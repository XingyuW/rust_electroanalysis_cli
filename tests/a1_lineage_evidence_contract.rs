use rust_electroanalysis_cli::domain::{
    AcquisitionFamilyId, AggregateExperimentScopeId, ArtifactAcquisitionFamilies,
    ArtifactDependency, ArtifactDependencyRole, ArtifactExperimentScope, ArtifactId,
    ArtifactIdentity, ArtifactKind, ArtifactLineageCatalog, ArtifactLineageNode,
    ArtifactLineageState, ExperimentId, LineageResolutionReason, LineageResolutionStatus,
    ResolvedAcquisitionFamilies, ScopeKey, artifact_identity_from_payload, resolve_lineage,
    semantic_sha256,
};
use rust_electroanalysis_cli::evidence::{
    CovarianceAxisId, CovarianceAxisValidationError, CovarianceQuantityKind,
    EvidenceArtifactSource, EvidenceAvailability, EvidenceBundle, EvidenceBundleBuilder,
    EvidenceBundleError, EvidenceDirection, EvidenceExperimentScope, EvidenceIndependence,
    EvidenceIndependenceAssessment, EvidencePairKey, EvidenceQuantity, EvidenceRecord,
    EvidenceScopeDerivation, EvidenceSourceClass, EvidenceSourceRef, EvidenceStrength,
    EvidenceTarget, EvidenceUncertaintyModel, EvidenceValidity, LegacySourceFingerprint,
    PairCovarianceDerivation, StrengthSource, TimescaleCrossCovariance, TimescalePairUncertainty,
    TimescalePairUncertaintySource, TimescaleTransformRegistry, analytic_delta_method_covariance,
    classify_independence, extract_direct_covariance, labeled_eis_covariance, validate_ucum_unit,
};
use rust_electroanalysis_cli::model::IdentifiabilityRequirementKind;
use rust_electroanalysis_cli::{
    AdapterContext, EvidenceBundleInputs, adapt_transient_analysis, assemble_evidence_bundle,
    legacy_context, try_adapt_calibration_observations,
};

fn id(label: &str) -> ArtifactId {
    ArtifactId::from_semantic_bytes(label.as_bytes())
}
fn family(label: &str) -> AcquisitionFamilyId {
    AcquisitionFamilyId::new(label).unwrap()
}
fn experiment(label: &str) -> ExperimentId {
    ExperimentId::new(label).unwrap()
}

fn identity(artifact_id: ArtifactId, families: ArtifactAcquisitionFamilies) -> ArtifactIdentity {
    let semantic_sha256 = artifact_id.0.strip_prefix("sha256:").unwrap().to_string();
    ArtifactIdentity {
        artifact_id,
        artifact_kind: ArtifactKind::SignalAnalysis,
        schema_version: 1,
        producer_version: "test".into(),
        experiment_scope: ArtifactExperimentScope::Single {
            experiment_id: experiment("exp-1"),
        },
        sensor_scope: ScopeKey::Specific("sensor-1".into()),
        channel_scope: ScopeKey::Specific("channel-1".into()),
        acquisition_families: families,
        semantic_sha256,
    }
}

fn known_node(artifact_id: ArtifactId, family_id: &str) -> ArtifactLineageNode {
    ArtifactLineageNode {
        identity: identity(
            artifact_id,
            ArtifactAcquisitionFamilies::known([family(family_id)]).unwrap(),
        ),
        direct_dependencies: vec![],
    }
}

fn record(id: &str, artifact_id: ArtifactId, quantity: Option<EvidenceQuantity>) -> EvidenceRecord {
    EvidenceRecord {
        evidence_id: rust_electroanalysis_cli::evidence::EvidenceId(id.into()),
        target: EvidenceTarget::ModelComponent(rust_electroanalysis_cli::evidence::ComponentId(
            "component".into(),
        )),
        source: EvidenceSourceRef {
            artifact: EvidenceArtifactSource::Known {
                artifact_id,
                artifact_kind: ArtifactKind::SignalAnalysis,
            },
            field_path: "$.value".into(),
        },
        experiment_scope: EvidenceExperimentScope::Single {
            experiment_id: experiment("exp-1"),
            derivation: EvidenceScopeDerivation::ArtifactScope,
        },
        source_class: EvidenceSourceClass::Observed,
        direction: EvidenceDirection::Supports,
        availability: EvidenceAvailability::Available,
        strength: EvidenceStrength::NotAssessed,
        validity: EvidenceValidity::Valid,
        quantity,
        strength_source: StrengthSource::NotAssessed,
        strength_derivation: None,
        threshold_provenance: vec![],
        lineage_artifact_ids: vec![],
        warnings: vec![],
    }
}

#[test]
fn a1_t01_t04_scope_identity_and_propagation_are_deterministic() {
    let members = vec![
        experiment("exp-b"),
        experiment("exp-a"),
        experiment("exp-b"),
    ];
    let aggregate = ArtifactExperimentScope::aggregate("calibration-analysis-v1", members).unwrap();
    let ArtifactExperimentScope::Aggregate {
        aggregate_scope_id,
        member_experiment_ids,
    } = &aggregate
    else {
        panic!("aggregate")
    };
    assert_eq!(
        member_experiment_ids,
        &vec![experiment("exp-a"), experiment("exp-b")]
    );
    assert_eq!(
        aggregate_scope_id,
        &AggregateExperimentScopeId::derive("calibration-analysis-v1", member_experiment_ids)
            .unwrap()
    );
    assert_ne!(
        aggregate_scope_id,
        &AggregateExperimentScopeId::derive("other-v1", member_experiment_ids).unwrap()
    );
    assert!(ArtifactExperimentScope::aggregate("bad", [experiment("only")]).is_err());
    assert!(matches!(
        ArtifactExperimentScope::propagate_with_kind(
            "test-propagation-v1",
            [
                ArtifactExperimentScope::Single {
                    experiment_id: experiment("exp-a")
                },
                ArtifactExperimentScope::Single {
                    experiment_id: experiment("exp-b")
                },
            ]
        ),
        ArtifactExperimentScope::Aggregate { .. }
    ));
    assert!(matches!(
        ArtifactExperimentScope::propagate_with_kind(
            "test-propagation-v1",
            [
                ArtifactExperimentScope::Unknown,
                ArtifactExperimentScope::Single {
                    experiment_id: experiment("exp-a")
                },
            ]
        ),
        ArtifactExperimentScope::Unknown
    ));
}

#[test]
fn a1_rv002_producer_scopes_are_specific_and_deterministic() {
    let members = [
        experiment("exp-a"),
        experiment("exp-b"),
        experiment("exp-a"),
    ];
    let mechanism =
        ArtifactExperimentScope::aggregate("mechanism-trend-v1", members.clone()).unwrap();
    let health = ArtifactExperimentScope::aggregate("health-trend-v1", members).unwrap();
    assert_ne!(mechanism, health);
    let ArtifactExperimentScope::Aggregate {
        member_experiment_ids,
        ..
    } = mechanism
    else {
        panic!("aggregate")
    };
    assert_eq!(
        member_experiment_ids,
        vec![experiment("exp-a"), experiment("exp-b")]
    );
}

#[test]
fn a1_rv004_ucum_units_reject_arbitrary_and_check_timescale_dimension() {
    assert!(validate_ucum_unit("V").is_ok());
    assert!(validate_ucum_unit("s").is_ok());
    assert!(validate_ucum_unit("not-a-unit").is_err());
    let invalid_tau = record(
        "bad-tau",
        id("bad-tau"),
        Some(EvidenceQuantity {
            value: 1.0,
            unit: "V".into(),
            uncertainty: None,
        }),
    );
    let valid_tau = record(
        "good-tau",
        id("good-tau"),
        Some(EvidenceQuantity {
            value: 2.0,
            unit: "s".into(),
            uncertainty: None,
        }),
    );
    let pair = EvidencePairKey::canonical(
        invalid_tau.evidence_id.clone(),
        valid_tau.evidence_id.clone(),
    )
    .unwrap();
    let mut catalog = ArtifactLineageCatalog::default();
    catalog
        .insert(known_node(id("bad-tau"), "family-a"))
        .unwrap();
    catalog
        .insert(known_node(id("good-tau"), "family-b"))
        .unwrap();
    let mut builder = EvidenceBundleBuilder::new(
        EvidenceExperimentScope::Unknown,
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        catalog,
    );
    builder.add_record(invalid_tau);
    builder.add_record(valid_tau);
    builder.add_timescale_pair_uncertainty(TimescalePairUncertainty {
        pair,
        covariance: TimescaleCrossCovariance::TauSpace {
            covariance_tau_s2: 0.1,
        },
        source: TimescalePairUncertaintySource {
            source_artifact: EvidenceArtifactSource::Known {
                artifact_id: id("bad-tau"),
                artifact_kind: ArtifactKind::SignalAnalysis,
            },
            left_source_field_path: "$.a".into(),
            right_source_field_path: "$.b".into(),
            covariance_source_field_path: "$.c".into(),
            derivation: PairCovarianceDerivation::ExtractedCovarianceMatrixEntry,
        },
    });
    assert!(matches!(
        builder.build(),
        Err(EvidenceBundleError::TimescaleCovarianceUnitMismatch)
    ));
}

#[test]
fn a1_rv005_rfc8785_hashes_normalize_numbers_keys_and_strings() {
    assert_eq!(
        semantic_sha256(&serde_json::json!({"b": 1e-7, "a": -0.0, "text": "€"})).unwrap(),
        semantic_sha256(&serde_json::json!({"text": "€", "a": 0.0, "b": 0.0000001})).unwrap()
    );
    assert_ne!(
        semantic_sha256(&serde_json::json!({"a": 1})).unwrap(),
        semantic_sha256(&serde_json::json!({"a": 2})).unwrap()
    );
    assert!(semantic_sha256(&f64::NAN).is_err());
}

#[test]
fn a1_semantic_hash_is_stable_for_ordering_and_changes_for_science_or_dependencies() {
    let dependency_a = ArtifactDependency {
        artifact_id: id("dep-a"),
        artifact_kind: ArtifactKind::SignalAnalysis,
        role: ArtifactDependencyRole::DerivedFrom,
    };
    let dependency_b = ArtifactDependency {
        artifact_id: id("dep-b"),
        artifact_kind: ArtifactKind::TransientAnalysis,
        role: ArtifactDependencyRole::Initialization,
    };
    let scope = ArtifactExperimentScope::Single {
        experiment_id: experiment("exp-1"),
    };
    let families = ArtifactAcquisitionFamilies::known([family("family-a")]).unwrap();
    let first = artifact_identity_from_payload(
        ArtifactKind::SignalAnalysis,
        1,
        "producer-v1",
        scope.clone(),
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        families.clone(),
        &[dependency_a.clone(), dependency_b.clone()],
        &serde_json::json!({"value": 1.0, "name": "feature"}),
    )
    .unwrap();
    let reordered = artifact_identity_from_payload(
        ArtifactKind::SignalAnalysis,
        1,
        "producer-v1",
        scope.clone(),
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        families.clone(),
        &[dependency_b.clone(), dependency_a.clone()],
        &serde_json::json!({"name": "feature", "value": 1.0}),
    )
    .unwrap();
    assert_eq!(first, reordered);
    let changed_science = artifact_identity_from_payload(
        ArtifactKind::SignalAnalysis,
        1,
        "producer-v1",
        scope.clone(),
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        families.clone(),
        &[dependency_a.clone(), dependency_b.clone()],
        &serde_json::json!({"value": 2.0, "name": "feature"}),
    )
    .unwrap();
    assert_ne!(first.artifact_id, changed_science.artifact_id);
    let changed_dependency = artifact_identity_from_payload(
        ArtifactKind::SignalAnalysis,
        1,
        "producer-v1",
        scope,
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        families,
        &[dependency_a],
        &serde_json::json!({"value": 1.0, "name": "feature"}),
    )
    .unwrap();
    assert_ne!(first.artifact_id, changed_dependency.artifact_id);
}

#[test]
fn acq_family_unknown_is_distinct_and_never_dropped() {
    let known =
        ArtifactAcquisitionFamilies::known([family(" B "), family("A"), family("A")]).unwrap();
    assert_eq!(
        known,
        ArtifactAcquisitionFamilies::Known(vec![family("A"), family("B")])
    );
    assert!(
        ArtifactAcquisitionFamilies::Known(vec![])
            .validate()
            .is_err()
    );
    assert_eq!(
        known.union(&ArtifactAcquisitionFamilies::Unknown),
        ArtifactAcquisitionFamilies::Unknown
    );
    assert_eq!(
        ResolvedAcquisitionFamilies::Known(vec![family("A")])
            .union(&ResolvedAcquisitionFamilies::Unknown),
        ResolvedAcquisitionFamilies::Unknown
    );
}

#[test]
fn lineage_root_is_state_based_and_preserves_missing_and_cycles() {
    let root_id = id("root");
    let child_id = id("child");
    let missing_id = id("missing");
    let root_identity = identity(
        root_id.clone(),
        ArtifactAcquisitionFamilies::known([family("root-family")]).unwrap(),
    );
    let root = ArtifactLineageState::Known {
        identity: root_identity.clone(),
        direct_dependencies: vec![
            ArtifactDependency {
                artifact_id: child_id.clone(),
                artifact_kind: ArtifactKind::SignalAnalysis,
                role: ArtifactDependencyRole::DerivedFrom,
            },
            ArtifactDependency {
                artifact_id: missing_id.clone(),
                artifact_kind: ArtifactKind::SignalAnalysis,
                role: ArtifactDependencyRole::AuxiliaryInput,
            },
        ],
    };
    let mut catalog = ArtifactLineageCatalog::default();
    catalog
        .insert(known_node(root_id.clone(), "root-family"))
        .unwrap();
    catalog
        .artifacts
        .get_mut(&root_id)
        .unwrap()
        .direct_dependencies = match &root {
        ArtifactLineageState::Known {
            direct_dependencies,
            ..
        } => direct_dependencies.clone(),
        _ => unreachable!(),
    };
    catalog
        .insert(known_node(child_id.clone(), "child-family"))
        .unwrap();
    let resolved = resolve_lineage(&root, &catalog);
    assert_eq!(resolved.root_artifact_id, Some(root_id));
    assert_eq!(resolved.status, LineageResolutionStatus::Incomplete);
    assert_eq!(resolved.missing_artifact_ids, vec![missing_id.clone()]);
    assert!(resolved.ancestor_artifact_ids.contains(&child_id));

    let legacy = resolve_lineage(&ArtifactLineageState::default(), &catalog);
    assert_eq!(legacy.status, LineageResolutionStatus::Incomplete);
    assert_eq!(legacy.root_artifact_id, None);
    assert_eq!(
        legacy.acquisition_families,
        ResolvedAcquisitionFamilies::Unknown
    );
    assert_eq!(
        legacy.reasons,
        vec![LineageResolutionReason::LegacyUnknownRoot]
    );

    let cycle_root_id = id("cycle-root");
    let cycle_child_id = id("cycle-child");
    let cycle_root_identity = identity(
        cycle_root_id.clone(),
        ArtifactAcquisitionFamilies::known([family("cycle-root")]).unwrap(),
    );
    let cycle_root = ArtifactLineageState::Known {
        identity: cycle_root_identity.clone(),
        direct_dependencies: vec![ArtifactDependency {
            artifact_id: cycle_child_id.clone(),
            artifact_kind: ArtifactKind::SignalAnalysis,
            role: ArtifactDependencyRole::DerivedFrom,
        }],
    };
    let mut cycle_catalog = ArtifactLineageCatalog::default();
    cycle_catalog
        .insert(ArtifactLineageNode {
            identity: cycle_root_identity,
            direct_dependencies: match &cycle_root {
                ArtifactLineageState::Known {
                    direct_dependencies,
                    ..
                } => direct_dependencies.clone(),
                _ => unreachable!(),
            },
        })
        .unwrap();
    cycle_catalog
        .insert(ArtifactLineageNode {
            identity: identity(
                cycle_child_id.clone(),
                ArtifactAcquisitionFamilies::known([family("cycle-child")]).unwrap(),
            ),
            direct_dependencies: vec![ArtifactDependency {
                artifact_id: cycle_root_id,
                artifact_kind: ArtifactKind::SignalAnalysis,
                role: ArtifactDependencyRole::DerivedFrom,
            }],
        })
        .unwrap();
    let cycle = resolve_lineage(&cycle_root, &cycle_catalog);
    assert_eq!(cycle.status, LineageResolutionStatus::CycleDetected);
    assert!(
        cycle
            .reasons
            .iter()
            .any(|reason| matches!(reason, LineageResolutionReason::CycleDetected { .. }))
    );
}

#[test]
fn evidence_combinations_and_legacy_sources_are_conservative() {
    let mut missing = record("missing", id("missing-source"), None);
    missing.availability = EvidenceAvailability::Missing;
    missing.direction = EvidenceDirection::Neutral;
    missing.validity = EvidenceValidity::NotAssessed;
    assert!(missing.validate().is_ok());
    missing.strength = EvidenceStrength::Strong;
    assert_eq!(
        missing.validate(),
        Err(EvidenceBundleError::MissingEvidenceCombination)
    );

    let mut not_applicable = record("na", id("na-source"), None);
    not_applicable.availability = EvidenceAvailability::NotApplicable;
    not_applicable.direction = EvidenceDirection::NotApplicable;
    not_applicable.validity = EvidenceValidity::NotAssessed;
    assert!(not_applicable.validate().is_ok());

    let mut assessed = record("assessed", id("assessed-source"), None);
    assessed.strength = EvidenceStrength::Moderate;
    assert_eq!(
        assessed.validate(),
        Err(EvidenceBundleError::AssessedStrengthMissingSource)
    );

    let legacy_source = EvidenceArtifactSource::LegacyUnknown {
        artifact_kind: ArtifactKind::SignalAnalysis,
        source_fingerprint: LegacySourceFingerprint::from_bytes(b"legacy"),
    };
    assert!(!matches!(
        legacy_source,
        EvidenceArtifactSource::Known { .. }
    ));
    let _ = EvidenceUncertaintyModel::LogNormal {
        variance_ln_tau_s: 0.1,
    };
}

#[test]
fn pairwise_independence_and_builder_use_exact_provenance() {
    let a = id("a");
    let b = id("b");
    let c = id("c");
    let mut catalog = ArtifactLineageCatalog::default();
    catalog.insert(known_node(a.clone(), "family-a")).unwrap();
    catalog.insert(known_node(b.clone(), "family-b")).unwrap();
    catalog.insert(known_node(c.clone(), "family-a")).unwrap();
    let left = record("a-evidence", a.clone(), None);
    let right = record("b-evidence", b.clone(), None);
    let shared = record("c-evidence", c, None);
    let independent = classify_independence(&left, &right, &catalog);
    assert_eq!(
        independent.classification,
        EvidenceIndependence::Independent
    );
    assert_eq!(
        classify_independence(&left, &shared, &catalog).classification,
        EvidenceIndependence::PartiallyDependent
    );
    let mut legacy = right.clone();
    legacy.evidence_id = rust_electroanalysis_cli::evidence::EvidenceId("legacy-evidence".into());
    legacy.source.artifact = EvidenceArtifactSource::LegacyUnknown {
        artifact_kind: ArtifactKind::SignalAnalysis,
        source_fingerprint: LegacySourceFingerprint::from_bytes(b"legacy"),
    };
    assert_eq!(
        classify_independence(&left, &legacy, &catalog).classification,
        EvidenceIndependence::Unknown
    );

    let mut builder = EvidenceBundleBuilder::new(
        EvidenceExperimentScope::Unknown,
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        catalog,
    );
    builder.add_record(left.clone());
    builder.add_record(right.clone());
    let pair =
        EvidencePairKey::canonical(left.evidence_id.clone(), right.evidence_id.clone()).unwrap();
    builder.add_independence_assessment(EvidenceIndependenceAssessment {
        pair: EvidencePairKey {
            left_evidence_id: pair.right_evidence_id.clone(),
            right_evidence_id: pair.left_evidence_id.clone(),
        },
        classification: independent.classification,
        algorithm_id: "lineage.v1".into(),
        left_lineage_status: independent.right_lineage_status,
        right_lineage_status: independent.left_lineage_status,
        shared_ancestor_artifact_ids: vec![],
        shared_acquisition_families: vec![],
        reasons: vec![],
    });
    let bundle = builder.build().unwrap();
    assert_eq!(bundle.independence_assessments[0].pair, pair);
}

#[test]
fn labeled_covariance_and_timescale_pair_are_exact_and_durable() {
    let matrix = labeled_eis_covariance(
        &[
            ("CPE1".into(), "Q".into(), "Ohm^-1 s^alpha".into()),
            ("CPE1".into(), "alpha".into(), "".into()),
        ],
        vec![vec![1.0, 0.1], vec![0.1, 2.0]],
    )
    .unwrap();
    assert!(
        matrix
            .lookup_exact(&rust_electroanalysis_cli::evidence::CovarianceAxisId(
                "eis.parameter:CPE1:q".into()
            ))
            .is_some()
    );
    assert!(
        matrix
            .lookup_exact(&rust_electroanalysis_cli::evidence::CovarianceAxisId(
                "eis.parameter:CPE1:alpha".into()
            ))
            .is_some()
    );
    assert_ne!(
        matrix.lookup_exact(&rust_electroanalysis_cli::evidence::CovarianceAxisId(
            "eis.parameter:CPE1".into()
        )),
        matrix.lookup_exact(&rust_electroanalysis_cli::evidence::CovarianceAxisId(
            "eis.parameter:CPE1:q".into()
        ))
    );
    assert_eq!(
        labeled_eis_covariance(
            &[
                ("CPE1".into(), "Q".into(), "".into()),
                ("CPE1".into(), "Q".into(), "".into())
            ],
            vec![vec![1.0, 0.0], vec![0.0, 1.0]]
        ),
        Err(CovarianceAxisValidationError::DuplicateCovarianceAxisId)
    );
    assert!(matches!(
        labeled_eis_covariance(
            &[("CPE1".into(), "unknown".into(), "".into())],
            vec![vec![1.0]]
        ),
        Err(CovarianceAxisValidationError::UnknownEisParameterKey)
    ));
    assert_eq!(
        matrix.axes[1].quantity_kind,
        CovarianceQuantityKind::Parameter
    );
    let q_axis = CovarianceAxisId("eis.parameter:CPE1:q".into());
    let alpha_axis = CovarianceAxisId("eis.parameter:CPE1:alpha".into());
    assert_eq!(
        extract_direct_covariance(&matrix, &q_axis, &alpha_axis).unwrap(),
        0.1
    );
    assert!(
        extract_direct_covariance(
            &matrix,
            &CovarianceAxisId("eis.parameter:CPE1:wrong".into()),
            &alpha_axis
        )
        .is_err()
    );
    let mut registry = TimescaleTransformRegistry::default();
    registry.register("approved.delta.v1").unwrap();
    let jacobian = rust_electroanalysis_cli::evidence::TimescaleJacobianDefinition {
        source_axis_ids: vec![q_axis.clone(), alpha_axis.clone()],
        coefficients: vec![2.0, 3.0],
        units: vec!["1".into(), "1".into()],
    };
    assert!(
        matches!(analytic_delta_method_covariance(&registry, "approved.delta.v1", &matrix, &jacobian, &jacobian).unwrap(), TimescaleCrossCovariance::TauSpace { covariance_tau_s2 } if (covariance_tau_s2 - 23.2).abs() < 1e-12)
    );
    assert!(
        analytic_delta_method_covariance(&registry, "unregistered", &matrix, &jacobian, &jacobian)
            .is_err()
    );

    let a = id("timescale-a");
    let b = id("timescale-b");
    let mut catalog = ArtifactLineageCatalog::default();
    catalog.insert(known_node(a.clone(), "family-a")).unwrap();
    catalog.insert(known_node(b.clone(), "family-b")).unwrap();
    let left = record(
        "tau-a",
        a.clone(),
        Some(EvidenceQuantity {
            value: 1.0,
            unit: "s".into(),
            uncertainty: Some(EvidenceUncertaintyModel::LogNormal {
                variance_ln_tau_s: 0.1,
            }),
        }),
    );
    let right = record(
        "tau-b",
        b,
        Some(EvidenceQuantity {
            value: 2.0,
            unit: "s".into(),
            uncertainty: Some(EvidenceUncertaintyModel::LogNormal {
                variance_ln_tau_s: 0.2,
            }),
        }),
    );
    let pair =
        EvidencePairKey::canonical(left.evidence_id.clone(), right.evidence_id.clone()).unwrap();
    let mut builder = EvidenceBundleBuilder::new(
        EvidenceExperimentScope::Unknown,
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        catalog,
    );
    builder.add_record(left);
    builder.add_record(right);
    builder.add_timescale_pair_uncertainty(TimescalePairUncertainty {
        pair: pair.clone(),
        covariance: TimescaleCrossCovariance::LogSpace {
            covariance_ln_tau: 0.03,
        },
        source: TimescalePairUncertaintySource {
            source_artifact: EvidenceArtifactSource::Known {
                artifact_id: a,
                artifact_kind: ArtifactKind::SignalAnalysis,
            },
            left_source_field_path: "$.tau_a".into(),
            right_source_field_path: "$.tau_b".into(),
            covariance_source_field_path: "$.covariance".into(),
            derivation: PairCovarianceDerivation::ExtractedCovarianceMatrixEntry,
        },
    });
    let bundle = builder.build().unwrap();
    assert_eq!(
        bundle
            .lookup_timescale_pair_uncertainty(&pair)
            .unwrap()
            .pair,
        pair
    );
    assert!(bundle.semantic_hash().unwrap().len() == 64);
}

#[test]
fn a1_t13_open_identifiability_kinds_preserve_known_and_custom_strings() {
    let known = serde_json::to_string(&IdentifiabilityRequirementKind::ModeSeparation).unwrap();
    assert_eq!(known, "\"mode_separation\"");
    let custom: IdentifiabilityRequirementKind =
        serde_json::from_str("\"future_requirement_v2\"").unwrap();
    assert_eq!(custom.as_str(), "future_requirement_v2");
    assert_eq!(
        serde_json::to_string(&custom).unwrap(),
        "\"future_requirement_v2\""
    );
    assert!(serde_json::from_str::<IdentifiabilityRequirementKind>("\"\"").is_err());
}

#[test]
fn a1_fixture_set_is_tracked_and_migrates_conservatively() {
    let legacy: ArtifactLineageState =
        serde_json::from_str(include_str!("fixtures/a1/legacy_lineage_state.json")).unwrap();
    assert!(matches!(legacy, ArtifactLineageState::LegacyUnknown { .. }));
    let known: ArtifactLineageState =
        serde_json::from_str(include_str!("fixtures/a1/current_known_lineage_state.json")).unwrap();
    let ArtifactLineageState::Known { identity, .. } = known else {
        panic!("known fixture")
    };
    identity.validate().unwrap();
    let aggregate: ArtifactExperimentScope =
        serde_json::from_str(include_str!("fixtures/a1/aggregate_scope.json")).unwrap();
    assert!(matches!(
        aggregate,
        ArtifactExperimentScope::Aggregate { .. }
    ));
    let legacy_covariance: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/a1/legacy_unlabeled_covariance.json")).unwrap();
    assert!(legacy_covariance.get("parameter_covariance").is_some());
    assert!(
        legacy_covariance
            .get("labeled_parameter_covariance")
            .is_some_and(serde_json::Value::is_null)
    );
    let current_covariance: rust_electroanalysis_cli::evidence::LabeledCovarianceMatrix =
        serde_json::from_str(include_str!("fixtures/a1/current_labeled_covariance.json")).unwrap();
    assert!(
        current_covariance
            .lookup_exact(&rust_electroanalysis_cli::evidence::CovarianceAxisId(
                "eis.parameter:CPE1:q".into()
            ))
            .is_some()
    );
}

#[test]
fn a1_adapter_reads_public_transient_artifact_without_inventing_strength() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/a0_artifact_contracts/schema1/transient_analysis.schema1.json");
    let artifact: rust_electroanalysis_cli::results::TransientAnalysisReport =
        rust_electroanalysis_cli::domain::read_artifact(&path).unwrap();
    let context = legacy_context(
        ArtifactKind::TransientAnalysis,
        br"legacy-transient",
        EvidenceExperimentScope::Unknown,
    );
    let records = adapt_transient_analysis(&artifact, &context);
    assert!(!records.is_empty());
    assert!(
        records
            .iter()
            .all(|record| record.strength == EvidenceStrength::NotAssessed)
    );
    assert!(records.iter().all(|record| matches!(
        record.source.artifact,
        EvidenceArtifactSource::LegacyUnknown { .. }
    )));
    let output = std::env::temp_dir().join(format!("a1-lineage-{}.json", std::process::id()));
    rust_electroanalysis_cli::domain::write_artifact(&output, &artifact).unwrap();
    let text = std::fs::read_to_string(&output).unwrap();
    assert!(text.contains("\"lineage\""));
    assert!(text.contains("LegacyUnknown"));
    let _ = std::fs::remove_file(output);
    let _ = AdapterContext::new(
        EvidenceArtifactSource::LegacyUnknown {
            artifact_kind: ArtifactKind::TransientAnalysis,
            source_fingerprint: LegacySourceFingerprint::from_bytes(b"same"),
        },
        EvidenceExperimentScope::Unknown,
    );
}

#[test]
fn a1_rv003_production_assembly_reads_public_artifact_and_builds_relations() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/a0_artifact_contracts/schema1/transient_analysis.schema1.json");
    let transient: rust_electroanalysis_cli::results::TransientAnalysisReport =
        rust_electroanalysis_cli::domain::read_artifact(&path).unwrap();
    let bundle = assemble_evidence_bundle(EvidenceBundleInputs {
        transient: Some(transient),
        ..EvidenceBundleInputs::default()
    })
    .unwrap();
    assert!(!bundle.records.is_empty());
    assert!(!bundle.independence_assessments.is_empty());
    assert!(
        bundle
            .records
            .iter()
            .all(|record| record.strength == EvidenceStrength::NotAssessed)
    );
}

#[test]
fn a1_rv003_production_eis_labeled_covariance_builds_timescale_relation() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/a0_artifact_contracts/eis_fit_schema2_correct_kind.json");
    let mut eis: rust_electroanalysis_cli::results::EisFitArtifact =
        rust_electroanalysis_cli::domain::read_artifact(&path).unwrap();
    eis.lineage =
        serde_json::from_str(include_str!("fixtures/a1/current_known_lineage_state.json")).unwrap();
    eis.parameters = vec![
        rust_electroanalysis_cli::results::EisFittedParameter {
            name: "tau".into(),
            element_id: "Wo1".into(),
            element_type: "Wo".into(),
            semantic_role: None,
            unit: "s".into(),
            value: 1.0,
            standard_error: None,
            lower_bound: None,
            upper_bound: None,
            at_bound: false,
        },
        rust_electroanalysis_cli::results::EisFittedParameter {
            name: "tau".into(),
            element_id: "Ws1".into(),
            element_type: "Ws".into(),
            semantic_role: None,
            unit: "s".into(),
            value: 2.0,
            standard_error: None,
            lower_bound: None,
            upper_bound: None,
            at_bound: false,
        },
    ];
    eis.statistics.labeled_parameter_covariance = Some(
        labeled_eis_covariance(
            &[
                ("Wo1".into(), "tau".into(), "s".into()),
                ("Ws1".into(), "tau".into(), "s".into()),
            ],
            vec![vec![0.1, 0.02], vec![0.02, 0.2]],
        )
        .unwrap(),
    );
    let bundle = assemble_evidence_bundle(EvidenceBundleInputs {
        eis_fit: Some(eis),
        ..EvidenceBundleInputs::default()
    })
    .unwrap();
    assert_eq!(bundle.timescale_pair_uncertainties.len(), 1);
    let source = &bundle.timescale_pair_uncertainties[0].source;
    assert!(source.left_source_field_path.contains("parameters[0]"));
    assert!(source.right_source_field_path.contains("parameters[1]"));
}

#[test]
fn a1_deserializers_reject_invalid_axes_pairs_and_known_lineage() {
    let matrix =
        labeled_eis_covariance(&[("R1".into(), "R".into(), "Ohm".into())], vec![vec![1.0]])
            .unwrap();
    let mut invalid_matrix = serde_json::to_value(&matrix).unwrap();
    invalid_matrix["axes"] = serde_json::json!([matrix.axes[0].clone(), matrix.axes[0].clone()]);
    invalid_matrix["values"] = serde_json::json!([[1.0, 0.0], [0.0, 1.0]]);
    assert!(
        serde_json::from_value::<rust_electroanalysis_cli::evidence::LabeledCovarianceMatrix>(
            invalid_matrix
        )
        .is_err()
    );

    let left_id = id("deserialize-left");
    let right_id = id("deserialize-right");
    let mut catalog = ArtifactLineageCatalog::default();
    catalog
        .insert(known_node(left_id.clone(), "left-family"))
        .unwrap();
    catalog
        .insert(known_node(right_id.clone(), "right-family"))
        .unwrap();
    let left = record("left", left_id, None);
    let right = record("right", right_id, None);
    let expected = classify_independence(&left, &right, &catalog);
    let pair =
        EvidencePairKey::canonical(left.evidence_id.clone(), right.evidence_id.clone()).unwrap();
    let mut builder = EvidenceBundleBuilder::new(
        EvidenceExperimentScope::Unknown,
        ScopeKey::Unspecified,
        ScopeKey::Unspecified,
        catalog,
    );
    builder.add_record(left);
    builder.add_record(right);
    builder.add_independence_assessment(EvidenceIndependenceAssessment {
        pair: pair.clone(),
        classification: expected.classification,
        algorithm_id: "lineage.v1".into(),
        left_lineage_status: expected.left_lineage_status,
        right_lineage_status: expected.right_lineage_status,
        shared_ancestor_artifact_ids: expected.shared_ancestor_artifact_ids,
        shared_acquisition_families: expected.shared_acquisition_families,
        reasons: expected.reasons,
    });
    let bundle = builder.build().unwrap();
    let mut invalid_bundle = serde_json::to_value(&bundle).unwrap();
    invalid_bundle["independence_assessments"][0]["pair"] = serde_json::json!({
        "left_evidence_id": pair.right_evidence_id,
        "right_evidence_id": pair.left_evidence_id,
    });
    assert!(serde_json::from_value::<EvidenceBundle>(invalid_bundle).is_err());

    let mut invalid_lineage = serde_json::to_value(ArtifactLineageState::Known {
        identity: identity(id("valid-lineage"), ArtifactAcquisitionFamilies::Unknown),
        direct_dependencies: vec![],
    })
    .unwrap();
    invalid_lineage["Known"]["identity"]["artifact_id"] = serde_json::json!("invalid");
    assert!(serde_json::from_value::<ArtifactLineageState>(invalid_lineage).is_err());
}

#[test]
fn a1_rv006_selected_member_scope_is_producer_proven_and_typed() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/a0_artifact_contracts/schema1/calibration_observations.schema1.json");
    let mut artifact: rust_electroanalysis_cli::results::CalibrationObservationSet =
        rust_electroanalysis_cli::domain::read_artifact(&path).unwrap();
    let aggregate = ArtifactExperimentScope::aggregate(
        "calibration-observations-v1",
        [experiment("a0-experiment"), experiment("other-experiment")],
    )
    .unwrap();
    let context = AdapterContext::new(
        EvidenceArtifactSource::LegacyUnknown {
            artifact_kind: ArtifactKind::CalibrationObservations,
            source_fingerprint: LegacySourceFingerprint::from_bytes(b"calibration"),
        },
        EvidenceExperimentScope::from_artifact_scope(&aggregate),
    );
    let selected = try_adapt_calibration_observations(&artifact, &context).unwrap();
    assert!(matches!(
        selected[0].experiment_scope,
        EvidenceExperimentScope::Single {
            derivation: EvidenceScopeDerivation::MemberRecord { ref source_field_path, .. },
            ..
        } if source_field_path == "$.observations[0].experiment_id"
    ));
    artifact.observations[0].experiment_id = "not-a-member".into();
    assert_eq!(
        try_adapt_calibration_observations(&artifact, &context),
        Err(EvidenceBundleError::ScopeMemberRecordMismatch)
    );
    artifact.observations[0].experiment_id.clear();
    assert_eq!(
        try_adapt_calibration_observations(&artifact, &context),
        Err(EvidenceBundleError::ScopeRecordMissingExperimentId)
    );
}

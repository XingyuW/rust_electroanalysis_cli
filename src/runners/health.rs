use crate::{
    health::{self, baseline::Context},
    health_config::{LoadedHealthConfig, PhaseCHealthEvidenceConfig},
    results::{HealthDomain, HealthWarning, SensorHealthAssessment, SensorHealthBaseline},
    runners::RunnerError,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Deserialize)]
pub struct HealthManifest {
    pub schema_version: u32,
    pub records: Vec<HealthManifestRecord>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct HealthManifestRecord {
    pub record_id: String,
    pub signal_results: PathBuf,
    #[serde(default)]
    pub transient_results: Option<PathBuf>,
    #[serde(default)]
    pub calibration_results: Option<PathBuf>,
    #[serde(default)]
    pub eis_fit: Option<PathBuf>,
    #[serde(default)]
    pub mechanism_results: Option<PathBuf>,
    #[serde(default)]
    pub metadata: Option<PathBuf>,
    #[serde(default)]
    pub independent_value: Option<f64>,
}
pub fn baseline(
    workspace: &Path,
    manifest: &Path,
    config_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let loaded = LoadedHealthConfig::load(workspace, config_path)?;
    let path = resolve(workspace, manifest);
    let man: HealthManifest = read_toml(&path)?;
    if man.schema_version != 1 {
        return Err(RunnerError::Message(
            "unsupported health manifest schema".into(),
        ));
    }
    let base = path.parent().unwrap_or(workspace);
    let mut records = Vec::new();
    let mut lineage_sources = Vec::new();
    let mut provenance = None;
    for r in &man.records {
        let signal_path = resolve(base, &r.signal_results);
        let signal: crate::results::SignalAnalysisReport =
            crate::domain::read_artifact(&signal_path)?;
        lineage_sources.push((
            signal.lineage.clone(),
            crate::domain::ArtifactDependencyRole::DerivedFrom,
        ));
        provenance.get_or_insert(signal.provenance.clone());
        let mut fs = health::features::from_signal(&signal);
        if let Some(p) = &r.transient_results {
            let t: crate::results::TransientAnalysisReport =
                crate::domain::read_artifact(&resolve(base, p))?;
            lineage_sources.push((
                t.lineage.clone(),
                crate::domain::ArtifactDependencyRole::TransformationInput,
            ));
            fs.extend(health::features::from_transient(&t));
        }
        if let Some(p) = &r.calibration_results {
            let c: crate::results::CalibrationAnalysisReport =
                crate::domain::read_artifact(&resolve(base, p))?;
            lineage_sources.push((
                c.lineage.clone(),
                crate::domain::ArtifactDependencyRole::TransformationInput,
            ));
            fs.extend(health::features::from_calibration(&c));
        }
        if let Some(p) = &r.eis_fit {
            let e: crate::results::EisFitArtifact =
                crate::domain::read_artifact(&resolve(base, p))?;
            lineage_sources.push((
                e.lineage.clone(),
                crate::domain::ArtifactDependencyRole::TransformationInput,
            ));
            fs.extend(health::features::from_eis(&e));
        }
        if let Some(p) = &r.mechanism_results {
            let m: crate::results::MechanismAnalysisReport =
                crate::domain::read_artifact(&resolve(base, p))?;
            lineage_sources.push((
                m.lineage.clone(),
                crate::domain::ArtifactDependencyRole::TransformationInput,
            ));
            fs.extend(health::features::from_mechanism(&m));
        }
        let context = r
            .metadata
            .as_ref()
            .map(|p| load_context(&resolve(base, p)))
            .transpose()?
            .unwrap_or_default();
        records.push((r.record_id.clone(), fs, context));
    }
    let provenance = provenance
        .ok_or_else(|| RunnerError::Message("health baseline manifest is empty".into()))?;
    let mut b = health::baseline::build_with_contexts(
        "health-baseline",
        &records,
        provenance,
        loaded.config.baseline.minimum_required_records,
    );
    let fallback_scope = lineage_scope(&b.lineage);
    b.lineage = derived_lineage(
        crate::domain::ArtifactKind::HealthBaseline,
        &b,
        &lineage_sources,
        fallback_scope,
    );
    let dest = output_file(workspace, output, &loaded.config.export.baseline_filename);
    crate::domain::write_artifact(&dest, &b)?;
    println!("Health baseline written to {}", dest.display());
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn build_legacy_assessment(
    workspace: &Path,
    signal_path: &Path,
    transient: Option<&Path>,
    calibration: Option<&Path>,
    eis: Option<&Path>,
    mechanism: Option<&Path>,
    baseline_path: Option<&Path>,
    metadata: Option<&Path>,
    config_path: Option<&Path>,
    _output: Option<&Path>,
) -> Result<SensorHealthAssessment, RunnerError> {
    let loaded = LoadedHealthConfig::load(workspace, config_path)?;
    let signal_path = resolve(workspace, signal_path);
    let signal: crate::results::SignalAnalysisReport = crate::domain::read_artifact(&signal_path)?;
    let mut lineage_sources = vec![(
        signal.lineage.clone(),
        crate::domain::ArtifactDependencyRole::DerivedFrom,
    )];
    let mut features = health::features::from_signal(&signal);
    let mut missing = Vec::new();
    if let Some(p) = transient {
        let r: crate::results::TransientAnalysisReport =
            crate::domain::read_artifact(&resolve(workspace, p))?;
        lineage_sources.push((
            r.lineage.clone(),
            crate::domain::ArtifactDependencyRole::TransformationInput,
        ));
        features.extend(health::features::from_transient(&r));
    } else {
        missing.push(HealthDomain::DynamicResponse);
    }
    if let Some(p) = calibration {
        let r: crate::results::CalibrationAnalysisReport =
            crate::domain::read_artifact(&resolve(workspace, p))?;
        lineage_sources.push((
            r.lineage.clone(),
            crate::domain::ArtifactDependencyRole::TransformationInput,
        ));
        features.extend(health::features::from_calibration(&r));
    } else {
        missing.push(HealthDomain::Calibration);
    }
    if let Some(p) = eis {
        let r: crate::results::EisFitArtifact =
            crate::domain::read_artifact(&resolve(workspace, p))?;
        lineage_sources.push((
            r.lineage.clone(),
            crate::domain::ArtifactDependencyRole::TransformationInput,
        ));
        features.extend(health::features::from_eis(&r));
    } else {
        missing.push(HealthDomain::Impedance);
    }
    if let Some(p) = mechanism {
        let r: crate::results::MechanismAnalysisReport =
            crate::domain::read_artifact(&resolve(workspace, p))?;
        lineage_sources.push((
            r.lineage.clone(),
            crate::domain::ArtifactDependencyRole::TransformationInput,
        ));
        features.extend(health::features::from_mechanism(&r));
    } else {
        missing.push(HealthDomain::MechanismEvidence);
    }
    let mut warnings = signal
        .warnings
        .iter()
        .map(|_| HealthWarning::AssessmentBasedOnWarningBearingFits)
        .collect::<Vec<_>>();
    let base = baseline_path.map(|p| resolve(workspace, p));
    let baseline: Option<SensorHealthBaseline> = if let Some(p) = base.as_deref() {
        let baseline: SensorHealthBaseline = crate::domain::read_artifact(p)?;
        lineage_sources.push((
            baseline.lineage.clone(),
            crate::domain::ArtifactDependencyRole::Prior,
        ));
        if baseline.records.len() < loaded.config.baseline.minimum_required_records {
            warnings.push(HealthWarning::InsufficientBaselineRecords);
        }
        Some(baseline)
    } else {
        warnings.push(HealthWarning::MissingBaseline);
        None
    };
    let current_context = metadata
        .map(|p| load_context(&resolve(workspace, p)))
        .transpose()?
        .unwrap_or_default();
    let base_context = baseline
        .as_ref()
        .map(|b| Context {
            sensor_id: None,
            sensor_type: b.sensor_type.clone(),
            sensor_design: b.sensor_design.clone(),
            analyte: b.analyte.clone(),
            sample_matrix: b.sample_matrix.clone(),
            temperature_k: b.temperature_domain_k.map(|x| (x.0 + x.1) / 2.0),
            temperature_values_k: b
                .temperature_domain_k
                .map(|x| vec![x.0, x.1])
                .unwrap_or_default(),
            experiment_id: None,
            metadata_source: None,
        })
        .unwrap_or_default();
    let mut comparisons = Vec::new();
    for f in &features {
        let (cmp, reason) = if let Some(b) = &baseline {
            let bdist = b.feature_distributions.iter().find(|x| x.feature == f.name);
            let (c, r) = health::normalization::comparable(
                &current_context,
                &base_context,
                &loaded.config.comparability,
            );
            (
                c,
                r.or_else(|| {
                    bdist
                        .is_none()
                        .then_some("feature absent from baseline".into())
                }),
            )
        } else {
            (
                crate::results::FeatureComparability::Unknown,
                Some("baseline unavailable".into()),
            )
        };
        let dist = baseline
            .as_ref()
            .and_then(|b| b.feature_distributions.iter().find(|x| x.feature == f.name));
        let mut c = health::normalization::compare_with_config(
            f,
            dist,
            cmp,
            &loaded.config.normalization,
            None,
        );
        c.override_reason = reason;
        if matches!(cmp, crate::results::FeatureComparability::NotComparable) {
            warnings.push(HealthWarning::FeatureNoncomparable);
        }
        comparisons.push(c);
    }
    let (evaluations, findings) = health::rules::evaluate_with_baseline_records(
        &loaded.config.rules,
        &features,
        &comparisons,
        loaded
            .config
            .assessment
            .minimum_domains_for_mechanistic_finding,
        baseline
            .as_ref()
            .map(|baseline| baseline.records.len())
            .unwrap_or(0),
    );
    let domains = features.iter().map(|f| f.domain).collect::<BTreeSet<_>>();
    if domains.len() < loaded.config.assessment.minimum_domains_for_assessment {
        warnings.push(HealthWarning::InsufficientEvidenceDomains);
    }
    let mut assessment = health::assessment::assemble(
        &format!("health:{}", signal.analysis_id),
        signal.sensor_id.clone(),
        signal.experiment_id.clone(),
        features,
        comparisons,
        evaluations,
        findings,
        missing,
        loaded.config,
        signal.provenance,
        warnings,
    );
    let fallback_scope = lineage_scope(&assessment.lineage);
    assessment.lineage = derived_lineage(
        crate::domain::ArtifactKind::HealthAssessment,
        &assessment,
        &lineage_sources,
        fallback_scope,
    );
    Ok(assessment)
}

#[derive(Debug, Clone, Copy)]
struct PhaseCHealthInputPaths<'a> {
    transient: Option<&'a Path>,
    calibration: Option<&'a Path>,
    baseline: Option<&'a Path>,
    estimation: Option<&'a Path>,
    model: Option<&'a Path>,
    mechanism: Option<&'a Path>,
    lineage_catalog: Option<&'a Path>,
}

#[allow(clippy::too_many_arguments)]
pub fn assess(
    workspace: &Path,
    signal_path: &Path,
    transient: Option<&Path>,
    calibration: Option<&Path>,
    eis: Option<&Path>,
    legacy_mechanism_results: Option<&Path>,
    baseline_path: Option<&Path>,
    metadata: Option<&Path>,
    legacy_config_path: Option<&Path>,
    phase_c_config: Option<&Path>,
    estimation_artifact: Option<&Path>,
    model_artifact: Option<&Path>,
    phase_c_mechanism_artifact: Option<&Path>,
    lineage_catalog: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let phase_c_only = [
        estimation_artifact,
        model_artifact,
        phase_c_mechanism_artifact,
        lineage_catalog,
    ]
    .into_iter()
    .any(|path| path.is_some());
    match (phase_c_config, phase_c_only) {
        (None, false) => assess_legacy(
            workspace,
            signal_path,
            transient,
            calibration,
            eis,
            legacy_mechanism_results,
            baseline_path,
            metadata,
            legacy_config_path,
            output,
        ),
        (None, true) => Err(RunnerError::Message(
            "Phase-C artifact flags require --phase-c-config".into(),
        )),
        (Some(config), _) => assess_phase_c(
            workspace,
            signal_path,
            transient,
            calibration,
            eis,
            legacy_mechanism_results,
            baseline_path,
            metadata,
            legacy_config_path,
            config,
            PhaseCHealthInputPaths {
                transient,
                calibration,
                baseline: baseline_path,
                estimation: estimation_artifact,
                model: model_artifact,
                mechanism: phase_c_mechanism_artifact,
                lineage_catalog,
            },
            output,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn assess_legacy(
    workspace: &Path,
    signal_path: &Path,
    transient: Option<&Path>,
    calibration: Option<&Path>,
    eis: Option<&Path>,
    mechanism: Option<&Path>,
    baseline_path: Option<&Path>,
    metadata: Option<&Path>,
    config_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let assessment = build_legacy_assessment(
        workspace,
        signal_path,
        transient,
        calibration,
        eis,
        mechanism,
        baseline_path,
        metadata,
        config_path,
        output,
    )?;
    export_legacy_assessment(workspace, output, &assessment)
}

#[allow(clippy::too_many_arguments)]
fn assess_phase_c(
    workspace: &Path,
    signal_path: &Path,
    transient: Option<&Path>,
    calibration: Option<&Path>,
    eis: Option<&Path>,
    legacy_mechanism: Option<&Path>,
    baseline_path: Option<&Path>,
    metadata: Option<&Path>,
    legacy_config: Option<&Path>,
    phase_c_config_path: &Path,
    paths: PhaseCHealthInputPaths<'_>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let config_path = resolve(workspace, phase_c_config_path);
    let loaded_phase_c = PhaseCHealthEvidenceConfig::load(&config_path)?;
    let mechanism_path = reconcile_mechanism_paths(workspace, legacy_mechanism, paths.mechanism)?;
    let mut assessment = build_legacy_assessment(
        workspace,
        signal_path,
        transient,
        calibration,
        eis,
        mechanism_path.as_deref(),
        baseline_path,
        metadata,
        legacy_config,
        output,
    )?;
    let current_context = metadata
        .map(|path| load_context(&resolve(workspace, path)))
        .transpose()?
        .unwrap_or_default();
    let inputs = load_phase_c_inputs(
        workspace,
        signal_path,
        paths,
        mechanism_path.as_deref(),
        current_context,
        assessment.configuration.comparability.clone(),
    )?;
    let eligible = crate::health::phase_c::validate_source_compatibility(&inputs, None)?;
    let bundle =
        crate::health::phase_c::prepare_phase_c_evidence(&eligible, &loaded_phase_c.config)?;
    let mut dimensions = crate::health::phase_c::evaluate_all_dimensions(
        &bundle,
        &eligible,
        &loaded_phase_c.config,
    )?;
    let mechanism_for_interpretation = mechanism_path
        .as_ref()
        .map(|path| crate::domain::read_artifact::<crate::results::MechanismAnalysisReport>(path))
        .transpose()?;
    let signal_for_mechanism: crate::results::SignalAnalysisReport =
        crate::domain::read_artifact(&resolve(workspace, signal_path))?;
    let mechanism_for_interpretation = mechanism_for_interpretation
        .filter(|report| phase_c_mechanism_is_eligible(&signal_for_mechanism, report));
    if let Some(report) = &mechanism_for_interpretation {
        let mut seen = std::collections::BTreeSet::new();
        for row in &report.hypothesis_assessments {
            if row.definition.hypothesis_id.is_empty()
                || row.definition.hypothesis_id != row.current.hypothesis_id
                || !seen.insert(row.current.hypothesis_id.clone())
            {
                return Err(crate::health::error::HealthError::InvalidEvidence {
                    source_name: "mechanism_analysis".into(),
                    field: "hypothesis_assessments".into(),
                }
                .into());
            }
        }
    }
    for dimension in &mut dimensions {
        // A Phase-B hypothesis only informs the dimension to which this
        // configuration explicitly binds it.  Passing the whole mechanism
        // report here would let a supported SignalIntegrity hypothesis alter,
        // for example, CalibrationHealth as well.
        let mapped_mechanism = mechanism_for_interpretation.as_ref().map(|report| {
            let mut mapped = report.clone();
            mapped.hypothesis_assessments.retain(|row| {
                loaded_phase_c
                    .config
                    .phase_b_hypothesis_bindings
                    .iter()
                    .any(|binding| {
                        binding.hypothesis_id == row.current.hypothesis_id
                            && binding.health_dimension == dimension.dimension
                    })
            });
            mapped
        });
        crate::health::phase_c::derive_interpretation_category(
            dimension,
            mapped_mechanism.as_ref(),
        )?;
        crate::health::phase_c::derive_causal_status(
            dimension,
            &bundle,
            mapped_mechanism.as_ref(),
            &loaded_phase_c.config,
        )?;
    }
    crate::health::phase_c::populate_consumed_artifact_ids(&mut dimensions, &bundle);
    let sources = crate::health::phase_c::consumed_lineage_sources(&eligible, &dimensions);
    let phase_c =
        crate::health::phase_c::compose_phase_c_report(&loaded_phase_c, dimensions, bundle)?;
    assessment = assemble_phase_c_assessment(assessment, phase_c, &sources)?;
    export_assessment(workspace, output, &assessment)
}

fn phase_c_mechanism_is_eligible(
    signal: &crate::results::SignalAnalysisReport,
    mechanism: &crate::results::MechanismAnalysisReport,
) -> bool {
    mechanism.schema_version == 4
        && matches!(
            (&signal.lineage, &mechanism.lineage),
            (
                crate::domain::ArtifactLineageState::Known { identity: left, .. },
                crate::domain::ArtifactLineageState::Known { identity: right, .. },
            ) if left.experiment_scope == right.experiment_scope
                && left.sensor_scope == right.sensor_scope
                && left.channel_scope == right.channel_scope
        )
}

fn assemble_phase_c_assessment(
    mut assessment: SensorHealthAssessment,
    phase_c: crate::results::PhaseCSensorHealthEvidenceReport,
    consumed_sources: &[(
        crate::domain::ArtifactLineageState,
        crate::domain::ArtifactDependencyRole,
    )],
) -> Result<SensorHealthAssessment, RunnerError> {
    let scope = lineage_scope(&assessment.lineage);
    assessment.schema_version = 4;
    assessment.overall_status = phase_c.overall_status;
    assessment.phase_c = Some(phase_c);
    assessment.lineage = derived_lineage_schema(
        crate::domain::ArtifactKind::HealthAssessment,
        &assessment,
        consumed_sources,
        scope,
        4,
    );
    Ok(assessment)
}

fn load_phase_c_inputs(
    workspace: &Path,
    signal_path: &Path,
    paths: PhaseCHealthInputPaths<'_>,
    mechanism_path: Option<&Path>,
    current_context: Context,
    comparability: crate::health_config::ComparabilityConfig,
) -> Result<crate::health::phase_c::PhaseCHealthInputs, RunnerError> {
    let signal = crate::domain::read_artifact(&resolve(workspace, signal_path))?;
    let baseline = paths
        .baseline
        .map(|path| crate::domain::read_artifact(&resolve(workspace, path)))
        .transpose()?;
    let transient = paths
        .transient
        .map(|path| crate::domain::read_artifact(&resolve(workspace, path)))
        .transpose()?;
    let calibration = paths
        .calibration
        .map(|path| crate::domain::read_artifact(&resolve(workspace, path)))
        .transpose()?;
    let estimation = paths
        .estimation
        .map(|path| crate::domain::read_artifact(&resolve(workspace, path)))
        .transpose()?;
    let model = paths
        .model
        .map(|path| crate::domain::read_artifact(&resolve(workspace, path)))
        .transpose()?;
    let mechanism = mechanism_path
        .map(crate::domain::read_artifact)
        .transpose()?;
    let catalog = paths
        .lineage_catalog
        .map(|path| load_lineage_catalog(&resolve(workspace, path)))
        .transpose()?;
    Ok(crate::health::phase_c::assemble_phase_c_inputs(
        signal,
        baseline,
        transient,
        calibration,
        estimation,
        model,
        mechanism,
        catalog,
        current_context,
        comparability,
    ))
}

fn load_lineage_catalog(path: &Path) -> Result<crate::domain::ArtifactLineageCatalog, RunnerError> {
    let catalog: crate::domain::ArtifactLineageCatalog = serde_json::from_slice(&fs::read(path)?)
        .map_err(|error| {
        crate::health::error::HealthError::LineageCatalogInvalid {
            message: error.to_string(),
        }
    })?;
    if catalog.schema_version != 1 {
        return Err(crate::health::error::HealthError::LineageCatalogInvalid {
            message: "catalog schema must be 1".into(),
        }
        .into());
    }
    let mut validated = crate::domain::ArtifactLineageCatalog::default();
    for (key, node) in catalog.artifacts {
        if key != node.identity.artifact_id {
            return Err(crate::health::error::HealthError::LineageCatalogInvalid {
                message: "catalog map key does not match node identity".into(),
            }
            .into());
        }
        validated.insert(node).map_err(|error| {
            crate::health::error::HealthError::LineageCatalogInvalid {
                message: error.to_string(),
            }
        })?;
    }
    Ok(validated)
}

fn reconcile_mechanism_paths(
    workspace: &Path,
    legacy: Option<&Path>,
    phase_c: Option<&Path>,
) -> Result<Option<PathBuf>, RunnerError> {
    match (legacy, phase_c) {
        (Some(left), Some(right)) => {
            let left_path = resolve(workspace, left);
            let right_path = resolve(workspace, right);
            let left_report: crate::results::MechanismAnalysisReport =
                crate::domain::read_artifact(&left_path)?;
            let right_report: crate::results::MechanismAnalysisReport =
                crate::domain::read_artifact(&right_path)?;
            let left_id = match &left_report.lineage {
                crate::domain::ArtifactLineageState::Known { identity, .. } => {
                    Some(identity.artifact_id.clone())
                }
                _ => None,
            };
            let right_id = match &right_report.lineage {
                crate::domain::ArtifactLineageState::Known { identity, .. } => {
                    Some(identity.artifact_id.clone())
                }
                _ => None,
            };
            if left_id.is_none() || left_id != right_id {
                return Err(
                    crate::health::error::HealthError::ConflictingEvidenceInput {
                        left: left_path.display().to_string(),
                        right: right_path.display().to_string(),
                    }
                    .into(),
                );
            }
            Ok(Some(left_path))
        }
        (Some(path), None) | (None, Some(path)) => Ok(Some(resolve(workspace, path))),
        (None, None) => Ok(None),
    }
}

pub fn trend(
    workspace: &Path,
    manifest: &Path,
    baseline_path: Option<&Path>,
    config_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let loaded = LoadedHealthConfig::load(workspace, config_path)?;
    let path = resolve(workspace, manifest);
    let man: HealthTrendManifest = read_toml(&path)?;
    if man.schema_version != 1 {
        return Err(RunnerError::Message(
            "unsupported health trend manifest schema".into(),
        ));
    }
    let base = path.parent().unwrap_or(workspace);
    let baseline = baseline_path
        .map(|p| crate::domain::read_artifact::<SensorHealthBaseline>(&resolve(workspace, p)))
        .transpose()?;
    let mut lineage_sources = baseline
        .as_ref()
        .map(|artifact| {
            vec![(
                artifact.lineage.clone(),
                crate::domain::ArtifactDependencyRole::Prior,
            )]
        })
        .unwrap_or_default();
    let mut all =
        std::collections::BTreeMap::<String, Vec<(String, Option<f64>, Option<f64>)>>::new();
    let mut provenance = None;
    for r in man.records {
        let a: SensorHealthAssessment =
            crate::domain::read_artifact(&resolve(base, &r.assessment))?;
        lineage_sources.push((
            a.lineage.clone(),
            crate::domain::ArtifactDependencyRole::DerivedFrom,
        ));
        provenance.get_or_insert(a.provenance.clone());
        for f in a.features {
            all.entry(f.name.clone()).or_default().push((
                r.record_id.clone(),
                f.value,
                r.independent_value,
            ));
        }
    }
    let mut trends = Vec::new();
    for (name, points) in all {
        let b = baseline.as_ref().and_then(|x| {
            x.feature_distributions
                .iter()
                .find(|f| f.feature == name)
                .and_then(|f| f.mean)
        });
        trends.push(health::trend::calculate(&name, points, b));
    }
    let p =
        provenance.ok_or_else(|| RunnerError::Message("health trend manifest is empty".into()))?;
    let mut report = health::trend::report("health-trend", trends, p);
    let fallback_scope = lineage_scope(&report.lineage);
    report.lineage = derived_lineage(
        crate::domain::ArtifactKind::HealthTrend,
        &report,
        &lineage_sources,
        fallback_scope,
    );
    let dir = output_dir(workspace, output, "health_trend");
    fs::create_dir_all(&dir)?;
    crate::domain::write_artifact(&dir.join(&loaded.config.export.trends_filename), &report)?;
    let mut w = csv::Writer::from_path(dir.join("health_trends.csv"))?;
    w.write_record([
        "feature",
        "record_id",
        "independent_value",
        "value",
        "absolute_change",
        "relative_change",
        "log_change",
    ])?;
    for t in &report.trends {
        for p in &t.points {
            w.write_record([
                t.feature.clone(),
                p.record_id.clone(),
                fmt(p.independent_value),
                fmt(p.value),
                fmt(p.absolute_change),
                fmt(p.relative_change),
                fmt(p.log_change),
            ])?;
        }
    }
    println!("Health trend written to {}", dir.display());
    Ok(())
}
pub fn report(workspace: &Path, results: &Path, output: Option<&Path>) -> Result<(), RunnerError> {
    let r: SensorHealthAssessment = crate::domain::read_artifact(&resolve(workspace, results))?;
    let dest = output_file(workspace, output, "health_report.txt");
    fs::write(&dest, human_report(&r))?;
    println!("Health report written to {}", dest.display());
    Ok(())
}
fn export_legacy_assessment(
    workspace: &Path,
    output: Option<&Path>,
    r: &SensorHealthAssessment,
) -> Result<(), RunnerError> {
    let dir = output_dir(workspace, output, "health");
    fs::create_dir_all(&dir)?;
    crate::domain::write_legacy_sensor_health_assessment_v3(
        &dir.join(&r.configuration.export.assessment_filename),
        r,
    )?;
    write_assessment_sidecars(&dir, r)
}

fn export_assessment(
    workspace: &Path,
    output: Option<&Path>,
    r: &SensorHealthAssessment,
) -> Result<(), RunnerError> {
    let dir = output_dir(workspace, output, "health");
    fs::create_dir_all(&dir)?;
    let c = &r.configuration.export;
    crate::domain::write_artifact(&dir.join(&c.assessment_filename), r)?;
    write_assessment_sidecars(&dir, r)
}

fn write_assessment_sidecars(dir: &Path, r: &SensorHealthAssessment) -> Result<(), RunnerError> {
    let c = &r.configuration.export;
    let mut f = csv::Writer::from_path(dir.join(&c.features_filename))?;
    f.write_record(["feature", "domain", "value", "unit", "source"])?;
    for x in &r.features {
        f.write_record([
            x.name.clone(),
            format!("{:?}", x.domain),
            fmt(x.value),
            x.unit.clone(),
            x.source.clone(),
        ])?;
    }
    let mut w = csv::Writer::from_path(dir.join(&c.findings_filename))?;
    w.write_record([
        "finding",
        "severity",
        "confidence",
        "triggered_rules",
        "supporting_domains",
        "alternatives",
    ])?;
    for x in &r.findings {
        w.write_record([
            format!("{:?}", x.finding),
            format!("{:?}", x.severity),
            format!("{:?}", x.confidence),
            x.triggered_rules.join(";"),
            x.supporting_evidence
                .iter()
                .map(|e| format!("{:?}", e.domain))
                .collect::<Vec<_>>()
                .join(";"),
            x.alternative_explanations.join(";"),
        ])?;
    }
    fs::write(dir.join(&c.report_filename), human_report(r))?;
    if r.configuration.plotting.enabled {
        crate::plottings::health_plot::plot_health_assessment(r, dir)?;
    }
    println!("Health assessment written to {}", dir.display());
    Ok(())
}

fn lineage_scope(
    lineage: &crate::domain::ArtifactLineageState,
) -> crate::domain::ArtifactExperimentScope {
    match lineage {
        crate::domain::ArtifactLineageState::Known { identity, .. } => {
            identity.experiment_scope.clone()
        }
        crate::domain::ArtifactLineageState::LegacyUnknown { .. } => {
            crate::domain::ArtifactExperimentScope::Unknown
        }
    }
}

fn derived_lineage<T: Serialize>(
    artifact_kind: crate::domain::ArtifactKind,
    artifact: &T,
    sources: &[(
        crate::domain::ArtifactLineageState,
        crate::domain::ArtifactDependencyRole,
    )],
    fallback_scope: crate::domain::ArtifactExperimentScope,
) -> crate::domain::ArtifactLineageState {
    derived_lineage_schema(artifact_kind, artifact, sources, fallback_scope, 3)
}

fn derived_lineage_schema<T: Serialize>(
    artifact_kind: crate::domain::ArtifactKind,
    artifact: &T,
    sources: &[(
        crate::domain::ArtifactLineageState,
        crate::domain::ArtifactDependencyRole,
    )],
    fallback_scope: crate::domain::ArtifactExperimentScope,
    schema_version: u32,
) -> crate::domain::ArtifactLineageState {
    let (source_scope, acquisition_families) = crate::domain::lineage_scope_and_families(
        match artifact_kind {
            crate::domain::ArtifactKind::HealthBaseline => "health-baseline-v1",
            crate::domain::ArtifactKind::HealthAssessment => "health-assessment-v1",
            crate::domain::ArtifactKind::HealthTrend => "health-trend-v1",
            _ => return crate::domain::current_unknown_lineage(3),
        },
        sources.iter().map(|(lineage, _)| lineage),
    );
    let experiment_scope = match source_scope {
        crate::domain::ArtifactExperimentScope::Unknown => fallback_scope,
        scope => scope,
    };
    let dependencies = sources
        .iter()
        .filter_map(|(lineage, role)| crate::domain::dependency_from_lineage(lineage, role.clone()))
        .collect::<Vec<_>>();
    crate::domain::known_lineage_from_artifact(
        artifact_kind,
        schema_version,
        format!("rust_electroanalysis_cli@{}", env!("CARGO_PKG_VERSION")),
        experiment_scope,
        crate::domain::ScopeKey::Unspecified,
        crate::domain::ScopeKey::Unspecified,
        acquisition_families,
        dependencies,
        artifact,
    )
    .unwrap_or_else(|_| crate::domain::current_unknown_lineage(schema_version))
}
fn human_report(r: &SensorHealthAssessment) -> String {
    let mut s = format!(
        "Sensor health assessment\n=========================\nStatus: {:?}\nFeatures: {}\nFindings: {}\nMissing domains: {:?}\nWarnings: {:?}\n",
        r.overall_status,
        r.features.len(),
        r.findings.len(),
        r.missing_domains,
        r.warnings
    );
    for f in &r.findings {
        s.push_str(&format!("Finding {:?} severity {:?}, confidence {:?}; supporting domains: {:?}; alternatives: {:?}\n",f.finding,f.severity,f.confidence,f.supporting_evidence.iter().map(|e|e.domain).collect::<Vec<_>>(),f.alternative_explanations));
    }
    s
}
#[derive(Debug, Clone, Deserialize)]
struct HealthTrendManifest {
    pub schema_version: u32,
    pub records: Vec<HealthTrendRecord>,
}
#[derive(Debug, Clone, Deserialize)]
struct HealthTrendRecord {
    pub record_id: String,
    pub assessment: PathBuf,
    #[serde(default)]
    pub independent_value: Option<f64>,
}
fn load_context(p: &Path) -> Result<Context, RunnerError> {
    let d = crate::domain::load_experiment_metadata(p)?;
    let temperature_values_k = d
        .environmental_data
        .iter()
        .filter(|series| series.name.to_ascii_lowercase().contains("temp"))
        .flat_map(|series| {
            let celsius =
                series.unit.eq_ignore_ascii_case("c") || series.unit.eq_ignore_ascii_case("°c");
            series
                .values
                .iter()
                .flatten()
                .copied()
                .filter(|value| value.is_finite())
                .map(move |value| if celsius { value + 273.15 } else { value })
        })
        .collect::<Vec<_>>();
    let temperature_k = if temperature_values_k.is_empty() {
        None
    } else {
        Some(temperature_values_k.iter().sum::<f64>() / temperature_values_k.len() as f64)
    };
    Ok(Context {
        sensor_id: d.sensor.sensor_id,
        sensor_type: d.sensor.sensor_type.clone(),
        sensor_design: d.sensor.model.or(d.sensor.sensor_type),
        analyte: d.sensor.analyte,
        sample_matrix: Some(d.sample_matrix),
        temperature_k,
        temperature_values_k,
        experiment_id: Some(d.experiment_id),
        metadata_source: Some(p.display().to_string()),
    })
}
fn read_toml<T: DeserializeOwned>(p: &Path) -> Result<T, RunnerError> {
    Ok(toml::from_str(&fs::read_to_string(p)?)?)
}
fn resolve(w: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        w.join(p)
    }
}
fn output_dir(w: &Path, p: Option<&Path>, d: &str) -> PathBuf {
    p.map(|x| resolve(w, x))
        .unwrap_or_else(|| w.join("output").join(d))
}
fn output_file(w: &Path, p: Option<&Path>, d: &str) -> PathBuf {
    let p = p
        .map(|x| resolve(w, x))
        .unwrap_or_else(|| w.join("output").join("health"));
    if p.extension().is_some() {
        p
    } else {
        p.join(d)
    }
}
fn fmt(v: Option<f64>) -> String {
    v.filter(|x| x.is_finite())
        .map(|x| format!("{x:.12e}"))
        .unwrap_or_default()
}

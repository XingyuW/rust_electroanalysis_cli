//! Deterministic Phase-D CSV table writers.

use crate::{
    domain::ArtifactLineageState,
    reporting::{AvailabilityReason, PublicReportError, projection::PublicReportProjection},
};
use csv::{Terminator, WriterBuilder};
use serde::Serialize;
use std::{fs::File, path::Path};

pub fn format_public_f64(value: f64) -> Result<String, &'static str> {
    if !value.is_finite() {
        return Err("non-finite number in public projection");
    }
    if value == 0.0 {
        Ok("0".into())
    } else {
        Ok(value.to_string())
    }
}

pub fn write_selected_tables(
    root: &Path,
    projection: &PublicReportProjection,
    selected: &[crate::report_config::TableId],
) -> Result<Vec<String>, PublicReportError> {
    let table_dir = root.join("tables");
    std::fs::create_dir_all(&table_dir).map_err(|source| PublicReportError::Write {
        path: table_dir.clone(),
        source,
    })?;
    let mut paths = Vec::new();
    for id in selected {
        let filename = match id {
            crate::report_config::TableId::MechanismEvidence => "mechanism_evidence.csv",
            crate::report_config::TableId::HealthDimensions => "sensor_health_dimensions.csv",
            crate::report_config::TableId::EvidenceProvenance => "evidence_provenance.csv",
            crate::report_config::TableId::ArtifactLineage => "artifact_lineage.csv",
            crate::report_config::TableId::TimescaleComparison => "timescale_comparison.csv",
            crate::report_config::TableId::ModelConsistency => "model_consistency.csv",
            crate::report_config::TableId::CurrentVsBaseline => "current_vs_baseline.csv",
        };
        let path = table_dir.join(filename);
        match id {
            crate::report_config::TableId::MechanismEvidence => {
                mechanism_evidence(&path, projection)?
            }
            crate::report_config::TableId::HealthDimensions => {
                health_dimensions(&path, projection)?
            }
            crate::report_config::TableId::EvidenceProvenance => {
                evidence_provenance(&path, projection)?
            }
            crate::report_config::TableId::ArtifactLineage => artifact_lineage(&path, projection)?,
            crate::report_config::TableId::TimescaleComparison => {
                timescale_comparison(&path, projection)?
            }
            crate::report_config::TableId::ModelConsistency => {
                model_consistency(&path, projection)?
            }
            crate::report_config::TableId::CurrentVsBaseline => {
                current_vs_baseline(&path, projection)?
            }
        }
        paths.push(format!("tables/{filename}"));
    }
    Ok(paths)
}

fn writer(path: &Path) -> Result<csv::Writer<File>, PublicReportError> {
    let file = File::create(path).map_err(|source| PublicReportError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(WriterBuilder::new()
        .terminator(Terminator::Any(b'\n'))
        .from_writer(file))
}

fn csv_error(path: &Path, source: csv::Error) -> PublicReportError {
    PublicReportError::Csv {
        path: path.to_path_buf(),
        source,
    }
}
fn number(value: Option<f64>) -> Result<String, PublicReportError> {
    value
        .map(format_public_f64)
        .transpose()
        .map_err(|detail| PublicReportError::StagingValidation {
            path: Path::new("public report projection").to_path_buf(),
            detail: detail.into(),
        })
        .map(|value| value.unwrap_or_else(|| "NA".into()))
}
fn token<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}
fn joined<T: Serialize>(values: &[T]) -> String {
    if values.is_empty() {
        "[]".into()
    } else {
        values.iter().map(token).collect::<Vec<_>>().join(";")
    }
}

fn mechanism_evidence(
    path: &Path,
    projection: &PublicReportProjection,
) -> Result<(), PublicReportError> {
    let mut writer = writer(path)?;
    writer
        .write_record([
            "hypothesis_id",
            "display_name",
            "evidence_level",
            "reason_codes",
            "validation_status",
            "temporal_statuses",
            "timescale_statuses",
            "amplitude_statuses",
            "repeatability_statuses",
            "identifiability_statuses",
            "contradiction_requirement_ids",
            "component_ids",
            "history_ids",
            "legacy_status",
        ])
        .map_err(|source| csv_error(path, source))?;
    if projection.mechanism_is_legacy() {
        for row in &projection.mechanism.legacy_hypotheses {
            writer
                .write_record([
                    row.hypothesis_id.as_str(),
                    "NA",
                    "NA",
                    "[]",
                    "NA",
                    "[]",
                    "[]",
                    "[]",
                    "[]",
                    "[]",
                    "[]",
                    "[]",
                    "[]",
                    "phase_b_v1_not_serialized",
                ])
                .map_err(|source| csv_error(path, source))?;
        }
    } else {
        let mut rows = projection
            .mechanism
            .hypothesis_assessments
            .iter()
            .collect::<Vec<_>>();
        rows.sort_by(|a, b| a.definition.hypothesis_id.cmp(&b.definition.hypothesis_id));
        for row in rows {
            let current = &row.current;
            let temporal = current
                .temporal_join_assessments
                .iter()
                .map(|item| token(&item.outcome))
                .collect::<Vec<_>>()
                .join(";");
            let timescale = current
                .timescale_assessments
                .iter()
                .map(|item| token(&item.status))
                .collect::<Vec<_>>()
                .join(";");
            let amplitude = current
                .amplitude_assessments
                .iter()
                .map(|item| token(&item.status))
                .collect::<Vec<_>>()
                .join(";");
            let repeatability = current
                .repeatability_assessments
                .iter()
                .map(|item| token(&item.status))
                .collect::<Vec<_>>()
                .join(";");
            let identifiability = current
                .identifiability_assessments
                .iter()
                .map(|item| token(&item.status))
                .collect::<Vec<_>>()
                .join(";");
            let contradictions = current
                .contradiction_summaries
                .iter()
                .map(|item| item.requirement_id.as_str())
                .collect::<Vec<_>>()
                .join(";");
            let history = current
                .history
                .iter()
                .map(|item| item.history_id.as_str())
                .collect::<Vec<_>>()
                .join(";");
            writer
                .write_record([
                    current.hypothesis_id.as_str(),
                    row.definition.display_name.as_str(),
                    token(&current.evidence_level).as_str(),
                    joined(&current.reason_codes).as_str(),
                    token(&current.validation_status).as_str(),
                    cell(&temporal).as_str(),
                    cell(&timescale).as_str(),
                    cell(&amplitude).as_str(),
                    cell(&repeatability).as_str(),
                    cell(&identifiability).as_str(),
                    cell(&contradictions).as_str(),
                    joined(&row.definition.target_components).as_str(),
                    cell(&history).as_str(),
                    "current",
                ])
                .map_err(|source| csv_error(path, source))?;
        }
    }
    writer.flush().map_err(|source| PublicReportError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn health_dimensions(
    path: &Path,
    projection: &PublicReportProjection,
) -> Result<(), PublicReportError> {
    let mut writer = writer(path)?;
    writer
        .write_record([
            "dimension",
            "display_label",
            "status",
            "evidence_state",
            "reason_codes",
            "interpretation_category",
            "causal_status",
            "source_evidence_ids",
            "excluded_evidence_ids",
            "source_artifact_ids",
            "legacy_status",
        ])
        .map_err(|source| csv_error(path, source))?;
    match &projection.health.phase_c {
        Some(report) => {
            for row in &report.dimension_assessments {
                writer
                    .write_record([
                        token(&row.dimension).as_str(),
                        crate::reporting::claims::health_status_text(row.status),
                        token(&row.status).as_str(),
                        token(&row.evidence_state).as_str(),
                        joined(&row.reason_codes).as_str(),
                        token(&row.interpretation_category).as_str(),
                        token(&row.causal_status).as_str(),
                        row.source_evidence_ids
                            .iter()
                            .map(|id| id.0.as_str())
                            .collect::<Vec<_>>()
                            .join(";")
                            .as_str(),
                        row.excluded_evidence_ids
                            .iter()
                            .map(|id| id.0.as_str())
                            .collect::<Vec<_>>()
                            .join(";")
                            .as_str(),
                        row.source_artifact_ids
                            .iter()
                            .map(|id| id.0.as_str())
                            .collect::<Vec<_>>()
                            .join(";")
                            .as_str(),
                        "current",
                    ])
                    .map_err(|source| csv_error(path, source))?;
            }
        }
        None => writer
            .write_record([
                "NA",
                "NA",
                "NA",
                "NA",
                "[]",
                "NA",
                "NA",
                "[]",
                "[]",
                "[]",
                "legacy_phase_c_not_serialized",
            ])
            .map_err(|source| csv_error(path, source))?,
    }
    writer.flush().map_err(|source| PublicReportError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn evidence_provenance(
    path: &Path,
    projection: &PublicReportProjection,
) -> Result<(), PublicReportError> {
    let mut writer = writer(path)?;
    writer
        .write_record([
            "assessment_scope",
            "evidence_id",
            "target",
            "source_class",
            "direction",
            "availability",
            "validity",
            "quantity_value",
            "quantity_unit",
            "uncertainty",
            "source_artifact_kind",
            "source_artifact_id_or_fingerprint",
            "source_field_path",
            "experiment_scope",
            "acquisition_families",
            "temporal_support",
        ])
        .map_err(|source| csv_error(path, source))?;
    if let Some(phase_c) = &projection.health.phase_c {
        for record in &phase_c.evidence_bundle.records {
            let (kind, id) = match &record.source.artifact {
                crate::evidence::EvidenceArtifactSource::Known {
                    artifact_id,
                    artifact_kind,
                } => (artifact_kind.as_str().to_string(), artifact_id.0.clone()),
                crate::evidence::EvidenceArtifactSource::LegacyUnknown {
                    artifact_kind,
                    source_fingerprint,
                } => (
                    artifact_kind.as_str().to_string(),
                    source_fingerprint.0.clone(),
                ),
            };
            writer
                .write_record([
                    "sensor_health",
                    record.evidence_id.0.as_str(),
                    token(&record.target).as_str(),
                    token(&record.source_class).as_str(),
                    token(&record.direction).as_str(),
                    token(&record.availability).as_str(),
                    token(&record.validity).as_str(),
                    number(record.quantity.as_ref().map(|value| value.value))?.as_str(),
                    record
                        .quantity
                        .as_ref()
                        .map(|value| value.unit.as_str())
                        .unwrap_or("NA"),
                    record
                        .quantity
                        .as_ref()
                        .and_then(|value| value.uncertainty.as_ref())
                        .map(token)
                        .unwrap_or_else(|| "NA".into())
                        .as_str(),
                    kind.as_str(),
                    id.as_str(),
                    record.source.field_path.as_str(),
                    token(&record.experiment_scope).as_str(),
                    "NA",
                    "NA",
                ])
                .map_err(|source| csv_error(path, source))?;
        }
    }
    writer.flush().map_err(|source| PublicReportError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn artifact_lineage(
    path: &Path,
    projection: &PublicReportProjection,
) -> Result<(), PublicReportError> {
    let mut writer = writer(path)?;
    writer
        .write_record([
            "root_input_flag",
            "row_kind",
            "root_artifact_kind",
            "root_artifact_id",
            "lineage_state",
            "direct_dependency_role",
            "direct_dependency_kind",
            "direct_dependency_id",
            "catalog_supplied",
            "root_catalog_entry_present",
        ])
        .map_err(|source| csv_error(path, source))?;
    let catalog_supplied = projection.lineage_catalog.is_some().to_string();
    for (flag, lineage) in projection.supplied_lineages() {
        match lineage {
            ArtifactLineageState::Known {
                identity,
                direct_dependencies,
            } => {
                let member = projection
                    .lineage_catalog
                    .as_ref()
                    .map(|catalog| {
                        catalog
                            .artifacts
                            .contains_key(&identity.artifact_id)
                            .to_string()
                    })
                    .unwrap_or_else(|| "NA".into());
                writer
                    .write_record([
                        flag,
                        "root",
                        identity.artifact_kind.as_str(),
                        identity.artifact_id.0.as_str(),
                        "known",
                        "NA",
                        "NA",
                        "NA",
                        catalog_supplied.as_str(),
                        member.as_str(),
                    ])
                    .map_err(|source| csv_error(path, source))?;
                for dependency in direct_dependencies {
                    writer
                        .write_record([
                            flag,
                            "direct_dependency",
                            "NA",
                            "NA",
                            "known",
                            token(&dependency.role).as_str(),
                            dependency.artifact_kind.as_str(),
                            dependency.artifact_id.0.as_str(),
                            catalog_supplied.as_str(),
                            "NA",
                        ])
                        .map_err(|source| csv_error(path, source))?;
                }
            }
            ArtifactLineageState::LegacyUnknown { .. } => writer
                .write_record([
                    flag,
                    "root",
                    "NA",
                    "NA",
                    "legacy_unknown",
                    "NA",
                    "NA",
                    "NA",
                    catalog_supplied.as_str(),
                    "NA",
                ])
                .map_err(|source| csv_error(path, source))?,
        }
    }
    writer.flush().map_err(|source| PublicReportError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn timescale_comparison(
    path: &Path,
    projection: &PublicReportProjection,
) -> Result<(), PublicReportError> {
    let mut writer = writer(path)?;
    writer
        .write_record([
            "comparison_id",
            "record_id",
            "eis_timescale_id",
            "eis_value_s",
            "eis_standard_error_s",
            "transient_timescale_id",
            "transient_value_s",
            "transient_standard_error_s",
            "ratio",
            "log10_distance",
            "symmetric_relative_difference",
            "confidence_interval_overlap",
            "compatibility_probability",
            "evidence_level",
            "supporting_evidence",
            "contradictory_evidence",
            "alternative_explanations",
            "warnings",
        ])
        .map_err(|source| csv_error(path, source))?;
    let mut rows = projection.mechanism.comparisons.iter().collect::<Vec<_>>();
    rows.sort_by(|a, b| a.comparison_id.cmp(&b.comparison_id));
    for row in rows {
        let eis = projection
            .mechanism
            .eis_timescales
            .iter()
            .find(|value| value.timescale_id == row.eis_timescale_id);
        let transient = projection
            .mechanism
            .transient_timescales
            .iter()
            .find(|value| value.timescale_id == row.transient_timescale_id);
        writer
            .write_record([
                row.comparison_id.as_str(),
                row.record_id.as_str(),
                row.eis_timescale_id.as_str(),
                number(eis.map(|v| v.value_s))?.as_str(),
                number(eis.and_then(|v| v.standard_error_s))?.as_str(),
                row.transient_timescale_id.as_str(),
                number(transient.map(|v| v.value_s))?.as_str(),
                number(transient.and_then(|v| v.standard_error_s))?.as_str(),
                number(row.ratio)?.as_str(),
                number(row.log10_distance)?.as_str(),
                number(row.symmetric_relative_difference)?.as_str(),
                row.confidence_interval_overlap
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "NA".into())
                    .as_str(),
                number(row.compatibility_probability)?.as_str(),
                token(&row.evidence_level).as_str(),
                cell(&row.supporting_evidence.join(";")).as_str(),
                cell(&row.contradictory_evidence.join(";")).as_str(),
                cell(&row.alternative_explanations.join(";")).as_str(),
                row.warnings
                    .iter()
                    .map(|w| w.message.as_str())
                    .collect::<Vec<_>>()
                    .join(";")
                    .as_str(),
            ])
            .map_err(|source| csv_error(path, source))?;
    }
    writer.flush().map_err(|source| PublicReportError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn model_consistency(
    path: &Path,
    projection: &PublicReportProjection,
) -> Result<(), PublicReportError> {
    let mut writer = writer(path)?;
    writer
        .write_record([
            "availability",
            "time_s",
            "observed_voltage_v",
            "predicted_voltage_v",
            "unexplained_residual_v",
            "uncertainty_status",
            "validity_status",
            "equilibrium_status",
        ])
        .map_err(|source| csv_error(path, source))?;
    let Some(model) = &projection.model else {
        writer
            .write_record(["not_provided", "NA", "NA", "NA", "NA", "NA", "NA", "NA"])
            .map_err(|source| csv_error(path, source))?;
        return writer.flush().map_err(|source| PublicReportError::Write {
            path: path.to_path_buf(),
            source,
        });
    };
    let mut rows = model.points.iter().enumerate().collect::<Vec<_>>();
    rows.sort_by(|(i, a), (j, b)| a.time_s.total_cmp(&b.time_s).then(i.cmp(j)));
    for (_, row) in rows {
        writer
            .write_record([
                "available",
                format_public_f64(row.time_s)
                    .map_err(|detail| PublicReportError::StagingValidation {
                        path: path.to_path_buf(),
                        detail: detail.into(),
                    })?
                    .as_str(),
                number(row.observed_voltage_v)?.as_str(),
                format_public_f64(row.predicted_voltage_v)
                    .map_err(|detail| PublicReportError::StagingValidation {
                        path: path.to_path_buf(),
                        detail: detail.into(),
                    })?
                    .as_str(),
                number(row.unexplained_residual_v)?.as_str(),
                token(&row.uncertainty.status).as_str(),
                if row.validity.is_valid {
                    "valid"
                } else {
                    "invalid"
                },
                token(&row.equilibrium.status).as_str(),
            ])
            .map_err(|source| csv_error(path, source))?;
    }
    writer.flush().map_err(|source| PublicReportError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn current_vs_baseline(
    path: &Path,
    projection: &PublicReportProjection,
) -> Result<(), PublicReportError> {
    let mut writer = writer(path)?;
    writer
        .write_record([
            "availability",
            "feature",
            "unit",
            "current_value",
            "baseline_value",
            "comparability",
            "absolute_difference",
            "relative_difference",
            "log_ratio",
            "z_score",
            "robust_z_score",
            "empirical_percentile",
            "baseline_sample_count",
            "override_reason",
            "warnings",
        ])
        .map_err(|source| csv_error(path, source))?;
    let mut rows = projection
        .health
        .baseline_comparison
        .iter()
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.feature.cmp(&b.feature));
    if rows.is_empty() {
        writer
            .write_record([
                "not_serialized",
                "NA",
                "NA",
                "NA",
                "NA",
                "NA",
                "NA",
                "NA",
                "NA",
                "NA",
                "NA",
                "NA",
                "NA",
                "NA",
                "[]",
            ])
            .map_err(|source| csv_error(path, source))?;
    }
    for row in rows {
        let units = projection
            .health
            .features
            .iter()
            .filter(|feature| feature.name == row.feature && !feature.unit.is_empty())
            .collect::<Vec<_>>();
        let reason = match row.comparability {
            // Unit authority is a separate required presentation boundary.
            // Never infer a unit from another feature, even if upstream
            // comparability is also unavailable.
            _ if units.len() != 1 => Some(AvailabilityReason::UnitAuthorityUnavailable),
            crate::results::FeatureComparability::NotComparable => {
                Some(AvailabilityReason::NotComparable)
            }
            crate::results::FeatureComparability::Unknown => {
                Some(AvailabilityReason::ComparisonUnknown)
            }
            _ if !row.current_value.is_some_and(f64::is_finite)
                || !row.baseline_value.is_some_and(f64::is_finite) =>
            {
                Some(AvailabilityReason::NoComparableFinitePair)
            }
            _ => None,
        };
        if let Some(reason) = reason {
            writer
                .write_record([
                    token(&reason).as_str(),
                    row.feature.as_str(),
                    "NA",
                    "NA",
                    "NA",
                    token(&row.comparability).as_str(),
                    "NA",
                    "NA",
                    "NA",
                    "NA",
                    "NA",
                    "NA",
                    "NA",
                    row.override_reason.as_deref().unwrap_or("NA"),
                    "[]",
                ])
                .map_err(|source| csv_error(path, source))?;
        } else {
            let warning = matches!(
                row.comparability,
                crate::results::FeatureComparability::ComparableWithWarnings
            )
            .then_some("baseline_comparable_with_warnings")
            .unwrap_or("[]");
            writer
                .write_record([
                    "available",
                    row.feature.as_str(),
                    units[0].unit.as_str(),
                    number(row.current_value)?.as_str(),
                    number(row.baseline_value)?.as_str(),
                    token(&row.comparability).as_str(),
                    number(row.absolute_difference)?.as_str(),
                    number(row.relative_difference)?.as_str(),
                    number(row.log_ratio)?.as_str(),
                    number(row.z_score)?.as_str(),
                    number(row.robust_z_score)?.as_str(),
                    number(row.empirical_percentile)?.as_str(),
                    row.baseline_sample_count.to_string().as_str(),
                    row.override_reason.as_deref().unwrap_or("NA"),
                    warning,
                ])
                .map_err(|source| csv_error(path, source))?;
        }
    }
    writer.flush().map_err(|source| PublicReportError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn cell(value: &str) -> String {
    if value.is_empty() {
        "[]".into()
    } else {
        value.to_string()
    }
}

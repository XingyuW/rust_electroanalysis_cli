//! Deterministic JSON and Markdown public documents.

use crate::reporting::{
    claims::{
        REQUIRED_DISCLAIMER, causal_status_text, evidence_state_text, health_status_text,
        mechanism_level_text, unavailable_text,
    },
    projection::PublicReportProjection,
};
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Serialize)]
struct Summary<'a> {
    schema_version: u32,
    output_kind: &'static str,
    renderer_contract: &'static str,
    route: &'static str,
    input_references: Vec<InputReference>,
    compatibility: Compatibility,
    mechanism: Mechanism<'a>,
    sensor_health: Health<'a>,
    optional_sources: Vec<OptionalSource>,
    lineage: Lineage,
    outputs: Outputs,
    limitations: Vec<Limitation>,
    rendering: Rendering,
}

#[derive(Serialize)]
#[serde(tag = "input_kind", rename_all = "snake_case")]
enum InputReference {
    Artifact {
        input_flag: &'static str,
        supplied_path_basename: Option<String>,
        artifact_kind: Option<crate::domain::ArtifactKind>,
        schema_version: Option<u32>,
        availability: &'static str,
    },
    LineageCatalog {
        supplied_path_basename: Option<String>,
        schema_version: Option<u32>,
        availability: &'static str,
        validation: &'static str,
    },
}
#[derive(Serialize)]
struct Compatibility {
    required_pair: &'static str,
    optional: Vec<CompatibilityRecord>,
}
#[derive(Serialize)]
struct CompatibilityRecord {
    input_flag: &'static str,
    against_flag: &'static str,
    status: &'static str,
    mismatch_axis: Option<&'static str>,
}
#[derive(Serialize)]
struct Mechanism<'a> {
    availability: &'static str,
    analysis_id: &'a str,
    hypothesis_count: usize,
    comparison_count: usize,
    legacy_phase_b_assessment: bool,
}
#[derive(Serialize)]
struct Health<'a> {
    availability: &'static str,
    assessment_id: &'a str,
    overall_status: crate::results::OverallHealthStatus,
    dimension_count: usize,
    legacy_phase_c_assessment: bool,
}
#[derive(Serialize)]
struct OptionalSource {
    kind: &'static str,
    availability: &'static str,
}
#[derive(Serialize)]
struct Lineage {
    catalog_supplied: bool,
    root_count: usize,
}
#[derive(Serialize)]
struct Outputs {
    tables: Vec<&'static str>,
    figures: Vec<&'static str>,
}
#[derive(Serialize)]
struct Limitation {
    code: &'static str,
    message: String,
}
#[derive(Serialize)]
struct Rendering {
    json_schema: &'static str,
    numeric_format: &'static str,
    csv_newline: &'static str,
    timestamp: Option<String>,
}

pub fn write_public_summary_json(
    root: &Path,
    projection: &PublicReportProjection,
    table_ids: &[crate::report_config::TableId],
    figure_ids: &[crate::report_config::FigureId],
) -> Result<String, crate::reporting::PublicReportError> {
    let path = root.join("public_summary.schema1.json");
    let summary = Summary {
        schema_version: 1,
        output_kind: "phase_d_public_scientific_output",
        renderer_contract: "mhi_v1_phase_d_public_output_v1",
        route: "electroanalysis report render",
        input_references: input_references(projection),
        compatibility: Compatibility {
            required_pair: compatibility_token(projection.required_compatibility.status),
            optional: projection
                .optional_compatibility
                .iter()
                .map(|(input, against, outcome)| CompatibilityRecord {
                    input_flag: trim_flag(input),
                    against_flag: trim_flag(against),
                    status: compatibility_token(outcome.status),
                    mismatch_axis: None,
                })
                .collect(),
        },
        mechanism: Mechanism {
            availability: "available",
            analysis_id: &projection.mechanism.analysis_id,
            hypothesis_count: projection.mechanism.hypothesis_assessments.len(),
            comparison_count: projection.mechanism.comparisons.len(),
            legacy_phase_b_assessment: projection.mechanism_is_legacy(),
        },
        sensor_health: Health {
            availability: "available",
            assessment_id: &projection.health.assessment_id,
            overall_status: projection.health.overall_status,
            dimension_count: projection
                .health
                .phase_c
                .as_ref()
                .map_or(0, |phase| phase.dimension_assessments.len()),
            legacy_phase_c_assessment: projection.health_is_legacy(),
        },
        optional_sources: optional_sources(projection),
        lineage: Lineage {
            catalog_supplied: projection.lineage_catalog.is_some(),
            root_count: projection.supplied_lineages().len(),
        },
        outputs: Outputs {
            tables: table_ids.iter().map(|value| value.as_str()).collect(),
            figures: figure_ids.iter().map(|value| value.as_str()).collect(),
        },
        limitations: limitations(projection),
        rendering: Rendering {
            json_schema: "public_summary.schema1",
            numeric_format: "rust_display_normalized_negative_zero_v1",
            csv_newline: "LF",
            timestamp: None,
        },
    };
    let mut bytes = serde_json::to_vec_pretty(&summary).map_err(|source| {
        crate::reporting::PublicReportError::Serialization {
            path: path.clone(),
            source,
        }
    })?;
    bytes.push(b'\n');
    fs::write(&path, bytes).map_err(|source| crate::reporting::PublicReportError::Write {
        path: path.clone(),
        source,
    })?;
    Ok("public_summary.schema1.json".into())
}

pub fn write_markdown_report(
    root: &Path,
    projection: &PublicReportProjection,
) -> Result<String, crate::reporting::PublicReportError> {
    let path = root.join("scientific_report.md");
    let mut text = String::new();
    for heading in [
        "Analysis identity and renderer boundary",
        "Input artifacts and compatibility state",
    ] {
        text.push_str("# ");
        text.push_str(heading);
        text.push_str("\n\n");
    }
    text.push_str(REQUIRED_DISCLAIMER);
    text.push_str("\n\n");
    text.push_str("# Mechanism assessment\n\n");
    if projection.mechanism_is_legacy() {
        text.push_str(
            "Legacy mechanism artifact; Phase B V1 hypothesis assessment unavailable.\n\n",
        );
    } else {
        let mut rows = projection
            .mechanism
            .hypothesis_assessments
            .iter()
            .collect::<Vec<_>>();
        rows.sort_by(|a, b| a.definition.hypothesis_id.cmp(&b.definition.hypothesis_id));
        for row in rows {
            text.push_str("## ");
            text.push_str(&row.definition.hypothesis_id);
            text.push_str(" — ");
            text.push_str(&row.definition.display_name);
            text.push_str("\n\n");
            text.push_str(mechanism_level_text(row.current.evidence_level.clone()));
            text.push_str("\n\n");
        }
    }
    text.push_str("# Sensor-health assessment\n\n");
    if let Some(phase_c) = &projection.health.phase_c {
        for row in &phase_c.dimension_assessments {
            text.push_str("## ");
            text.push_str(
                serde_json::to_string(&row.dimension)
                    .unwrap_or_default()
                    .trim_matches('"'),
            );
            text.push_str("\n\n");
            text.push_str(health_status_text(row.status));
            text.push_str("; ");
            text.push_str(evidence_state_text(row.evidence_state));
            text.push_str("; ");
            text.push_str(causal_status_text(row.causal_status));
            text.push_str("\n\n");
        }
    } else {
        text.push_str("Legacy schema-3 health assessment: Phase C nine-dimension assessment was not serialized.\n\n");
    }
    for heading in [
        "Key evidence and contradictions",
        "Uncertainty and data-quality limitations",
        "Current-versus-baseline comparison",
        "Optional analysis projections",
        "Figures",
        "Tables",
        "Lineage and provenance",
        "Reproducibility metadata",
    ] {
        text.push_str("# ");
        text.push_str(heading);
        text.push_str("\n\n");
    }
    fs::write(&path, text).map_err(|source| crate::reporting::PublicReportError::Write {
        path: path.clone(),
        source,
    })?;
    Ok("scientific_report.md".into())
}

fn input_references(projection: &PublicReportProjection) -> Vec<InputReference> {
    let mut rows = vec![
        artifact_reference(
            "mechanism",
            crate::domain::ArtifactKind::MechanismAnalysis,
            projection.mechanism.schema_version,
        ),
        artifact_reference(
            "health",
            crate::domain::ArtifactKind::HealthAssessment,
            projection.health.schema_version,
        ),
    ];
    rows.push(optional_reference(
        "eis",
        projection.eis.as_ref().map(|value| value.schema_version),
        crate::domain::ArtifactKind::EisFit,
    ));
    rows.push(optional_reference(
        "transient",
        projection
            .transient
            .as_ref()
            .map(|value| value.schema_version),
        crate::domain::ArtifactKind::TransientAnalysis,
    ));
    rows.push(optional_reference(
        "calibration",
        projection
            .calibration
            .as_ref()
            .map(|value| value.schema_version),
        crate::domain::ArtifactKind::CalibrationAnalysis,
    ));
    rows.push(optional_reference(
        "calibration_observations",
        projection
            .calibration_observations
            .as_ref()
            .map(|value| value.schema_version),
        crate::domain::ArtifactKind::CalibrationObservations,
    ));
    rows.push(optional_reference(
        "signal",
        projection.signal.as_ref().map(|value| value.schema_version),
        crate::domain::ArtifactKind::SignalAnalysis,
    ));
    rows.push(optional_reference(
        "estimation",
        projection
            .estimation
            .as_ref()
            .map(|value| value.schema_version),
        crate::domain::ArtifactKind::StateEstimation,
    ));
    rows.push(optional_reference(
        "model",
        projection.model.as_ref().map(|value| value.schema_version),
        crate::domain::ArtifactKind::ModelAnalysis,
    ));
    rows.push(InputReference::LineageCatalog {
        supplied_path_basename: projection
            .lineage_catalog
            .as_ref()
            .map(|_| "lineage_catalog.json".into()),
        schema_version: projection.lineage_catalog.as_ref().map(|_| 1),
        availability: if projection.lineage_catalog.is_some() {
            "available"
        } else {
            "not_provided"
        },
        validation: if projection.lineage_catalog.is_some() {
            "validated"
        } else {
            "not_applicable"
        },
    });
    rows
}
fn artifact_reference(
    flag: &'static str,
    kind: crate::domain::ArtifactKind,
    schema: u32,
) -> InputReference {
    InputReference::Artifact {
        input_flag: flag,
        supplied_path_basename: None,
        artifact_kind: Some(kind),
        schema_version: Some(schema),
        availability: "available",
    }
}
fn optional_reference(
    flag: &'static str,
    schema: Option<u32>,
    kind: crate::domain::ArtifactKind,
) -> InputReference {
    InputReference::Artifact {
        input_flag: flag,
        supplied_path_basename: None,
        artifact_kind: schema.map(|_| kind),
        schema_version: schema,
        availability: if schema.is_some() {
            "available"
        } else {
            "not_provided"
        },
    }
}
fn optional_sources(projection: &PublicReportProjection) -> Vec<OptionalSource> {
    [
        ("eis", projection.eis.is_some()),
        ("transient", projection.transient.is_some()),
        ("calibration", projection.calibration.is_some()),
        ("signal", projection.signal.is_some()),
        ("estimation", projection.estimation.is_some()),
        ("model", projection.model.is_some()),
        ("lineage_catalog", projection.lineage_catalog.is_some()),
    ]
    .into_iter()
    .map(|(kind, supplied)| OptionalSource {
        kind,
        availability: if supplied {
            "available"
        } else {
            "not_provided"
        },
    })
    .collect()
}
fn limitations(projection: &PublicReportProjection) -> Vec<Limitation> {
    let mut items = Vec::new();
    if projection.mechanism_is_legacy() {
        items.push(Limitation {
            code: "legacy_input",
            message: "Legacy mechanism artifact; Phase B V1 hypothesis assessment unavailable."
                .into(),
        });
    }
    if projection.health_is_legacy() {
        items.push(Limitation{code:"legacy_input",message:"Legacy schema-3 health assessment: Phase C nine-dimension assessment was not serialized.".into()});
    }
    if projection.lineage_catalog.is_none() {
        items.push(Limitation {
            code: "catalog_not_supplied",
            message: format!(
                "{}; only serialized direct lineage is shown.",
                unavailable_text(crate::reporting::AvailabilityReason::CatalogNotSupplied)
            ),
        });
    }
    items
}
fn trim_flag(value: &'static str) -> &'static str {
    value.trim_start_matches("--")
}
fn compatibility_token(value: crate::reporting::reader::CompatibilityStatus) -> &'static str {
    match value {
        crate::reporting::reader::CompatibilityStatus::Compatible => "compatible",
        crate::reporting::reader::CompatibilityStatus::LegacyUnknown => "legacy_unknown",
        crate::reporting::reader::CompatibilityStatus::NotProvided => "not_provided",
        crate::reporting::reader::CompatibilityStatus::NotApplicable => "not_applicable",
    }
}

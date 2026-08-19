//! Clap-neutral configuration for the certified Phase-D public report route.

use crate::reporting::PublicReportError;
use serde::Serialize;
use std::{collections::BTreeSet, path::PathBuf, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    All,
    Json,
    Markdown,
}

impl FromStr for ReportFormat {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "all" => Ok(Self::All),
            "json" => Ok(Self::Json),
            "markdown" => Ok(Self::Markdown),
            _ => Err(format!("unsupported report format {value}")),
        }
    }
}

impl ReportFormat {
    pub const fn writes_json(self) -> bool {
        matches!(self, Self::All | Self::Json)
    }

    pub const fn writes_markdown(self) -> bool {
        matches!(self, Self::All | Self::Markdown)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FigureId {
    MechanismTimescale,
    SensorHealthDimensionStatus,
    CurrentVsBaseline,
    EisNyquist,
    EisBode,
    TransientResponse,
    CalibrationPerformance,
    SignalDiagnostics,
    EstimationObservedPredicted,
    ModelObservedPredicted,
    Lineage,
}

impl FigureId {
    pub const ALL: [Self; 11] = [
        Self::MechanismTimescale,
        Self::SensorHealthDimensionStatus,
        Self::CurrentVsBaseline,
        Self::EisNyquist,
        Self::EisBode,
        Self::TransientResponse,
        Self::CalibrationPerformance,
        Self::SignalDiagnostics,
        Self::EstimationObservedPredicted,
        Self::ModelObservedPredicted,
        Self::Lineage,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MechanismTimescale => "mechanism_timescale",
            Self::SensorHealthDimensionStatus => "sensor_health_dimension_status",
            Self::CurrentVsBaseline => "current_vs_baseline",
            Self::EisNyquist => "eis_nyquist",
            Self::EisBode => "eis_bode",
            Self::TransientResponse => "transient_response",
            Self::CalibrationPerformance => "calibration_performance",
            Self::SignalDiagnostics => "signal_diagnostics",
            Self::EstimationObservedPredicted => "estimation_observed_predicted",
            Self::ModelObservedPredicted => "model_observed_predicted",
            Self::Lineage => "lineage",
        }
    }
}

impl FromStr for FigureId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.as_str() == value)
            .ok_or(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TableId {
    MechanismEvidence,
    HealthDimensions,
    EvidenceProvenance,
    ArtifactLineage,
    TimescaleComparison,
    ModelConsistency,
    CurrentVsBaseline,
}

impl TableId {
    pub const ALL: [Self; 7] = [
        Self::MechanismEvidence,
        Self::HealthDimensions,
        Self::EvidenceProvenance,
        Self::ArtifactLineage,
        Self::TimescaleComparison,
        Self::ModelConsistency,
        Self::CurrentVsBaseline,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MechanismEvidence => "mechanism_evidence",
            Self::HealthDimensions => "health_dimensions",
            Self::EvidenceProvenance => "evidence_provenance",
            Self::ArtifactLineage => "artifact_lineage",
            Self::TimescaleComparison => "timescale_comparison",
            Self::ModelConsistency => "model_consistency",
            Self::CurrentVsBaseline => "current_vs_baseline",
        }
    }
}

impl FromStr for TableId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.as_str() == value)
            .ok_or(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionMode {
    Default,
    Explicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportSelection {
    pub figures: Vec<FigureId>,
    pub tables: Vec<TableId>,
    pub figures_mode: SelectionMode,
    pub tables_mode: SelectionMode,
}

impl ReportSelection {
    pub fn parse(figures: Option<&str>, tables: Option<&str>) -> Result<Self, PublicReportError> {
        Ok(Self {
            figures: parse_selector(figures, "figures", FigureId::ALL, FigureId::from_str)?,
            tables: parse_selector(tables, "tables", TableId::ALL, TableId::from_str)?,
            figures_mode: if figures.is_some() {
                SelectionMode::Explicit
            } else {
                SelectionMode::Default
            },
            tables_mode: if tables.is_some() {
                SelectionMode::Explicit
            } else {
                SelectionMode::Default
            },
        })
    }
}

fn parse_selector<T: Copy + Ord>(
    supplied: Option<&str>,
    selector: &'static str,
    all: impl IntoIterator<Item = T>,
    parse: impl Fn(&str) -> Result<T, ()>,
) -> Result<Vec<T>, PublicReportError> {
    let Some(supplied) = supplied else {
        return Ok(all.into_iter().collect());
    };
    if supplied.is_empty() {
        return Err(PublicReportError::InvalidSelection {
            selector,
            value: supplied.to_string(),
        });
    }
    if supplied == "all" {
        return Ok(all.into_iter().collect());
    }
    if supplied == "none" {
        return Ok(Vec::new());
    }
    let mut values = Vec::new();
    let mut seen = BTreeSet::new();
    for token in supplied.split(',') {
        if token == "all" || token == "none" || token.is_empty() {
            return Err(PublicReportError::InvalidSelection {
                selector,
                value: token.to_string(),
            });
        }
        let parsed = parse(token).map_err(|_| PublicReportError::InvalidSelection {
            selector,
            value: token.to_string(),
        })?;
        if !seen.insert(parsed) {
            return Err(PublicReportError::InvalidSelection {
                selector,
                value: token.to_string(),
            });
        }
        values.push(parsed);
    }
    Ok(values)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportRenderOptions {
    pub mechanism: PathBuf,
    pub health: PathBuf,
    pub output_dir: PathBuf,
    pub lineage_catalog: Option<PathBuf>,
    pub eis: Option<PathBuf>,
    pub transient: Option<PathBuf>,
    pub calibration: Option<PathBuf>,
    pub calibration_observations: Option<PathBuf>,
    pub signal: Option<PathBuf>,
    pub estimation: Option<PathBuf>,
    pub model: Option<PathBuf>,
    pub format: ReportFormat,
    pub selection: ReportSelection,
    pub overwrite: bool,
}

impl ReportRenderOptions {
    pub fn validate_pairing(&self) -> Result<(), PublicReportError> {
        if self.calibration.is_some() != self.calibration_observations.is_some() {
            return Err(PublicReportError::InvalidCombination {
                detail: "--calibration and --calibration-observations must be supplied together",
            });
        }
        Ok(())
    }
}

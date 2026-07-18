//! Structured command-line parsing and legacy-flag normalization.
//!
//! The derive-based [`Cli`] tree is the canonical interface.  The old flat
//! flags remain represented in [`LegacyArgs`] and are normalized into the same
//! [`CommandSpec`] values, so existing scripts keep working while new users
//! get explicit command boundaries.

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum, error::ErrorKind};
use std::path::PathBuf;
use thiserror::Error;

/// Controls which category of plots the plotting command produces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum PlotTarget {
    /// Generate EIS, regular, and generic plots.
    All,
    /// Generate EIS (Nyquist / Bode) plots only.
    Eis,
    /// Generate regular (Pb-sensor / CHI timeseries) plots only.
    #[value(alias = "regular", alias = "pb", alias = "pb-sensor", alias = "chi")]
    RegularPlot,
    /// Generate only the generic (`[[generic_plot]]`) plots.
    #[value(alias = "generic-plot")]
    GenericPlot,
}

/// Top-level derive-based CLI parser.
#[derive(Debug, Parser)]
#[command(
    name = "electroanalysis",
    bin_name = "electroanalysis",
    about = "Electrochemical data analysis and equivalent-circuit workflows",
    version,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    #[command(flatten)]
    pub legacy: LegacyArgs,
}

/// Structured command tree exposed by the binary.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate configured EIS, regular, and/or generic plots.
    Plot(PlotCommand),
    /// Run an EIS fit or equivalent-circuit search.
    Eis {
        #[command(subcommand)]
        command: EisCommand,
    },
    /// Analyze potentiometric transient responses around experimental events.
    Transient {
        #[command(subcommand)]
        command: TransientCommand,
    },
    /// Extract, fit, validate, and use equilibrium potentiometric calibrations.
    Calibration {
        #[command(subcommand)]
        command: CalibrationCommand,
    },
    /// Compare EIS-derived and transient-derived characteristic timescales.
    Mechanism {
        #[command(subcommand)]
        command: MechanismCommand,
    },
    /// Characterize signal quality and residual structure.
    Signal {
        #[command(subcommand)]
        command: SignalCommand,
    },
    /// Construct baselines and assess sensor health.
    Health {
        #[command(subcommand)]
        command: HealthCommand,
    },
    /// Estimate latent activity and sensor-response states from time-resolved measurements.
    Estimate {
        #[command(subcommand)]
        command: EstimateCommand,
    },
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
pub enum EstimateCommand {
    Run(EstimateRunCommand),
    Validate(EstimateValidateCommand),
    Simulate(EstimateSimulateCommand),
    Compare(EstimateCompareCommand),
    Report(EstimateReportCommand),
}

#[derive(Debug, Args)]
pub struct EstimateRunCommand {
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub metadata: PathBuf,
    #[arg(long)]
    pub channel: String,
    #[arg(long, value_name = "NAME")]
    pub sheet: Option<String>,
    #[arg(long = "calibration-model")]
    pub calibration_model: PathBuf,
    #[arg(long = "signal-results")]
    pub signal_results: Option<PathBuf>,
    #[arg(long = "transient-results")]
    pub transient_results: Option<PathBuf>,
    #[arg(long = "calibration-results")]
    pub calibration_results: Option<PathBuf>,
    #[arg(long = "eis-fit")]
    pub eis_fit: Option<PathBuf>,
    #[arg(long = "mechanism-results")]
    pub mechanism_results: Option<PathBuf>,
    #[arg(long = "health-baseline")]
    pub health_baseline: Option<PathBuf>,
    #[arg(long = "health-assessment")]
    pub health_assessment: Option<PathBuf>,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub filter: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long)]
    pub seed: Option<u64>,
}
#[derive(Debug, Args)]
pub struct EstimateValidateCommand {
    #[arg(long)]
    pub results: PathBuf,
    #[arg(long)]
    pub truth: PathBuf,
    #[arg(long)]
    pub output: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub struct EstimateSimulateCommand {
    #[arg(long)]
    pub scenario: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub seed: Option<u64>,
}
#[derive(Debug, Args)]
pub struct EstimateCompareCommand {
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub metadata: PathBuf,
    #[arg(long)]
    pub channel: String,
    #[arg(long, value_name = "NAME")]
    pub sheet: Option<String>,
    #[arg(long = "calibration-model")]
    pub calibration_model: PathBuf,
    #[arg(long)]
    pub filters: Option<String>,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub struct EstimateReportCommand {
    #[arg(long)]
    pub results: PathBuf,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct PlotCommand {
    /// Plot category. Defaults to all configured plot workflows.
    #[arg(value_enum, default_value_t = PlotTarget::All, value_name = "TARGET")]
    pub target: PlotTarget,
    /// Override the plotting TOML file.
    #[arg(long = "plot-config", alias = "config", value_name = "PATH")]
    pub plot_config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum EisCommand {
    /// Fit one EIS data file with its resolved or explicitly supplied circuit.
    Fit(EisFitCommand),
    /// Search one EIS file or all supported EIS files in a directory.
    Search(EisSearchCommand),
    /// Export a durable JSON artifact for one EIS fit.
    ExportFit(EisExportFitCommand),
}

#[derive(Debug, Subcommand)]
pub enum MechanismCommand {
    Compare(MechanismCompareCommand),
    Trend(MechanismTrendCommand),
    Report(MechanismReportCommand),
}

#[derive(Debug, Subcommand)]
pub enum SignalCommand {
    Characterize(SignalCharacterizeCommand),
    Compare(SignalCompareCommand),
    Residuals(SignalResidualsCommand),
}

#[derive(Debug, Args)]
pub struct SignalCharacterizeCommand {
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub metadata: Option<PathBuf>,
    #[arg(long)]
    pub channel: String,
    #[arg(long, value_name = "NAME")]
    pub sheet: Option<String>,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub struct SignalCompareCommand {
    #[arg(long)]
    pub manifest: PathBuf,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub struct SignalResidualsCommand {
    #[arg(long)]
    pub transient_results: Option<PathBuf>,
    #[arg(long)]
    pub calibration_results: Option<PathBuf>,
    #[arg(long)]
    pub eis_fit: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum HealthCommand {
    Baseline(HealthBaselineCommand),
    Assess(HealthAssessCommand),
    Trend(HealthTrendCommand),
    Report(HealthReportCommand),
}
#[derive(Debug, Args)]
pub struct HealthBaselineCommand {
    #[arg(long)]
    pub manifest: PathBuf,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub struct HealthAssessCommand {
    #[arg(long)]
    pub signal_results: PathBuf,
    #[arg(long)]
    pub transient_results: Option<PathBuf>,
    #[arg(long)]
    pub calibration_results: Option<PathBuf>,
    #[arg(long)]
    pub eis_fit: Option<PathBuf>,
    #[arg(long)]
    pub mechanism_results: Option<PathBuf>,
    #[arg(long)]
    pub baseline: Option<PathBuf>,
    #[arg(long)]
    pub metadata: Option<PathBuf>,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub struct HealthTrendCommand {
    #[arg(long)]
    pub manifest: PathBuf,
    #[arg(long)]
    pub baseline: Option<PathBuf>,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub struct HealthReportCommand {
    #[arg(long)]
    pub results: PathBuf,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum TransientCommand {
    /// Fit configured transient models to one or more eligible events.
    Fit(TransientFitCommand),
}

#[derive(Debug, Subcommand)]
pub enum CalibrationCommand {
    /// Extract equilibrium calibration observations from concentration events.
    Extract(CalibrationExtractCommand),
    /// Fit configured calibration models to observations.
    Fit(CalibrationFitCommand),
    /// Validate a stored calibration model against observations.
    Validate(CalibrationValidateCommand),
    /// Predict activity or concentration from a stored calibration model.
    Predict(CalibrationPredictCommand),
}

#[derive(Debug, Args)]
pub struct CalibrationExtractCommand {
    #[arg(long, value_name = "PATH")]
    pub input: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub metadata: PathBuf,
    #[arg(long, value_name = "NAME")]
    pub channel: String,
    #[arg(long, value_name = "NAME")]
    pub sheet: Option<String>,
    #[arg(long, value_name = "PATH")]
    pub transient_results: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CalibrationFitCommand {
    #[arg(long, value_name = "PATH")]
    pub observations: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    #[arg(long, value_name = "MODEL")]
    pub model: Option<String>,
    #[arg(long, value_name = "CRITERION")]
    pub selection: Option<String>,
    #[arg(long, value_name = "N")]
    pub bootstrap: Option<usize>,
    #[arg(long, value_name = "N")]
    pub seed: Option<u64>,
}

#[derive(Debug, Args)]
pub struct CalibrationValidateCommand {
    #[arg(long, value_name = "PATH")]
    pub model: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub observations: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CalibrationPredictCommand {
    #[arg(long, value_name = "PATH")]
    pub model: PathBuf,
    #[arg(long, value_name = "V")]
    pub potential: Option<f64>,
    /// Temperature in degrees Celsius; converted to kelvin internally.
    #[arg(long, value_name = "C")]
    pub temperature: Option<f64>,
    #[arg(long, value_name = "PATH")]
    pub input: Option<PathBuf>,
    #[arg(long, value_name = "NAME", requires = "input")]
    pub channel: Option<String>,
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum TransientEventKindArg {
    #[value(name = "concentration-step")]
    ConcentrationStep,
    #[value(name = "flow-change")]
    FlowChange,
    #[value(name = "temperature-change")]
    TemperatureChange,
    #[value(name = "ionic-strength-change")]
    IonicStrengthChange,
    #[value(name = "interferent-addition")]
    InterferentAddition,
    #[value(name = "flush-start")]
    FlushStart,
    #[value(name = "reading-start")]
    ReadingStart,
    #[value(name = "flush-end")]
    FlushEnd,
    #[value(name = "manual-annotation")]
    ManualAnnotation,
}

impl TransientEventKindArg {
    fn as_str(self) -> &'static str {
        match self {
            Self::ConcentrationStep => "concentration-step",
            Self::FlowChange => "flow-change",
            Self::TemperatureChange => "temperature-change",
            Self::IonicStrengthChange => "ionic-strength-change",
            Self::InterferentAddition => "interferent-addition",
            Self::FlushStart => "flush-start",
            Self::ReadingStart => "reading-start",
            Self::FlushEnd => "flush-end",
            Self::ManualAnnotation => "manual-annotation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum TransientModelArg {
    Single,
    Double,
    #[value(name = "double-drift")]
    DoubleDrift,
    Stretched,
    All,
}

impl TransientModelArg {
    fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Double => "double",
            Self::DoubleDrift => "double-drift",
            Self::Stretched => "stretched",
            Self::All => "all",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum TransientSelectionArg {
    Aic,
    Bic,
}

impl TransientSelectionArg {
    fn as_str(self) -> &'static str {
        match self {
            Self::Aic => "aic",
            Self::Bic => "bic",
        }
    }
}

#[derive(Debug, Args)]
pub struct TransientFitCommand {
    /// Input time-series data file.
    #[arg(long, value_name = "PATH")]
    pub input: PathBuf,
    /// Experiment metadata TOML file.
    #[arg(long, value_name = "PATH")]
    pub metadata: PathBuf,
    /// Measurement channel name, for example `E1/V` or `potential`.
    #[arg(long, value_name = "NAME")]
    pub channel: String,
    #[arg(long, value_name = "NAME")]
    pub sheet: Option<String>,
    /// Transient configuration override.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    /// Output directory for transient reports and figures.
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Event category to analyze.
    #[arg(long, value_enum, default_value_t = TransientEventKindArg::ConcentrationStep)]
    pub event_kind: TransientEventKindArg,
    /// Zero-based index among eligible events.
    #[arg(long, value_name = "N")]
    pub event_index: Option<usize>,
    /// Fit one model or all configured models.
    #[arg(long, value_enum)]
    pub model: Option<TransientModelArg>,
    /// Information criterion used for model selection.
    #[arg(long, value_enum)]
    pub selection: Option<TransientSelectionArg>,
    /// Residual bootstrap iteration override.
    #[arg(long, value_name = "N")]
    pub bootstrap: Option<usize>,
    /// Reproducibility seed override.
    #[arg(long, value_name = "N")]
    pub seed: Option<u64>,
}

#[derive(Debug, Args)]
pub struct EisFitCommand {
    /// Input CHI EIS file.
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,
    /// Circuit expression override, for example `R0-p(CPE1,R1)`.
    #[arg(
        short = 'c',
        long = "circuit",
        alias = "model",
        value_name = "EXPRESSION"
    )]
    pub circuit_model: Option<String>,
    /// Write the fit report to this path instead of stdout.
    #[arg(short = 'o', long = "output", value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Optional durable JSON artifact destination.
    #[arg(long, value_name = "PATH")]
    pub artifact: Option<PathBuf>,
    /// Optional human-readable artifact report destination.
    #[arg(long, value_name = "PATH")]
    pub report: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct EisExportFitCommand {
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,
    #[arg(
        short = 'c',
        long = "circuit",
        alias = "model",
        value_name = "EXPRESSION"
    )]
    pub circuit_model: Option<String>,
    #[arg(long, value_name = "PATH")]
    pub artifact: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub report: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct MechanismCompareCommand {
    #[arg(long, value_name = "PATH")]
    pub eis_fit: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub transient_results: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub calibration_results: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub metadata: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct MechanismTrendCommand {
    #[arg(long, value_name = "PATH")]
    pub manifest: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct MechanismReportCommand {
    #[arg(long, value_name = "PATH")]
    pub results: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct EisSearchCommand {
    /// EIS file or directory to search.
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,
    /// Override the analysis TOML file.
    #[arg(long = "search-config", alias = "config", value_name = "PATH")]
    pub search_config: Option<PathBuf>,
    /// Report file or output directory override.
    #[arg(long = "search-output", value_name = "PATH")]
    pub search_output: Option<PathBuf>,
    /// Maximum number of ranked candidates to retain.
    #[arg(long = "search-top", value_name = "N")]
    pub search_top: Option<usize>,
}

/// The legacy flat options accepted before the subcommand migration.
#[derive(Debug, Default, Args)]
pub struct LegacyArgs {
    /// Legacy plot selector (`all`, `eis`, `regular-plot`, or `generic`).
    #[arg(long = "plot", value_enum, value_name = "TARGET")]
    pub plot: Option<PlotTarget>,
    /// Legacy plotting configuration override.
    #[arg(long = "plot-config", value_name = "PATH")]
    pub plot_config: Option<PathBuf>,
    /// Legacy EIS-search target.
    #[arg(long = "search-eis", value_name = "PATH")]
    pub search_eis: Option<PathBuf>,
    /// Legacy search configuration override.
    #[arg(long = "search-config", value_name = "PATH")]
    pub search_config: Option<PathBuf>,
    /// Legacy search report output override.
    #[arg(long = "search-output", value_name = "PATH")]
    pub search_output: Option<PathBuf>,
    /// Legacy ranked-candidate limit.
    #[arg(long = "search-top", value_name = "N")]
    pub search_top: Option<usize>,
}

/// Normalized command values consumed by the application layer.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandSpec {
    Plot {
        target: PlotTarget,
        plot_config_path: Option<PathBuf>,
    },
    EisFit {
        input: PathBuf,
        circuit_model: Option<String>,
        output: Option<PathBuf>,
        artifact: Option<PathBuf>,
        report: Option<PathBuf>,
    },
    EisExportFit {
        input: PathBuf,
        circuit_model: Option<String>,
        artifact: PathBuf,
        report: Option<PathBuf>,
    },
    EisSearch {
        input: PathBuf,
        search_config_path: Option<PathBuf>,
        search_output: Option<PathBuf>,
        search_top: Option<usize>,
    },
    TransientFit {
        input: PathBuf,
        metadata: PathBuf,
        channel: String,
        sheet: Option<String>,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
        event_kind: String,
        event_index: Option<usize>,
        model: Option<String>,
        selection: Option<String>,
        bootstrap: Option<usize>,
        seed: Option<u64>,
    },
    CalibrationExtract {
        input: PathBuf,
        metadata: PathBuf,
        channel: String,
        sheet: Option<String>,
        transient_results: Option<PathBuf>,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    CalibrationFit {
        observations: PathBuf,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
        model: Option<String>,
        selection: Option<String>,
        bootstrap: Option<usize>,
        seed: Option<u64>,
    },
    CalibrationValidate {
        model: PathBuf,
        observations: PathBuf,
        output: Option<PathBuf>,
    },
    CalibrationPredict {
        model: PathBuf,
        potential: Option<f64>,
        temperature: Option<f64>,
        input: Option<PathBuf>,
        channel: Option<String>,
        output: Option<PathBuf>,
    },
    MechanismCompare {
        eis_fit: PathBuf,
        transient_results: PathBuf,
        calibration_results: Option<PathBuf>,
        metadata: Option<PathBuf>,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    MechanismTrend {
        manifest: PathBuf,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    MechanismReport {
        results: PathBuf,
        output: Option<PathBuf>,
    },
    SignalCharacterize {
        input: PathBuf,
        metadata: Option<PathBuf>,
        channel: String,
        sheet: Option<String>,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    SignalCompare {
        manifest: PathBuf,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    SignalResiduals {
        transient_results: Option<PathBuf>,
        calibration_results: Option<PathBuf>,
        eis_fit: Option<PathBuf>,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    HealthBaseline {
        manifest: PathBuf,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    HealthAssess {
        signal_results: PathBuf,
        transient_results: Option<PathBuf>,
        calibration_results: Option<PathBuf>,
        eis_fit: Option<PathBuf>,
        mechanism_results: Option<PathBuf>,
        baseline: Option<PathBuf>,
        metadata: Option<PathBuf>,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    HealthTrend {
        manifest: PathBuf,
        baseline: Option<PathBuf>,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    HealthReport {
        results: PathBuf,
        output: Option<PathBuf>,
    },
    EstimateRun {
        input: PathBuf,
        metadata: PathBuf,
        channel: String,
        sheet: Option<String>,
        calibration_model: PathBuf,
        signal_results: Option<PathBuf>,
        transient_results: Option<PathBuf>,
        calibration_results: Option<PathBuf>,
        eis_fit: Option<PathBuf>,
        mechanism_results: Option<PathBuf>,
        health_baseline: Option<PathBuf>,
        health_assessment: Option<PathBuf>,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
        filter: Option<String>,
        model: Option<String>,
        seed: Option<u64>,
    },
    EstimateValidate {
        results: PathBuf,
        truth: PathBuf,
        output: Option<PathBuf>,
    },
    EstimateSimulate {
        scenario: Option<PathBuf>,
        output: Option<PathBuf>,
        seed: Option<u64>,
    },
    EstimateCompare {
        input: PathBuf,
        metadata: PathBuf,
        channel: String,
        sheet: Option<String>,
        calibration_model: PathBuf,
        filters: Option<String>,
        config_path: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    EstimateReport {
        results: PathBuf,
        output: Option<PathBuf>,
    },
}

/// Errors raised while parsing or validating command-line arguments.
#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Parse(#[from] clap::Error),
    #[error("invalid command combination: {0}")]
    InvalidCombination(String),
}

/// Compatibility representation retained for callers of the former parser.
/// New code should consume [`CommandSpec`].
#[derive(Debug, Clone, PartialEq)]
pub struct CliArgs {
    pub command: Option<CommandSpec>,
    pub plot_target: Option<PlotTarget>,
    pub plot_config_path: Option<PathBuf>,
    pub search_target: Option<PathBuf>,
    pub search_config_path: Option<PathBuf>,
    pub search_output: Option<PathBuf>,
    pub search_top: Option<usize>,
    pub fit_target: Option<PathBuf>,
    pub fit_circuit_model: Option<String>,
    pub fit_output: Option<PathBuf>,
    pub help_text: Option<String>,
}

impl CliArgs {
    fn from_command(command: Option<CommandSpec>) -> Self {
        let mut result = Self {
            command: command.clone(),
            plot_target: None,
            plot_config_path: None,
            search_target: None,
            search_config_path: None,
            search_output: None,
            search_top: None,
            fit_target: None,
            fit_circuit_model: None,
            fit_output: None,
            help_text: None,
        };

        match command {
            Some(CommandSpec::Plot {
                target,
                plot_config_path,
            }) => {
                result.plot_target = Some(target);
                result.plot_config_path = plot_config_path;
            }
            Some(CommandSpec::EisFit {
                input,
                circuit_model,
                output,
                artifact: _,
                report: _,
            }) => {
                result.fit_target = Some(input);
                result.fit_circuit_model = circuit_model;
                result.fit_output = output;
            }
            Some(CommandSpec::EisExportFit { input, .. }) => {
                result.fit_target = Some(input);
            }
            Some(CommandSpec::EisSearch {
                input,
                search_config_path,
                search_output,
                search_top,
            }) => {
                result.search_target = Some(input);
                result.search_config_path = search_config_path;
                result.search_output = search_output;
                result.search_top = search_top;
            }
            Some(CommandSpec::TransientFit { .. }) => {}
            Some(CommandSpec::CalibrationExtract { .. })
            | Some(CommandSpec::CalibrationFit { .. })
            | Some(CommandSpec::CalibrationValidate { .. })
            | Some(CommandSpec::CalibrationPredict { .. }) => {}
            Some(CommandSpec::MechanismCompare { .. })
            | Some(CommandSpec::MechanismTrend { .. })
            | Some(CommandSpec::MechanismReport { .. })
            | Some(CommandSpec::SignalCharacterize { .. })
            | Some(CommandSpec::SignalCompare { .. })
            | Some(CommandSpec::SignalResiduals { .. })
            | Some(CommandSpec::HealthBaseline { .. })
            | Some(CommandSpec::HealthAssess { .. })
            | Some(CommandSpec::HealthTrend { .. })
            | Some(CommandSpec::HealthReport { .. }) => {}
            Some(CommandSpec::EstimateRun { .. })
            | Some(CommandSpec::EstimateValidate { .. })
            | Some(CommandSpec::EstimateSimulate { .. })
            | Some(CommandSpec::EstimateCompare { .. })
            | Some(CommandSpec::EstimateReport { .. }) => {}
            None => {}
        }

        result
    }
}

/// Parse derive-based arguments and normalize both structured and legacy forms.
pub fn parse_cli_args(args: &[String]) -> Result<CliArgs, CliError> {
    let parsed = match Cli::try_parse_from(args.iter().map(String::as_str)) {
        Ok(parsed) => parsed,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            let mut result = CliArgs::from_command(None);
            result.help_text = Some(error.to_string());
            return Ok(result);
        }
        Err(error) => return Err(CliError::Parse(error)),
    };

    normalize_cli(parsed)
}

fn normalize_cli(parsed: Cli) -> Result<CliArgs, CliError> {
    let legacy = parsed.legacy;
    let legacy_used = legacy.plot.is_some()
        || legacy.plot_config.is_some()
        || legacy.search_eis.is_some()
        || legacy.search_config.is_some()
        || legacy.search_output.is_some()
        || legacy.search_top.is_some();

    if parsed.command.is_some() && legacy_used {
        return Err(CliError::InvalidCombination(
            "structured subcommands cannot be combined with legacy --plot/--search-* flags"
                .to_string(),
        ));
    }

    let command = if let Some(command) = parsed.command {
        match command {
            Command::Plot(command) => CommandSpec::Plot {
                target: command.target,
                plot_config_path: command.plot_config,
            },
            Command::Eis { command } => match command {
                EisCommand::Fit(command) => CommandSpec::EisFit {
                    input: command.input,
                    circuit_model: command.circuit_model,
                    output: command.output,
                    artifact: command.artifact,
                    report: command.report,
                },
                EisCommand::ExportFit(command) => CommandSpec::EisExportFit {
                    input: command.input,
                    circuit_model: command.circuit_model,
                    artifact: command.artifact,
                    report: command.report,
                },
                EisCommand::Search(command) => {
                    validate_search_top(command.search_top)?;
                    CommandSpec::EisSearch {
                        input: command.input,
                        search_config_path: command.search_config,
                        search_output: command.search_output,
                        search_top: command.search_top,
                    }
                }
            },
            Command::Transient { command } => match command {
                TransientCommand::Fit(command) => CommandSpec::TransientFit {
                    input: command.input,
                    metadata: command.metadata,
                    channel: command.channel,
                    sheet: command.sheet,
                    config_path: command.config,
                    output: command.output,
                    event_kind: command.event_kind.as_str().to_string(),
                    event_index: command.event_index,
                    model: command.model.map(|model| model.as_str().to_string()),
                    selection: command
                        .selection
                        .map(|selection| selection.as_str().to_string()),
                    bootstrap: command.bootstrap,
                    seed: command.seed,
                },
            },
            Command::Calibration { command } => match command {
                CalibrationCommand::Extract(command) => CommandSpec::CalibrationExtract {
                    input: command.input,
                    metadata: command.metadata,
                    channel: command.channel,
                    sheet: command.sheet,
                    transient_results: command.transient_results,
                    config_path: command.config,
                    output: command.output,
                },
                CalibrationCommand::Fit(command) => CommandSpec::CalibrationFit {
                    observations: command.observations,
                    config_path: command.config,
                    output: command.output,
                    model: command.model,
                    selection: command.selection,
                    bootstrap: command.bootstrap,
                    seed: command.seed,
                },
                CalibrationCommand::Validate(command) => CommandSpec::CalibrationValidate {
                    model: command.model,
                    observations: command.observations,
                    output: command.output,
                },
                CalibrationCommand::Predict(command) => {
                    if command.potential.is_none() && command.input.is_none() {
                        return Err(CliError::InvalidCombination(
                            "calibration predict requires --potential or --input".to_string(),
                        ));
                    }
                    if command.potential.is_some() && command.input.is_some() {
                        return Err(CliError::InvalidCombination(
                            "calibration predict accepts either --potential or --input, not both"
                                .to_string(),
                        ));
                    }
                    if command.input.is_some() && command.channel.is_none() {
                        return Err(CliError::InvalidCombination(
                            "calibration predict with --input requires --channel".to_string(),
                        ));
                    }
                    CommandSpec::CalibrationPredict {
                        model: command.model,
                        potential: command.potential,
                        temperature: command.temperature,
                        input: command.input,
                        channel: command.channel,
                        output: command.output,
                    }
                }
            },
            Command::Mechanism { command } => match command {
                MechanismCommand::Compare(command) => CommandSpec::MechanismCompare {
                    eis_fit: command.eis_fit,
                    transient_results: command.transient_results,
                    calibration_results: command.calibration_results,
                    metadata: command.metadata,
                    config_path: command.config,
                    output: command.output,
                },
                MechanismCommand::Trend(command) => CommandSpec::MechanismTrend {
                    manifest: command.manifest,
                    config_path: command.config,
                    output: command.output,
                },
                MechanismCommand::Report(command) => CommandSpec::MechanismReport {
                    results: command.results,
                    output: command.output,
                },
            },
            Command::Signal { command } => match command {
                SignalCommand::Characterize(c) => CommandSpec::SignalCharacterize {
                    input: c.input,
                    metadata: c.metadata,
                    channel: c.channel,
                    sheet: c.sheet,
                    config_path: c.config,
                    output: c.output,
                },
                SignalCommand::Compare(c) => CommandSpec::SignalCompare {
                    manifest: c.manifest,
                    config_path: c.config,
                    output: c.output,
                },
                SignalCommand::Residuals(c) => CommandSpec::SignalResiduals {
                    transient_results: c.transient_results,
                    calibration_results: c.calibration_results,
                    eis_fit: c.eis_fit,
                    config_path: c.config,
                    output: c.output,
                },
            },
            Command::Health { command } => match command {
                HealthCommand::Baseline(c) => CommandSpec::HealthBaseline {
                    manifest: c.manifest,
                    config_path: c.config,
                    output: c.output,
                },
                HealthCommand::Assess(c) => CommandSpec::HealthAssess {
                    signal_results: c.signal_results,
                    transient_results: c.transient_results,
                    calibration_results: c.calibration_results,
                    eis_fit: c.eis_fit,
                    mechanism_results: c.mechanism_results,
                    baseline: c.baseline,
                    metadata: c.metadata,
                    config_path: c.config,
                    output: c.output,
                },
                HealthCommand::Trend(c) => CommandSpec::HealthTrend {
                    manifest: c.manifest,
                    baseline: c.baseline,
                    config_path: c.config,
                    output: c.output,
                },
                HealthCommand::Report(c) => CommandSpec::HealthReport {
                    results: c.results,
                    output: c.output,
                },
            },
            Command::Estimate { command } => match command {
                EstimateCommand::Run(c) => CommandSpec::EstimateRun {
                    input: c.input,
                    metadata: c.metadata,
                    channel: c.channel,
                    sheet: c.sheet,
                    calibration_model: c.calibration_model,
                    signal_results: c.signal_results,
                    transient_results: c.transient_results,
                    calibration_results: c.calibration_results,
                    eis_fit: c.eis_fit,
                    mechanism_results: c.mechanism_results,
                    health_baseline: c.health_baseline,
                    health_assessment: c.health_assessment,
                    config_path: c.config,
                    output: c.output,
                    filter: c.filter,
                    model: c.model,
                    seed: c.seed,
                },
                EstimateCommand::Validate(c) => CommandSpec::EstimateValidate {
                    results: c.results,
                    truth: c.truth,
                    output: c.output,
                },
                EstimateCommand::Simulate(c) => CommandSpec::EstimateSimulate {
                    scenario: c.scenario,
                    output: c.output,
                    seed: c.seed,
                },
                EstimateCommand::Compare(c) => CommandSpec::EstimateCompare {
                    input: c.input,
                    metadata: c.metadata,
                    channel: c.channel,
                    sheet: c.sheet,
                    calibration_model: c.calibration_model,
                    filters: c.filters,
                    config_path: c.config,
                    output: c.output,
                },
                EstimateCommand::Report(c) => CommandSpec::EstimateReport {
                    results: c.results,
                    output: c.output,
                },
            },
        }
    } else if let Some(search_target) = legacy.search_eis {
        if legacy.plot_config.is_some() || legacy.plot.is_some() {
            return Err(CliError::InvalidCombination(
                "use either --plot or --search-eis in one invocation, not both".to_string(),
            ));
        }
        validate_search_top(legacy.search_top)?;
        CommandSpec::EisSearch {
            input: search_target,
            search_config_path: legacy.search_config,
            search_output: legacy.search_output,
            search_top: legacy.search_top,
        }
    } else {
        if legacy.search_config.is_some()
            || legacy.search_output.is_some()
            || legacy.search_top.is_some()
        {
            return Err(CliError::InvalidCombination(
                "--search-config, --search-output, and --search-top require --search-eis"
                    .to_string(),
            ));
        }
        CommandSpec::Plot {
            target: legacy.plot.unwrap_or(PlotTarget::All),
            plot_config_path: legacy.plot_config,
        }
    };

    Ok(CliArgs::from_command(Some(command)))
}

fn validate_search_top(search_top: Option<usize>) -> Result<(), CliError> {
    if search_top == Some(0) {
        return Err(CliError::InvalidCombination(
            "--search-top must be greater than zero".to_string(),
        ));
    }
    Ok(())
}

/// Print the derive-generated help synopsis using a caller-supplied program name.
pub fn print_usage(program: &str) {
    let _ = program;
    let mut command = Cli::command();
    match command.print_help() {
        Ok(()) => println!(),
        Err(error) => eprintln!("failed to render CLI help: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandSpec, PlotTarget, parse_cli_args};

    fn parse(values: &[&str]) -> super::CliArgs {
        let args = std::iter::once("electroanalysis")
            .chain(values.iter().copied())
            .map(str::to_string)
            .collect::<Vec<_>>();
        parse_cli_args(&args).expect("CLI should parse")
    }

    #[test]
    fn structured_plot_command_defaults_to_all() {
        let parsed = parse(&["plot"]);
        assert_eq!(
            parsed.command,
            Some(CommandSpec::Plot {
                target: PlotTarget::All,
                plot_config_path: None,
            })
        );
    }

    #[test]
    fn legacy_plot_flags_normalize_to_structured_command() {
        let parsed = parse(&["--plot", "regular-plot", "--plot-config", "legacy.toml"]);
        assert_eq!(
            parsed.command,
            Some(CommandSpec::Plot {
                target: PlotTarget::RegularPlot,
                plot_config_path: Some("legacy.toml".into()),
            })
        );
        assert_eq!(parsed.plot_target, Some(PlotTarget::RegularPlot));
    }

    #[test]
    fn structured_search_preserves_all_search_overrides() {
        let parsed = parse(&[
            "eis",
            "search",
            "data/sample.csv",
            "--search-config",
            "analysis.toml",
            "--search-output",
            "reports",
            "--search-top",
            "7",
        ]);
        assert_eq!(
            parsed.command,
            Some(CommandSpec::EisSearch {
                input: "data/sample.csv".into(),
                search_config_path: Some("analysis.toml".into()),
                search_output: Some("reports".into()),
                search_top: Some(7),
            })
        );
    }

    #[test]
    fn invalid_legacy_plot_search_combination_is_clear() {
        let args = [
            "electroanalysis".to_string(),
            "--plot".to_string(),
            "eis".to_string(),
            "--search-eis".to_string(),
            "data".to_string(),
        ];
        let error = parse_cli_args(&args).expect_err("mixed modes must fail");
        assert!(error.to_string().contains("either --plot or --search-eis"));
    }

    #[test]
    fn fit_command_exposes_named_fit_options() {
        let parsed = parse(&[
            "eis",
            "fit",
            "sample.csv",
            "--circuit",
            "R0-p(CPE1,R1)",
            "--output",
            "fit.txt",
        ]);
        assert_eq!(parsed.fit_target, Some("sample.csv".into()));
        assert_eq!(parsed.fit_circuit_model.as_deref(), Some("R0-p(CPE1,R1)"));
        assert_eq!(parsed.fit_output, Some("fit.txt".into()));
    }
}

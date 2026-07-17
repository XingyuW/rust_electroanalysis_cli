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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandSpec {
    Plot {
        target: PlotTarget,
        plot_config_path: Option<PathBuf>,
    },
    EisFit {
        input: PathBuf,
        circuit_model: Option<String>,
        output: Option<PathBuf>,
    },
    EisSearch {
        input: PathBuf,
        search_config_path: Option<PathBuf>,
        search_output: Option<PathBuf>,
        search_top: Option<usize>,
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
            }) => {
                result.fit_target = Some(input);
                result.fit_circuit_model = circuit_model;
                result.fit_output = output;
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

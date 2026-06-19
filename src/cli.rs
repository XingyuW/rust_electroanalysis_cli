//! CLI argument parsing and usage help for the `rust_plots` binary.
//!
//! This module owns all knowledge about command-line flags: what they mean,
//! how they are validated, and what structured value (`CliArgs`) they produce.
//! Nothing in here performs I/O beyond writing the usage synopsis.

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Controls which category of plots the `--plot` flag produces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlotTarget {
    /// Generate EIS, regular, and generic plots.
    All,
    /// Generate EIS (Nyquist / Bode) plots only.
    Eis,
    /// Generate regular (Pb-sensor / CHI timeseries) plots only.
    RegularPlot,
    /// Generate only the generic (`[[generic_plot]]`) plots.
    ///
    /// These use the domain-agnostic `PlotData` pathway and have no
    /// hardcoded axis-label defaults — labels are fully configured in
    /// `plot_config.toml` or default to `"X Values"` / `"Y Values"`.
    GenericPlot,
}

/// Structured representation of every flag accepted by the binary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    /// Which plot kind to generate; `None` means `--help` was requested.
    pub plot_target: Option<PlotTarget>,
    /// Optional override for the plot-config TOML file path.
    pub plot_config_path: Option<PathBuf>,
    /// Target file or directory for the `--search-eis` workflow.
    pub search_target: Option<PathBuf>,
    /// Optional override for the ECM-search-config TOML file path.
    pub search_config_path: Option<PathBuf>,
    /// Optional output path (file or directory) for the search report.
    pub search_output: Option<PathBuf>,
    /// Optional override for the number of top-ranked candidates to keep.
    pub search_top: Option<usize>,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Parse the raw argument list (as returned by `std::env::args()`) into a
/// structured [`CliArgs`].
///
/// Returns `Ok(CliArgs { plot_target: None, .. })` when `--help` / `-h` is
/// passed; the caller should print usage and exit cleanly.
pub fn parse_cli_args(args: &[String]) -> Result<CliArgs, Box<dyn std::error::Error>> {
    let mut iter = args.iter().skip(1).peekable();
    let mut plot_target = PlotTarget::All;
    let mut plot_config_path = None;
    let mut search_target = None;
    let mut search_config_path = None;
    let mut search_output = None;
    let mut search_top = None;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                return Ok(CliArgs {
                    plot_target: None,
                    plot_config_path: None,
                    search_target: None,
                    search_config_path: None,
                    search_output: None,
                    search_top,
                });
            }
            "--plot" => {
                let value = iter
                    .next()
                    .ok_or("Missing value for --plot. Use eis, regular-plot, or all.")?;
                plot_target = match value.as_str() {
                    "all" => PlotTarget::All,
                    "eis" => PlotTarget::Eis,
                    "regular" | "regular-plot" | "pb" | "pb-sensor" | "chi" => {
                        PlotTarget::RegularPlot
                    }
                    "generic" | "generic-plot" => PlotTarget::GenericPlot,
                    _ => return Err(format!(
                        "Unsupported plot target '{value}'. Use eis, regular-plot, generic, or all."
                    )
                    .into()),
                };
            }
            "--plot-config" => {
                let value = iter
                    .next()
                    .ok_or("Missing value for --plot-config. Provide a path to a TOML file.")?;
                plot_config_path = Some(PathBuf::from(value));
            }
            "--search-eis" => {
                let value = iter
                    .next()
                    .ok_or("Missing value for --search-eis. Provide a file or directory path.")?;
                search_target = Some(PathBuf::from(value));
            }
            "--search-config" => {
                let value = iter
                    .next()
                    .ok_or("Missing value for --search-config. Provide a path to a TOML file.")?;
                search_config_path = Some(PathBuf::from(value));
            }
            "--search-output" => {
                let value = iter.next().ok_or(
                    "Missing value for --search-output. Provide a report path or directory.",
                )?;
                search_output = Some(PathBuf::from(value));
            }
            "--search-top" => {
                let value = iter
                    .next()
                    .ok_or("Missing value for --search-top. Provide a positive integer.")?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|error| format!("Invalid --search-top value '{value}': {error}"))?;
                if parsed == 0 {
                    return Err("--search-top must be greater than zero".into());
                }
                search_top = Some(parsed);
            }
            other => {
                return Err(format!(
                    "Unsupported argument '{other}'. Use --plot <eis|regular-plot|generic|all>, \
                     --search-eis <path>, --search-config <path>, --search-output <path>, \
                     --search-top <n>, --plot-config <path>, or --help."
                )
                .into());
            }
        }
    }

    // Cross-flag validation
    if search_target.is_some() && plot_config_path.is_some() {
        return Err("--plot-config is only used with plotting commands, not --search-eis".into());
    }
    if search_target.is_none() && search_config_path.is_some() {
        return Err("--search-config is only used with --search-eis".into());
    }
    if search_target.is_some() && args.iter().any(|arg| arg == "--plot") {
        return Err("Use either --plot or --search-eis in one invocation, not both".into());
    }

    Ok(CliArgs {
        // When search mode is active there is no plot target.
        plot_target: if search_target.is_some() {
            None
        } else {
            Some(plot_target)
        },
        plot_config_path,
        search_target,
        search_config_path,
        search_output,
        search_top,
    })
}

/// Print a brief synopsis to stdout.
pub fn print_usage(program: &str) {
    println!("Usage: {program} [--plot <eis|regular-plot|generic|all>] [--plot-config <path>]");
    println!(
        "       {program} --search-eis <file-or-dir> \
         [--search-config <path>] [--search-output <path>] [--search-top <n>]"
    );
    println!("  --plot eis           Generate only EIS figures");
    println!(
        "  --plot regular-plot  Generate only regular plots (pb-sensor is accepted as a legacy alias)"
    );
    println!("  --plot generic       Generate only generic (PlotData-based) plots");
    println!("  --plot all           Generate all figures: EIS, regular, and generic (default)");
    println!("  --plot-config path   Use an alternative plotting TOML file");
    println!(
        "  --search-eis path   Run equivalent-circuit discovery on one EIS file \
         or on all supported EIS files in a directory"
    );
    println!("  --search-config path Use an alternative analysis TOML file");
    println!(
        "  --search-output path Export the ranked search table; \
         for a directory search this is treated as an output directory"
    );
    println!("  --search-top n      Override the ranked-candidate limit from config/analysis.toml");
}

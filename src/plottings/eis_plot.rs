//! EIS plotting and fit-report rendering.
//!
//! Produces Nyquist/Bode figures for single files, directories, and ranked
//! ECM-search outputs while reusing the shared renderer configuration model.

use crate::DEFAULT_LOG_BASE;
use crate::chi_file::{EISData, EISFitResult};
use crate::impedance::{CircuitModelResolver, EcmSearchReport, RankedEcmCandidate};
use crate::plottings::plotting::{
    AxisScale, PlotAxisScale, PlotColor, PlotLegendPosition, PublicationConfig,
    plot_rendered_series_hq, plot_rendered_series_panels_hq,
};

use std::error::Error;
use std::fs;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

const BASELINE_CIRCUIT_MODEL: &str = "R0-p(CPE1,R1)";
const WARBURG_CIRCUIT_MODEL: &str = "R0-p(CPE1,R1)-W2";
const GENERALIZED_WARBURG_CIRCUIT_MODEL: &str = "R0-p(CPE1,R1)-Gw2";

/// Output summary for a single EIS input file.
#[derive(Debug, Clone)]
pub struct EISPlotOutcome {
    pub input_file: PathBuf,
    pub output_base: PathBuf,
    pub data: EISData,
    pub fit_report_path: PathBuf,
}

/// Output summary for directory-mode EIS plotting.
#[derive(Debug, Clone)]
pub struct EISDirectoryPlotOutcome {
    pub plotted: Vec<EISPlotOutcome>,
    pub combined_output_base: PathBuf,
}

/// Output summary for ranked ECM-search overlay/individual plots.
#[derive(Debug, Clone)]
pub struct RankedSearchPlotOutcome {
    pub output_base: PathBuf,
    pub plotted_candidates: usize,
    pub nyquist_overlay_path: PathBuf,
    pub magnitude_overlay_path: PathBuf,
    pub phase_overlay_path: PathBuf,
    pub individual_output_bases: Vec<PathBuf>,
}

pub fn eis_individual_publication_config() -> PublicationConfig {
    PublicationConfig {
        width_inches: 7.8,
        height_inches: 5.8,
        font_size_pt: 23.0,
        line_width: 3,
        plot_ratio_x: 10.0,
        plot_ratio_y: 6.8,
        legend_font_ratio: 0.68,
        png_font: "Arial Bold".to_string(),
        svg_font: "Arial".to_string(),
        experimental_marker_radius: 6,
        experimental_color: Some(PlotColor::rgb(22, 94, 131)),
        series_palette: vec![],
        fitted_line_width: Some(4),
        fitted_color: Some(PlotColor::rgba(210, 105, 30, 0.80)),
        legend_position: PlotLegendPosition::UpperLeft,
        ..Default::default()
    }
}

pub fn eis_combined_publication_config() -> PublicationConfig {
    PublicationConfig {
        width_inches: 8.6,
        height_inches: 6.0,
        font_size_pt: 22.0,
        line_width: 3,
        plot_ratio_x: 10.5,
        plot_ratio_y: 6.6,
        legend_font_ratio: 0.62,
        png_font: "Arial Bold".to_string(),
        svg_font: "Arial".to_string(),
        experimental_marker_radius: 5,
        series_palette: vec![
            PlotColor::rgb(22, 94, 131),
            PlotColor::rgb(178, 80, 25),
            PlotColor::rgb(46, 125, 50),
            PlotColor::rgb(111, 66, 193),
        ],
        fitted_line_width: Some(4),
        legend_position: PlotLegendPosition::LowerRight,
        ..Default::default()
    }
}

pub fn eis_publication_config() -> PublicationConfig {
    eis_individual_publication_config()
}

fn ranked_candidate_to_fit_result(candidate: &RankedEcmCandidate) -> EISFitResult {
    EISFitResult {
        circuit_model: format!("rank {}: {}", candidate.rank, candidate.circuit_string),
        fitted_parameters: candidate.fitted_parameters.clone(),
        parameter_names: candidate.parameter_names.clone(),
        parameter_units: candidate.parameter_units.clone(),
        fitted_z_re: candidate.fitted_z_re.clone(),
        fitted_z_im: candidate.fitted_z_im.clone(),
        fitted_magnitude: candidate.fitted_magnitude.clone(),
        fitted_phase: candidate.fitted_phase.clone(),
    }
}

fn ranked_search_fits(report: &EcmSearchReport, top_n: usize) -> Vec<EISFitResult> {
    report
        .ranked_candidates
        .iter()
        .take(top_n.min(report.ranked_candidates.len()))
        .map(ranked_candidate_to_fit_result)
        .collect()
}

pub fn best_ranked_search_fit(report: &EcmSearchReport) -> Option<EISFitResult> {
    report
        .ranked_candidates
        .first()
        .map(ranked_candidate_to_fit_result)
}

fn nyquist_plot_config(config: &PublicationConfig) -> PublicationConfig {
    config
        .clone()
        .with_default_axis_labels("Z' (Ohm)", "-Z'' (Ohm)")
}

fn bode_plot_config(config: &PublicationConfig, y_label: &str) -> PublicationConfig {
    let resolved = config
        .clone()
        .with_default_axis_scales(
            Some(AxisScale::Log {
                base: DEFAULT_LOG_BASE,
            }),
            None,
        )
        .with_default_scientific_notation(Some(false), None);
    let use_log_tick_exponents = matches!(resolved.x_scale, AxisScale::Log { .. })
        && !(resolved.sci_notation_x_is_explicit && resolved.sci_notation_x);
    let resolved = resolved.with_default_log_tick_exponents(Some(use_log_tick_exponents), None);
    let x_label = bode_frequency_axis_label(resolved.x_scale, resolved.x_log_ticks_as_exponents);

    resolved.with_default_axis_labels(x_label.as_str(), y_label)
}

fn bode_frequency_axis_label(x_scale: AxisScale, log_ticks_as_exponents: bool) -> String {
    if !log_ticks_as_exponents {
        return "Frequency (Hz)".to_string();
    }

    match x_scale {
        AxisScale::Linear => "Frequency (Hz)".to_string(),
        AxisScale::Log { base } => format!("log{}(Frequency / Hz)", format_log_base(base)),
    }
}

fn format_log_base(base: f64) -> String {
    if !base.is_finite() {
        return base.to_string();
    }
    if (base - DEFAULT_LOG_BASE).abs() < 1e-12 {
        return "e".to_string();
    }

    let mut rendered = format!("{base:.6}");
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
    }
    rendered
}

fn plot_eis_series_hq(
    figname: &str,
    rendered_series: &[Vec<crate::plottings::PlotSeries>],
    config: &PublicationConfig,
    plot_all_in_one: bool,
) -> Result<(), Box<dyn Error>> {
    plot_rendered_series_hq(
        figname,
        rendered_series,
        config,
        plot_all_in_one,
        PlotAxisScale::Linear,
    )
}

fn plot_eis_series_panels_hq(
    figname: &str,
    rendered_series: &[Vec<crate::plottings::PlotSeries>],
    config: &PublicationConfig,
) -> Result<(), Box<dyn Error>> {
    plot_rendered_series_panels_hq(figname, rendered_series, config, PlotAxisScale::Linear)
}

pub fn plot_ranked_search_report<P: AsRef<Path>>(
    data: &EISData,
    report: &EcmSearchReport,
    output_base: P,
    config: &PublicationConfig,
    top_n: usize,
) -> Result<RankedSearchPlotOutcome, Box<dyn Error>> {
    let output_base = output_base.as_ref().to_path_buf();
    let plotted_candidates = top_n.min(report.ranked_candidates.len());

    if plotted_candidates == 0 {
        return Err("no ranked candidates available to plot".into());
    }

    let fits = ranked_search_fits(report, plotted_candidates);

    let nyquist_config = nyquist_plot_config(config);
    let magnitude_config = bode_plot_config(config, "|Z| (Ohm)");
    let phase_config = bode_plot_config(config, "Phase (deg)");

    let nyquist_overlay_path = append_output_suffix(&output_base, "_nyquist_overlay");
    plot_eis_series_hq(
        nyquist_overlay_path.to_string_lossy().as_ref(),
        &[data.nyquist_series_for_fits(&fits)],
        &nyquist_config,
        true,
    )?;

    let magnitude_overlay_path = append_output_suffix(&output_base, "_bode_magnitude_overlay");
    plot_eis_series_hq(
        magnitude_overlay_path.to_string_lossy().as_ref(),
        &[data.bode_magnitude_series_for_fits(&fits)],
        &magnitude_config,
        true,
    )?;

    let phase_overlay_path = append_output_suffix(&output_base, "_bode_phase_overlay");
    plot_eis_series_hq(
        phase_overlay_path.to_string_lossy().as_ref(),
        &[data.bode_phase_series_for_fits(&fits)],
        &phase_config,
        true,
    )?;

    let mut individual_output_bases = Vec::with_capacity(plotted_candidates);
    for (index, fit) in fits.iter().enumerate() {
        let rank = report.ranked_candidates[index].rank;
        let individual_output_base =
            append_output_suffix(&output_base, &format!("_rank_{rank:02}"));
        plot_eis_series_hq(
            append_output_suffix(&individual_output_base, "_nyquist")
                .to_string_lossy()
                .as_ref(),
            &[data.nyquist_series_for_fit(fit)],
            &nyquist_config,
            true,
        )?;
        plot_eis_series_hq(
            append_output_suffix(&individual_output_base, "_bode_magnitude")
                .to_string_lossy()
                .as_ref(),
            &[data.bode_magnitude_series_for_fit(fit)],
            &magnitude_config,
            true,
        )?;
        plot_eis_series_hq(
            append_output_suffix(&individual_output_base, "_bode_phase")
                .to_string_lossy()
                .as_ref(),
            &[data.bode_phase_series_for_fit(fit)],
            &phase_config,
            true,
        )?;
        individual_output_bases.push(individual_output_base);
    }

    Ok(RankedSearchPlotOutcome {
        output_base,
        plotted_candidates,
        nyquist_overlay_path,
        magnitude_overlay_path,
        phase_overlay_path,
        individual_output_bases,
    })
}

// Use Instructions:
// 1. Call `plot_eis_file` for individual EIS data files, providing the file path, output base path, plot configuration, and optionally a circuit model.
pub fn plot_eis_file<P: AsRef<Path>>(
    file_path: P,
    output_base: P,
    config: &PublicationConfig,
    circuit_model: Option<&str>,
) -> Result<EISPlotOutcome, Box<dyn Error>> {
    let input_file = file_path.as_ref().to_path_buf();
    let output_base = output_base.as_ref().to_path_buf();
    let resolver = CircuitModelResolver::load_or_default()?;
    let mut data = EISData::parse_file_with_resolver(&input_file, &resolver)?;
    let ranking_metric = resolver.model_selection.ranking_metric;
    let warburg_aic_threshold = resolver.model_selection.warburg_aic_threshold;

    if let Some(circuit_model) = circuit_model {
        data = data.with_circuit_model(circuit_model);
    }
    let candidate_models = candidate_circuit_models(&data.circuit_model);
    let candidate_fits = candidate_models
        .iter()
        .map(|model| data.fit_circuit_for_model(model))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| -> Box<dyn Error> { err.into() })?;
    let ranked_fits = data.ranked_fits_by(&candidate_fits, ranking_metric);
    let preferred_idx =
        data.preferred_fit_index(&ranked_fits, ranking_metric, warburg_aic_threshold);
    let primary_fit = ranked_fits[preferred_idx].fit.clone();
    let comparison_fit = ranked_fits
        .iter()
        .enumerate()
        .find(|(idx, _)| *idx != preferred_idx)
        .map(|(_, ranked_fit)| ranked_fit.fit.clone())
        .or_else(|| ranked_fits.get(1).map(|ranked_fit| ranked_fit.fit.clone()))
        .or_else(|| ranked_fits.first().map(|ranked_fit| ranked_fit.fit.clone()))
        .unwrap_or_else(|| primary_fit.clone());
    let fits: Vec<_> = ranked_fits
        .iter()
        .map(|ranked_fit| ranked_fit.fit.clone())
        .collect();

    let nyquist_config = nyquist_plot_config(config);
    let magnitude_config = bode_plot_config(config, "|Z| (Ohm)");
    let phase_config = bode_plot_config(config, "Phase (deg)");

    let nyquist_output = append_output_suffix(&output_base, "_nyquist");

    plot_eis_series_hq(
        nyquist_output.to_string_lossy().as_ref(),
        &[data.nyquist_series_for_fit(&primary_fit)],
        &nyquist_config,
        true,
    )?;

    let comparison_nyquist_output = append_output_suffix(&output_base, "_nyquist_comparison");
    plot_eis_series_panels_hq(
        comparison_nyquist_output.to_string_lossy().as_ref(),
        &[
            data.nyquist_series_for_fit(&primary_fit),
            data.nyquist_series_for_fit(&comparison_fit),
        ],
        &nyquist_config,
    )?;

    let overlay_nyquist_output = append_output_suffix(&output_base, "_nyquist_overlay");
    plot_eis_series_hq(
        overlay_nyquist_output.to_string_lossy().as_ref(),
        &[data.nyquist_series_for_fits(&fits)],
        &nyquist_config,
        true,
    )?;

    let magnitude_output = append_output_suffix(&output_base, "_bode_magnitude");
    plot_eis_series_hq(
        magnitude_output.to_string_lossy().as_ref(),
        &[data.bode_magnitude_series_for_fit(&primary_fit)],
        &magnitude_config,
        true,
    )?;

    let magnitude_comparison_output =
        append_output_suffix(&output_base, "_bode_magnitude_comparison");
    plot_eis_series_panels_hq(
        magnitude_comparison_output.to_string_lossy().as_ref(),
        &[
            data.bode_magnitude_series_for_fit(&primary_fit),
            data.bode_magnitude_series_for_fit(&comparison_fit),
        ],
        &magnitude_config,
    )?;

    let phase_output = append_output_suffix(&output_base, "_bode_phase");
    plot_eis_series_hq(
        phase_output.to_string_lossy().as_ref(),
        &[data.bode_phase_series_for_fit(&primary_fit)],
        &phase_config,
        true,
    )?;

    let phase_comparison_output = append_output_suffix(&output_base, "_bode_phase_comparison");
    plot_eis_series_panels_hq(
        phase_comparison_output.to_string_lossy().as_ref(),
        &[
            data.bode_phase_series_for_fit(&primary_fit),
            data.bode_phase_series_for_fit(&comparison_fit),
        ],
        &phase_config,
    )?;

    let fit_report_path = append_output_suffix(&output_base, "_fit_report.txt");
    fs::write(
        &fit_report_path,
        data.format_fit_report(&fits, ranking_metric, warburg_aic_threshold),
    )?;

    Ok(EISPlotOutcome {
        input_file,
        output_base,
        data,
        fit_report_path,
    })
}

fn append_output_suffix(base: &Path, suffix: &str) -> PathBuf {
    let parent = base.parent().unwrap_or_else(|| Path::new(""));
    let filename = base
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("eis_plot");
    parent.join(format!("{filename}{suffix}"))
}

fn compose_output_base_name(prefix: &str, stem: &str, fallback: &str) -> String {
    let trimmed_prefix = prefix.trim();
    let trimmed_stem = stem.trim();

    match (trimmed_prefix.is_empty(), trimmed_stem.is_empty()) {
        (true, true) => fallback.to_string(),
        (true, false) => trimmed_stem.to_string(),
        (false, true) => trimmed_prefix.to_string(),
        (false, false) => format!("{trimmed_prefix}_{trimmed_stem}"),
    }
}

fn candidate_circuit_models(primary_model: &str) -> Vec<&'static str> {
    let normalized = normalize_model_name(primary_model);
    let warburg = normalize_model_name(WARBURG_CIRCUIT_MODEL);
    let generalized_warburg = normalize_model_name(GENERALIZED_WARBURG_CIRCUIT_MODEL);

    if normalized == generalized_warburg || normalized == warburg {
        vec![
            BASELINE_CIRCUIT_MODEL,
            WARBURG_CIRCUIT_MODEL,
            GENERALIZED_WARBURG_CIRCUIT_MODEL,
        ]
    } else {
        vec![BASELINE_CIRCUIT_MODEL, WARBURG_CIRCUIT_MODEL]
    }
}

fn normalize_model_name(model: &str) -> String {
    model
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect::<String>()
}

// 2. Call `plot_eis_directory` to process all EIS data files in a specified directory, providing the input directory, output directory, output file prefix, plot configuration, and optionally a circuit model.
pub fn plot_eis_directory<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    output_prefix: &str,
    config: &PublicationConfig,
    circuit_model: Option<&str>,
) -> Result<Vec<EISPlotOutcome>, Box<dyn Error>> {
    Ok(plot_eis_directory_with_configs(
        input_dir,
        output_dir,
        output_prefix,
        config,
        config,
        circuit_model,
    )?
    .plotted)
}

pub fn plot_eis_directory_with_configs<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    output_prefix: &str,
    individual_config: &PublicationConfig,
    combined_config: &PublicationConfig,
    circuit_model: Option<&str>,
) -> Result<EISDirectoryPlotOutcome, Box<dyn Error>> {
    let input_dir = input_dir.as_ref();
    let output_dir = output_dir.as_ref();
    let mut entries = read_dir(input_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    let mut outcomes = Vec::new();
    let mut nyquist_series = Vec::new();
    let mut magnitude_series = Vec::new();
    let mut phase_series = Vec::new();
    let combined_output_base =
        output_dir
            .join("combined")
            .join(compose_output_base_name(output_prefix, "all", "all"));
    let resolver = CircuitModelResolver::load_or_default()?;
    let ranking_metric = resolver.model_selection.ranking_metric;
    let warburg_aic_threshold = resolver.model_selection.warburg_aic_threshold;

    for entry in entries {
        let path = entry.path();
        // Skip non-file entries (e.g., directories)
        // Skip files that don't have a .csv extension (assuming EIS data files are in CSV format)
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("eis_plot");
        let output_base =
            output_dir.join(compose_output_base_name(output_prefix, stem, "eis_plot"));
        let mut data = EISData::parse_file_with_resolver(&path, &resolver)?;

        if let Some(circuit_model) = circuit_model {
            data = data.with_circuit_model(circuit_model);
        }
        let candidate_models = candidate_circuit_models(&data.circuit_model);
        let candidate_fits = candidate_models
            .iter()
            .map(|model| data.fit_circuit_for_model(model))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| -> Box<dyn Error> { err.into() })?;
        let ranked_fits = data.ranked_fits_by(&candidate_fits, ranking_metric);
        let preferred_idx =
            data.preferred_fit_index(&ranked_fits, ranking_metric, warburg_aic_threshold);
        let fit = ranked_fits[preferred_idx].fit.clone();
        let comparison_fit = ranked_fits
            .iter()
            .enumerate()
            .find(|(idx, _)| *idx != preferred_idx)
            .map(|(_, ranked_fit)| ranked_fit.fit.clone())
            .or_else(|| ranked_fits.get(1).map(|ranked_fit| ranked_fit.fit.clone()))
            .or_else(|| ranked_fits.first().map(|ranked_fit| ranked_fit.fit.clone()))
            .unwrap_or_else(|| fit.clone());
        let fits: Vec<_> = ranked_fits
            .iter()
            .map(|ranked_fit| ranked_fit.fit.clone())
            .collect();

        let nyquist_config = nyquist_plot_config(individual_config);
        let magnitude_config = bode_plot_config(individual_config, "|Z| (Ohm)");
        let phase_config = bode_plot_config(individual_config, "Phase (deg)");

        let nyquist_output = append_output_suffix(&output_base, "_nyquist");
        plot_eis_series_hq(
            nyquist_output.to_string_lossy().as_ref(),
            &[data.nyquist_series_for_fit(&fit)],
            &nyquist_config,
            true,
        )?;

        let comparison_nyquist_output = append_output_suffix(&output_base, "_nyquist_comparison");
        plot_eis_series_panels_hq(
            comparison_nyquist_output.to_string_lossy().as_ref(),
            &[
                data.nyquist_series_for_fit(&fit),
                data.nyquist_series_for_fit(&comparison_fit),
            ],
            &nyquist_config,
        )?;

        let overlay_nyquist_output = append_output_suffix(&output_base, "_nyquist_overlay");
        plot_eis_series_hq(
            overlay_nyquist_output.to_string_lossy().as_ref(),
            &[data.nyquist_series_for_fits(&fits)],
            &nyquist_config,
            true,
        )?;

        let magnitude_output = append_output_suffix(&output_base, "_bode_magnitude");
        plot_eis_series_hq(
            magnitude_output.to_string_lossy().as_ref(),
            &[data.bode_magnitude_series_for_fit(&fit)],
            &magnitude_config,
            true,
        )?;

        let magnitude_comparison_output =
            append_output_suffix(&output_base, "_bode_magnitude_comparison");
        plot_eis_series_panels_hq(
            magnitude_comparison_output.to_string_lossy().as_ref(),
            &[
                data.bode_magnitude_series_for_fit(&fit),
                data.bode_magnitude_series_for_fit(&comparison_fit),
            ],
            &magnitude_config,
        )?;

        let phase_output = append_output_suffix(&output_base, "_bode_phase");
        plot_eis_series_hq(
            phase_output.to_string_lossy().as_ref(),
            &[data.bode_phase_series_for_fit(&fit)],
            &phase_config,
            true,
        )?;

        let phase_comparison_output = append_output_suffix(&output_base, "_bode_phase_comparison");
        plot_eis_series_panels_hq(
            phase_comparison_output.to_string_lossy().as_ref(),
            &[
                data.bode_phase_series_for_fit(&fit),
                data.bode_phase_series_for_fit(&comparison_fit),
            ],
            &phase_config,
        )?;

        let fit_report_path = append_output_suffix(&output_base, "_fit_report.txt");
        fs::write(
            &fit_report_path,
            data.format_fit_report(&fits, ranking_metric, warburg_aic_threshold),
        )?;

        nyquist_series.push(data.nyquist_series_for_fit(&fit));
        magnitude_series.push(data.bode_magnitude_series_for_fit(&fit));
        phase_series.push(data.bode_phase_series_for_fit(&fit));

        let outcome = EISPlotOutcome {
            input_file: path,
            output_base,
            data,
            fit_report_path,
        };
        outcomes.push(outcome);
    }

    if outcomes.is_empty() {
        return Err(format!("No valid EIS datasets found in {}", input_dir.display()).into());
    }

    let nyquist_combined_config = nyquist_plot_config(combined_config);
    let magnitude_combined_config = bode_plot_config(combined_config, "|Z| (Ohm)");
    let phase_combined_config = bode_plot_config(combined_config, "Phase (deg)");

    let combined_nyquist_output = append_output_suffix(&combined_output_base, "_nyquist");
    plot_eis_series_hq(
        combined_nyquist_output.to_string_lossy().as_ref(),
        &nyquist_series,
        &nyquist_combined_config,
        true,
    )?;

    let combined_magnitude_output = append_output_suffix(&combined_output_base, "_bode_magnitude");
    plot_eis_series_hq(
        combined_magnitude_output.to_string_lossy().as_ref(),
        &magnitude_series,
        &magnitude_combined_config,
        true,
    )?;

    let combined_phase_output = append_output_suffix(&combined_output_base, "_bode_phase");
    plot_eis_series_hq(
        combined_phase_output.to_string_lossy().as_ref(),
        &phase_series,
        &phase_combined_config,
        true,
    )?;

    Ok(EISDirectoryPlotOutcome {
        plotted: outcomes,
        combined_output_base,
    })
}

#[cfg(test)]
mod tests {
    use super::{bode_plot_config, nyquist_plot_config};
    use crate::DEFAULT_LOG_BASE;
    use crate::plottings::{AxisScale, PublicationConfig};

    #[test]
    fn bode_plots_default_to_natural_log_frequency_axis() {
        let cfg = bode_plot_config(&PublicationConfig::default(), "|Z| (Ohm)");

        match cfg.x_scale {
            AxisScale::Log { base } => assert!((base - DEFAULT_LOG_BASE).abs() < 1e-10),
            AxisScale::Linear => panic!("expected Bode x-axis to default to natural log"),
        }
        assert!(!cfg.sci_notation_x);
        assert!(cfg.x_log_ticks_as_exponents);
        assert_eq!(cfg.x_label, "loge(Frequency / Hz)");
        assert_eq!(cfg.y_label, "|Z| (Ohm)");
    }

    #[test]
    fn bode_plots_respect_explicit_axis_scale_overrides() {
        let cfg = PublicationConfig {
            x_scale: AxisScale::Linear,
            x_scale_is_explicit: true,
            ..PublicationConfig::default()
        };

        let resolved = bode_plot_config(&cfg, "Phase (deg)");

        assert!(matches!(resolved.x_scale, AxisScale::Linear));
        assert!(!resolved.x_log_ticks_as_exponents);
        assert_eq!(resolved.x_label, "Frequency (Hz)");
        assert_eq!(resolved.y_label, "Phase (deg)");
    }

    #[test]
    fn bode_plots_respect_explicit_scientific_notation_overrides() {
        let cfg = PublicationConfig {
            sci_notation_x: true,
            sci_notation_x_is_explicit: true,
            ..PublicationConfig::default()
        };

        let resolved = bode_plot_config(&cfg, "|Z| (Ohm)");

        assert!(resolved.sci_notation_x);
        assert!(!resolved.x_log_ticks_as_exponents);
        match resolved.x_scale {
            AxisScale::Log { base } => assert!((base - DEFAULT_LOG_BASE).abs() < 1e-10),
            AxisScale::Linear => panic!("expected Bode x-axis to remain logarithmic"),
        }
        assert_eq!(resolved.x_label, "Frequency (Hz)");
    }

    #[test]
    fn bode_plots_reflect_explicit_log_base_in_default_axis_label() {
        let cfg = PublicationConfig {
            x_scale: AxisScale::Log { base: 2.0 },
            x_scale_is_explicit: true,
            sci_notation_x: false,
            sci_notation_x_is_explicit: true,
            ..PublicationConfig::default()
        };

        let resolved = bode_plot_config(&cfg, "Phase (deg)");

        match resolved.x_scale {
            AxisScale::Log { base } => assert!((base - 2.0).abs() < 1e-10),
            AxisScale::Linear => panic!("expected Bode x-axis to remain logarithmic"),
        }
        assert!(resolved.x_log_ticks_as_exponents);
        assert_eq!(resolved.x_label, "log2(Frequency / Hz)");
    }

    #[test]
    fn nyquist_plot_keeps_linear_axis_defaults() {
        let cfg = nyquist_plot_config(&PublicationConfig::default());

        assert!(matches!(cfg.x_scale, AxisScale::Linear));
        assert!(matches!(cfg.y_scale, AxisScale::Linear));
        assert_eq!(cfg.x_label, "Z' (Ohm)");
        assert_eq!(cfg.y_label, "-Z'' (Ohm)");
    }
}

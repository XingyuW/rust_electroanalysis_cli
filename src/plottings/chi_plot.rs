//! CHI (regular plot) rendering pipeline.
//!
//! This module wraps `ElectrochemData` parsing and delegates figure generation
//! to the shared high-quality plotting engine with CHI-specific defaults.

use crate::data_file::ElectrochemData;
use crate::data_file::value_transform::AxisTransforms;
use crate::plottings::plotting::{PlotColor, PlotLegendPosition, PublicationConfig, plot_hq};

use std::error::Error;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

/// Result for one successful regular-plot render.
#[derive(Debug, Clone)]
pub struct ChiPlotOutcome {
    pub input_file: PathBuf,
    pub output_base: PathBuf,
    pub data: ElectrochemData,
}

/// One skipped input encountered during directory-mode plotting.
#[derive(Debug, Clone)]
pub struct ChiPlotSkip {
    pub input_file: PathBuf,
    pub reason: String,
}

/// Aggregated summary for a directory regular-plot run.
#[derive(Debug, Clone)]
pub struct ChiDirectoryPlotOutcome {
    pub plotted: Vec<ChiPlotOutcome>,
    pub skipped: Vec<ChiPlotSkip>,
    pub combined_output_base: PathBuf,
    pub individual_output_base: PathBuf,
}

pub fn pb_sensor_individual_publication_config() -> PublicationConfig {
    PublicationConfig {
        width_inches: 8.4,
        height_inches: 5.8,
        font_size_pt: 22.0,
        line_width: 3,
        plot_ratio_x: 10.6,
        plot_ratio_y: 6.2,
        legend_font_ratio: 0.62,
        x_label: "Time (s)".to_string(),
        y_label: "Potential (V)".to_string(),
        png_font: "Arial Bold".to_string(),
        svg_font: "Arial".to_string(),
        experimental_marker_radius: 5,
        experimental_color: Some(PlotColor::rgb(29, 78, 137)),
        series_palette: vec![],
        legend_position: PlotLegendPosition::UpperRight,
        ..Default::default()
    }
}

pub fn pb_sensor_combined_publication_config() -> PublicationConfig {
    PublicationConfig {
        width_inches: 9.2,
        height_inches: 6.0,
        font_size_pt: 22.0,
        line_width: 3,
        plot_ratio_x: 11.0,
        plot_ratio_y: 6.2,
        legend_font_ratio: 0.56,
        x_label: "Time (s)".to_string(),
        y_label: "Potential (V)".to_string(),
        png_font: "Arial Bold".to_string(),
        svg_font: "Arial".to_string(),
        experimental_marker_radius: 4,
        series_palette: vec![
            PlotColor::rgb(29, 78, 137),
            PlotColor::rgb(198, 64, 52),
            PlotColor::rgb(61, 135, 97),
            PlotColor::rgb(122, 81, 149),
            PlotColor::rgb(232, 136, 39),
            PlotColor::rgb(0, 128, 128),
            PlotColor::rgb(188, 108, 37),
            PlotColor::rgb(87, 117, 144),
            PlotColor::rgb(173, 32, 32),
        ],
        legend_position: PlotLegendPosition::LowerLeft,
        ..Default::default()
    }
}

pub fn pb_sensor_publication_config() -> PublicationConfig {
    pb_sensor_combined_publication_config()
}

pub fn plot_chi_file<P: AsRef<Path>>(
    file_path: P,
    output_base: P,
    config: &PublicationConfig,
) -> Result<ChiPlotOutcome, Box<dyn Error>> {
    plot_chi_file_with_transform(file_path, output_base, config, &AxisTransforms::default())
}

pub fn plot_chi_file_with_transform<P: AsRef<Path>>(
    file_path: P,
    output_base: P,
    config: &PublicationConfig,
    transform: &AxisTransforms,
) -> Result<ChiPlotOutcome, Box<dyn Error>> {
    let input_file = file_path.as_ref().to_path_buf();
    let output_base = output_base.as_ref().to_path_buf();
    let data = ElectrochemData::parse_file(&input_file)?;
    let mut transformed = vec![data.clone()];
    apply_axis_transforms_to_electrochem(&mut transformed, transform);

    let plot_config = config
        .clone()
        .with_default_axis_labels("Time (s)", "Potential (V)");
    plot_hq(
        output_base.to_string_lossy().as_ref(),
        &transformed,
        &plot_config,
        true,
    )?;

    Ok(ChiPlotOutcome {
        input_file,
        output_base,
        data,
    })
}

pub fn plot_chi_directory<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    output_prefix: &str,
    config: &PublicationConfig,
) -> Result<ChiDirectoryPlotOutcome, Box<dyn Error>> {
    plot_chi_directory_with_configs(input_dir, output_dir, output_prefix, config, config)
}

pub fn plot_chi_directory_with_configs<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    output_prefix: &str,
    individual_config: &PublicationConfig,
    combined_config: &PublicationConfig,
) -> Result<ChiDirectoryPlotOutcome, Box<dyn Error>> {
    plot_chi_directory_with_configs_and_transforms(
        input_dir,
        output_dir,
        output_prefix,
        individual_config,
        combined_config,
        &AxisTransforms::default(),
        &AxisTransforms::default(),
    )
}

pub fn plot_chi_directory_with_configs_and_transforms<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    output_prefix: &str,
    individual_config: &PublicationConfig,
    combined_config: &PublicationConfig,
    individual_transform: &AxisTransforms,
    combined_transform: &AxisTransforms,
) -> Result<ChiDirectoryPlotOutcome, Box<dyn Error>> {
    let input_dir = input_dir.as_ref();
    let output_dir = output_dir.as_ref();
    let individual_output_base = output_dir
        .join("individual")
        .join(base_output_name(output_prefix, "chi_plot"));
    let combined_output_base = output_dir
        .join("combined")
        .join(base_output_name_with_suffix(
            output_prefix,
            "overlay",
            "overlay",
        ));

    let mut entries = read_dir(input_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    let mut plotted = Vec::new();
    let mut skipped = Vec::new();
    let mut datasets = Vec::new();

    for entry in entries {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if !is_supported_chi_path(&path) {
            skipped.push(ChiPlotSkip {
                input_file: path,
                reason: "Unsupported file type".to_string(),
            });
            continue;
        }

        match ElectrochemData::parse_file_series(&path) {
            Ok(parsed_series) => {
                for data in parsed_series {
                    let output_base = append_output_suffix(
                        &individual_output_base,
                        &sanitize_output_component(data.label.as_str()),
                    );
                    datasets.push(data.clone());
                    plotted.push(ChiPlotOutcome {
                        input_file: path.clone(),
                        output_base,
                        data,
                    });
                }
            }
            Err(err) => skipped.push(ChiPlotSkip {
                input_file: path,
                reason: err.to_string(),
            }),
        }
    }

    if datasets.is_empty() {
        return Err(format!("No valid CHI datasets found in {}", input_dir.display()).into());
    }

    let mut individual_datasets = datasets.clone();
    let mut combined_datasets = datasets;
    apply_axis_transforms_to_electrochem(&mut individual_datasets, individual_transform);
    apply_axis_transforms_to_electrochem(&mut combined_datasets, combined_transform);

    let individual_plot_config = individual_config
        .clone()
        .with_default_axis_labels("Time (s)", "Potential (V)");
    let combined_plot_config = combined_config
        .clone()
        .with_default_axis_labels("Time (s)", "Potential (V)");

    plot_hq(
        individual_output_base.to_string_lossy().as_ref(),
        &individual_datasets,
        &individual_plot_config,
        false,
    )?;
    plot_hq(
        combined_output_base.to_string_lossy().as_ref(),
        &combined_datasets,
        &combined_plot_config,
        true,
    )?;

    Ok(ChiDirectoryPlotOutcome {
        plotted,
        skipped,
        combined_output_base,
        individual_output_base,
    })
}

fn apply_axis_transforms_to_electrochem(
    datasets: &mut [ElectrochemData],
    transforms: &AxisTransforms,
) {
    if transforms.is_empty() {
        return;
    }
    for dataset in datasets.iter_mut() {
        if let Some(ref tx) = transforms.x {
            tx.apply_vec(&mut dataset.x_values);
        }
        if let Some(ref ty) = transforms.y {
            ty.apply_vec(&mut dataset.y_values);
        }
    }
}

fn is_supported_chi_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase()),
        Some(extension) if matches!(extension.as_str(), "csv" | "txt" | "dat")
    )
}

fn sanitize_output_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if ch.is_ascii_whitespace() || matches!(ch, '-' | '_' | '.') {
            out.push('_');
        }
    }

    let out = out.trim_matches('_');
    if out.is_empty() {
        "series".to_string()
    } else {
        out.to_string()
    }
}

fn append_output_suffix(base: &Path, suffix: &str) -> PathBuf {
    let parent = base.parent().unwrap_or_else(|| Path::new(""));
    let filename = base
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("chi_plot");
    parent.join(format!("{filename}_{suffix}"))
}

fn base_output_name(prefix: &str, fallback: &str) -> String {
    let trimmed_prefix = prefix.trim();
    if trimmed_prefix.is_empty() {
        fallback.to_string()
    } else {
        trimmed_prefix.to_string()
    }
}

fn base_output_name_with_suffix(prefix: &str, suffix: &str, fallback: &str) -> String {
    let trimmed_prefix = prefix.trim();
    if trimmed_prefix.is_empty() {
        fallback.to_string()
    } else {
        format!("{trimmed_prefix}_{suffix}")
    }
}

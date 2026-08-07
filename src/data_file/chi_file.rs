//! Parsers and adapters for CHI/EIS text exports.
//!
//! The functions in this file convert instrument-oriented CSV/TXT input into
//! strongly typed series (`ElectrochemData`, `EISData`) while preserving
//! metadata used later by plotting and ECM search reporting.

use crate::domain::{
    DataParsingError, FittingError, MeasurementChannel, MultiChannelMeasurement, ParseDiagnostics,
    PlottingError,
};
use crate::impedance;
use crate::impedance::{
    CircuitModelContext, CircuitModelResolver, FitRankingMetric, format_fitted_circuit_composition,
};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::Path;

use crate::plottings::{PlotDataSeries, PlotSeries};

fn normalize_header_name(name: &str) -> String {
    name.trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect()
}

fn build_electrochem_series_labels(base_label: &str, y_headers: &[String]) -> Vec<String> {
    let y_indices = 1..=y_headers.len();
    if y_headers.len() <= 1 {
        return vec![base_label.to_string()];
    }

    let normalized_headers: Vec<_> = y_headers
        .iter()
        .map(|header| normalize_header_name(header))
        .collect();
    let mut header_counts = BTreeMap::new();
    for normalized in &normalized_headers {
        if !normalized.is_empty() {
            *header_counts.entry(normalized.clone()).or_insert(0usize) += 1;
        }
    }

    y_headers
        .iter()
        .zip(y_indices)
        .zip(normalized_headers.iter())
        .map(|((header, y_index), normalized)| {
            let trimmed = header.trim();
            let is_duplicate =
                !normalized.is_empty() && header_counts.get(normalized).copied().unwrap_or(0) > 1;
            let suffix = if trimmed.is_empty() {
                format!("column {}", y_index + 1)
            } else if is_duplicate {
                format!("{trimmed} (col {})", y_index + 1)
            } else {
                trimmed.to_string()
            };
            format!("{base_label} - {suffix}")
        })
        .collect()
}

fn file_label<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

// ElectrochemData is a more general struct for parsing and representing electrochemical data from CHI files, which may include various test types beyond just EIS. It captures common metadata and provides a flexible structure for x-y data pairs, making it suitable for OCPT, CV, and other electrochemical techniques. The EISData struct is specifically tailored for impedance spectroscopy data, with additional fields and methods relevant to EIS analysis and fitting.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElectrochemData {
    pub date: String,
    pub test_type: String,
    pub instrument_model: String,
    pub x_values: Vec<f64>,
    pub y_values: Vec<f64>,
    pub label: String,
}

impl ElectrochemData {
    pub fn parse_file_with_diagnostics<P: AsRef<Path>>(
        path: P,
    ) -> Result<(Self, ParseDiagnostics), DataParsingError> {
        let path = path.as_ref();
        let diagnostics = crate::data_file::parse_measurement_file(path)?.diagnostics;
        Ok((Self::parse_file(path)?, diagnostics))
    }

    pub fn parse_file_series_with_diagnostics<P: AsRef<Path>>(
        path: P,
    ) -> Result<(Vec<Self>, ParseDiagnostics), DataParsingError> {
        let path = path.as_ref();
        let diagnostics = crate::data_file::parse_measurement_file(path)?.diagnostics;
        Ok((Self::parse_file_series(path)?, diagnostics))
    }

    pub fn to_multi_channel_measurement(
        &self,
    ) -> Result<MultiChannelMeasurement, DataParsingError> {
        if self.x_values.len() != self.y_values.len() {
            return Err(DataParsingError::invalid(format!(
                "electrochemical data has {} x values and {} y values",
                self.x_values.len(),
                self.y_values.len()
            )));
        }
        MultiChannelMeasurement::new(
            self.x_values.clone(),
            vec![MeasurementChannel::from_values(
                self.label.clone(),
                "",
                self.y_values.clone(),
            )],
        )
    }

    pub fn series_count<P: AsRef<Path>>(path: P) -> Result<usize, DataParsingError> {
        let path = path.as_ref();
        let dataset = crate::data_file::electrodata_domain_adapter::read_dataset(path)?;
        let count = dataset.measurement_channels().len();
        (count > 0)
            .then_some(count)
            .ok_or_else(|| DataParsingError::invalid_at(path, "missing time/potential header"))
    }

    pub fn parse_file_series<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, DataParsingError> {
        let path = path.as_ref();
        let dataset = crate::data_file::electrodata_domain_adapter::read_dataset(path)?;
        let view = dataset.time_series_view()?;
        let date = dataset
            .metadata
            .acquisition
            .recorded_at
            .map(|value| value.to_string())
            .unwrap_or_default();
        let test_type = dataset
            .metadata
            .acquisition
            .technique
            .clone()
            .unwrap_or_default();
        let instrument_model = dataset
            .metadata
            .acquisition
            .instrument_model
            .clone()
            .unwrap_or_default();
        // Preserve source coordinates for plotting. A scientific caller that
        // needs seconds must request and record that conversion explicitly.
        let x_values = view.coordinate_values().to_vec();
        let y_descriptors = view.measurements;
        let y_headers = y_descriptors
            .iter()
            .map(|column| {
                column
                    .original_name
                    .clone()
                    .unwrap_or_else(|| column.name.clone())
            })
            .collect::<Vec<_>>();
        let labels = build_electrochem_series_labels(&file_label(path), &y_headers);

        let mut datasets = Vec::new();
        for (descriptor, label) in y_descriptors.into_iter().zip(labels) {
            let y_values = descriptor.values.clone();
            let (x_values, y_values): (Vec<_>, Vec<_>) = x_values
                .iter()
                .zip(y_values)
                .filter_map(|(x, y)| (*x).zip(y))
                .unzip();
            if x_values.is_empty() || y_values.is_empty() {
                continue;
            }
            datasets.push(Self {
                date: date.clone(),
                test_type: test_type.clone(),
                instrument_model: instrument_model.clone(),
                x_values,
                y_values,
                label,
            });
        }

        if datasets.is_empty() {
            return Err(DataParsingError::invalid_at(
                path,
                "no numeric time/potential data rows found",
            ));
        }

        Ok(datasets)
    }

    #[allow(dead_code)]
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Self, DataParsingError> {
        let path = path.as_ref();
        Self::parse_file_series(path)?
            .into_iter()
            .next()
            .ok_or_else(|| {
                DataParsingError::invalid_at(path, "no numeric time/potential data rows found")
            })
    }
}

impl PlotDataSeries for ElectrochemData {
    fn label(&self) -> &str {
        &self.label
    }

    fn x_values(&self) -> &[f64] {
        &self.x_values
    }

    fn y_values(&self) -> &[f64] {
        &self.y_values
    }
}

impl TryFrom<ElectrochemData> for MultiChannelMeasurement {
    type Error = DataParsingError;

    fn try_from(data: ElectrochemData) -> Result<Self, Self::Error> {
        data.to_multi_channel_measurement()
    }
}

// EISData is a specialized struct for representing electrochemical impedance spectroscopy data parsed from CHI files. It includes fields for frequency, phase, real and imaginary impedance components, as well as metadata and circuit model information. The struct provides methods for parsing EIS-specific data formats, fitting to equivalent circuit models, and generating plot series for visualization. This struct is designed to encapsulate all relevant information and functionality needed for EIS analysis within the rust_plots library.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EISFitResult {
    pub circuit_model: String,
    pub fitted_parameters: Vec<f64>,
    pub parameter_names: Vec<String>,
    pub parameter_units: Vec<String>,
    pub fitted_z_re: Vec<f64>,
    pub fitted_z_im: Vec<f64>,
    pub fitted_magnitude: Vec<f64>,
    pub fitted_phase: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
pub struct EISFitMetrics {
    pub weighted_sse: f64,
    pub weighted_rmse: f64,
    pub aic: f64,
    pub real_rmse: f64,
    pub imag_rmse: f64,
    pub max_modulus_error: f64,
}

#[derive(Debug, Clone)]
pub struct RankedEISFit {
    pub fit: EISFitResult,
    pub metrics: EISFitMetrics,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EISData {
    /// Public fields are retained for pre-1.0 source compatibility. New code
    /// should use `from_impedance`, `with_source_bode`, and the accessors
    /// below so source-measured and derived Bode quantities cannot be
    /// accidentally conflated.
    pub date: String,
    pub test_type: String,
    pub instrument_model: String,
    pub freq: Vec<f64>,
    pub phase: Vec<f64>,
    pub z_re: Vec<f64>,
    pub z_im: Vec<f64>,
    /// Source-measured magnitude, including source null positions.
    pub measured_magnitude: Option<Vec<Option<f64>>>,
    /// Source-measured phase. `phase` remains a complete analysis vector and
    /// derives only where the supplied source phase is missing.
    pub measured_phase: Option<Vec<Option<f64>>>,
    /// Magnitude calculated from the source real/imaginary components.
    pub derived_magnitude: Vec<f64>,
    /// Phase calculated from the source real/imaginary components.
    pub derived_phase: Vec<f64>,
    pub label: String,
    pub metadata: BTreeMap<String, String>,
    pub circuit_model: String,
}

impl EISData {
    /// Supported public construction path for EIS data derived from complex
    /// impedance. Source magnitude/phase are absent until `with_source_bode`
    /// is explicitly called.
    pub fn from_impedance(freq: Vec<f64>, z_re: Vec<f64>, z_im: Vec<f64>) -> Self {
        let derived_magnitude = z_re
            .iter()
            .zip(&z_im)
            .map(|(real, imaginary)| real.hypot(*imaginary))
            .collect::<Vec<_>>();
        let derived_phase = z_re
            .iter()
            .zip(&z_im)
            .map(|(real, imaginary)| imaginary.atan2(*real).to_degrees())
            .collect::<Vec<_>>();
        Self {
            date: String::new(),
            test_type: String::new(),
            instrument_model: String::new(),
            freq,
            phase: derived_phase.clone(),
            z_re,
            z_im,
            measured_magnitude: None,
            measured_phase: None,
            derived_magnitude,
            derived_phase,
            label: String::new(),
            metadata: BTreeMap::new(),
            circuit_model: String::new(),
        }
    }

    /// Attaches optional source Bode columns without changing the complex-
    /// impedance-derived quantities. The complete legacy analysis `phase`
    /// vector uses a source value only where it is present.
    pub fn with_source_bode(
        mut self,
        measured_magnitude: Option<Vec<Option<f64>>>,
        measured_phase: Option<Vec<Option<f64>>>,
    ) -> Self {
        self.measured_magnitude = measured_magnitude;
        self.measured_phase = measured_phase;
        self.phase = self
            .derived_phase
            .iter()
            .enumerate()
            .map(|(index, derived)| {
                self.measured_phase
                    .as_ref()
                    .and_then(|values| values.get(index).copied().flatten())
                    .unwrap_or(*derived)
            })
            .collect();
        self
    }

    pub fn source_measured_magnitude(&self) -> Option<&[Option<f64>]> {
        self.measured_magnitude.as_deref()
    }

    pub fn source_measured_phase(&self) -> Option<&[Option<f64>]> {
        self.measured_phase.as_deref()
    }

    pub fn derived_magnitude(&self) -> &[f64] {
        &self.derived_magnitude
    }

    pub fn derived_phase(&self) -> &[f64] {
        &self.derived_phase
    }
    #[allow(dead_code)]
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Self, DataParsingError> {
        let resolver =
            CircuitModelResolver::load_or_default().map_err(DataParsingError::Configuration)?;
        Self::parse_file_with_resolver(path, &resolver)
    }

    pub fn parse_file_with_resolver<P: AsRef<Path>>(
        path: P,
        resolver: &CircuitModelResolver,
    ) -> Result<Self, DataParsingError> {
        Self::parse_file_with_resolver_and_sheet(path, resolver, None)
    }

    /// Canonical EIS ingestion with an optional provider worksheet selector.
    pub fn parse_file_with_resolver_and_sheet<P: AsRef<Path>>(
        path: P,
        resolver: &CircuitModelResolver,
        sheet_name: Option<&str>,
    ) -> Result<Self, DataParsingError> {
        let path = path.as_ref();
        let dataset = crate::data_file::electrodata_domain_adapter::read_dataset_with_sheet(
            path, sheet_name,
        )?;
        let mut data = Self::try_from(&dataset)?;
        let label = file_label(path);
        let context = CircuitModelContext {
            filename: label.clone(),
            metadata: data.metadata.clone(),
        };
        data.label = label;
        data.circuit_model = resolver.resolve(&context);
        Ok(data)
    }

    pub fn parse_file_with_sheet<P: AsRef<Path>>(
        path: P,
        sheet_name: Option<&str>,
    ) -> Result<Self, DataParsingError> {
        let resolver =
            CircuitModelResolver::load_or_default().map_err(DataParsingError::Configuration)?;
        Self::parse_file_with_resolver_and_sheet(path, &resolver, sheet_name)
    }

    pub fn with_circuit_model(mut self, circuit_model: impl Into<String>) -> Self {
        self.circuit_model = circuit_model.into();
        self
    }

    pub fn point_count(&self) -> usize {
        self.freq.len()
    }

    pub fn fit_circuit(&self) -> Result<EISFitResult, FittingError> {
        self.compute_fit_result()
    }

    pub fn fit_circuit_for_model(&self, circuit_model: &str) -> Result<EISFitResult, FittingError> {
        self.compute_fit_result_for_model(circuit_model)
    }

    pub fn fitted_parameters(&self) -> Result<Vec<f64>, FittingError> {
        Ok(self.compute_fit_result()?.fitted_parameters)
    }

    pub fn fit_metrics(&self, fit: &EISFitResult) -> EISFitMetrics {
        let mut weighted_sum = 0.0;
        let mut real_sum = 0.0;
        let mut imag_sum = 0.0;
        let mut max_modulus_error = 0.0_f64;
        let mut count = 0.0;

        for idx in 0..self
            .freq
            .len()
            .min(fit.fitted_z_re.len())
            .min(fit.fitted_z_im.len())
        {
            let re_residual = fit.fitted_z_re[idx] - self.z_re[idx];
            let im_residual = fit.fitted_z_im[idx] - self.z_im[idx];
            let weight = self.z_re[idx].hypot(self.z_im[idx]).max(1.0);

            weighted_sum += (re_residual / weight).powi(2) + (im_residual / weight).powi(2);
            real_sum += re_residual.powi(2);
            imag_sum += im_residual.powi(2);
            max_modulus_error = max_modulus_error.max(re_residual.hypot(im_residual));
            count += 1.0;
        }

        if count == 0.0 {
            return EISFitMetrics {
                weighted_sse: 0.0,
                weighted_rmse: 0.0,
                aic: f64::INFINITY,
                real_rmse: 0.0,
                imag_rmse: 0.0,
                max_modulus_error: 0.0,
            };
        }

        let residual_count = 2.0 * count;
        let parameter_count = fit.fitted_parameters.len() as f64;
        let rss_per_observation = (weighted_sum / residual_count).max(1e-30);
        let aic = residual_count * rss_per_observation.ln() + 2.0 * parameter_count;

        EISFitMetrics {
            weighted_sse: weighted_sum,
            weighted_rmse: (weighted_sum / residual_count).sqrt(),
            aic,
            real_rmse: (real_sum / count).sqrt(),
            imag_rmse: (imag_sum / count).sqrt(),
            max_modulus_error,
        }
    }

    pub fn ranked_fits(&self, fits: &[EISFitResult]) -> Vec<RankedEISFit> {
        self.ranked_fits_by(fits, FitRankingMetric::Aic)
    }

    pub fn ranked_fits_by(
        &self,
        fits: &[EISFitResult],
        ranking_metric: FitRankingMetric,
    ) -> Vec<RankedEISFit> {
        let mut ranked: Vec<_> = fits
            .iter()
            .cloned()
            .map(|fit| RankedEISFit {
                metrics: self.fit_metrics(&fit),
                fit,
            })
            .collect();

        ranked.sort_by(|lhs, rhs| {
            let lhs_score = match ranking_metric {
                FitRankingMetric::Aic => lhs.metrics.aic,
                FitRankingMetric::WeightedRmse => lhs.metrics.weighted_rmse,
            };
            let rhs_score = match ranking_metric {
                FitRankingMetric::Aic => rhs.metrics.aic,
                FitRankingMetric::WeightedRmse => rhs.metrics.weighted_rmse,
            };

            lhs_score.partial_cmp(&rhs_score).unwrap_or(Ordering::Equal)
        });

        ranked
    }

    pub fn preferred_fit_index(
        &self,
        ranked_fits: &[RankedEISFit],
        ranking_metric: FitRankingMetric,
        warburg_aic_threshold: f64,
    ) -> usize {
        if ranked_fits.is_empty() {
            return 0;
        }

        if ranking_metric != FitRankingMetric::Aic {
            return 0;
        }

        let best = &ranked_fits[0];
        if !is_warburg_extended_model(&best.fit.circuit_model) {
            return 0;
        }

        let baseline_idx = ranked_fits
            .iter()
            .position(|ranked_fit| !is_warburg_extended_model(&ranked_fit.fit.circuit_model));

        let Some(baseline_idx) = baseline_idx else {
            return 0;
        };

        let baseline = &ranked_fits[baseline_idx];
        let aic_improvement = baseline.metrics.aic - best.metrics.aic;
        if aic_improvement >= warburg_aic_threshold {
            0
        } else {
            baseline_idx
        }
    }

    pub fn format_fit_report(
        &self,
        fits: &[EISFitResult],
        ranking_metric: FitRankingMetric,
        warburg_aic_threshold: f64,
    ) -> String {
        let mut report = String::new();
        let ranked_fits = self.ranked_fits_by(fits, ranking_metric);
        let preferred_idx =
            self.preferred_fit_index(&ranked_fits, ranking_metric, warburg_aic_threshold);
        let (ranking_metric_label, alternate_metric_label) = match ranking_metric {
            FitRankingMetric::Aic => ("AIC", "weighted RMSE"),
            FitRankingMetric::WeightedRmse => ("weighted RMSE", "AIC"),
        };

        report.push_str(&format!("Label: {}\n", self.label));
        report.push_str(&format!("Date: {}\n", self.date));
        report.push_str(&format!("Test Type: {}\n", self.test_type));
        report.push_str(&format!("Instrument Model: {}\n", self.instrument_model));
        report.push_str(&format!("Data Points: {}\n", self.point_count()));
        report.push_str(&format!(
            "Ranking Metric: {} (alternate metric shown alongside: {})\n",
            ranking_metric_label, alternate_metric_label,
        ));

        if let Some(best) = ranked_fits.first() {
            let runner_up = ranked_fits.get(1);
            let runner_up_name = ranked_fits
                .get(1)
                .map(|runner_up| runner_up.fit.circuit_model.as_str())
                .unwrap_or("n/a");
            let selected = &ranked_fits[preferred_idx];

            match ranking_metric {
                FitRankingMetric::Aic => {
                    let delta_aic = runner_up
                        .map(|runner_up| runner_up.metrics.aic - best.metrics.aic)
                        .unwrap_or(0.0);
                    report.push_str(&format!(
                        "Ranking Summary: best AIC model = {}; configured metric (AIC) = {:.6}; alternate metric (weighted RMSE) = {:.6}; ΔAIC vs next best ({}) = {:.2}; selected model = {}\n",
                        best.fit.circuit_model,
                        best.metrics.aic,
                        best.metrics.weighted_rmse,
                        runner_up_name,
                        delta_aic,
                        selected.fit.circuit_model,
                    ));

                    if is_warburg_extended_model(&best.fit.circuit_model)
                        && selected.fit.circuit_model != best.fit.circuit_model
                    {
                        report.push_str(&format!(
                            "Selection Threshold: ΔAIC improvement for -W2 ({:.2}) is below threshold {:.2}; baseline model retained.\n",
                            delta_aic,
                            warburg_aic_threshold,
                        ));
                    } else if is_warburg_extended_model(&selected.fit.circuit_model) {
                        report.push_str(&format!(
                            "Selection Threshold: -W2 selected because ΔAIC improvement meets threshold {:.2}.\n",
                            warburg_aic_threshold,
                        ));
                    }
                }
                FitRankingMetric::WeightedRmse => {
                    let improvement_pct = runner_up
                        .map(|runner_up| {
                            if runner_up.metrics.weighted_rmse > 0.0 {
                                100.0
                                    * (runner_up.metrics.weighted_rmse - best.metrics.weighted_rmse)
                                    / runner_up.metrics.weighted_rmse
                            } else {
                                0.0
                            }
                        })
                        .unwrap_or(0.0);
                    report.push_str(&format!(
                        "Ranking Summary: best weighted RMSE model = {}; configured metric (weighted RMSE) = {:.6}; alternate metric (AIC) = {:.6}; improvement vs next best ({}) = {:.2}%; selected model = {}\n",
                        best.fit.circuit_model,
                        best.metrics.weighted_rmse,
                        best.metrics.aic,
                        runner_up_name,
                        improvement_pct,
                        selected.fit.circuit_model,
                    ));
                }
            }
        }
        report.push('\n');

        for (rank, ranked_fit) in ranked_fits.iter().enumerate() {
            let fit = &ranked_fit.fit;
            let metrics = ranked_fit.metrics;
            report.push_str(&format!("Rank {}: {}\n", rank + 1, fit.circuit_model));
            match ranking_metric {
                FitRankingMetric::Aic => {
                    report.push_str(&format!(
                        "  Configured Metric (AIC): {:.6} | Alternate Metric (weighted RMSE): {:.6}\n",
                        metrics.aic,
                        metrics.weighted_rmse,
                    ));
                }
                FitRankingMetric::WeightedRmse => {
                    report.push_str(&format!(
                        "  Configured Metric (weighted RMSE): {:.6} | Alternate Metric (AIC): {:.6}\n",
                        metrics.weighted_rmse,
                        metrics.aic,
                    ));
                }
            }
            report.push_str(&format!("  AIC: {:.6}\n", metrics.aic));
            report.push_str(&format!("  Weighted SSE: {:.6}\n", metrics.weighted_sse));
            report.push_str(&format!("  Weighted RMSE: {:.6}\n", metrics.weighted_rmse));
            report.push_str(&format!("  Real RMSE: {:.6} Ohm\n", metrics.real_rmse));
            report.push_str(&format!("  Imag RMSE: {:.6} Ohm\n", metrics.imag_rmse));
            report.push_str(&format!(
                "  Max |ΔZ|: {:.6} Ohm\n",
                metrics.max_modulus_error
            ));
            match format_fitted_circuit_composition(
                &fit.circuit_model,
                &fit.fitted_parameters,
                "  ",
            ) {
                Ok(composition_report) => {
                    report.push_str(&composition_report);
                    report.push('\n');
                }
                Err(error) => {
                    report.push_str(&format!("  Circuit Composition: unavailable ({error})\n"));
                }
            }

            report.push('\n');
        }

        report
    }

    pub fn nyquist_series_for_fit(&self, fit: &EISFitResult) -> Vec<PlotSeries> {
        self.nyquist_series_for_fits(std::slice::from_ref(fit))
    }

    pub fn nyquist_series_for_fits(&self, fits: &[EISFitResult]) -> Vec<PlotSeries> {
        let experimental_points = self.points();
        let mut series = vec![PlotSeries::experimental(
            self.label().to_string(),
            experimental_points,
        )];

        for fit in fits {
            let fitted_points = finite_points(
                fit.fitted_z_re
                    .iter()
                    .zip(fit.fitted_z_im.iter())
                    .map(|(re, im)| (*re, -*im)),
            );
            series.push(PlotSeries::fitted(
                format!("{} fit [{}]", self.label(), fit.circuit_model),
                fitted_points,
            ));
        }

        series
    }

    pub fn bode_magnitude_series_for_fit(&self, fit: &EISFitResult) -> Vec<PlotSeries> {
        self.bode_magnitude_series_for_fits(std::slice::from_ref(fit))
    }

    pub fn bode_magnitude_series_for_fits(&self, fits: &[EISFitResult]) -> Vec<PlotSeries> {
        let source_magnitude = self.measured_magnitude.as_ref();
        let experimental_points =
            sorted_frequency_points(self.freq.iter().enumerate().filter_map(|(index, freq)| {
                source_magnitude
                    .and_then(|values| values.get(index).copied().flatten())
                    .or_else(|| self.derived_magnitude.get(index).copied())
                    .map(|magnitude| (*freq, magnitude))
            }));
        let experimental_label = if source_magnitude.is_some() {
            self.label().to_string()
        } else {
            format!("{} derived |Z|", self.label())
        };
        let mut series = vec![PlotSeries::experimental(
            experimental_label,
            experimental_points,
        )];

        for fit in fits {
            let fitted_points = sorted_frequency_points(
                self.freq
                    .iter()
                    .zip(fit.fitted_magnitude.iter())
                    .map(|(freq, magnitude)| (*freq, *magnitude)),
            );
            series.push(PlotSeries::fitted(
                format!("{} fit [{}]", self.label(), fit.circuit_model),
                fitted_points,
            ));
        }

        series
    }

    pub fn bode_phase_series_for_fit(&self, fit: &EISFitResult) -> Vec<PlotSeries> {
        self.bode_phase_series_for_fits(std::slice::from_ref(fit))
    }

    pub fn bode_phase_series_for_fits(&self, fits: &[EISFitResult]) -> Vec<PlotSeries> {
        let experimental_points = sorted_frequency_points(
            self.freq
                .iter()
                .zip(self.phase.iter())
                .map(|(freq, phase)| (*freq, *phase)),
        );
        let experimental_label = if self.measured_phase.is_some() {
            self.label().to_string()
        } else {
            format!("{} derived phase", self.label())
        };
        let mut series = vec![PlotSeries::experimental(
            experimental_label,
            experimental_points,
        )];

        for fit in fits {
            let fitted_points = sorted_frequency_points(
                self.freq
                    .iter()
                    .zip(fit.fitted_phase.iter())
                    .map(|(freq, phase)| (*freq, *phase)),
            );
            series.push(PlotSeries::fitted(
                format!("{} fit [{}]", self.label(), fit.circuit_model),
                fitted_points,
            ));
        }

        series
    }

    fn compute_fit_result(&self) -> Result<EISFitResult, FittingError> {
        self.compute_fit_result_for_model(&self.circuit_model)
    }

    fn compute_fit_result_for_model(
        &self,
        circuit_model: &str,
    ) -> Result<EISFitResult, FittingError> {
        let fit = impedance::fit_circuit(
            circuit_model,
            &self.freq,
            &self.z_re,
            &self.z_im,
            &self.phase,
        )?;

        Ok(EISFitResult {
            circuit_model: circuit_model.to_string(),
            fitted_parameters: fit.fitted_parameters,
            parameter_names: fit.parameter_names,
            parameter_units: fit.parameter_units,
            fitted_z_re: fit.fitted_z_re,
            fitted_z_im: fit.fitted_z_im,
            fitted_magnitude: fit.fitted_magnitude,
            fitted_phase: fit.fitted_phase,
        })
    }
}

impl PlotDataSeries for EISData {
    fn label(&self) -> &str {
        &self.label
    }

    fn x_values(&self) -> &[f64] {
        &self.z_re
    }

    fn y_values(&self) -> &[f64] {
        &self.z_im
    }

    fn points(&self) -> Vec<(f64, f64)> {
        finite_points(
            self.x_values()
                .iter()
                .zip(self.y_values().iter())
                // Invert the imaginary part to match the common convention for Nyquist plots where the positive y-axis points upwards.
                .map(|(x, y)| (*x, *y * -1.0)),
        )
    }

    fn plot_series(&self) -> Result<Vec<PlotSeries>, PlottingError> {
        let fit = self.fit_circuit()?;
        Ok(self.nyquist_series_for_fit(&fit))
    }
}

fn finite_points(points: impl IntoIterator<Item = (f64, f64)>) -> Vec<(f64, f64)> {
    points
        .into_iter()
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .collect()
}

fn sorted_frequency_points(points: impl IntoIterator<Item = (f64, f64)>) -> Vec<(f64, f64)> {
    let mut points: Vec<_> = points
        .into_iter()
        .filter(|(freq, value)| freq.is_finite() && *freq > 0.0 && value.is_finite())
        .collect();

    points.sort_by(|lhs, rhs| lhs.0.partial_cmp(&rhs.0).unwrap_or(Ordering::Equal));
    points
}

fn is_warburg_extended_model(model: &str) -> bool {
    let normalized = model
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect::<String>();
    normalized.contains("-w") || normalized.ends_with(")w2")
}

// Unit tests for CHI file parsing and EIS data handling, including circuit
// model resolution and fitting logic.
//
// Tests cover parsing of metadata and data columns, circuit model selection
// based on metadata and filename, loading of resolver rules from configuration
// files, and validation of fitting results for synthetic EIS data.

#[cfg(test)]
mod tests {
    use super::{EISData, EISFitResult, ElectrochemData, file_label};
    use crate::impedance::{
        CircuitModelResolver, CircuitModelRule, DEFAULT_CIRCUIT_MODEL_CONFIG_PATH,
        DEFAULT_EIS_CIRCUIT_MODEL, FitRankingMetric, Impedance, parse_circuit_string,
    };
    use std::f64::consts::PI;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_file(prefix: &str, content: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}_{timestamp}.csv"));
        fs::write(&path, content).expect("failed to write temp CHI file");
        path
    }

    fn write_temp_toml(prefix: &str, content: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}_{timestamp}.toml"));
        fs::write(&path, content).expect("failed to write temp TOML file");
        path
    }

    #[test]
    fn parses_ocpt_time_and_potential_columns() {
        let sample = "Mar. 6, 2026   13:55:19\nOpen Circuit Potential - Time\nFile: test.csv\nData Source:  Experiment\nInstrument Model:  CHI760F\nHeader:\nNote:\n\nSample Interval (V) = 0.1\nRun Time (sec) = 400\n\nTime/sec, Potential/V\n\n1.000e-1, 2.482e-1\n2.000e-1, 2.469e-1\n";
        let path = write_temp_file("ocpt_parse", sample);

        let parsed = ElectrochemData::parse_file(&path).expect("failed to parse OCPT sample");
        fs::remove_file(path).ok();

        assert_eq!(parsed.instrument_model, "CHI760F");
        assert_eq!(parsed.x_values, vec![0.1, 0.2]);
        assert_eq!(parsed.y_values, vec![0.2482, 0.2469]);
    }

    #[test]
    fn parses_ocpt_with_alternate_header_names() {
        let sample = "Mar. 6, 2026   13:55:19\nOpen Circuit Potential - Time\nFile: test.txt\nData Source:  Experiment\nInstrument Model:  CHI760F\n\nTime/s, Voltage/V\n\n1.000e-1, 2.482e-1\n2.000e-1, 2.469e-1\n";
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ocpt_parse_alt_{timestamp}.txt"));
        fs::write(&path, sample).expect("failed to write temp CHI file");

        let parsed = ElectrochemData::parse_file(&path).expect("failed to parse OCPT sample");
        fs::remove_file(path).ok();

        assert_eq!(parsed.x_values, vec![0.1, 0.2]);
        assert_eq!(parsed.y_values, vec![0.2482, 0.2469]);
    }

    #[test]
    fn parses_multi_column_ocpt_into_independent_series() {
        let sample = "Mar. 6, 2026   13:55:19\nOpen Circuit Potential - Time\nFile: test.csv\nData Source:  Experiment\nInstrument Model:  CHI760F\n\nTime/sec, E1/V, E2/V\n\n1.000e-1, 2.482e-1, 2.300e-1\n2.000e-1, 2.469e-1, 2.280e-1\n";
        let path = write_temp_file("ocpt_parse_multi", sample);

        let parsed = ElectrochemData::parse_file_series(&path)
            .expect("failed to parse multi-column OCPT sample");
        fs::remove_file(path.clone()).ok();

        let stem = file_label(&path);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].label, format!("{stem} - E1/V"));
        assert_eq!(parsed[1].label, format!("{stem} - E2/V"));
        assert_eq!(parsed[0].x_values, vec![0.1, 0.2]);
        assert_eq!(parsed[0].y_values, vec![0.2482, 0.2469]);
        assert_eq!(parsed[1].x_values, vec![0.1, 0.2]);
        assert_eq!(parsed[1].y_values, vec![0.23, 0.228]);
    }

    #[test]
    fn rejects_duplicate_canonical_channel_roles_with_structured_context() {
        let sample = "Mar. 6, 2026   13:55:19\nOpen Circuit Potential - Time\nFile: test.csv\nData Source:  Experiment\nInstrument Model:  CHI760F\n\nTime/sec, E1/V, E1/V\n\n1.000e-1, 2.482e-1, 2.300e-1\n2.000e-1, 2.469e-1, 2.280e-1\n";
        let path = write_temp_file("ocpt_parse_multi_dupe", sample);

        let error = ElectrochemData::parse_file_series(&path)
            .expect_err("duplicate canonical channel roles must remain structured input errors");
        fs::remove_file(path.clone()).ok();
        assert!(error.to_string().contains("ch1_v"));
    }

    #[test]
    fn parses_eis_columns_from_header_names() {
        let sample = "Mar. 12, 2026   15:48:13\nA.C. Impedance\nFile: test.csv\nData Source:  Experiment\nInstrument Model:  CHI760F\nHeader:\nNote:\n\nInit E (V) = 0.02\nHigh Frequency (Hz) = 1e+5\nLow Frequency (Hz) = 0.01\n\nFreq/Hz, Z'/ohm, Z\"/ohm, Z/ohm, Phase/deg\n\n8.252e+4, 2.691e+2, -1.998e+1, 2.699e+2, -4.2\n6.812e+4, 2.716e+2, -1.553e+1, 2.720e+2, -3.3\n";
        let path = write_temp_file("eis_parse", sample);

        let parsed = EISData::parse_file(&path).expect("failed to parse EIS sample");
        fs::remove_file(path).ok();

        assert_eq!(parsed.instrument_model, "CHI760F");
        assert_eq!(parsed.freq, vec![82520.0, 68120.0]);
        assert_eq!(parsed.z_re, vec![269.1, 271.6]);
        assert_eq!(parsed.z_im, vec![-19.98, -15.53]);
        assert_eq!(parsed.phase, vec![-4.2, -3.3]);
        assert_eq!(
            parsed.metadata.get("highfrequency(hz)"),
            Some(&"1e+5".to_string())
        );
        assert_eq!(parsed.circuit_model, DEFAULT_EIS_CIRCUIT_MODEL);
    }

    #[test]
    fn selects_circuit_model_from_metadata() {
        let sample = "Mar. 12, 2026   15:48:13\nA.C. Impedance\nFile: test.csv\nData Source:  Experiment\nInstrument Model:  CHI760F\nEquivalent Circuit: R0-W1\n\nFreq/Hz, Z'/ohm, Z\"/ohm, Z/ohm, Phase/deg\n\n8.252e+4, 2.691e+2, -1.998e+1, 2.699e+2, -4.2\n";
        let path = write_temp_file("eis_metadata_model", sample);

        let parsed = EISData::parse_file(&path).expect("failed to parse EIS sample");
        fs::remove_file(path).ok();

        assert_eq!(parsed.circuit_model, "R0-W1");
    }

    #[test]
    fn selects_circuit_model_from_filename_tag() {
        let sample = "Mar. 12, 2026   15:48:13\nA.C. Impedance\nFile: test.csv\nData Source:  Experiment\nInstrument Model:  CHI760F\n\nFreq/Hz, Z'/ohm, Z\"/ohm, Z/ohm, Phase/deg\n\n8.252e+4, 2.691e+2, -1.998e+1, 2.699e+2, -4.2\n";
        let path = write_temp_file("eis_file_circuit=R0-W1", sample);

        let parsed = EISData::parse_file(&path).expect("failed to parse EIS sample");
        fs::remove_file(path).ok();

        assert_eq!(parsed.circuit_model, "R0-W1");
    }

    #[test]
    fn selects_circuit_model_from_custom_rule() {
        let sample = "Mar. 12, 2026   15:48:13\nA.C. Impedance\nFile: test.csv\nData Source:  Experiment\nInstrument Model:  CHI760F\n\nFreq/Hz, Z'/ohm, Z\"/ohm, Z/ohm, Phase/deg\n\n8.252e+4, 2.691e+2, -1.998e+1, 2.699e+2, -4.2\n";
        let path = write_temp_file("qd_li_sensor", sample);

        let resolver = CircuitModelResolver::default()
            .with_rule(CircuitModelRule::new("R0-p(CPE1,R1)-W2").with_filename_contains("li"));

        let parsed = EISData::parse_file_with_resolver(&path, &resolver)
            .expect("failed to parse EIS sample");
        fs::remove_file(path).ok();

        assert_eq!(parsed.circuit_model, "R0-p(CPE1,R1)-W2");
    }

    #[test]
    fn loads_circuit_rules_from_toml_config() {
        let config = r#"
fallback_model = "R0-p(CPE1,R1)"

    [model_selection]
    ranking_metric = "weighted_rmse"
    warburg_aic_threshold = 6.5

[[rules]]
circuit_model = "R0-W1"
filename_contains = ["qd"]

[rules.metadata_contains]
instrumentmodel = "chi760f"
"#;

        let path = write_temp_toml("circuit_resolver", config);
        let resolver = CircuitModelResolver::from_config_file(&path).expect("load config");
        fs::remove_file(path).ok();

        assert_eq!(resolver.fallback_model, DEFAULT_EIS_CIRCUIT_MODEL);
        assert_eq!(
            resolver.model_selection.ranking_metric,
            FitRankingMetric::WeightedRmse
        );
        assert_eq!(resolver.model_selection.warburg_aic_threshold, 6.5);
        assert_eq!(resolver.rules.len(), 1);
        assert_eq!(resolver.rules[0].circuit_model, "R0-W1");
    }

    #[test]
    fn parse_file_uses_config_file_when_present() {
        let sample = "Mar. 12, 2026   15:48:13\nA.C. Impedance\nFile: test.csv\nData Source:  Experiment\nInstrument Model:  CHI760F\n\nFreq/Hz, Z'/ohm, Z\"/ohm, Z/ohm, Phase/deg\n\n8.252e+4, 2.691e+2, -1.998e+1, 2.699e+2, -4.2\n";
        let csv_path = write_temp_file("qd_dataset", sample);
        let original_dir = std::env::current_dir().expect("current dir");
        let temp_dir = std::env::temp_dir().join(format!(
            "rust_plots_config_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        ));

        fs::create_dir_all(&temp_dir).expect("create temp dir");
        let config_path = temp_dir.join(DEFAULT_CIRCUIT_MODEL_CONFIG_PATH);
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).expect("create config dir");
        }
        fs::write(
            &config_path,
            "fallback_model = \"R0-p(CPE1,R1)\"\n\n[model_selection]\nranking_metric = \"aic\"\nwarburg_aic_threshold = 4.0\n\n[[rules]]\ncircuit_model = \"R0-W1\"\nfilename_contains = [\"qd\"]\n",
        )
        .expect("write config");

        std::env::set_current_dir(&temp_dir).expect("set temp dir");
        let parsed = EISData::parse_file(&csv_path).expect("parse with config");
        std::env::set_current_dir(&original_dir).expect("restore current dir");

        fs::remove_file(csv_path).ok();
        fs::remove_file(config_path).ok();
        fs::remove_dir_all(temp_dir).ok();

        assert_eq!(parsed.circuit_model, "R0-W1");
    }

    #[test]
    fn computes_fitted_parameters_for_eis_data() {
        let circuit = parse_circuit_string(DEFAULT_EIS_CIRCUIT_MODEL).expect("parse circuit");
        let params = vec![8.0, 2.5e-5, 0.88, 125.0];
        let freq = vec![
            100_000.0, 50_000.0, 10_000.0, 5_000.0, 1_000.0, 500.0, 100.0, 50.0,
        ];

        let mut z_re = Vec::with_capacity(freq.len());
        let mut z_im = Vec::with_capacity(freq.len());
        let mut phase = Vec::with_capacity(freq.len());

        for &f in &freq {
            let z = circuit.calculate(2.0 * PI * f, &params);
            z_re.push(z.re);
            z_im.push(z.im);
            phase.push(z.im.atan2(z.re).to_degrees());
        }

        let data = EISData {
            date: "2026-03-13".to_string(),
            test_type: "A.C. Impedance".to_string(),
            instrument_model: "Synthetic".to_string(),
            freq,
            phase,
            z_re,
            z_im,
            measured_magnitude: None,
            measured_phase: None,
            derived_magnitude: Vec::new(),
            derived_phase: Vec::new(),
            label: "synthetic".to_string(),
            metadata: Default::default(),
            circuit_model: DEFAULT_EIS_CIRCUIT_MODEL.to_string(),
        };

        let fit = data.fit_circuit().expect("fit synthetic data");

        assert_eq!(fit.parameter_names.len(), 4);
        assert_eq!(fit.parameter_units.len(), 4);
        assert_eq!(fit.fitted_parameters.len(), 4);
        assert_eq!(fit.fitted_z_re.len(), data.freq.len());
        assert!(fit.fitted_parameters.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn fit_report_includes_element_counts_and_element_breakdown() {
        let data = EISData {
            date: "2026-03-13".to_string(),
            test_type: "A.C. Impedance".to_string(),
            instrument_model: "Synthetic".to_string(),
            freq: vec![10_000.0, 1_000.0, 100.0],
            phase: vec![-5.0, -15.0, -30.0],
            z_re: vec![4.0, 8.0, 20.0],
            z_im: vec![-1.0, -5.0, -12.0],
            measured_magnitude: None,
            measured_phase: None,
            derived_magnitude: Vec::new(),
            derived_phase: Vec::new(),
            label: "synthetic".to_string(),
            metadata: Default::default(),
            circuit_model: DEFAULT_EIS_CIRCUIT_MODEL.to_string(),
        };

        let fit = EISFitResult {
            circuit_model: DEFAULT_EIS_CIRCUIT_MODEL.to_string(),
            fitted_parameters: vec![8.0, 2.5e-5, 0.88, 125.0],
            parameter_names: vec![
                "R_0".to_string(),
                "Q_1".to_string(),
                "alpha_1".to_string(),
                "R_1".to_string(),
            ],
            parameter_units: vec![
                "Ohm".to_string(),
                "Ohm^-1 s^alpha".to_string(),
                "".to_string(),
                "Ohm".to_string(),
            ],
            fitted_z_re: vec![4.1, 8.1, 19.9],
            fitted_z_im: vec![-1.1, -4.9, -12.1],
            fitted_magnitude: vec![4.244997, 9.466784, 23.285618],
            fitted_phase: vec![-15.01836063115067, -31.16913932790742, -31.304280336346846],
        };

        let report = data.format_fit_report(&[fit], FitRankingMetric::Aic, 4.0);

        assert!(report.contains("Element Counts:"));
        assert!(report.contains("R (Resistor) = 2"));
        assert!(report.contains("CPE (Constant Phase Element) = 1"));
        assert!(report.contains("- R0 [R: Resistor]"));
        assert!(report.contains("- CPE1 [CPE: Constant Phase Element]"));
    }
}

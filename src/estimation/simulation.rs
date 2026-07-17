use crate::{
    domain::{AnalysisProvenance, MeasurementChannel, MultiChannelMeasurement},
    estimation::error::EstimationError,
    results::{
        ActivityModelKind, CalibrationDomain, CalibrationFitStatistics, CalibrationModelKind,
        CalibrationParameter, NernstSlopeMode, ResponseSign, StoredCalibrationModel,
        TemperatureMode,
    },
};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SimulationScenario {
    pub schema_version: u32,
    pub seed: u64,
    pub sample_count: usize,
    pub start_time_s: f64,
    pub interval_s: f64,
    pub irregular_jitter_s: f64,
    pub initial_log10_activity: f64,
    pub activity_step_time_s: Option<f64>,
    pub activity_step_log10: f64,
    pub activity_pulse_time_s: Option<f64>,
    pub activity_pulse_duration_s: f64,
    pub activity_pulse_log10: f64,
    pub activity_ramp_rate_log10_per_s: f64,
    pub baseline_initial_v: f64,
    pub baseline_drift_v_per_s: f64,
    pub baseline_random_walk_sd_v: f64,
    pub polarization_initial_v: f64,
    pub polarization_tau_s: f64,
    pub polarization_pulse_time_s: Option<f64>,
    pub polarization_pulse_v: f64,
    pub sensitivity_initial: f64,
    pub sensitivity_drift_per_s: f64,
    pub temperature_celsius: f64,
    pub temperature_ramp_celsius_per_s: f64,
    pub measurement_noise_sd_v: f64,
    pub missing_fraction: f64,
    pub outlier_fraction: f64,
    pub outlier_magnitude_v: f64,
}
impl Default for SimulationScenario {
    fn default() -> Self {
        Self {
            schema_version: 1,
            seed: 42,
            sample_count: 200,
            start_time_s: 0.0,
            interval_s: 1.0,
            irregular_jitter_s: 0.0,
            initial_log10_activity: -3.0,
            activity_step_time_s: Some(80.0),
            activity_step_log10: 1.0,
            activity_pulse_time_s: None,
            activity_pulse_duration_s: 10.0,
            activity_pulse_log10: 0.0,
            activity_ramp_rate_log10_per_s: 0.0,
            baseline_initial_v: 0.0,
            baseline_drift_v_per_s: 0.0,
            baseline_random_walk_sd_v: 0.0,
            polarization_initial_v: 0.0,
            polarization_tau_s: 30.0,
            polarization_pulse_time_s: None,
            polarization_pulse_v: 0.0,
            sensitivity_initial: 1.0,
            sensitivity_drift_per_s: 0.0,
            temperature_celsius: 25.0,
            temperature_ramp_celsius_per_s: 0.0,
            measurement_noise_sd_v: 0.0005,
            missing_fraction: 0.0,
            outlier_fraction: 0.0,
            outlier_magnitude_v: 0.05,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationTruthPoint {
    pub timestamp_s: f64,
    pub log10_activity: f64,
    pub activity: f64,
    pub baseline_offset_v: f64,
    pub polarization_v: f64,
    pub sensitivity_scale: Option<f64>,
    pub temperature_k: f64,
    pub observed_potential_v: Option<f64>,
    pub outlier: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationOutput {
    pub schema_version: u32,
    pub scenario: SimulationScenario,
    pub observations: Vec<SimulationTruthPoint>,
    pub provenance: Option<AnalysisProvenance>,
}

pub fn simulate_scenario(
    scenario: &SimulationScenario,
) -> Result<SimulationOutput, EstimationError> {
    if scenario.sample_count < 2 || !scenario.interval_s.is_finite() || scenario.interval_s <= 0.0 {
        return Err(EstimationError::invalid(
            "simulation sample_count and interval must be valid",
        ));
    }
    for (name, value) in [
        ("irregular_jitter_s", scenario.irregular_jitter_s),
        ("polarization_tau_s", scenario.polarization_tau_s),
        (
            "activity_pulse_duration_s",
            scenario.activity_pulse_duration_s,
        ),
        ("measurement_noise_sd_v", scenario.measurement_noise_sd_v),
        ("missing_fraction", scenario.missing_fraction),
        ("outlier_fraction", scenario.outlier_fraction),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(EstimationError::invalid(format!(
                "simulation {name} must be finite and nonnegative"
            )));
        }
    }
    if scenario.missing_fraction > 1.0 || scenario.outlier_fraction > 1.0 {
        return Err(EstimationError::invalid(
            "simulation missing and outlier fractions must not exceed one",
        ));
    }
    let mut rng = rand::rngs::StdRng::seed_from_u64(scenario.seed);
    let slope = 0.05916;
    let e0 = 0.2;
    let mut p = scenario.polarization_initial_v;
    let mut baseline = scenario.baseline_initial_v;
    let mut out = Vec::with_capacity(scenario.sample_count);
    let mut t = scenario.start_time_s;
    for i in 0..scenario.sample_count {
        if i > 0 {
            let dt = (scenario.interval_s
                + if scenario.irregular_jitter_s > 0.0 {
                    rng.gen_range(-scenario.irregular_jitter_s..scenario.irregular_jitter_s)
                } else {
                    0.0
                })
            .max(1e-9);
            t += dt;
            p *= (-dt / scenario.polarization_tau_s.max(1e-9)).exp();
            if scenario.baseline_random_walk_sd_v > 0.0 {
                baseline += scenario.baseline_random_walk_sd_v * dt.sqrt() * normal(&mut rng);
            }
        }
        if scenario
            .polarization_pulse_time_s
            .is_some_and(|pulse| (t - pulse).abs() <= scenario.interval_s.max(1e-9) / 2.0)
        {
            p += scenario.polarization_pulse_v;
        }
        let mut loga = scenario.initial_log10_activity
            + scenario.activity_ramp_rate_log10_per_s * (t - scenario.start_time_s);
        if scenario.activity_step_time_s.is_some_and(|x| t >= x) {
            loga += scenario.activity_step_log10;
        }
        if scenario.activity_pulse_time_s.is_some_and(|pulse| {
            t >= pulse && t < pulse + scenario.activity_pulse_duration_s.max(0.0)
        }) {
            loga += scenario.activity_pulse_log10;
        }
        let baseline = baseline + scenario.baseline_drift_v_per_s * (t - scenario.start_time_s);
        let sensitivity = scenario.sensitivity_initial
            + scenario.sensitivity_drift_per_s * (t - scenario.start_time_s);
        let temp = 273.15
            + scenario.temperature_celsius
            + scenario.temperature_ramp_celsius_per_s * (t - scenario.start_time_s);
        let mut potential = e0 + sensitivity * slope * loga + baseline + p;
        let outlier =
            scenario.outlier_fraction > 0.0 && rng.r#gen::<f64>() < scenario.outlier_fraction;
        if outlier {
            potential += scenario.outlier_magnitude_v;
        }
        if scenario.measurement_noise_sd_v > 0.0 {
            potential += scenario.measurement_noise_sd_v * normal(&mut rng);
        }
        let missing =
            scenario.missing_fraction > 0.0 && rng.r#gen::<f64>() < scenario.missing_fraction;
        out.push(SimulationTruthPoint {
            timestamp_s: t,
            log10_activity: loga,
            activity: 10_f64.powf(loga),
            baseline_offset_v: baseline,
            polarization_v: p,
            sensitivity_scale: Some(sensitivity),
            temperature_k: temp,
            observed_potential_v: (!missing).then_some(potential),
            outlier,
        });
    }
    Ok(SimulationOutput {
        schema_version: 1,
        scenario: scenario.clone(),
        observations: out,
        provenance: None,
    })
}
fn normal(rng: &mut impl Rng) -> f64 {
    let u1 = rng.r#gen::<f64>().max(1e-12);
    let u2 = rng.r#gen::<f64>();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

pub fn simulation_model() -> StoredCalibrationModel {
    StoredCalibrationModel {
        schema_version: 1,
        analyte: "synthetic".into(),
        ion_charge: 1,
        model_kind: CalibrationModelKind::Nernst,
        activity_model: ActivityModelKind::Ideal,
        temperature_mode: TemperatureMode::Constant,
        slope_mode: NernstSlopeMode::Free,
        response_sign: ResponseSign::Positive,
        parameters: vec![
            CalibrationParameter {
                name: "E0".into(),
                unit: "V".into(),
                value: 0.2,
                standard_error: None,
                lower_bound: None,
                upper_bound: None,
                source: Some("simulation".into()),
            },
            CalibrationParameter {
                name: "slope".into(),
                unit: "V/decade".into(),
                value: 0.05916,
                standard_error: None,
                lower_bound: None,
                upper_bound: None,
                source: Some("simulation".into()),
            },
        ],
        selectivity_coefficients: Vec::new(),
        valid_domain: CalibrationDomain {
            log10_activity_min: Some(-8.0),
            log10_activity_max: Some(1.0),
            ..Default::default()
        },
        training_statistics: CalibrationFitStatistics::default(),
        configuration: crate::calibration_config::ResolvedCalibrationConfig::default(),
        provenance: AnalysisProvenance {
            software_version: "simulation".into(),
            input_path: PathBuf::from("simulation"),
            input_sha256: "simulation".into(),
            configuration_path: None,
            configuration_sha256: None,
            generation_timestamp: 0,
            git_commit: None,
        },
    }
}

pub fn output_measurement(
    output: &SimulationOutput,
) -> Result<MultiChannelMeasurement, EstimationError> {
    let time = output.observations.iter().map(|x| x.timestamp_s).collect();
    let values = output
        .observations
        .iter()
        .map(|x| x.observed_potential_v)
        .collect();
    MultiChannelMeasurement::new(time, vec![MeasurementChannel::new("E1", "V", values)])
        .map_err(|x| EstimationError::invalid(x.to_string()))
}

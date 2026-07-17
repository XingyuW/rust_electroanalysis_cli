use rust_electroanalysis_cli::{
    domain::{AnalysisProvenance, MeasurementChannel, MultiChannelMeasurement},
    health::rules,
    health_config::{
        FeatureCondition, FeatureOperator, HealthFindingKind, HealthRule, HealthSeverity,
    },
    results::{FeatureComparability, HealthDomain, HealthFeature, SignalWarning},
    signal::{allan, correlation, drift, psd, sampling, spikes},
    signal_config::{
        AllanConfig, CorrelationConfig, PsdConfig, SamplingConfig, SamplingPolicy, SpikesConfig,
    },
};
use std::path::PathBuf;

fn provenance() -> AnalysisProvenance {
    AnalysisProvenance {
        software_version: "test".into(),
        input_path: PathBuf::from("synthetic.csv"),
        input_sha256: "synthetic".into(),
        configuration_path: None,
        configuration_sha256: None,
        generation_timestamp: 1,
        git_commit: None,
    }
}

#[test]
fn synthetic_white_noise_psd_and_allan_are_finite() {
    let values = (0..1024)
        .map(|i| ((i as f64 * 12.9898).sin() * 43758.5453).fract() - 0.5)
        .collect::<Vec<_>>();
    let time = (0..values.len()).map(|i| i as f64).collect::<Vec<_>>();
    let config = PsdConfig {
        segment_points: 256,
        overlap_fraction: 0.5,
        ..Default::default()
    };
    let psd = psd::welch(&time, &values, &config).expect("regular signal should support PSD");
    assert!(psd.psd.iter().all(|v| v.is_finite()));
    assert!(psd.parseval_relative_error.is_some_and(|v| v < 0.5));
    let allan = allan::overlapping(
        &time,
        &values,
        &AllanConfig {
            tau_points: 10,
            ..Default::default()
        },
    )
    .expect("Allan deviation");
    assert!(
        allan
            .points
            .iter()
            .all(|p| p.deviation.is_some_and(|v| v.is_finite()))
    );
}

#[test]
fn drift_spikes_and_common_mode_are_non_destructive() {
    let time = (0..100).map(|i| i as f64).collect::<Vec<_>>();
    let mut values = time
        .iter()
        .map(|t| Some(0.2 + 2.0e-3 * t))
        .collect::<Vec<_>>();
    values[50] = Some(1.0);
    let original = values.clone();
    let estimate = drift::estimate(
        &time,
        &values.iter().flatten().copied().collect::<Vec<_>>(),
        rust_electroanalysis_cli::results::DriftModelKind::TheilSen,
    );
    assert!((estimate.slope_v_per_s.unwrap() - 2.0e-3).abs() < 1.0e-5);
    let spikes = spikes::detect(
        &time,
        &values,
        &SpikesConfig {
            minimum_local_observations: 3,
            mad_threshold: 3.0,
            ..Default::default()
        },
    );
    assert!(spikes.flagged.iter().any(|x| x.index == 50));
    assert_eq!(values, original);
    let channel_b = values
        .iter()
        .map(|v| v.map(|x| 2.0 * x + 0.1))
        .collect::<Vec<_>>();
    let c = correlation::pair(
        "a",
        &time,
        &values,
        "b",
        &time,
        &channel_b,
        &CorrelationConfig::default(),
    );
    assert!(c.pearson.unwrap() > 0.99);
}

#[test]
fn irregular_sampling_policy_is_explicit_and_resampling_is_marked() {
    let time = vec![0.0, 1.0, 2.5, 3.5, 10.0];
    let values = time.iter().map(|t| Some(*t)).collect::<Vec<_>>();
    let rejected = sampling::analyze_sampling(
        &time,
        &values,
        &SamplingConfig {
            policy: SamplingPolicy::RequireRegular,
            ..Default::default()
        },
    );
    assert!(rejected.is_err());
    let (analysis, resampled_time, resampled_values) = sampling::analyze_sampling(
        &time,
        &values,
        &SamplingConfig {
            policy: SamplingPolicy::ResampleLinear,
            resample_interval_s: Some(1.0),
            maximum_interpolation_gap_s: 2.0,
            ..Default::default()
        },
    )
    .expect("resampling");
    assert!(analysis.interpolation_count > 0);
    assert_eq!(resampled_time.len(), resampled_values.len());
    assert!(analysis.interpolation_gap_exceeded);
}

#[test]
fn health_mechanism_rule_requires_independent_domains() {
    let rule = HealthRule {
        rule_id: "fouling".into(),
        finding: HealthFindingKind::ProbableFouling,
        severity: HealthSeverity::Major,
        all_of: vec![FeatureCondition {
            feature: "transient.tau_slow".into(),
            operator: FeatureOperator::RelativeIncreaseGreaterThan,
            value: Some(1.0),
        }],
        any_of: vec![FeatureCondition {
            feature: "calibration.slope_efficiency".into(),
            operator: FeatureOperator::RelativeDecreaseGreaterThan,
            value: Some(0.2),
        }],
        minimum_evidence_domains: 2,
        minimum_baseline_records: 3,
        alternative_explanations: vec!["environmental mismatch".into()],
    };
    let one_domain = vec![HealthFeature {
        name: "transient.tau_slow".into(),
        value: Some(2.0),
        unit: "s".into(),
        domain: HealthDomain::DynamicResponse,
        source: "test".into(),
        warning: None,
    }];
    let comparisons = vec![rust_electroanalysis_cli::results::BaselineComparison {
        feature: "transient.tau_slow".into(),
        current_value: Some(2.0),
        baseline_value: Some(1.0),
        comparability: FeatureComparability::Comparable,
        absolute_difference: Some(1.0),
        relative_difference: Some(1.0),
        log_ratio: Some(0.69),
        z_score: None,
        robust_z_score: None,
        percentile_position: None,
        override_reason: None,
    }];
    let (evaluations, findings) = rules::evaluate(&[rule], &one_domain, &comparisons, 2);
    assert!(!evaluations[0].triggered);
    assert!(findings.is_empty());
    let _ = provenance();
    let _ = SignalWarning::RecordTooShort;
}

#[test]
fn measurement_analysis_preserves_source_channels() {
    let measurement = MultiChannelMeasurement::new(
        vec![0.0, 1.0, 2.0, 3.0],
        vec![MeasurementChannel::from_values(
            "potential",
            "V",
            vec![0.1, 0.2, 0.3, 0.4],
        )],
    )
    .unwrap();
    assert_eq!(
        measurement.channels[0].values,
        vec![Some(0.1), Some(0.2), Some(0.3), Some(0.4)]
    );
}

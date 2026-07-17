//! Transparent comparison and evidence classification.

use crate::results::{
    CharacteristicTimescale, EvidenceLevel, MechanismWarning, ResolvedMechanismConfig,
    TimescaleComparison,
};

pub fn compare_timescales(
    record_id: &str,
    eis: &CharacteristicTimescale,
    transient: &CharacteristicTimescale,
    config: &ResolvedMechanismConfig,
) -> TimescaleComparison {
    let mut supporting = Vec::new();
    let mut contradictory = Vec::new();
    let mut assumptions = vec!["numerical timescale compatibility is treated as statistical association, not mechanism proof".to_string()];
    let alternatives = vec!["different processes can have similar characteristic timescales".to_string(), "model misspecification or unresolved frequency/observation windows can produce apparent agreement".to_string()];
    let mut warnings = Vec::new();
    let valid = eis.value_s.is_finite()
        && transient.value_s.is_finite()
        && eis.value_s > 0.0
        && transient.value_s > 0.0;
    let (ratio, log_distance, relative) = if valid {
        let ratio = (eis.value_s / transient.value_s).max(transient.value_s / eis.value_s);
        (
            Some(ratio),
            Some((eis.value_s.log10() - transient.value_s.log10()).abs()),
            Some(
                (eis.value_s - transient.value_s).abs() / ((eis.value_s + transient.value_s) / 2.0),
            ),
        )
    } else {
        warnings.push(MechanismWarning {
            kind: "nonpositive_timescale".to_string(),
            message: "comparison requires finite positive timescales".to_string(),
        });
        (None, None, None)
    };
    let overlap = eis
        .confidence_interval_s
        .zip(transient.confidence_interval_s)
        .map(|(a, b)| a.0 <= b.1 && b.0 <= a.1);
    let probability = compatibility_probability(
        eis,
        transient,
        config.compatibility_ratio_lower,
        config.compatibility_ratio_upper,
        config.monte_carlo_samples,
        config.seed,
    );
    let level = match (ratio, log_distance) {
        (Some(r), Some(d)) if r <= config.ratio_strong && d <= config.log_distance_strong => {
            supporting.push(format!(
                "ratio {r:.4} and log10 distance {d:.4} meet the strong numerical thresholds"
            ));
            EvidenceLevel::Strong
        }
        (Some(r), Some(d)) if r <= config.ratio_moderate && d <= config.log_distance_moderate => {
            supporting.push(format!(
                "ratio {r:.4} and log10 distance {d:.4} meet the moderate numerical thresholds"
            ));
            EvidenceLevel::Moderate
        }
        (Some(r), Some(d)) if r <= config.ratio_weak && d <= config.log_distance_weak => {
            supporting.push(format!(
                "ratio {r:.4} and log10 distance {d:.4} indicate weak temporal compatibility"
            ));
            EvidenceLevel::Weak
        }
        (Some(r), Some(d)) => {
            contradictory.push(format!(
                "ratio {r:.4} and log10 distance {d:.4} exceed configured compatibility thresholds"
            ));
            EvidenceLevel::Contradictory
        }
        _ => EvidenceLevel::Insufficient,
    };
    if overlap == Some(true) {
        supporting
            .push("confidence intervals overlap; this is not a formal hypothesis test".to_string());
    } else if overlap == Some(false) {
        contradictory.push("confidence intervals do not overlap".to_string());
    }
    if eis.validity != crate::results::TimescaleValidity::Valid {
        warnings.push(MechanismWarning {
            kind: "eis_timescale_warning".to_string(),
            message: "EIS timescale carries derivation or identifiability warnings".to_string(),
        });
    }
    if transient.validity != crate::results::TimescaleValidity::Valid {
        warnings.push(MechanismWarning {
            kind: "transient_timescale_warning".to_string(),
            message: "transient timescale carries fit or observation-window warnings".to_string(),
        });
    }
    if probability.is_none() {
        assumptions.push("compatibility probability unavailable because uncertainty intervals/covariance were unavailable".to_string());
    }
    TimescaleComparison {
        comparison_id: format!(
            "{record_id}:{}:{}",
            eis.timescale_id, transient.timescale_id
        ),
        record_id: record_id.to_string(),
        eis_timescale_id: eis.timescale_id.clone(),
        transient_timescale_id: transient.timescale_id.clone(),
        ratio,
        log10_distance: log_distance,
        symmetric_relative_difference: relative,
        confidence_interval_overlap: overlap,
        compatibility_probability: probability,
        evidence_level: level,
        supporting_evidence: supporting,
        contradictory_evidence: contradictory,
        assumptions,
        alternative_explanations: alternatives,
        warnings,
    }
}

fn compatibility_probability(
    eis: &CharacteristicTimescale,
    transient: &CharacteristicTimescale,
    lower: f64,
    upper: f64,
    samples: usize,
    seed: u64,
) -> Option<f64> {
    let eis_se = eis.standard_error_s?;
    let transient_se = transient.standard_error_s?;
    if samples == 0 || !eis_se.is_finite() || !transient_se.is_finite() {
        return None;
    }
    let mut state = seed.max(1);
    let mut compatible = 0usize;
    for _ in 0..samples {
        let x = eis.value_s + eis_se * standard_normal(&mut state);
        let y = transient.value_s + transient_se * standard_normal(&mut state);
        if x > 0.0 && y > 0.0 {
            let ratio = x / y;
            if ratio >= lower && ratio <= upper {
                compatible += 1;
            }
        }
    }
    Some(compatible as f64 / samples as f64)
}

fn standard_normal(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    let u1 = ((*state >> 11) as f64) / ((1u64 << 53) as f64);
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    let u2 = ((*state >> 11) as f64) / ((1u64 << 53) as f64);
    (-2.0 * u1.max(f64::MIN_POSITIVE).ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

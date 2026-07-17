use super::{drift, spikes, statistics};
use crate::{
    results::EisFitArtifact,
    results::{EisResidualSummary, ResidualSummary, SignalWarning},
    signal_config::{PsdConfig, SpikesConfig},
};
pub fn time_summary(
    time: &[f64],
    residuals: &[f64],
    psd_config: Option<&PsdConfig>,
    spike_config: &SpikesConfig,
) -> ResidualSummary {
    let opts = residuals.iter().map(|v| Some(*v)).collect::<Vec<_>>();
    let d = statistics::descriptive(&opts, &[0.5], 0.95);
    let acf = autocorrelation(residuals);
    let lag1 = acf.get(1).copied();
    let dw = (residuals.len() > 1).then(|| {
        let num = residuals
            .windows(2)
            .map(|w| (w[1] - w[0]).powi(2))
            .sum::<f64>();
        let den = residuals.iter().map(|v| v * v).sum::<f64>();
        if den > 0.0 { num / den } else { 0.0 }
    });
    let runs = runs_statistic(residuals);
    let drift = if time.len() == residuals.len() {
        Some(drift::estimate(
            time,
            residuals,
            crate::results::DriftModelKind::OrdinaryLinear,
        ))
    } else {
        None
    };
    let spike = spikes::detect(time, &opts, spike_config);
    let psd = psd_config.and_then(|c| crate::signal::psd::welch(time, residuals, c).ok());
    ResidualSummary {
        mean: d.mean,
        standard_deviation: d.standard_deviation,
        rmse: d.rms,
        lag1_autocorrelation: lag1,
        durbin_watson: dw,
        runs_statistic: runs,
        drift,
        spike_fraction: spike.flagged_fraction,
        autocorrelation: acf,
        psd,
    }
}
pub fn eis_summary(artifact: &EisFitArtifact) -> EisResidualSummary {
    let r = &artifact.residuals.points;
    let n = r.len();
    let mean = |f: fn(&crate::results::EisResidualPoint) -> f64| {
        if n > 0 {
            Some(r.iter().map(f).sum::<f64>() / n as f64)
        } else {
            None
        }
    };
    let mut by_freq = r
        .iter()
        .map(|p| (p.frequency_hz, p.magnitude))
        .collect::<Vec<_>>();
    by_freq.sort_by(|a, b| a.0.total_cmp(&b.0));
    let half = n / 2;
    let low = if half > 0 {
        Some(by_freq[..half].iter().map(|p| p.1).sum::<f64>() / half as f64)
    } else {
        None
    };
    let high = if n - half > 0 {
        Some(by_freq[half..].iter().map(|p| p.1).sum::<f64>() / (n - half) as f64)
    } else {
        None
    };
    let signs = r.iter().map(|p| p.real.signum()).collect::<Vec<_>>();
    let runs = signs.windows(2).filter(|w| w[0] != w[1]).count();
    EisResidualSummary {
        observations: n,
        real_mean: mean(|p| p.real),
        imaginary_mean: mean(|p| p.imaginary),
        magnitude_mean: mean(|p| p.magnitude),
        low_frequency_bias: low,
        high_frequency_bias: high,
        systematic_sign_runs: runs,
        frequency_dependent_magnitude: by_freq,
    }
}
fn autocorrelation(v: &[f64]) -> Vec<f64> {
    let m = statistics::mean(v).unwrap_or(0.0);
    let den = v.iter().map(|x| (x - m).powi(2)).sum::<f64>();
    if den == 0.0 {
        return vec![1.0];
    }
    (0..v.len().min(50))
        .map(|lag| {
            v.iter()
                .skip(lag)
                .zip(v)
                .map(|(a, b)| (a - m) * (b - m))
                .sum::<f64>()
                / den
        })
        .collect()
}
fn runs_statistic(v: &[f64]) -> Option<f64> {
    if v.len() < 2 {
        return None;
    }
    let m = statistics::mean(v)?;
    let signs = v
        .iter()
        .map(|x| if *x >= m { 1 } else { -1 })
        .collect::<Vec<_>>();
    Some(signs.windows(2).filter(|w| w[0] != w[1]).count() as f64)
}
pub fn warning() -> SignalWarning {
    SignalWarning::ResidualArtifactIncompatible
}

use super::{error::SignalError, statistics};
use crate::{
    results::{PsdAnalysis, PsdBandPower, PsdPeak, SignalWarning},
    signal_config::{DetrendKind, PsdConfig},
};
use rustfft::{FftPlanner, num_complex::Complex};

pub fn welch(time: &[f64], values: &[f64], config: &PsdConfig) -> Result<PsdAnalysis, SignalError> {
    if values.len() != time.len() || values.len() < 4 {
        return Err(SignalError::invalid(
            "PSD requires at least four aligned samples",
        ));
    }
    if time.windows(2).any(|w| w[1] <= w[0]) {
        return Err(SignalError::Sampling(
            "PSD requires increasing timestamps".into(),
        ));
    }
    let dt = statistics::median(&mut time.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>())
        .ok_or_else(|| SignalError::Sampling("sampling interval unavailable".into()))?;
    let fs = 1.0 / dt;
    let n = config.segment_points.min(values.len());
    if n < 4 {
        return Err(SignalError::invalid("PSD segment is too short"));
    }
    let overlap = (n as f64 * config.overlap_fraction).round() as usize;
    if overlap >= n {
        return Err(SignalError::invalid(
            "PSD overlap must be less than segment length",
        ));
    }
    let step = n - overlap;
    let fft_len = config.fft_length.unwrap_or(n).max(n).next_power_of_two();
    let mut segments = Vec::new();
    let mut start = 0;
    while start + n <= values.len() {
        segments.push(start);
        start += step;
    }
    if segments.is_empty() {
        return Err(SignalError::invalid("PSD segment longer than data"));
    }
    let window = (0..n)
        .map(|i| {
            if config.window.eq_ignore_ascii_case("hann") {
                0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (n - 1) as f64).cos()
            } else {
                1.0
            }
        })
        .collect::<Vec<_>>();
    let window_power = window.iter().map(|v| v * v).sum::<f64>();
    let mut avg = vec![0.0; fft_len / 2 + 1];
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_len);
    for offset in &segments {
        let mut x = vec![Complex::new(0.0, 0.0); fft_len];
        let segment = &values[*offset..*offset + n];
        let mean = statistics::mean(segment).unwrap_or(0.0);
        let slope = if matches!(config.detrend, DetrendKind::Linear) {
            let xm = (n - 1) as f64 / 2.0;
            let ym = mean;
            let den = (0..n).map(|i| (i as f64 - xm).powi(2)).sum::<f64>();
            Some(
                (0..n)
                    .map(|i| (i as f64 - xm) * (segment[i] - ym))
                    .sum::<f64>()
                    / den,
            )
        } else {
            None
        };
        for i in 0..n {
            let trend = match config.detrend {
                DetrendKind::None => 0.0,
                DetrendKind::Mean => mean,
                DetrendKind::Linear => {
                    mean + slope.unwrap_or(0.0) * (i as f64 - (n - 1) as f64 / 2.0)
                }
            };
            x[i] = Complex::new((segment[i] - trend) * window[i], 0.0);
        }
        fft.process(&mut x);
        for k in 0..=fft_len / 2 {
            let mut p = x[k].norm_sqr() / (fs * window_power);
            if k > 0 && k < fft_len / 2 {
                p *= 2.0;
            }
            avg[k] += p;
        }
    }
    for p in &mut avg {
        *p /= segments.len() as f64;
    }
    let freq = (0..=fft_len / 2)
        .map(|k| k as f64 * fs / fft_len as f64)
        .collect::<Vec<_>>();
    let asd = avg.iter().map(|v| v.max(0.0).sqrt()).collect::<Vec<_>>();
    let integrated = integrate(&freq, &avg);
    let mut warnings = Vec::new();
    if segments.len() < 4 {
        warnings.push(SignalWarning::InsufficientWelchSegments);
    }
    if let Some(f) = freq.get(1)
        && *f > 0.1 * fs
    {
        warnings.push(SignalWarning::PoorPsdResolution);
    }
    let peaks = (1..avg.len().saturating_sub(1))
        .filter(|i| avg[*i] >= avg[*i - 1] && avg[*i] >= avg[*i + 1])
        .map(|i| PsdPeak {
            frequency_hz: freq[i],
            psd: avg[i],
            interpretation: None,
        })
        .collect::<Vec<_>>();
    let mut peaks = peaks;
    peaks.sort_by(|a, b| b.psd.total_cmp(&a.psd));
    peaks.truncate(5);
    let centroid = integrated.filter(|p| *p > 0.0).map(|_| {
        freq.iter().zip(&avg).map(|(f, p)| f * p).sum::<f64>() / avg.iter().sum::<f64>().max(1e-30)
    });
    let roll = integrated.and_then(|total| {
        let target = 0.9 * total;
        let mut sum = 0.0;
        freq.windows(2).enumerate().find_map(|(i, w)| {
            sum += (avg[i] + avg[i + 1]) * 0.5 * (w[1] - w[0]);
            (sum >= target).then_some(w[1])
        })
    });
    let var = statistics::stddev(values).map(|v| v * v);
    let rel = var
        .zip(integrated)
        .map(|(a, b)| (b - a).abs() / a.max(1e-30));
    if rel.is_some_and(|v| v > config.parseval_tolerance) {
        warnings.push(SignalWarning::PoorParsevalAgreement);
    }
    let bands = config
        .frequency_bands
        .iter()
        .map(|b| {
            let p = integrate_band(&freq, &avg, b.minimum_hz, b.maximum_hz);
            PsdBandPower {
                name: b.name.clone(),
                minimum_hz: b.minimum_hz,
                maximum_hz: b.maximum_hz,
                integrated_power: p,
                fraction: p
                    .zip(integrated)
                    .and_then(|(x, t)| (t > 0.0).then_some(x / t)),
            }
        })
        .collect();
    Ok(PsdAnalysis {
        frequency_hz: freq,
        psd_unit: "unit^2/Hz".into(),
        psd: avg,
        amplitude_spectral_density: asd,
        segment_points: n,
        segment_count: segments.len(),
        overlap_fraction: config.overlap_fraction,
        frequency_resolution_hz: Some(fs / fft_len as f64),
        nyquist_hz: Some(fs / 2.0),
        dominant_peaks: peaks,
        total_integrated_power: integrated,
        band_powers: bands,
        spectral_centroid_hz: centroid,
        spectral_rolloff_hz: roll,
        parseval_time_variance: var,
        parseval_integrated_power: integrated,
        parseval_relative_error: rel,
        warnings,
    })
}
fn integrate(x: &[f64], y: &[f64]) -> Option<f64> {
    (x.len() > 1).then(|| {
        x.windows(2)
            .zip(y.windows(2))
            .map(|(a, b)| (b[0] + b[1]) * 0.5 * (a[1] - a[0]))
            .sum()
    })
}
fn integrate_band(x: &[f64], y: &[f64], lo: f64, hi: f64) -> Option<f64> {
    let p = (0..x.len().saturating_sub(1))
        .filter(|i| x[*i + 1] >= lo && x[*i] <= hi)
        .map(|i| (y[i] + y[i + 1]) * 0.5 * (x[i + 1] - x[i]))
        .sum::<f64>();
    (p.is_finite()).then_some(p)
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

/// MESA Sine Wave (Ehlers)
/// Maximum Entropy Spectral Analysis based adaptive sine wave.
/// Outputs sine and cosine components of the dominant cycle.
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let vals = close.to_f64_vec();
    let n = vals.len();
    if n < period + 2 {
        return Err(IndError::DataInsufficient(period + 2));
    }

    // MESA high-pass filter to remove trend
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut hp = vec![f64::NAN; n];
    hp[0] = 0.0;
    for i in 1..n {
        if hp[i - 1].is_finite() {
            hp[i] = 0.5 * (1.0 + alpha) * (vals[i] - vals[i - 1]) + alpha * hp[i - 1];
        }
    }

    // MESA real (in-phase) and imaginary (quadrature) components
    let mut real_part = vec![f64::NAN; n];
    let mut imag_part = vec![f64::NAN; n];

    for i in 2..n {
        if hp[i].is_finite() && hp[i - 1].is_finite() && hp[i - 2].is_finite() {
            // Compute dominant cycle frequency estimate
            let delta = 0.5 * (period as f64).max(6.0);
            let beta = 2.0 * (2.0 * std::f64::consts::PI / delta).cos();

            // Real and imaginary using adaptive band-pass filter
            if i >= 3 && real_part[i - 1].is_finite() && real_part[i - 2].is_finite() {
                real_part[i] =
                    0.5 * (1.0 - alpha) * (hp[i] - hp[i - 2]) + beta * (1.0 + alpha) * real_part[i - 1]
                        - alpha * real_part[i - 2];
            } else {
                real_part[i] = 0.5 * (1.0 - alpha) * (hp[i] - hp[i - 2]);
            }
        }
    }

    // Imaginary component (90-degree phase shifted from real)
    for i in 4..n {
        if hp[i].is_finite() && hp[i - 1].is_finite() {
            let delta = 0.5 * (period as f64).max(6.0);
            let beta = 2.0 * (2.0 * std::f64::consts::PI / delta).cos();

            if i >= 5 && imag_part[i - 1].is_finite() && imag_part[i - 2].is_finite() {
                imag_part[i] = 0.5 * (1.0 - alpha) * (hp[i - 1] - hp[i - 3])
                    + beta * (1.0 + alpha) * imag_part[i - 1]
                    - alpha * imag_part[i - 2];
            } else {
                imag_part[i] = 0.5 * (1.0 - alpha) * (hp[i - 1] - hp[i - 3]);
            }
        }
    }

    // Compute adaptive dominant cycle period from correlation
    let mut smooth_period = vec![f64::NAN; n];
    for i in period + 2..n {
        let mut sx = 0.0f64;
        let mut sy = 0.0f64;
        let mut sxx = 0.0f64;
        let mut sxy = 0.0f64;
        let mut syy = 0.0f64;
        for j in 0..period {
            let idx = i - j;
            if real_part[idx].is_finite() && imag_part[idx].is_finite() {
                sx += real_part[idx];
                sy += imag_part[idx];
                sxx += real_part[idx] * real_part[idx];
                sxy += real_part[idx] * imag_part[idx];
                syy += imag_part[idx] * imag_part[idx];
            }
        }
        let denom = ((period as f64 * sxx - sx * sx) * (period as f64 * syy - sy * sy)).sqrt();
        if denom > 1e-12 {
            let corr = (period as f64 * sxy - sx * sy) / denom;
            let corr_clamped = corr.clamp(-0.999, 0.999);
            let phase_angle = corr_clamped.acos();
            if phase_angle > 1e-8 {
                smooth_period[i] = (2.0 * std::f64::consts::PI / phase_angle).clamp(6.0, 100.0);
            }
        }
        if !smooth_period[i].is_finite() && smooth_period[i - 1].is_finite() {
            smooth_period[i] = smooth_period[i - 1];
        }
    }

    // Compute phase from real and imaginary components
    // Then compute sine and cosine of accumulated phase
    let mut sine = vec![f64::NAN; n];
    let mut cosine = vec![f64::NAN; n];
    let mut dc_phase = 0.0f64;

    for i in period + 2..n {
        if real_part[i].is_finite() && imag_part[i].is_finite() {
            // Phase accumulation using smooth period
            if smooth_period[i].is_finite() && smooth_period[i] > 0.0 {
                dc_phase += 2.0 * std::f64::consts::PI / smooth_period[i];
            } else {
                dc_phase += 2.0 * std::f64::consts::PI / period as f64;
            }

            // Normalize phase
            while dc_phase > std::f64::consts::PI {
                dc_phase -= 2.0 * std::f64::consts::PI;
            }
            while dc_phase < -std::f64::consts::PI {
                dc_phase += 2.0 * std::f64::consts::PI;
            }

            sine[i] = dc_phase.sin();
            cosine[i] = dc_phase.cos();
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("MESA_SINE({})", period),
            values: Column::F64(sine),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("MESA_COSINE({})", period),
            values: Column::F64(cosine),
            style: OutputStyle::Line,
        },
    ])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

/// Hilbert Transform Sine Wave (Ehlers)
/// Outputs sine and lead sine components of the dominant cycle.
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(7.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let vals = close.to_f64_vec();
    let n = vals.len();
    if n < period + 6 {
        return Err(IndError::DataInsufficient(period + 6));
    }

    // Step 1: Smooth with 4-bar WMA
    let mut smooth = vec![f64::NAN; n];
    for i in 3..n {
        smooth[i] = (4.0 * vals[i] + 3.0 * vals[i - 1] + 2.0 * vals[i - 2] + vals[i - 3]) / 10.0;
    }

    // Step 2: Detrender = smooth[i] - smooth[i - period]
    let mut detrender = vec![f64::NAN; n];
    for i in period..n {
        if smooth[i].is_finite() && smooth[i - period].is_finite() {
            detrender[i] = smooth[i] - smooth[i - period];
        }
    }

    // Step 3: Quadrature component Q1 using Hilbert FIR filter
    // Step 4: In-phase component I1 = delayed detrender
    let mut q1 = vec![f64::NAN; n];
    let mut i1 = vec![f64::NAN; n];
    for i in 6..n {
        if detrender[i].is_finite()
            && detrender[i - 2].is_finite()
            && detrender[i - 4].is_finite()
            && detrender[i - 6].is_finite()
        {
            q1[i] = (0.0962 * detrender[i]
                + 0.5769 * detrender[i - 2]
                - 0.5769 * detrender[i - 4]
                - 0.0962 * detrender[i - 6])
                * 0.5;
        }
        if i >= 3 && detrender[i - 3].is_finite() {
            i1[i] = detrender[i - 3];
        }
    }

    // Step 5: Phase from I1 and Q1, compute delta phase and smooth
    let mut phase = vec![f64::NAN; n];
    let mut inst_period = vec![f64::NAN; n];

    for i in 7..n {
        if q1[i].is_finite() && i1[i].is_finite() {
            phase[i] = q1[i].atan2(i1[i]);
            if phase[i - 1].is_finite() {
                let mut dp = phase[i] - phase[i - 1];
                // Handle phase wraparound
                if dp < -std::f64::consts::PI {
                    dp += 2.0 * std::f64::consts::PI;
                } else if dp > std::f64::consts::PI {
                    dp -= 2.0 * std::f64::consts::PI;
                }
                if dp != 0.0 {
                    inst_period[i] = 2.0 * std::f64::consts::PI / dp;
                }
            }
        }
    }

    // Smooth the instantaneous period
    let mut smooth_period = vec![f64::NAN; n];
    for i in 7..n {
        if inst_period[i].is_finite() {
            if !smooth_period[i - 1].is_finite() {
                smooth_period[i] = inst_period[i];
            } else {
                // EMA smoothing
                let alpha = 2.0 / (period as f64 + 1.0);
                smooth_period[i] = alpha * inst_period[i] + (1.0 - alpha) * smooth_period[i - 1];
            }
            // Clamp
            smooth_period[i] = smooth_period[i].clamp(6.0, 50.0);
        }
    }

    // Step 6: Compute DC phase and sine components
    let mut sine = vec![f64::NAN; n];
    let mut lead_sine = vec![f64::NAN; n];
    let mut dc_phase = 0.0f64;

    for i in 7..n {
        if smooth_period[i].is_finite() && phase[i].is_finite() {
            let sp = smooth_period[i].max(1.0);
            dc_phase += 2.0 * std::f64::consts::PI / sp;
            // Normalize to [-pi, pi]
            while dc_phase > std::f64::consts::PI {
                dc_phase -= 2.0 * std::f64::consts::PI;
            }
            while dc_phase < -std::f64::consts::PI {
                dc_phase += 2.0 * std::f64::consts::PI;
            }
            sine[i] = dc_phase.sin();
            lead_sine[i] = (dc_phase + std::f64::consts::PI / 4.0).sin();
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("HT_SINE({})", period),
            values: Column::F64(sine),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("HT_LEADSINE({})", period),
            values: Column::F64(lead_sine),
            style: OutputStyle::Line,
        },
    ])
}

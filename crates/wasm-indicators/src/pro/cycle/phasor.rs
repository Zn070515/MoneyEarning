use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

/// Phasor Indicator (Ehlers)
/// Detects cycle mode: trending vs cycling.
/// When the phasor phase is stable, the market is cycling.
/// Outputs phasor phase and phasor magnitude.
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let vals = close.to_f64_vec();
    let n = vals.len();
    if n < period + 6 {
        return Err(IndError::DataInsufficient(period + 6));
    }

    // Smooth with 4-bar WMA
    let mut smooth = vec![f64::NAN; n];
    for i in 3..n {
        smooth[i] = (4.0 * vals[i] + 3.0 * vals[i - 1] + 2.0 * vals[i - 2] + vals[i - 3]) / 10.0;
    }

    // Detrender
    let mut detrender = vec![f64::NAN; n];
    for i in period..n {
        if smooth[i].is_finite() && smooth[i - period].is_finite() {
            detrender[i] = smooth[i] - smooth[i - period];
        }
    }

    // Quadrature component (Hilbert transform FIR)
    let mut quadrature = vec![f64::NAN; n];
    for i in 6..n {
        if detrender[i].is_finite()
            && detrender[i - 2].is_finite()
            && detrender[i - 4].is_finite()
            && detrender[i - 6].is_finite()
        {
            quadrature[i] = (0.0962 * detrender[i]
                + 0.5769 * detrender[i - 2]
                - 0.5769 * detrender[i - 4]
                - 0.0962 * detrender[i - 6])
                * 0.5;
        }
    }

    // In-phase component (delayed detrender)
    let mut in_phase = vec![f64::NAN; n];
    for i in 3..n {
        if detrender[i - 3].is_finite() {
            in_phase[i] = detrender[i - 3];
        }
    }

    // Phase and magnitude
    let mut phasor_phase = vec![f64::NAN; n];
    let mut phasor_mag = vec![f64::NAN; n];

    for i in 7..n {
        if quadrature[i].is_finite() && in_phase[i].is_finite() {
            let phase = quadrature[i].atan2(in_phase[i]);
            let mag = (quadrature[i] * quadrature[i] + in_phase[i] * in_phase[i]).sqrt();

            // Phase unwrapping for continuity
            if phasor_phase[i - 1].is_finite() {
                let mut dp = phase - phasor_phase[i - 1];
                if dp < -std::f64::consts::PI {
                    dp += 2.0 * std::f64::consts::PI;
                } else if dp > std::f64::consts::PI {
                    dp -= 2.0 * std::f64::consts::PI;
                }
                phasor_phase[i] = phasor_phase[i - 1] + dp;
            } else {
                phasor_phase[i] = phase;
            }
            phasor_mag[i] = mag;
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("PHASOR_PHASE({})", period),
            values: Column::F64(phasor_phase),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("PHASOR_MAG({})", period),
            values: Column::F64(phasor_mag),
            style: OutputStyle::Line,
        },
    ])
}

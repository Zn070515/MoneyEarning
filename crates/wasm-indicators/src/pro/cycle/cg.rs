use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

/// Center of Gravity (Ehlers)
/// Adaptive, essentially zero-lag oscillator.
/// Outputs CG line and a signal line (SMA of CG).
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(10.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let vals = close.to_f64_vec();
    let n = vals.len();
    if n < period + 1 {
        return Err(IndError::DataInsufficient(period + 1));
    }

    // Center of Gravity: CG[i] = -num/den
    // num = sum(j * close[i-j]) for j=0..period-1
    // den = sum(close[i-j]) for j=0..period-1
    let mut cg = vec![f64::NAN; n];
    for i in period - 1..n {
        let mut num = 0.0f64;
        let mut den = 0.0f64;
        for j in 0..period {
            let v = vals[i - j];
            if v.is_finite() {
                num += j as f64 * v;
                den += v;
            }
        }
        if den.abs() > 1e-12 {
            cg[i] = -num / den;
        }
    }

    // Signal line: SMA of CG
    let signal_period = (period / 2).max(2);
    let mut signal = vec![f64::NAN; n];
    for i in period - 1 + signal_period - 1..n {
        let mut sum = 0.0;
        let mut count = 0;
        for j in 0..signal_period {
            let v = cg[i - j];
            if v.is_finite() {
                sum += v;
                count += 1;
            }
        }
        if count > 0 {
            signal[i] = sum / count as f64;
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("CG({})", period),
            values: Column::F64(cg),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("CG_SIG({})", period),
            values: Column::F64(signal),
            style: OutputStyle::Line,
        },
    ])
}

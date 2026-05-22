use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ema(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![f64::NAN; n];
    if n == 0 {
        return out;
    }
    // Find the first non-NaN value as seed
    let mut seed_idx = None;
    for i in 0..n {
        if vals[i].is_finite() {
            out[i] = vals[i];
            seed_idx = Some(i);
            break;
        }
    }
    let start = match seed_idx {
        Some(idx) => idx,
        None => return out,
    };
    for i in start + 1..n {
        if vals[i].is_finite() {
            out[i] = alpha * vals[i] + (1.0 - alpha) * out[i - 1];
        } else {
            out[i] = out[i - 1];
        }
    }
    out
}

pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let long = params.get("long").copied().unwrap_or(25.0) as usize;
    let short = params.get("short").copied().unwrap_or(13.0) as usize;
    let signal = params.get("signal").copied().unwrap_or(7.0) as usize;

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    let min_required = long.max(short) + signal + 1;
    if n < min_required {
        return Err(IndError::DataInsufficient(min_required));
    }

    // Momentum: close change
    let mut mom = vec![f64::NAN; n];
    for i in 1..n {
        mom[i] = c[i] - c[i - 1];
    }

    // Absolute momentum
    let mut abs_mom = vec![f64::NAN; n];
    for i in 1..n {
        abs_mom[i] = mom[i].abs();
    }

    // Double-smoothed momentum: EMA(EMA(mom, long), short)
    let ema1_mom = ema(&mom, long);
    let ema2_mom = ema(&ema1_mom, short);

    // Double-smoothed absolute momentum: EMA(EMA(abs_mom, long), short)
    let ema1_abs = ema(&abs_mom, long);
    let ema2_abs = ema(&ema1_abs, short);

    // TSI = 100 * ema2_mom / ema2_abs
    let mut tsi = vec![f64::NAN; n];
    for i in 0..n {
        if ema2_abs[i].is_finite() && ema2_abs[i] != 0.0 && ema2_mom[i].is_finite() {
            tsi[i] = 100.0 * ema2_mom[i] / ema2_abs[i];
        }
    }

    // Signal line = EMA(tsi, signal)
    let signal_line = ema(&tsi, signal);

    Ok(vec![
        IndicatorOutput {
            name: format!("TSI({},{})", long, short),
            values: Column::F64(tsi),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("TSI_SIG({})", signal),
            values: Column::F64(signal_line),
            style: OutputStyle::Line,
        },
    ])
}

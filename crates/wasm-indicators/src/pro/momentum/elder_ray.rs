use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ema(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![f64::NAN; n];
    if n == 0 {
        return out;
    }
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
    let period = params.get("period").copied().unwrap_or(13.0) as usize;

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    if n < period + 1 {
        return Err(IndError::DataInsufficient(period + 1));
    }

    let ema_close = ema(&c, period);

    // Bull power = high - EMA(close)
    let mut bull = vec![f64::NAN; n];
    for i in 0..n {
        if ema_close[i].is_finite() {
            bull[i] = h[i] - ema_close[i];
        }
    }

    // Bear power = low - EMA(close)
    let mut bear = vec![f64::NAN; n];
    for i in 0..n {
        if ema_close[i].is_finite() {
            bear[i] = l[i] - ema_close[i];
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("BULL({})", period),
            values: Column::F64(bull),
            style: OutputStyle::Histogram,
        },
        IndicatorOutput {
            name: format!("BEAR({})", period),
            values: Column::F64(bear),
            style: OutputStyle::Histogram,
        },
    ])
}

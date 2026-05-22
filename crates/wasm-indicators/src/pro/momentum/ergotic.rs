use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn llv(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    let mut min_val = f64::MAX;
    for i in 0..n {
        min_val = min_val.min(vals[i]);
        if i >= period && vals[i - period] == min_val {
            min_val = vals[i - period + 1..=i]
                .iter()
                .cloned()
                .fold(f64::MAX, f64::min);
        }
        if i >= period - 1 {
            result[i] = min_val;
        }
    }
    result
}

fn hhv(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    let mut max_val = f64::MIN;
    for i in 0..n {
        max_val = max_val.max(vals[i]);
        if i >= period && vals[i - period] == max_val {
            max_val = vals[i - period + 1..=i]
                .iter()
                .cloned()
                .fold(f64::MIN, f64::max);
        }
        if i >= period - 1 {
            result[i] = max_val;
        }
    }
    result
}

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

fn sma(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut out = vec![f64::NAN; n];
    let mut sum = 0.0;
    let mut count = 0usize;
    for i in 0..n {
        if vals[i].is_finite() {
            sum += vals[i];
            count += 1;
        }
        if count > period && vals[i.max(period) - period].is_finite() {
            sum -= vals[i - period];
            count -= 1;
        }
        if count >= period {
            out[i] = sum / period as f64;
        }
    }
    out
}

pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let signal = params.get("signal").copied().unwrap_or(5.0) as usize;

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    let min_required = period + signal + 1;
    if n < min_required {
        return Err(IndError::DataInsufficient(min_required));
    }

    let ll = llv(&l, period);
    let hh = hhv(&h, period);

    // r = (close - lowest(low, period)) / (highest(high, period) - lowest(low, period))
    let mut r = vec![f64::NAN; n];
    for i in 0..n {
        if hh[i].is_finite() && ll[i].is_finite() {
            let denom = hh[i] - ll[i];
            if denom > 0.0 {
                r[i] = (c[i] - ll[i]) / denom;
            } else {
                r[i] = 0.5;
            }
        }
    }

    // Ergotic = 100 * EMA(r, signal)
    let ergotic_ema = ema(&r, signal);
    let mut ergotic = vec![f64::NAN; n];
    for i in 0..n {
        if ergotic_ema[i].is_finite() {
            ergotic[i] = 100.0 * ergotic_ema[i];
        }
    }

    // Signal line = SMA(ergotic, signal)
    let signal_line = sma(&ergotic, signal);

    Ok(vec![
        IndicatorOutput {
            name: format!("ERGOTIC({},{})", period, signal),
            values: Column::F64(ergotic),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("ERGOTIC_SIG({})", signal),
            values: Column::F64(signal_line),
            style: OutputStyle::Line,
        },
    ])
}

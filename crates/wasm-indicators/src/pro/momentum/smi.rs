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

pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let k_period = params.get("k_period").copied().unwrap_or(5.0) as usize;
    let k_smooth = params.get("k_smooth").copied().unwrap_or(3.0) as usize;
    let d_period = params.get("d_period").copied().unwrap_or(3.0) as usize;
    let signal = params.get("signal").copied().unwrap_or(5.0) as usize;

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    let min_required = k_period + k_smooth + d_period + signal;
    if n < min_required {
        return Err(IndError::DataInsufficient(min_required));
    }

    let ll = llv(&l, k_period);
    let hh = hhv(&h, k_period);

    // mid = (hh + ll) / 2, d_close = close - mid, r = (hh - ll) / 2
    let mut d_close = vec![f64::NAN; n];
    let mut r_half = vec![f64::NAN; n];
    for i in 0..n {
        if ll[i].is_finite() && hh[i].is_finite() {
            let mid = (hh[i] + ll[i]) / 2.0;
            d_close[i] = c[i] - mid;
            r_half[i] = (hh[i] - ll[i]) / 2.0;
        }
    }

    // smooth_d = EMA(EMA(d_close, k_smooth), d_period)
    let ema_d1 = ema(&d_close, k_smooth);
    let smooth_d = ema(&ema_d1, d_period);

    // smooth_r = EMA(EMA(r_half * 2, k_smooth), d_period) ... r is already r/2, so r*2 = (hh-ll)
    let mut r_full = vec![f64::NAN; n];
    for i in 0..n {
        if r_half[i].is_finite() {
            r_full[i] = r_half[i] * 2.0; // (hh - ll)
        }
    }
    let ema_r1 = ema(&r_full, k_smooth);
    let smooth_r = ema(&ema_r1, d_period);

    // SMI = 100 * smooth_d / smooth_r
    let mut smi = vec![f64::NAN; n];
    for i in 0..n {
        if smooth_r[i].is_finite() && smooth_r[i] != 0.0 && smooth_d[i].is_finite() {
            smi[i] = 100.0 * smooth_d[i] / smooth_r[i];
        }
    }

    // Signal line = EMA(smi, signal)
    let signal_line = ema(&smi, signal);

    Ok(vec![
        IndicatorOutput {
            name: format!("SMI({},{},{})", k_period, k_smooth, d_period),
            values: Column::F64(smi),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("SMI_SIG({})", signal),
            values: Column::F64(signal_line),
            style: OutputStyle::Line,
        },
    ])
}

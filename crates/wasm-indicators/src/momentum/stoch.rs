use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn llv(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    let mut min_val = f64::MAX;
    for i in 0..n {
        min_val = min_val.min(vals[i]);
        if i >= period && vals[i - period] == min_val {
            min_val = vals[i - period + 1..=i].iter().cloned().fold(f64::MAX, f64::min);
        }
        if i >= period - 1 { result[i] = min_val; }
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
            max_val = vals[i - period + 1..=i].iter().cloned().fold(f64::MIN, f64::max);
        }
        if i >= period - 1 { result[i] = max_val; }
    }
    result
}

fn sma(vals: &[f64], period: usize, weight: f64) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    let mut sma_val = 0.0;
    let mut started = false;
    for i in 0..n {
        if vals[i].is_nan() { continue; }
        if !started { sma_val = vals[i]; started = true; }
        else { sma_val = (vals[i] * weight + sma_val * (period as f64 - weight)) / period as f64; }
        result[i] = sma_val;
    }
    result
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let k_period = params.get("k_period").copied().unwrap_or(14.0) as usize;
    let k_slow = params.get("k_slow").copied().unwrap_or(3.0) as usize;
    let d_period = params.get("d_period").copied().unwrap_or(3.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let len = c.len();

    let llv_k = llv(&l, k_period);
    let hhv_k = hhv(&h, k_period);
    let mut fast_k = vec![f64::NAN; len];
    for i in 0..len {
        if llv_k[i].is_finite() && hhv_k[i].is_finite() {
            let denom = hhv_k[i] - llv_k[i];
            if denom > 0.0 { fast_k[i] = (c[i] - llv_k[i]) / denom * 100.0; }
            else { fast_k[i] = 50.0; }
        }
    }
    let k = sma(&fast_k, k_slow, 1.0);
    let d = sma(&k, d_period, 1.0);
    Ok(vec![
        IndicatorOutput { name: "K".into(), values: Column::F64(k), style: OutputStyle::Line },
        IndicatorOutput { name: "D".into(), values: Column::F64(d), style: OutputStyle::Line },
    ])
}

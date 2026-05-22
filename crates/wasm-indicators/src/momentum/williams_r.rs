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

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    let hhv_p = hhv(&h, period);
    let llv_p = llv(&l, period);
    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        if hhv_p[i].is_finite() && llv_p[i].is_finite() {
            let denom = hhv_p[i] - llv_p[i];
            if denom > 0.0 { result[i] = (hhv_p[i] - c[i]) / denom * -100.0; }
            else { result[i] = -50.0; }
        }
    }
    Ok(vec![IndicatorOutput {
        name: format!("WR({})", period), values: Column::F64(result), style: OutputStyle::Line,
    }])
}

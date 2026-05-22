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
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let upper = hhv(&h, period);
    let lower = llv(&l, period);
    let mut mid = vec![f64::NAN; h.len()];
    for i in 0..mid.len() {
        if upper[i].is_finite() && lower[i].is_finite() { mid[i] = (upper[i] + lower[i]) / 2.0; }
    }
    Ok(vec![
        IndicatorOutput { name: "UPPER".into(), values: Column::F64(upper), style: OutputStyle::Line },
        IndicatorOutput { name: "MID".into(), values: Column::F64(mid), style: OutputStyle::Line },
        IndicatorOutput { name: "LOWER".into(), values: Column::F64(lower), style: OutputStyle::Line },
    ])
}

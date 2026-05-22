use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = h.len();
    if n < 2 { return Err(IndError::DataInsufficient(2)); }

    let mut box_ratio = vec![f64::NAN; n];
    for i in 1..n {
        if v[i] != 0.0 && h[i] != l[i] {
            box_ratio[i] = (v[i] / 100_000_000.0) / ((h[i] - l[i]) / h[i]);
        }
    }

    let mut emv = vec![f64::NAN; n];
    for i in 1..n {
        let mp = (h[i] + l[i]) / 2.0;
        let mp_prev = (h[i-1] + l[i-1]) / 2.0;
        if box_ratio[i].is_finite() && box_ratio[i] != 0.0 {
            emv[i] = (mp - mp_prev) / box_ratio[i];
        }
    }

    let mut result = vec![f64::NAN; n];
    let mut sum = 0.0;
    let mut count = 0;
    for i in 0..n {
        if emv[i].is_finite() { sum += emv[i]; count += 1; }
        if i >= period && emv[i - period].is_finite() { sum -= emv[i - period]; count -= 1; }
        if count > 0 { result[i] = sum / count as f64; }
    }

    Ok(vec![IndicatorOutput {
        name: format!("EOM({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

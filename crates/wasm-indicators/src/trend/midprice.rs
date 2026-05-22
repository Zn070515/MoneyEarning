use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(10.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();
    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        if i < period - 1 {
            result[i] = f64::NAN;
        } else {
            let slice_h = &h[i + 1 - period..=i];
            let slice_l = &l[i + 1 - period..=i];
            let hh = slice_h.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let ll = slice_l.iter().cloned().fold(f64::INFINITY, f64::min);
            result[i] = (hh + ll) / 2.0;
        }
    }
    Ok(vec![IndicatorOutput {
        name: format!("MIDPRICE({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

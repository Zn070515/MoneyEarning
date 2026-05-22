use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(10.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        if i < period - 1 {
            result[i] = f64::NAN;
        } else {
            let slice = &c[i + 1 - period..=i];
            let mn = slice.iter().cloned().fold(f64::INFINITY, f64::min);
            let mx = slice.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            result[i] = (mn + mx) / 2.0;
        }
    }
    Ok(vec![IndicatorOutput {
        name: format!("MIDPOINT({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(3.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        let mut dec = true;
        for j in 0..period - 1 {
            if c[i - j] >= c[i - j - 1] { dec = false; break; }
        }
        result[i] = if dec { 1.0 } else { 0.0 };
    }
    Ok(vec![IndicatorOutput {
        name: format!("DECREASING({})", period),
        values: Column::F64(result),
        style: OutputStyle::Histogram,
    }])
}

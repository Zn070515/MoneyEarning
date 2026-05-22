use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];
    let total_weight = (period * (period + 1)) as f64 / 2.0;
    for i in period - 1..n {
        let mut sum = 0.0;
        for j in 0..period {
            sum += c[i - j] * (period - j) as f64;
        }
        result[i] = sum / total_weight;
    }
    Ok(vec![IndicatorOutput {
        name: format!("WMA({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

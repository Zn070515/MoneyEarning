use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        let slice = &c[i + 1 - period..=i];
        let mean: f64 = slice.iter().sum::<f64>() / period as f64;
        result[i] = slice.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / period as f64;
    }
    Ok(vec![IndicatorOutput {
        name: format!("VARIANCE({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

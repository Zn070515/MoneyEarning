use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(12.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];
    let mut up_count = 0usize;
    for i in 0..n {
        if i > 0 && c[i] > c[i - 1] { up_count += 1; }
        if i >= period && c[i - period] > c[i - period - 1] { up_count -= 1; }
        if i >= period - 1 {
            result[i] = up_count as f64 / period as f64 * 100.0;
        }
    }
    Ok(vec![IndicatorOutput {
        name: format!("PSY({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

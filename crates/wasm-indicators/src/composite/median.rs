use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        let mut slice: Vec<f64> = c[i + 1 - period..=i].to_vec();
        slice.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = period / 2;
        result[i] = if period % 2 == 0 {
            (slice[mid - 1] + slice[mid]) / 2.0
        } else {
            slice[mid]
        };
    }
    Ok(vec![IndicatorOutput {
        name: format!("MEDIAN({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

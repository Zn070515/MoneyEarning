use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        result[i] = (h[i] + l[i] + 2.0 * c[i]) / 4.0;
    }
    Ok(vec![IndicatorOutput {
        name: "WCP".into(),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    if n == 0 { return Err(IndError::DataInsufficient(period)); }
    let alpha = 1.0 / period as f64;
    let mut result = vec![f64::NAN; n];
    let mut rma = c[0];
    for i in 0..n {
        rma = alpha * c[i] + (1.0 - alpha) * rma;
        result[i] = rma;
    }
    Ok(vec![IndicatorOutput {
        name: format!("RMA({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

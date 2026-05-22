use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(12.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];
    for i in period..n {
        if c[i - period] > 0.0 {
            result[i] = (c[i] - c[i - period]) / c[i - period] * 100.0;
        }
    }
    Ok(vec![IndicatorOutput {
        name: format!("ROC({})", period), values: Column::F64(result), style: OutputStyle::Line,
    }])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(6.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];

    let mut sum: f64 = c[0..period].iter().sum();
    for i in period - 1..n {
        if i == period - 1 { /* use initial sum */ }
        else if i >= period { sum += c[i] - c[i - period]; }
        let sma = sum / period as f64;
        result[i] = if sma != 0.0 { (c[i] / sma - 1.0) * 100.0 } else { 0.0 };
    }

    Ok(vec![IndicatorOutput {
        name: format!("BIAS({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

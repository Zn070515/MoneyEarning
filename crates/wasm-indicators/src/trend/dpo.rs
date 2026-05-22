use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    if n < period { return Err(IndError::DataInsufficient(period)); }
    let offset = period / 2 + 1;

    let mut sma = vec![f64::NAN; n];
    let mut sum: f64 = c[0..period].iter().sum();
    for i in period - 1..n {
        if i == period - 1 { sma[i] = sum / period as f64; }
        else { sum += c[i] - c[i - period]; sma[i] = sum / period as f64; }
    }

    let mut result = vec![f64::NAN; n];
    for i in offset..n {
        if sma[i - offset].is_finite() {
            result[i] = c[i - offset] - sma[i - offset];
        }
    }
    Ok(vec![IndicatorOutput {
        name: format!("DPO({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];

    let half = (period as f64 / 2.0).ceil() as usize;
    let mut weights = Vec::with_capacity(period);
    for j in 0..half { weights.push((j + 1) as f64); }
    for j in half..period { weights.push((period - j) as f64); }
    let total_weight: f64 = weights.iter().sum();

    for i in period - 1..n {
        let mut sum = 0.0;
        for j in 0..period {
            sum += c[i - period + 1 + j] * weights[j];
        }
        result[i] = sum / total_weight;
    }

    Ok(vec![IndicatorOutput {
        name: format!("TRIMA({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

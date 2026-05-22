use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    if n < period + 1 { return Err(IndError::DataInsufficient(period + 1)); }

    // Standard deviation of price changes
    let mut changes = vec![f64::NAN; n];
    for i in 1..n { changes[i] = c[i] - c[i-1]; }

    let mut std_up = vec![f64::NAN; n];
    let mut std_down = vec![f64::NAN; n];

    for i in period..n {
        let slice = &changes[i + 1 - period..=i];
        let mean: f64 = slice.iter().filter(|v| v.is_finite()).sum::<f64>() / period as f64;
        let var_up: f64 = slice.iter().filter(|v| v.is_finite() && **v > mean).map(|v| (v - mean) * (v - mean)).sum();
        let var_down: f64 = slice.iter().filter(|v| v.is_finite() && **v < mean).map(|v| (v - mean) * (v - mean)).sum();
        let count_up = slice.iter().filter(|v| v.is_finite() && **v > mean).count().max(1);
        let count_down = slice.iter().filter(|v| v.is_finite() && **v < mean).count().max(1);
        std_up[i] = (var_up / count_up as f64).sqrt();
        std_down[i] = (var_down / count_down as f64).sqrt();
    }

    let mut result = vec![f64::NAN; n];
    for i in period..n {
        if std_down[i] != 0.0 && std_up[i].is_finite() && std_down[i].is_finite() {
            result[i] = std_up[i] / std_down[i];
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("RVI({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

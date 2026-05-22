use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(21.0) as usize;
    let annualize = params.get("annualize").copied().unwrap_or(252.0);
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let vals = close.to_f64_vec();
    let n = vals.len();
    if n < period + 1 {
        return Err(IndError::DataInsufficient(period + 1));
    }

    // Compute log returns: ln(close[i] / close[i-1])
    let mut log_returns = vec![f64::NAN; n];
    for i in 1..n {
        if vals[i] > 0.0 && vals[i - 1] > 0.0 {
            log_returns[i] = (vals[i] / vals[i - 1]).ln();
        }
    }

    // Rolling standard deviation of log returns, annualized
    let mut result = vec![f64::NAN; n];
    for i in period..n {
        let slice = &log_returns[i + 1 - period..=i];
        let count = slice.iter().filter(|v| v.is_finite()).count();
        if count < 2 {
            continue;
        }
        let mean: f64 = slice.iter().filter(|v| v.is_finite()).sum::<f64>() / count as f64;
        let var: f64 = slice
            .iter()
            .filter(|v| v.is_finite())
            .map(|v| (v - mean) * (v - mean))
            .sum::<f64>()
            / (count - 1) as f64;
        result[i] = var.sqrt() * annualize.sqrt() * 100.0;
    }

    Ok(vec![IndicatorOutput {
        name: format!("HV({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

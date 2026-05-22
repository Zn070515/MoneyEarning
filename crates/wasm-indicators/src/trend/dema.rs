use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ema_series(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![f64::NAN; n];
    if n == 0 { return out; }
    out[0] = data[0];
    for i in 1..n {
        out[i] = alpha * data[i] + (1.0 - alpha) * out[i - 1];
    }
    out
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    if c.is_empty() { return Err(IndError::DataInsufficient(period)); }
    let e1 = ema_series(&c, period);
    let e2 = ema_series(&e1, period);
    let mut result = vec![f64::NAN; c.len()];
    for i in 0..c.len() {
        if e1[i].is_finite() && e2[i].is_finite() {
            result[i] = 2.0 * e1[i] - e2[i];
        }
    }
    Ok(vec![IndicatorOutput {
        name: format!("DEMA({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

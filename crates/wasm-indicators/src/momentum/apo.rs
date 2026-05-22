use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ema_series(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![f64::NAN; n];
    if n == 0 { return out; }
    out[0] = data[0];
    for i in 1..n { out[i] = alpha * data[i] + (1.0 - alpha) * out[i - 1]; }
    out
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let fast = params.get("fast").copied().unwrap_or(12.0) as usize;
    let slow = params.get("slow").copied().unwrap_or(26.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let ema_f = ema_series(&c, fast);
    let ema_s = ema_series(&c, slow);
    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        if ema_f[i].is_finite() && ema_s[i].is_finite() { result[i] = ema_f[i] - ema_s[i]; }
    }
    Ok(vec![IndicatorOutput {
        name: format!("APO({},{})", fast, slow),
        values: Column::F64(result),
        style: OutputStyle::Histogram,
    }])
}

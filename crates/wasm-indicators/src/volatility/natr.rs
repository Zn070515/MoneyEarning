use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn rma_series(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    if n == 0 { return vec![]; }
    let alpha = 1.0 / period as f64;
    let mut out = vec![0.0; n];
    out[0] = data[0];
    for i in 1..n { out[i] = alpha * data[i] + (1.0 - alpha) * out[i - 1]; }
    out
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = h.len();
    if n < 2 { return Err(IndError::DataInsufficient(2)); }

    let mut tr = vec![f64::NAN; n];
    tr[0] = h[0] - l[0];
    for i in 1..n {
        tr[i] = (h[i] - l[i]).max((h[i] - c[i-1]).abs()).max((l[i] - c[i-1]).abs());
    }
    let atr = rma_series(&tr, period);

    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        if c[i] != 0.0 && atr[i].is_finite() { result[i] = atr[i] / c[i] * 100.0; }
    }

    Ok(vec![IndicatorOutput {
        name: format!("NATR({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

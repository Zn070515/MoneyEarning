use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ema_series(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    if n == 0 { return vec![]; }
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![f64::NAN; n];
    out[0] = data[0];
    for i in 1..n { out[i] = alpha * data[i] + (1.0 - alpha) * out[i - 1]; }
    out
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();
    if n < period + 1 { return Err(IndError::DataInsufficient(period + 1)); }

    let mut tr_vals = vec![f64::NAN; n];
    for i in 1..n {
        let close = df.get_f64("close", i - 1).unwrap_or(0.0);
        tr_vals[i] = (h[i] - l[i]).max((h[i] - close).abs()).max((l[i] - close).abs());
    }
    let tr_ema = ema_series(&tr_vals, period);

    let mut hl_diff = vec![f64::NAN; n];
    for i in 0..n { hl_diff[i] = h[i] - l[i]; }
    let hl_ema = ema_series(&hl_diff, period);

    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        if tr_ema[i].is_finite() && tr_ema[i] != 0.0 && hl_ema[i].is_finite() {
            result[i] = hl_ema[i] / tr_ema[i];
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("THERMO({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

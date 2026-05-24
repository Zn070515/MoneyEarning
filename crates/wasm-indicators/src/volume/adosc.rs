use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ad_line(df: &DataFrame) -> Result<Vec<f64>, IndError> {
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();

    let mut ad = vec![0.0; n];
    let mut cum = 0.0;
    for i in 0..n {
        let range = h[i] - l[i];
        let clv = if range != 0.0 { ((c[i] - l[i]) - (h[i] - c[i])) / range } else { 0.0 };
        cum += clv * v[i];
        ad[i] = cum;
    }
    Ok(ad)
}

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
    let fast = params.get("fast").copied().unwrap_or(3.0) as usize;
    let slow = params.get("slow").copied().unwrap_or(10.0) as usize;
    let ad = ad_line(df)?;
    let ema_f = ema_series(&ad, fast);
    let ema_s = ema_series(&ad, slow);
    let n = ad.len();
    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        if ema_f[i].is_finite() && ema_s[i].is_finite() { result[i] = ema_f[i] - ema_s[i]; }
    }
    Ok(vec![IndicatorOutput {
        name: format!("ADOSC({},{})", fast, slow),
        values: Column::F64(result),
        style: OutputStyle::Histogram,
    }])
}

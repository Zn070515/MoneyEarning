use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn rvgi(df: &DataFrame, period: usize) -> Result<Vec<f64>, IndError> {
    let open = df.column("open").ok_or(IndError::InvalidName)?;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let o = open.to_f64_vec();
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    let mut num = vec![0.0; n];
    let mut den = vec![0.0; n];
    for i in 0..n {
        num[i] = c[i] - o[i];
        den[i] = h[i] - l[i];
    }

    let mut num_sma = vec![f64::NAN; n];
    let mut den_sma = vec![f64::NAN; n];
    let mut num_sum = 0.0;
    let mut den_sum = 0.0;
    for i in 0..n {
        num_sum += num[i]; den_sum += den[i];
        if i >= period - 1 {
            num_sma[i] = num_sum / period as f64;
            den_sma[i] = den_sum / period as f64;
            num_sum -= num[i + 1 - period];
            den_sum -= den[i + 1 - period];
        }
    }

    let mut rvgi_vals = vec![f64::NAN; n];
    for i in 0..n {
        if den_sma[i].is_finite() && den_sma[i] != 0.0 { rvgi_vals[i] = num_sma[i] / den_sma[i]; }
    }

    let signal_period = 4usize;
    let mut signal_sum = 0.0;
    let mut valid_count = 0;
    let mut rvgi_signal = vec![f64::NAN; n];
    for i in 0..n {
        if rvgi_vals[i].is_finite() { signal_sum += rvgi_vals[i]; valid_count += 1; }
        if valid_count > signal_period && rvgi_vals[i.max(signal_period) - signal_period].is_finite() {
            signal_sum -= rvgi_vals[i - signal_period];
            valid_count -= 1;
        }
        if valid_count >= signal_period {
            rvgi_signal[i] = signal_sum / signal_period as f64;
        }
    }

    Ok(rvgi_signal)
}

fn ema_series(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![f64::NAN; n];
    if n == 0 || !data[0].is_finite() { return out; }
    out[0] = data[0];
    for i in 1..n {
        out[i] = if data[i].is_finite() { alpha * data[i] + (1.0 - alpha) * out[i - 1] } else { out[i - 1] };
    }
    out
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let rvgi_signal = rvgi(df, period)?;
    let result = ema_series(&rvgi_signal, period);
    Ok(vec![IndicatorOutput {
        name: format!("INERTIA({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

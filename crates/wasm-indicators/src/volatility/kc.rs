use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn calc_ema(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    if n < period { return result; }
    let multiplier = 2.0 / (period as f64 + 1.0);
    let sma_init = vals[0..period].iter().sum::<f64>() / period as f64;
    let mut ema = sma_init;
    for i in 0..n {
        if i < period - 1 { continue; }
        else if i == period - 1 { result[i] = ema; }
        else { ema = (vals[i] - ema) * multiplier + ema; result[i] = ema; }
    }
    result
}

fn calc_atr(h: &[f64], l: &[f64], c: &[f64], period: usize) -> Vec<f64> {
    let n = c.len();
    let mut tr = vec![0.0; n];
    for i in 1..n {
        let a = h[i] - l[i];
        let b = (h[i] - c[i - 1]).abs();
        let d = (l[i] - c[i - 1]).abs();
        tr[i] = a.max(b).max(d);
    }
    tr[0] = h[0] - l[0];
    let mut result = vec![f64::NAN; n];
    let mut atr = tr[0..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = atr;
    for i in period..n {
        atr = (atr * (period - 1) as f64 + tr[i]) / period as f64;
        result[i] = atr;
    }
    result
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let atr_period = params.get("atr_period").copied().unwrap_or(10.0) as usize;
    let multiplier = params.get("multiplier").copied().unwrap_or(2.0);
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    let mid = calc_ema(&c, period);
    let atr = calc_atr(&h, &l, &c, atr_period);
    let mut upper = vec![f64::NAN; n];
    let mut lower = vec![f64::NAN; n];
    for i in 0..n {
        if mid[i].is_finite() && atr[i].is_finite() {
            upper[i] = mid[i] + multiplier * atr[i];
            lower[i] = mid[i] - multiplier * atr[i];
        }
    }
    Ok(vec![
        IndicatorOutput { name: "MID".into(), values: Column::F64(mid), style: OutputStyle::Line },
        IndicatorOutput { name: "UPPER".into(), values: Column::F64(upper), style: OutputStyle::Line },
        IndicatorOutput { name: "LOWER".into(), values: Column::F64(lower), style: OutputStyle::Line },
    ])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

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
    let period = params.get("period").copied().unwrap_or(10.0) as usize;
    let multiplier = params.get("multiplier").copied().unwrap_or(3.0);
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    let atr = calc_atr(&h, &l, &c, period);
    let mut result = vec![f64::NAN; n];
    let mut upper = vec![f64::NAN; n];
    let mut lower = vec![f64::NAN; n];
    let mut trend = vec![true; n]; // true = up

    for i in (period - 1)..n {
        let hl2 = (h[i] + l[i]) / 2.0;
        let cur_upper = hl2 + multiplier * atr[i];
        let cur_lower = hl2 - multiplier * atr[i];

        if i == period - 1 {
            upper[i] = cur_upper;
            lower[i] = cur_lower;
            trend[i] = c[i] > ((upper[i] + lower[i]) / 2.0);
        } else {
            upper[i] = if cur_upper < upper[i - 1] || c[i - 1] > upper[i - 1] { cur_upper } else { upper[i - 1] };
            lower[i] = if cur_lower > lower[i - 1] || c[i - 1] < lower[i - 1] { cur_lower } else { lower[i - 1] };

            if trend[i - 1] && c[i] < lower[i] { trend[i] = false; }
            else if !trend[i - 1] && c[i] > upper[i] { trend[i] = true; }
            else { trend[i] = trend[i - 1]; }
        }
        result[i] = if trend[i] { lower[i] } else { upper[i] };
    }

    Ok(vec![IndicatorOutput {
        name: format!("ST({},{})", period, multiplier), values: Column::F64(result), style: OutputStyle::Dots,
    }])
}

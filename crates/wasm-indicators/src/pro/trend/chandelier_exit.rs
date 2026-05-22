use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn calc_atr(h: &[f64], l: &[f64], c: &[f64], period: usize) -> Vec<f64> {
    let n = c.len();
    let mut tr = vec![0.0; n];
    tr[0] = h[0] - l[0];
    for i in 1..n {
        let a = h[i] - l[i];
        let b = (h[i] - c[i - 1]).abs();
        let d = (l[i] - c[i - 1]).abs();
        tr[i] = a.max(b).max(d);
    }
    let mut result = vec![f64::NAN; n];
    if n < period {
        return result;
    }
    let mut atr = tr[0..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = atr;
    for i in period..n {
        atr = (atr * (period - 1) as f64 + tr[i]) / period as f64;
        result[i] = atr;
    }
    result
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(22.0) as usize;
    let atr_period = params.get("atr_period").copied().unwrap_or(22.0) as usize;
    let multiplier = params.get("multiplier").copied().unwrap_or(3.0);

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    if n < period.max(atr_period) {
        return Err(IndError::DataInsufficient(period.max(atr_period)));
    }

    let atr = calc_atr(&h, &l, &c, atr_period);

    let mut long_stop = vec![f64::NAN; n];
    let mut short_stop = vec![f64::NAN; n];

    for i in (period - 1)..n {
        let win_start = i + 1 - period;

        // Highest high over `period` bars
        let mut hh = f64::NEG_INFINITY;
        for j in win_start..=i {
            hh = hh.max(h[j]);
        }

        // Lowest low over `period` bars
        let mut ll = f64::INFINITY;
        for j in win_start..=i {
            ll = ll.min(l[j]);
        }

        if atr[i].is_finite() {
            // Long stop (trailing stop for long positions): below the highest high
            long_stop[i] = hh - multiplier * atr[i];
            // Short stop (trailing stop for short positions): above the lowest low
            short_stop[i] = ll + multiplier * atr[i];
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("CE({},{},{})", period, atr_period, multiplier),
        values: Column::F64(short_stop.clone()),
        style: OutputStyle::Band {
            upper: Column::F64(short_stop),
            lower: Column::F64(long_stop),
        },
    }])
}

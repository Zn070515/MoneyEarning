use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = c.len();

    let mut sma = vec![f64::NAN; n];
    if n >= period {
        let mut sum: f64 = c[0..period].iter().sum();
        for i in period - 1..n {
            if i == period - 1 {
                sma[i] = sum / period as f64;
            } else {
                sum += c[i] - c[i - period];
                sma[i] = sum / period as f64;
            }
        }
    }

    let mut hi_arr = vec![f64::NAN; n];
    let mut lo_arr = vec![f64::NAN; n];
    let mut mid_arr = vec![f64::NAN; n];
    for i in 0..n {
        if sma[i].is_finite() {
            hi_arr[i] = h[i].max(sma[i]);
            lo_arr[i] = l[i].min(sma[i]);
            mid_arr[i] = (hi_arr[i] + lo_arr[i]) / 2.0;
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("HILO_H({})", period),
            values: Column::F64(hi_arr),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("HILO_L({})", period),
            values: Column::F64(lo_arr),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("HILO_M({})", period),
            values: Column::F64(mid_arr),
            style: OutputStyle::Line,
        },
    ])
}

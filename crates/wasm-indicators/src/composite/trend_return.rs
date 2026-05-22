use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];

    let mut sma = vec![f64::NAN; n];
    let mut sum: f64 = 0.0;
    for i in 0..n {
        sum += c[i];
        if i >= period - 1 {
            if i >= period { sum -= c[i - period]; }
            sma[i] = sum / period as f64;
        }
    }

    for i in 0..n {
        if sma[i].is_finite() && sma[i] != 0.0 {
            result[i] = (c[i] / sma[i] - 1.0) * 100.0;
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("TREND_RET({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

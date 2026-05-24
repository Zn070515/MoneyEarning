use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();
    if n < period || period == 0 { return Err(IndError::DataInsufficient(period)); }

    let mut tr = vec![0.0; n];
    tr[0] = h[0] - l[0];
    for i in 1..n {
        let a = h[i] - l[i];
        let b = (h[i] - c[i - 1]).abs();
        let d = (l[i] - c[i - 1]).abs();
        tr[i] = a.max(b).max(d);
    }

    let mut result = vec![f64::NAN; n];
    let mut atr = tr[0..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = atr;
    for i in period..n {
        atr = (atr * (period - 1) as f64 + tr[i]) / period as f64;
        result[i] = atr;
    }

    Ok(vec![IndicatorOutput {
        name: format!("ATR({})", period), values: Column::F64(result), style: OutputStyle::Line,
    }])
}

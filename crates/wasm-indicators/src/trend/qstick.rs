use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let open = df.column("open").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let o = open.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();
    if n < period { return Err(IndError::DataInsufficient(period)); }

    let mut diff = vec![f64::NAN; n];
    for i in 0..n { diff[i] = c[i] - o[i]; }

    let mut result = vec![f64::NAN; n];
    let mut sum: f64 = diff[0..period].iter().filter(|v| v.is_finite()).sum();
    let count = period;
    for i in period - 1..n {
        if i == period - 1 { result[i] = sum / count as f64; }
        else { sum += diff[i] - diff[i - period]; result[i] = sum / count as f64; }
    }

    Ok(vec![IndicatorOutput {
        name: format!("QSTICK({})", period),
        values: Column::F64(result),
        style: OutputStyle::Histogram,
    }])
}

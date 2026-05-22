use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = h.len();

    let mut result = vec![f64::NAN; n];
    if n > 0 { result[0] = h[0] - l[0]; }
    for i in 1..n {
        result[i] = (h[i] - l[i]).max((h[i] - c[i-1]).abs()).max((l[i] - c[i-1]).abs());
    }

    Ok(vec![IndicatorOutput {
        name: "TR".into(),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

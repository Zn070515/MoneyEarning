use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();

    let mut result = vec![f64::NAN; n];
    let mut cum = 0.0;
    result[0] = 0.0;
    for i in 1..n {
        if c[i-1] != 0.0 {
            cum += v[i] * (c[i] - c[i-1]) / c[i-1];
        }
        result[i] = cum;
    }

    Ok(vec![IndicatorOutput {
        name: "PVT".into(),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();

    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        result[i] = c[i] * v[i];
    }

    Ok(vec![IndicatorOutput {
        name: "PVOL".into(),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

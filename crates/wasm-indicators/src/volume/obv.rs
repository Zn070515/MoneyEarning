use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let _ = params;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();
    let mut obv = vec![0.0; n];
    obv[0] = v[0];
    for i in 1..n {
        if c[i] > c[i - 1] { obv[i] = obv[i - 1] + v[i]; }
        else if c[i] < c[i - 1] { obv[i] = obv[i - 1] - v[i]; }
        else { obv[i] = obv[i - 1]; }
    }
    Ok(vec![IndicatorOutput {
        name: "OBV".into(), values: Column::F64(obv), style: OutputStyle::Line,
    }])
}

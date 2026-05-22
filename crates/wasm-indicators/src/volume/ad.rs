use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let _ = params;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();

    let mut result = vec![0.0; n];
    let mut adl = 0.0;
    for i in 0..n {
        let range = h[i] - l[i];
        let mfm = if range > 0.0 {
            ((c[i] - l[i]) - (h[i] - c[i])) / range
        } else { 0.0 };
        let mfv = mfm * v[i];
        adl += mfv;
        result[i] = adl;
    }
    Ok(vec![IndicatorOutput {
        name: "AD".into(), values: Column::F64(result), style: OutputStyle::Line,
    }])
}

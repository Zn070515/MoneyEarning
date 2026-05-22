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

    let mut result = vec![f64::NAN; n];
    let mut cum_pv = 0.0;
    let mut cum_v = 0.0;
    for i in 0..n {
        let typical = (h[i] + l[i] + c[i]) / 3.0;
        cum_pv += typical * v[i];
        cum_v += v[i];
        if cum_v > 0.0 { result[i] = cum_pv / cum_v; }
    }
    Ok(vec![IndicatorOutput {
        name: "VWAP".into(), values: Column::F64(result), style: OutputStyle::Line,
    }])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();

    let mut result = vec![f64::NAN; n];
    for i in (period - 1)..n {
        let mut sum_pv = 0.0;
        let mut sum_v = 0.0;
        for j in i + 1 - period..=i {
            sum_pv += c[j] * v[j];
            sum_v += v[j];
        }
        if sum_v > 0.0 { result[i] = sum_pv / sum_v; }
    }
    Ok(vec![IndicatorOutput {
        name: format!("VWMA({})", period), values: Column::F64(result), style: OutputStyle::Line,
    }])
}

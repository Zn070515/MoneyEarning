use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    let mut tp = vec![0.0; n];
    for i in 0..n { tp[i] = (h[i] + l[i] + c[i]) / 3.0; }

    let mut result = vec![f64::NAN; n];
    for i in (period - 1)..n {
        let slice = &tp[i + 1 - period..=i];
        let mean = slice.iter().sum::<f64>() / period as f64;
        let mad = slice.iter().map(|v| (v - mean).abs()).sum::<f64>() / period as f64;
        if mad > 0.0 { result[i] = (tp[i] - mean) / (0.015 * mad); }
    }
    Ok(vec![IndicatorOutput {
        name: format!("CCI({})", period), values: Column::F64(result), style: OutputStyle::Line,
    }])
}

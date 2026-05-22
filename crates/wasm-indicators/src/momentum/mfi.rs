use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();

    let mut tp = vec![0.0; n];
    for i in 0..n { tp[i] = (h[i] + l[i] + c[i]) / 3.0; }

    let mut pos_flow = vec![0.0; n];
    let mut neg_flow = vec![0.0; n];
    for i in 1..n {
        let mf = tp[i] * v[i];
        if tp[i] > tp[i - 1] { pos_flow[i] = mf; } else { neg_flow[i] = mf; }
    }

    let mut result = vec![f64::NAN; n];
    for i in period..n {
        let pos_sum: f64 = pos_flow[i + 1 - period..=i].iter().sum();
        let neg_sum: f64 = neg_flow[i + 1 - period..=i].iter().sum();
        if neg_sum > 0.0 {
            let ratio = pos_sum / neg_sum;
            result[i] = 100.0 - 100.0 / (1.0 + ratio);
        } else { result[i] = 100.0; }
    }
    Ok(vec![IndicatorOutput {
        name: format!("MFI({})", period), values: Column::F64(result), style: OutputStyle::Line,
    }])
}

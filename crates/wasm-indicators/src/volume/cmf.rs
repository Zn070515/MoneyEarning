use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
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
    for i in (period - 1)..n {
        let mut mf_sum = 0.0;
        let mut vol_sum = 0.0;
        for j in i + 1 - period..=i {
            let range = h[j] - l[j];
            let mfm = if range > 0.0 {
                ((c[j] - l[j]) - (h[j] - c[j])) / range
            } else { 0.0 };
            mf_sum += mfm * v[j];
            vol_sum += v[j];
        }
        if vol_sum > 0.0 { result[i] = mf_sum / vol_sum; }
    }
    Ok(vec![IndicatorOutput {
        name: format!("CMF({})", period), values: Column::F64(result), style: OutputStyle::Line,
    }])
}

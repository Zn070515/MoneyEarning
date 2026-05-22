use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let close2_col = params.get("col").map(|v| format!("close_{}", *v as usize)).unwrap_or_else(|| "close".to_string());
    let close2 = df.column(&close2_col).ok_or(IndError::InvalidParams(format!("找不到列: {}", close2_col)))?;
    let c1 = close.to_f64_vec();
    let c2 = close2.to_f64_vec();
    let n = c1.len().min(c2.len());

    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        let slice1 = &c1[i + 1 - period..=i];
        let slice2 = &c2[i + 1 - period..=i];
        let mean1: f64 = slice1.iter().sum::<f64>() / period as f64;
        let mean2: f64 = slice2.iter().sum::<f64>() / period as f64;
        let mut cov = 0.0;
        let mut var1 = 0.0;
        let mut var2 = 0.0;
        for j in 0..period {
            let d1 = slice1[j] - mean1;
            let d2 = slice2[j] - mean2;
            cov += d1 * d2;
            var1 += d1 * d1;
            var2 += d2 * d2;
        }
        let denom = (var1 * var2).sqrt();
        if denom != 0.0 { result[i] = cov / denom; }
    }

    Ok(vec![IndicatorOutput {
        name: format!("CORR({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

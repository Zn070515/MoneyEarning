use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];

    for i in period - 1..n {
        let start = i + 1 - period;
        let p = period as f64;
        let x_sum: f64 = (0..period).map(|k| k as f64).sum();
        let y_sum: f64 = c[start..=i].iter().sum();
        let xy_sum: f64 = (0..period).map(|k| k as f64 * c[start + k]).sum();
        let x2_sum: f64 = (0..period).map(|k| (k * k) as f64).sum();
        let denom = p * x2_sum - x_sum * x_sum;
        if denom != 0.0 {
            let slope = (p * xy_sum - x_sum * y_sum) / denom;
            let intercept = (y_sum - slope * x_sum) / p;
            result[i] = intercept + slope * (period - 1) as f64;
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("LINREG({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

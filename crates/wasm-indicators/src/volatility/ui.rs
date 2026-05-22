use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    if n < period { return Err(IndError::DataInsufficient(period)); }

    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        let slice = &c[i + 1 - period..=i];
        let max_c = slice.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mut sum_sq = 0.0;
        for &val in slice {
            if max_c != 0.0 {
                let drawdown_pct = (max_c - val) / max_c * 100.0;
                sum_sq += drawdown_pct * drawdown_pct;
            }
        }
        result[i] = (sum_sq / period as f64).sqrt();
    }

    Ok(vec![IndicatorOutput {
        name: format!("UI({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

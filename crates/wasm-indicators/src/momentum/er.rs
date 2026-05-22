use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(10.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];

    for i in period..n {
        let direction = (c[i] - c[i - period]).abs();
        let mut volatility = 0.0;
        for j in i + 1 - period..=i {
            volatility += (c[j] - c[j - 1]).abs();
        }
        if volatility != 0.0 { result[i] = direction / volatility; }
    }

    Ok(vec![IndicatorOutput {
        name: format!("ER({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

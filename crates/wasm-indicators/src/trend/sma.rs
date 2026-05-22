use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let vals = close.to_f64_vec();
    let n = vals.len();

    if n < period {
        return Err(IndError::DataInsufficient(period));
    }

    let mut result = Vec::with_capacity(n);
    let mut window_sum = vals[0..period].iter().sum::<f64>();

    for i in 0..n {
        if i < period - 1 {
            result.push(f64::NAN);
        } else if i == period - 1 {
            result.push(window_sum / period as f64);
        } else {
            window_sum += vals[i] - vals[i - period];
            result.push(window_sum / period as f64);
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("SMA({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

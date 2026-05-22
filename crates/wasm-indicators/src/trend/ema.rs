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
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Seed EMA with SMA for the first period
    let sma_init: f64 = vals[0..period].iter().sum::<f64>() / period as f64;
    let mut ema_val = sma_init;

    for i in 0..n {
        if i < period - 1 {
            result.push(f64::NAN);
        } else if i == period - 1 {
            result.push(ema_val);
        } else {
            ema_val = (vals[i] - ema_val) * multiplier + ema_val;
            result.push(ema_val);
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("EMA({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

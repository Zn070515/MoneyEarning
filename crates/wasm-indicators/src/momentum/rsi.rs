use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let vals = close.to_f64_vec();
    let n = vals.len();

    if n < period + 1 {
        return Err(IndError::DataInsufficient(period + 1));
    }

    let mut changes = Vec::with_capacity(n - 1);
    for i in 1..n {
        changes.push(vals[i] - vals[i - 1]);
    }

    let mut gains = Vec::with_capacity(n - 1);
    let mut losses = Vec::with_capacity(n - 1);
    for &c in &changes {
        if c >= 0.0 {
            gains.push(c);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-c);
        }
    }

    // Seed with SMA for first period
    let avg_gain_init = gains[0..period].iter().sum::<f64>() / period as f64;
    let avg_loss_init = losses[0..period].iter().sum::<f64>() / period as f64;

    let mut result = vec![f64::NAN; n];
    let mut avg_gain = avg_gain_init;
    let mut avg_loss = avg_loss_init;

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

        let rs = if avg_loss == 0.0 {
            100.0
        } else {
            avg_gain / avg_loss
        };
        result[i + 1] = 100.0 - (100.0 / (1.0 + rs));
    }

    Ok(vec![IndicatorOutput {
        name: format!("RSI({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

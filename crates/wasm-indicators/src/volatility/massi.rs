use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(25.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();
    if n < period + 8 { return Err(IndError::DataInsufficient(period + 8)); }

    let mut hl = vec![f64::NAN; n];
    for i in 0..n { hl[i] = h[i] - l[i]; }

    // EMA of high-low range (9-period)
    let alpha9 = 2.0 / (9.0 + 1.0);
    let mut ema9 = vec![f64::NAN; n];
    ema9[0] = hl[0];
    for i in 1..n { ema9[i] = alpha9 * hl[i] + (1.0 - alpha9) * ema9[i-1]; }

    // EMA of EMA9 (9-period)
    let mut ema2 = vec![f64::NAN; n];
    ema2[0] = ema9[0];
    for i in 1..n { ema2[i] = alpha9 * ema9[i] + (1.0 - alpha9) * ema2[i-1]; }

    let mut ratio = vec![f64::NAN; n];
    for i in 0..n {
        if ema2[i] != 0.0 { ratio[i] = ema9[i] / ema2[i]; }
    }

    let mut result = vec![f64::NAN; n];
    let mut sum = 0.0;
    for i in 0..n {
        if ratio[i].is_finite() { sum += ratio[i]; }
        if i >= period {
            if ratio[i - period].is_finite() { sum -= ratio[i - period]; }
            result[i] = sum;
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("MASSI({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

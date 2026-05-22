use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn sma(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    if n < period {
        return result;
    }
    let mut sum: f64 = vals[0..period].iter().sum();
    result[period - 1] = sum / period as f64;
    for i in period..n {
        sum = sum - vals[i - period] + vals[i];
        result[i] = sum / period as f64;
    }
    result
}

/// Awesome Oscillator (Bill Williams) — 5-period SMA(median) minus 34-period SMA(median).
///
/// Params: none (fixed periods 5 and 34).
///
/// Output: Histogram.
pub fn compute(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();

    if n < 34 {
        return Err(IndError::DataInsufficient(34));
    }

    // Median price = (high + low) / 2
    let mut median = vec![0.0; n];
    for i in 0..n {
        median[i] = (h[i] + l[i]) / 2.0;
    }

    let sma5 = sma(&median, 5);
    let sma34 = sma(&median, 34);

    let mut ao = vec![f64::NAN; n];
    for i in 34..n {
        if sma5[i].is_finite() && sma34[i].is_finite() {
            ao[i] = sma5[i] - sma34[i];
        }
    }

    Ok(vec![IndicatorOutput {
        name: "AO".to_string(),
        values: Column::F64(ao),
        style: OutputStyle::Histogram,
    }])
}

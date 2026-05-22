use std::collections::HashMap;
use wasm_core::{Column, DataFrame, IndError, IndicatorOutput, OutputStyle};

/// Compute Simple Moving Average for a given period.
/// Returns NaN for indices before the first full window.
fn sma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let mut sum = 0.0;
    for i in 0..n {
        sum += data[i];
        if i >= period {
            sum -= data[i - period];
        }
        if i >= period - 1 {
            out[i] = sum / period as f64;
        }
    }
    out
}

/// Volume Oscillator.
///
/// A MACD-like oscillator on volume: the percentage difference between
/// a fast SMA and a slow SMA of volume.
///
/// Params:
/// - `fast` (default 5, range 2-50): fast SMA period
/// - `slow` (default 10, range 5-100): slow SMA period
pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let fast = params.get("fast").copied().unwrap_or(5.0) as usize;
    let slow = params.get("slow").copied().unwrap_or(10.0) as usize;

    if fast >= slow {
        return Err(IndError::InvalidParams(
            "fast must be less than slow".into(),
        ));
    }

    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let v = volume.to_f64_vec();
    let n = v.len();

    if n < slow {
        return Err(IndError::DataInsufficient(slow));
    }

    let fast_ma = sma(&v, fast);
    let slow_ma = sma(&v, slow);

    let mut vo = vec![f64::NAN; n];
    for i in 0..n {
        if slow_ma[i].is_finite() && slow_ma[i] != 0.0 && fast_ma[i].is_finite() {
            vo[i] = ((fast_ma[i] - slow_ma[i]) / slow_ma[i]) * 100.0;
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("VO({},{})", fast, slow),
        values: Column::F64(vo),
        style: OutputStyle::Histogram,
    }])
}

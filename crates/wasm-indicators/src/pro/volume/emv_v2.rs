use std::collections::HashMap;
use wasm_core::{Column, DataFrame, IndError, IndicatorOutput, OutputStyle};

/// Compute Simple Moving Average for a given period.
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

/// Enhanced Ease of Movement (Richard Arms).
///
/// Measures how easily prices move relative to volume. A rising EMV
/// suggests an uptrend on low resistance; a falling EMV suggests
/// difficulty moving upward.
///
/// Params:
/// - `period` (default 14, range 2-100): SMA smoothing period
pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;

    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = h.len();

    if n < 2 {
        return Err(IndError::DataInsufficient(2));
    }

    let mut emv_raw = vec![f64::NAN; n];
    for i in 1..n {
        let range = h[i] - l[i];
        // Box ratio: volume divided by range. Avoid division by zero.
        let box_ratio = if range > 0.0 && v[i] > 0.0 {
            v[i] / range
        } else if v[i] > 0.0 {
            v[i] / 0.001
        } else {
            f64::NAN
        };

        if box_ratio.is_finite() && box_ratio != 0.0 {
            let mid_move = (h[i] + l[i]) / 2.0 - (h[i - 1] + l[i - 1]) / 2.0;
            emv_raw[i] = (mid_move / box_ratio) * 1_000_000.0;
        }
    }

    // Apply SMA smoothing
    let emv = sma(&emv_raw, period);

    Ok(vec![IndicatorOutput {
        name: format!("EMV_V2({})", period),
        values: Column::F64(emv),
        style: OutputStyle::Line,
    }])
}

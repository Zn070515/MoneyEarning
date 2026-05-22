use std::collections::HashMap;
use wasm_core::{Column, DataFrame, IndError, IndicatorOutput, OutputStyle};

/// Compute Exponential Moving Average.
/// Initialized with SMA of the first `period` values, then applies EMA smoothing.
fn ema(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let k = 2.0 / (period as f64 + 1.0);

    // Find first block of `period` finite values for SMA initialization.
    let mut sma = 0.0;
    let mut count: usize = 0;
    let mut init_idx = 0;
    for i in 0..n {
        if data[i].is_finite() {
            if count == 0 {
                init_idx = i;
            }
            sma += data[i];
            count += 1;
            if count == period {
                let init_val = sma / period as f64;
                out[init_idx + period - 1] = init_val;
                // Continue EMA from here
                for j in (init_idx + period)..n {
                    if data[j].is_finite() {
                        out[j] = data[j] * k + out[j - 1] * (1.0 - k);
                    } else {
                        out[j] = out[j - 1];
                    }
                }
                return out;
            }
        } else {
            count = 0;
            sma = 0.0;
        }
    }
    out
}

/// Klinger Volume Oscillator.
///
/// Uses price direction multiplied by volume to create a money-flow measure,
/// then applies a dual EMA difference to produce the KVO line and a signal line.
///
/// Params:
/// - `fast` (default 34, range 10-100): fast EMA period
/// - `slow` (default 55, range 20-200): slow EMA period
/// - `signal` (default 13, range 2-50): signal line EMA period
pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let fast = params.get("fast").copied().unwrap_or(34.0) as usize;
    let slow = params.get("slow").copied().unwrap_or(55.0) as usize;
    let signal = params.get("signal").copied().unwrap_or(13.0) as usize;

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;

    let c = close.to_f64_vec();
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();

    if n < 2 {
        return Err(IndError::DataInsufficient(2));
    }

    // Compute volume force: price-direction * volume
    let mut force = vec![f64::NAN; n];
    for i in 1..n {
        let prev_sum = c[i - 1] + h[i - 1] + l[i - 1];
        let curr_sum = c[i] + h[i] + l[i];
        let direction = if curr_sum > prev_sum {
            1.0
        } else if curr_sum < prev_sum {
            -1.0
        } else {
            0.0
        };
        force[i] = direction * v[i];
    }

    // KVO = EMA(force, fast) - EMA(force, slow)
    let ema_fast = ema(&force, fast);
    let ema_slow = ema(&force, slow);
    let mut kvo = vec![f64::NAN; n];
    for i in 0..n {
        if ema_fast[i].is_finite() && ema_slow[i].is_finite() {
            kvo[i] = ema_fast[i] - ema_slow[i];
        }
    }

    // Signal line = EMA(KVO, signal)
    let signal_line = ema(&kvo, signal);

    Ok(vec![
        IndicatorOutput {
            name: format!("KVO({},{},{})", fast, slow, signal),
            values: Column::F64(kvo),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("KVO_Signal({},{},{})", fast, slow, signal),
            values: Column::F64(signal_line),
            style: OutputStyle::Line,
        },
    ])
}

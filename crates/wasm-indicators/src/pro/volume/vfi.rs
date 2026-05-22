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

/// Volume Flow Indicator (Markos Katsanos).
///
/// Measures volume flow direction and strength using the typical price
/// movement weighted by volume, filtered through a cutoff and normalized
/// by a volume EMA.
///
/// Params:
/// - `period` (default 130, range 20-300): EMA period
/// - `coef` (default 0.2, range 0.05-0.5): cutoff coefficient
/// - `vcoef` (default 2.5, range 1-5): output scaling multiplier
pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(130.0) as usize;
    let coef = params.get("coef").copied().unwrap_or(0.2);
    let vcoef = params.get("vcoef").copied().unwrap_or(2.5);

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;

    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();

    if n < period + 1 {
        return Err(IndError::DataInsufficient(period + 1));
    }

    // typical price = (H + L + C) / 3
    let mut typical = vec![0.0; n];
    for i in 0..n {
        typical[i] = (h[i] + l[i] + c[i]) / 3.0;
    }

    // Compute rolling SMA of (typical * volume) for cutoff reference
    let mut vol_flow_raw = vec![f64::NAN; n];
    for i in 1..n {
        vol_flow_raw[i] = (typical[i] - typical[i - 1]) * v[i];
    }

    // Compute rolling standard deviation of close changes for volatility cutoff
    let mut close_changes = vec![f64::NAN; n];
    for i in 1..n {
        close_changes[i] = c[i] - c[i - 1];
    }

    // Compute the filtered flow using cutoff logic
    // cutoff = coef * vc * close, where vc approximates relative volatility
    let mut flow = vec![f64::NAN; n];
    for i in period..n {
        // Compute stddev of close changes over the window
        let mut sum = 0.0;
        let mut cnt = 0;
        for j in i + 1 - period..=i {
            if close_changes[j].is_finite() {
                sum += close_changes[j];
                cnt += 1;
            }
        }
        if cnt == 0 {
            continue;
        }
        let mean_change = sum / cnt as f64;
        let mut var_sum = 0.0;
        for j in i + 1 - period..=i {
            if close_changes[j].is_finite() {
                let diff = close_changes[j] - mean_change;
                var_sum += diff * diff;
            }
        }
        let stddev = (var_sum / cnt as f64).sqrt();

        // Compute cutoff threshold
        let vc = if stddev > 0.0 {
            (c[i] - c[i - 1]).abs().sqrt() / stddev
        } else {
            0.0
        };
        let cutoff = coef * vc * c[i];

        // Apply cutoff: only include flow above the noise threshold
        let raw_flow = (typical[i] - typical[i - 1]) * v[i];
        if raw_flow.abs() > cutoff {
            flow[i] = raw_flow;
        } else {
            flow[i] = 0.0;
        }
    }

    // EMA of flow and EMA of volume
    let ema_flow = ema(&flow, period);
    let ema_vol = ema(&v, period);

    let mut vfi = vec![f64::NAN; n];
    for i in 0..n {
        if ema_flow[i].is_finite() && ema_vol[i].is_finite() && ema_vol[i] != 0.0 {
            vfi[i] = (ema_flow[i] / ema_vol[i]) * vcoef;
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("VFI({},{},{})", period, coef, vcoef),
        values: Column::F64(vfi),
        style: OutputStyle::Line,
    }])
}

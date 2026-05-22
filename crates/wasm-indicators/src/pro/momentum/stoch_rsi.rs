use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn rsi_series(close: &[f64], period: usize) -> Vec<f64> {
    let n = close.len();
    let mut result = vec![f64::NAN; n];
    if n < period + 1 {
        return result;
    }

    let mut gains = Vec::with_capacity(n - 1);
    let mut losses = Vec::with_capacity(n - 1);
    for i in 1..n {
        let change = close[i] - close[i - 1];
        if change >= 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    let avg_gain_init = gains[0..period].iter().sum::<f64>() / period as f64;
    let avg_loss_init = losses[0..period].iter().sum::<f64>() / period as f64;

    let mut avg_gain = avg_gain_init;
    let mut avg_loss = avg_loss_init;

    // RSI at position period (index period in original array = index period-1 in changes)
    // Wait: first RSI value is at index period in the original array
    // gains[i] corresponds to change from close[i] to close[i+1]
    // So after processing gains[period-1], the RSI applies to close[period]
    // We need to compute RSI for close indices period..n
    // Let's compute them and place at the right indices.

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;
        let rs = if avg_loss == 0.0 {
            100.0
        } else {
            avg_gain / avg_loss
        };
        // This RSI applies to close[i+1]
        result[i + 1] = 100.0 - (100.0 / (1.0 + rs));
    }

    result
}

fn sma(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut out = vec![f64::NAN; n];
    let mut sum = 0.0;
    let mut count = 0usize;
    for i in 0..n {
        if vals[i].is_finite() {
            sum += vals[i];
            count += 1;
        }
        if count > period && vals[i.max(period) - period].is_finite() {
            sum -= vals[i - period];
            count -= 1;
        }
        if count >= period {
            out[i] = sum / period as f64;
        }
    }
    out
}

fn llv(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    let mut min_val = f64::MAX;
    for i in 0..n {
        min_val = min_val.min(vals[i]);
        if i >= period && vals[i - period] == min_val {
            min_val = vals[i - period + 1..=i]
                .iter()
                .cloned()
                .fold(f64::MAX, f64::min);
        }
        if i >= period - 1 {
            result[i] = min_val;
        }
    }
    result
}

fn hhv(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    let mut max_val = f64::MIN;
    for i in 0..n {
        max_val = max_val.max(vals[i]);
        if i >= period && vals[i - period] == max_val {
            max_val = vals[i - period + 1..=i]
                .iter()
                .cloned()
                .fold(f64::MIN, f64::max);
        }
        if i >= period - 1 {
            result[i] = max_val;
        }
    }
    result
}

pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let k = params.get("k").copied().unwrap_or(3.0) as usize;
    let d = params.get("d").copied().unwrap_or(3.0) as usize;

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    // Need enough data: RSI requires period+1, then stoch needs another period, plus smoothing
    let min_required = period * 2 + k + d;
    if n < min_required {
        return Err(IndError::DataInsufficient(min_required));
    }

    // Compute RSI series
    let rsi = rsi_series(&c, period);

    // Stochastic of RSI: stoch = (RSI - lowest(RSI, period)) / (highest(RSI, period) - lowest(RSI, period))
    let rsi_ll = llv(&rsi, period);
    let rsi_hh = hhv(&rsi, period);

    let mut stoch = vec![f64::NAN; n];
    for i in 0..n {
        if rsi_hh[i].is_finite() && rsi_ll[i].is_finite() {
            let denom = rsi_hh[i] - rsi_ll[i];
            if denom > 0.0 && rsi[i].is_finite() {
                stoch[i] = (rsi[i] - rsi_ll[i]) / denom * 100.0;
            } else if denom == 0.0 && rsi[i].is_finite() {
                stoch[i] = 50.0;
            }
        }
    }

    // k_line = SMA(stoch, k)
    let k_line = sma(&stoch, k);

    // d_line = SMA(k_line, d)
    let d_line = sma(&k_line, d);

    Ok(vec![
        IndicatorOutput {
            name: format!("STOCH_RSI_K({},{})", period, k),
            values: Column::F64(k_line),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("STOCH_RSI_D({},{})", period, d),
            values: Column::F64(d_line),
            style: OutputStyle::Line,
        },
    ])
}

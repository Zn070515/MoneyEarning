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

fn ema(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    if n < period {
        return result;
    }
    let multiplier = 2.0 / (period as f64 + 1.0);
    let first_sum: f64 = vals[0..period].iter().sum();
    result[period - 1] = first_sum / period as f64;
    for i in period..n {
        result[i] = (vals[i] - result[i - 1]) * multiplier + result[i - 1];
    }
    result
}

/// RSI computation returning full array (NaN before period+1).
fn rsi(close: &[f64], period: usize) -> Vec<f64> {
    let n = close.len();
    let mut result = vec![f64::NAN; n];
    if n < period + 1 {
        return result;
    }

    let mut gains = Vec::with_capacity(n - 1);
    let mut losses = Vec::with_capacity(n - 1);
    for i in 1..n {
        let diff = close[i] - close[i - 1];
        if diff >= 0.0 {
            gains.push(diff);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-diff);
        }
    }

    let avg_gain_init = gains[0..period].iter().sum::<f64>() / period as f64;
    let avg_loss_init = losses[0..period].iter().sum::<f64>() / period as f64;
    let mut avg_gain = avg_gain_init;
    let mut avg_loss = avg_loss_init;

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;
        let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
        result[i + 1] = 100.0 - (100.0 / (1.0 + rs));
    }
    result
}

/// MACD computation returning (macd_line, signal_line).
fn macd(close: &[f64], fast: usize, slow: usize, signal: usize) -> (Vec<f64>, Vec<f64>) {
    let n = close.len();
    let ema_fast = ema(close, fast);
    let ema_slow = ema(close, slow);

    let mut macd_line = vec![f64::NAN; n];
    for i in 0..n {
        if ema_fast[i].is_finite() && ema_slow[i].is_finite() {
            macd_line[i] = ema_fast[i] - ema_slow[i];
        }
    }

    let signal_line = ema(&macd_line, signal);
    (macd_line, signal_line)
}

/// Bollinger Bands computation returning (mid, upper, lower).
fn bollinger(close: &[f64], period: usize, stddev_mult: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = close.len();
    let mid = sma(close, period);
    let mut upper = vec![f64::NAN; n];
    let mut lower = vec![f64::NAN; n];
    for i in (period - 1)..n {
        let start = i + 1 - period;
        let slice = &close[start..=i];
        let mean = mid[i];
        let var = slice.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / period as f64;
        let sd = var.sqrt();
        upper[i] = mean + stddev_mult * sd;
        lower[i] = mean - stddev_mult * sd;
    }
    (mid, upper, lower)
}

/// Decision Point Score — composite score combining RSI, MACD, and Bollinger %b.
/// Score range: -100 to +100.
///
/// Params: rsi_period (default 14), macd_fast (default 12), macd_slow (default 26),
///         bb_period (default 20)
///
/// Output: Line.
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let rsi_period = params.get("rsi_period").copied().unwrap_or(14.0) as usize;
    let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
    let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
    let bb_period = params.get("bb_period").copied().unwrap_or(20.0) as usize;

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    let min_period = rsi_period.max(macd_slow).max(bb_period) + 1;
    if n < min_period {
        return Err(IndError::DataInsufficient(min_period));
    }

    // RSI
    let rsi_vals = rsi(&c, rsi_period);

    // MACD
    let (macd_line, signal_line) = macd(&c, macd_fast, macd_slow, 9);

    // Bollinger Bands
    let (_, bb_upper, bb_lower) = bollinger(&c, bb_period, 2.0);

    let mut score = vec![f64::NAN; n];

    for i in 0..n {
        // RSI component: (RSI - 50) / 50, clamped to [-1, 1]
        let rsi_comp = if rsi_vals[i].is_finite() {
            ((rsi_vals[i] - 50.0) / 50.0).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        // MACD component: sign(MACD - signal) * min(1, abs(diff)/signal * 10)
        let macd_comp = if macd_line[i].is_finite() && signal_line[i].is_finite() {
            let diff = macd_line[i] - signal_line[i];
            let magnitude = if signal_line[i].abs() > 1e-9 {
                (diff.abs() / signal_line[i].abs() * 10.0).min(1.0)
            } else {
                if diff.abs() > 1e-9 { 1.0 } else { 0.0 }
            };
            diff.signum() * magnitude
        } else {
            0.0
        };

        // Bollinger %b component: (close - lower) / (upper - lower), normalized to [-1, 1]
        let bb_comp = if bb_upper[i].is_finite() && bb_lower[i].is_finite() {
            let bandwidth = bb_upper[i] - bb_lower[i];
            if bandwidth > 1e-9 {
                let pct_b = (c[i] - bb_lower[i]) / bandwidth;
                ((pct_b - 0.5) * 2.0).clamp(-1.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        score[i] = (rsi_comp + macd_comp + bb_comp) / 3.0 * 100.0;
    }

    Ok(vec![IndicatorOutput {
        name: "DP_SCORE".to_string(),
        values: Column::F64(score),
        style: OutputStyle::Line,
    }])
}

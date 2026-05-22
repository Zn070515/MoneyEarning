//! ME Script built-in function library
//! Implements technical indicators, window functions, math, logic, and candle patterns

/// Compute Simple Moving Average
pub fn sma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    let p = period.min(n);
    if p == 0 { return result; }
    for i in p.saturating_sub(1)..n {
        let start = i.saturating_sub(p.saturating_sub(1));
        result[i] = data[start..=i].iter().sum::<f64>() / p as f64;
    }
    result
}

/// Compute Exponential Moving Average
pub fn ema(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    if n == 0 || period == 0 { return result; }
    let alpha = 2.0 / (period as f64 + 1.0);
    result[0] = data[0];
    for i in 1..n {
        result[i] = data[i] * alpha + result[i - 1] * (1.0 - alpha);
    }
    result
}

/// Weighted Moving Average
pub fn wma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    let p = period.min(n);
    if p == 0 { return result; }
    let weight_sum = (p * (p + 1)) as f64 / 2.0;
    for i in p - 1..n {
        let mut sum = 0.0;
        for j in 0..p {
            sum += data[i - j] * (p - j) as f64;
        }
        result[i] = sum / weight_sum;
    }
    result
}

/// Wilder's RMA (RSI-style smoothing)
pub fn rma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    if n == 0 || period == 0 { return result; }
    let alpha = 1.0 / period as f64;
    result[0] = data[0];
    for i in 1..n {
        result[i] = data[i] * alpha + result[i - 1] * (1.0 - alpha);
    }
    result
}

/// Hull Moving Average
pub fn hma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let half = period / 2;
    let sqrt_p = (period as f64).sqrt() as usize;
    if half == 0 || sqrt_p == 0 { return vec![0.0; n]; }
    let wma_half = wma(data, half);
    let wma_full = wma(data, period);
    let mut diff = vec![0.0; n];
    for i in 0..n {
        if wma_half[i].is_finite() && wma_full[i].is_finite() {
            diff[i] = 2.0 * wma_half[i] - wma_full[i];
        }
    }
    wma(&diff, sqrt_p)
}

/// Standard Deviation
pub fn stdev(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    let p = period.min(n);
    if p == 0 { return result; }
    for i in p.saturating_sub(1)..n {
        let start = i.saturating_sub(p.saturating_sub(1));
        let window = &data[start..=i];
        let mean = window.iter().sum::<f64>() / p as f64;
        let variance = window.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / p as f64;
        result[i] = variance.sqrt();
    }
    result
}

/// Z-Score
pub fn zscore(data: &[f64], period: usize) -> Vec<f64> {
    let ma = sma(data, period);
    let sd = stdev(data, period);
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        if sd[i].is_finite() && sd[i] > 1e-12 {
            result[i] = (data[i] - ma[i]) / sd[i];
        }
    }
    result
}

/// Rate of Change
pub fn roc(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    for i in period..n {
        if data[i - period].abs() > 1e-12 {
            result[i] = (data[i] - data[i - period]) / data[i - period] * 100.0;
        }
    }
    result
}

/// RSI
pub fn rsi(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![50.0; n];
    if n < period + 1 { return result; }
    let mut gains = 0.0;
    let mut losses = 0.0;
    for i in 1..=period {
        let diff = data[i] - data[i - 1];
        if diff > 0.0 { gains += diff; } else { losses += -diff; }
    }
    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;
    for i in period + 1..n {
        let diff = data[i] - data[i - 1];
        let gain = if diff > 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };
        avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;
        result[i] = if avg_loss > 0.0 {
            100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
        } else {
            100.0
        };
    }
    result
}

/// CCI (Commodity Channel Index)
pub fn cci(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let n = close.len();
    let mut result = vec![f64::NAN; n];
    let tp: Vec<f64> = (0..n).map(|i| (high[i] + low[i] + close[i]) / 3.0).collect();
    let ma = sma(&tp, period);
    let mut mean_dev = vec![0.0; n];
    for i in period - 1..n {
        let mut sum = 0.0;
        for j in i - period + 1..=i {
            sum += (tp[j] - ma[i]).abs();
        }
        mean_dev[i] = sum / period as f64;
    }
    for i in 0..n {
        if mean_dev[i] > 1e-12 {
            result[i] = (tp[i] - ma[i]) / (0.015 * mean_dev[i]);
        }
    }
    result
}

/// ATR (Average True Range)
pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let n = close.len();
    let mut tr = vec![0.0; n];
    tr[0] = high[0] - low[0];
    for i in 1..n {
        let h_l = high[i] - low[i];
        let h_pc = (high[i] - close[i - 1]).abs();
        let l_pc = (low[i] - close[i - 1]).abs();
        tr[i] = h_l.max(h_pc).max(l_pc);
    }
    rma(&tr, period)
}

/// Highest High Value over N periods
pub fn hhv(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = data[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    }
    result
}

/// Lowest Low Value over N periods
pub fn llv(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = data[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
    }
    result
}

/// Sum over N periods
pub fn sum(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = data[start..=i].iter().sum();
    }
    result
}

/// Reference N bars back
pub fn ref_n(data: &[f64], offset: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    for i in offset..n {
        result[i] = data[i - offset];
    }
    result
}

/// Count of true conditions in N periods
pub fn count(cond: &[bool], period: usize) -> Vec<f64> {
    let n = cond.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = cond[start..=i].iter().filter(|&&c| c).count() as f64;
    }
    result
}

/// True if condition was true every bar in N periods
pub fn every(cond: &[bool], period: usize) -> Vec<bool> {
    let n = cond.len();
    let mut result = vec![false; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = cond[start..=i].iter().all(|&c| c);
    }
    result
}

/// True if condition was true at least once in N periods
pub fn exist(cond: &[bool], period: usize) -> Vec<bool> {
    let n = cond.len();
    let mut result = vec![false; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = cond[start..=i].iter().any(|&c| c);
    }
    result
}

/// Bars since last true condition
pub fn barslast(cond: &[bool]) -> Vec<f64> {
    let n = cond.len();
    let mut result = vec![0.0; n];
    let mut last_true: Option<usize> = None;
    for i in 0..n {
        if cond[i] { last_true = Some(i); }
        result[i] = match last_true {
            Some(pos) => (i - pos) as f64,
            None => i as f64,
        };
    }
    result
}

/// Bars since first true condition
pub fn barssince(cond: &[bool]) -> Vec<f64> {
    let n = cond.len();
    let mut result = vec![0.0; n];
    let mut first_true: Option<usize> = None;
    for i in 0..n {
        if cond[i] && first_true.is_none() { first_true = Some(i); }
        result[i] = match first_true {
            Some(pos) => (i - pos) as f64,
            None => 0.0,
        };
    }
    result
}

/// Cross detection: a crosses above b (or below if above=false)
pub fn cross(a: &[f64], b: &[f64], above: bool) -> Vec<f64> {
    let n = a.len().min(b.len());
    let mut result = vec![0.0; a.len().max(b.len())];
    for i in 1..n {
        if above {
            result[i] = if a[i - 1] <= b[i - 1] && a[i] > b[i] { 1.0 } else { 0.0 };
        } else {
            result[i] = if a[i - 1] >= b[i - 1] && a[i] < b[i] { 1.0 } else { 0.0 };
        }
    }
    result
}

/// Filter: after condition is true, suppress for N periods
pub fn filter(cond: &[bool], period: usize) -> Vec<bool> {
    let n = cond.len();
    let mut result = vec![false; n];
    let mut block_until = 0;
    for i in 0..n {
        if i >= block_until && cond[i] {
            result[i] = true;
            block_until = i + period;
        }
    }
    result
}

/// Math functions (element-wise)
pub fn vec_abs(data: &[f64]) -> Vec<f64> { data.iter().map(|v| v.abs()).collect() }
pub fn vec_sqrt(data: &[f64]) -> Vec<f64> { data.iter().map(|v| v.sqrt()).collect() }
pub fn vec_log(data: &[f64]) -> Vec<f64> { data.iter().map(|v| if *v > 0.0 { v.log10() } else { f64::NAN }).collect() }
pub fn vec_ln(data: &[f64]) -> Vec<f64> { data.iter().map(|v| if *v > 0.0 { v.ln() } else { f64::NAN }).collect() }
pub fn vec_exp(data: &[f64]) -> Vec<f64> { data.iter().map(|v| v.exp()).collect() }
pub fn vec_round(data: &[f64]) -> Vec<f64> { data.iter().map(|v| v.round()).collect() }
pub fn vec_ceil(data: &[f64]) -> Vec<f64> { data.iter().map(|v| v.ceil()).collect() }
pub fn vec_floor(data: &[f64]) -> Vec<f64> { data.iter().map(|v| v.floor()).collect() }
pub fn vec_sign(data: &[f64]) -> Vec<f64> { data.iter().map(|v| if *v > 0.0 { 1.0 } else if *v < 0.0 { -1.0 } else { 0.0 }).collect() }

/// Element-wise max/min of two arrays
pub fn vec_max(a: &[f64], b: &[f64]) -> Vec<f64> {
    let n = a.len().min(b.len());
    (0..n).map(|i| a[i].max(b[i])).collect()
}
pub fn vec_min(a: &[f64], b: &[f64]) -> Vec<f64> {
    let n = a.len().min(b.len());
    (0..n).map(|i| a[i].min(b[i])).collect()
}

/// Sine/cosine for cycle analysis
pub fn vec_sin(data: &[f64]) -> Vec<f64> { data.iter().map(|v| v.sin()).collect() }
pub fn vec_cos(data: &[f64]) -> Vec<f64> { data.iter().map(|v| v.cos()).collect() }
pub fn vec_tan(data: &[f64]) -> Vec<f64> { data.iter().map(|v| v.tan()).collect() }

/// Between: check if x is between a and b (inclusive)
pub fn between(x: &[f64], a: &[f64], b: &[f64]) -> Vec<bool> {
    let n = x.len().min(a.len()).min(b.len());
    (0..n).map(|i| x[i] >= a[i].min(b[i]) && x[i] <= a[i].max(b[i])).collect()
}

/// Range: check if x is strictly between a and b
pub fn range(x: &[f64], a: &[f64], b: &[f64]) -> Vec<bool> {
    let n = x.len().min(a.len()).min(b.len());
    (0..n).map(|i| x[i] > a[i].min(b[i]) && x[i] < a[i].max(b[i])).collect()
}

/// OBV (On-Balance Volume)
pub fn obv(close: &[f64], volume: &[f64]) -> Vec<f64> {
    let n = close.len();
    let mut result = vec![0.0; n];
    result[0] = volume[0];
    for i in 1..n {
        if close[i] > close[i - 1] {
            result[i] = result[i - 1] + volume[i];
        } else if close[i] < close[i - 1] {
            result[i] = result[i - 1] - volume[i];
        } else {
            result[i] = result[i - 1];
        }
    }
    result
}

/// Compute O/H/L/C series from DataFrame field access helpers
pub fn get_field(data: &[f64], _field: &str) -> Vec<f64> {
    data.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma(&data, 3);
        assert!((result[2] - 2.0).abs() < 0.01);
        assert!((result[4] - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_cross() {
        let a = vec![1.0, 2.0, 3.0, 2.0, 1.0];
        let b = vec![1.5, 1.5, 1.5, 1.5, 1.5];
        let result = cross(&a, &b, true);
        assert!((result[1] - 1.0).abs() < 0.01); // a crosses above b at i=1
        assert!((result[2] - 0.0).abs() < 0.01); // no cross at i=2 (already above)
    }

    #[test]
    fn test_rsi() {
        let mut data = vec![10.0; 30];
        for i in 1..20 { data[i] = data[i - 1] + 0.1; }
        for i in 20..30 { data[i] = data[i - 1] - 0.1; }
        let result = rsi(&data, 14);
        assert!(result[25] >= 0.0 && result[25] <= 100.0);
    }

    #[test]
    fn test_hhv_llv() {
        let data = vec![1.0, 3.0, 2.0, 5.0, 4.0];
        let h = hhv(&data, 3);
        let l = llv(&data, 3);
        assert!((h[2] - 3.0).abs() < 0.01);
        assert!((h[3] - 5.0).abs() < 0.01);
        assert!((l[2] - 1.0).abs() < 0.01);
        assert!((l[3] - 2.0).abs() < 0.01);
    }
}

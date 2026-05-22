use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

/// Compute rolling Hurst exponent using Rescaled Range (R/S) analysis.
/// Values > 0.5 indicate trending, < 0.5 indicate mean-reversion, = 0.5 random walk.
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(100.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let vals = close.to_f64_vec();
    let n = vals.len();
    if n < period {
        return Err(IndError::DataInsufficient(period));
    }

    let mut result = vec![f64::NAN; n];

    for i in period - 1..n {
        let window = &vals[i + 1 - period..=i];
        let h = hurst_rs(window);
        result[i] = h;
    }

    Ok(vec![IndicatorOutput {
        name: format!("HURST({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

fn hurst_rs(data: &[f64]) -> f64 {
    let n = data.len();
    if n < 8 {
        return f64::NAN;
    }

    // Generate chunk sizes: n/2, n/4, n/8, ..., min 8
    let mut chunk_sizes: Vec<usize> = Vec::new();
    let mut sz = n / 2;
    while sz >= 8 && chunk_sizes.len() < 6 {
        chunk_sizes.push(sz);
        sz /= 2;
    }
    if chunk_sizes.is_empty() {
        return f64::NAN;
    }

    // Compute log-log data points for regression
    let mut log_sizes: Vec<f64> = Vec::new();
    let mut log_rs: Vec<f64> = Vec::new();

    for &chunk_sz in &chunk_sizes {
        let num_chunks = n / chunk_sz;
        if num_chunks == 0 {
            continue;
        }

        let mut rs_sum = 0.0;
        let mut valid_chunks = 0;

        for c in 0..num_chunks {
            let start = c * chunk_sz;
            let end = start + chunk_sz;
            let chunk = &data[start..end];

            let cnt = chunk.len() as f64;
            let mean = chunk.iter().sum::<f64>() / cnt;

            // Cumulative deviations
            let mut cum_dev = 0.0;
            let mut cum_max = f64::MIN;
            let mut cum_min = f64::MAX;
            for &v in chunk.iter() {
                cum_dev += v - mean;
                cum_max = cum_max.max(cum_dev);
                cum_min = cum_min.min(cum_dev);
            }

            let r = cum_max - cum_min;

            // Standard deviation
            let var = chunk.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / cnt;
            let s = var.sqrt();

            if s > 0.0 {
                rs_sum += r / s;
                valid_chunks += 1;
            }
        }

        if valid_chunks > 0 {
            log_sizes.push((chunk_sz as f64).ln());
            log_rs.push((rs_sum / valid_chunks as f64).ln());
        }
    }

    if log_sizes.len() < 2 {
        return f64::NAN;
    }

    // Linear regression: log(RS) = H * log(size) + intercept
    let m = log_sizes.len() as f64;
    let sum_x = log_sizes.iter().sum::<f64>();
    let sum_y = log_rs.iter().sum::<f64>();
    let sum_xy: f64 = log_sizes.iter().zip(log_rs.iter()).map(|(x, y)| x * y).sum();
    let sum_xx: f64 = log_sizes.iter().map(|x| x * x).sum();

    let denominator = m * sum_xx - sum_x * sum_x;
    if denominator.abs() < 1e-12 {
        return f64::NAN;
    }

    let h = (m * sum_xy - sum_x * sum_y) / denominator;

    // Clamp to reasonable range [0.0, 1.0]
    h.clamp(0.0, 1.0)
}

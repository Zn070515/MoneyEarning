use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(10.0) as usize;
    let fast = params.get("fast").copied().unwrap_or(2.0) as usize;
    let slow = params.get("slow").copied().unwrap_or(30.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    if n < period + 1 {
        return Err(IndError::DataInsufficient(period + 1));
    }

    let fast_sc = 2.0 / (fast as f64 + 1.0);
    let slow_sc = 2.0 / (slow as f64 + 1.0);

    let mut result = vec![f64::NAN; n];

    // Seed KAMA with SMA of first `period` bars
    let seed: f64 = c[0..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = seed;
    let mut kama = seed;

    for i in period..n {
        // Efficiency Ratio: absolute net change / sum of absolute bar-to-bar changes
        let direction = (c[i] - c[i - period]).abs();
        let mut volatility = 0.0;
        for j in (i - period + 1)..=i {
            volatility += (c[j] - c[j - 1]).abs();
        }
        let er = if volatility == 0.0 {
            0.0
        } else {
            direction / volatility
        };

        // Smoothing Constant
        let sc = (er * (fast_sc - slow_sc) + slow_sc).powi(2);

        // Kaufman Adaptive Moving Average
        kama = kama + sc * (c[i] - kama);
        result[i] = kama;
    }

    Ok(vec![IndicatorOutput {
        name: format!("KAMA({},{},{})", period, fast, slow),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(16.0) as usize;
    if period < 4 {
        return Err(IndError::InvalidParams(
            "period must be at least 4".into(),
        ));
    }
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    if n < period {
        return Err(IndError::DataInsufficient(period));
    }

    let half = period / 2;
    let mut result = vec![f64::NAN; n];

    // Seed FRAMA with SMA of first `period` bars
    let seed: f64 = c[0..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = seed;

    for i in period..n {
        let win_start = i - period + 1;

        // N1: range ratio in most recent half-window [i-half+1, i]
        let mut h1 = f64::NEG_INFINITY;
        let mut l1 = f64::INFINITY;
        for j in (i - half + 1)..=i {
            h1 = h1.max(c[j]);
            l1 = l1.min(c[j]);
        }
        let n1 = (h1 - l1) / half as f64;

        // N2: range ratio in older half-window [win_start, i-half]
        let mut h2 = f64::NEG_INFINITY;
        let mut l2 = f64::INFINITY;
        for j in win_start..=(i - half) {
            h2 = h2.max(c[j]);
            l2 = l2.min(c[j]);
        }
        let n2 = (h2 - l2) / half as f64;

        // N3: range ratio in full window [win_start, i]
        let mut h3 = f64::NEG_INFINITY;
        let mut l3 = f64::INFINITY;
        for j in win_start..=i {
            h3 = h3.max(c[j]);
            l3 = l3.min(c[j]);
        }
        let n3 = (h3 - l3) / period as f64;

        // Fractal dimension D
        let d = if n3 > 0.0 && n1 + n2 > 0.0 {
            ((n1 + n2).ln() - n3.ln()) / (2.0_f64).ln()
        } else {
            1.0
        };

        // Alpha from fractal dimension, clamped to [0.01, 1.0]
        let alpha = (-4.6 * (d - 1.0)).exp().clamp(0.01, 1.0);
        result[i] = result[i - 1] + alpha * (c[i] - result[i - 1]);
    }

    Ok(vec![IndicatorOutput {
        name: format!("FRAMA({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

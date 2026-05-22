use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn wma_series(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    if period == 0 || n < period {
        return result;
    }
    let total_weight = (period * (period + 1)) as f64 / 2.0;
    for i in (period - 1)..n {
        let mut sum = 0.0;
        for j in 0..period {
            sum += data[i - j] * (period - j) as f64;
        }
        result[i] = sum / total_weight;
    }
    result
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    if n < period {
        return Err(IndError::DataInsufficient(period));
    }

    let half_period = ((period as f64) / 2.0) as usize;
    let sqrt_period = ((period as f64).sqrt()).floor() as usize;

    let half = half_period.max(1);
    let sqrt = sqrt_period.max(1);

    let wma_half = wma_series(&c, half);
    let wma_full = wma_series(&c, period);

    // raw = 2 * WMA(n/2) - WMA(n)
    let mut raw = vec![f64::NAN; n];
    for i in 0..n {
        if wma_full[i].is_finite() && wma_half[i].is_finite() {
            raw[i] = 2.0 * wma_half[i] - wma_full[i];
        }
    }

    // HMA = WMA(raw, floor(sqrt(n)))
    let hma = wma_series(&raw, sqrt);

    Ok(vec![IndicatorOutput {
        name: format!("HMA({})", period),
        values: Column::F64(hma),
        style: OutputStyle::Line,
    }])
}

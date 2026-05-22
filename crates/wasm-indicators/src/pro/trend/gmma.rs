use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ema_series(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![f64::NAN; n];
    if n == 0 {
        return out;
    }
    out[0] = data[0];
    for i in 1..n {
        if data[i].is_finite() && out[i - 1].is_finite() {
            out[i] = alpha * data[i] + (1.0 - alpha) * out[i - 1];
        }
    }
    out
}

pub fn compute(
    df: &DataFrame,
    _params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();

    // GMMA fixed EMA periods: 6 short-term + 6 long-term
    let short_periods = [3usize, 5, 8, 10, 12, 15];
    let long_periods = [30usize, 35, 40, 45, 50, 60];

    let mut outputs = Vec::with_capacity(12);

    for &p in &short_periods {
        let ema = ema_series(&c, p);
        outputs.push(IndicatorOutput {
            name: format!("short_ema_{}", p),
            values: Column::F64(ema),
            style: OutputStyle::Line,
        });
    }

    for &p in &long_periods {
        let ema = ema_series(&c, p);
        outputs.push(IndicatorOutput {
            name: format!("long_ema_{}", p),
            values: Column::F64(ema),
            style: OutputStyle::Line,
        });
    }

    Ok(outputs)
}

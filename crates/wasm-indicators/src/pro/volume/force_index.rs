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

/// Force Index (Alexander Elder).
///
/// Measures the power behind price moves by multiplying the price change
/// by volume, then smoothing with an EMA.
///
/// Params:
/// - `period` (default 13, range 1-100): EMA smoothing period
pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(13.0) as usize;

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;

    let c = close.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = c.len();

    if n < 2 {
        return Err(IndError::DataInsufficient(2));
    }

    // raw_force[i] = (close[i] - close[i-1]) * volume[i]
    let mut raw_force = vec![f64::NAN; n];
    for i in 1..n {
        raw_force[i] = (c[i] - c[i - 1]) * v[i];
    }

    // Force Index = EMA(raw_force, period)
    let fi = ema(&raw_force, period);

    Ok(vec![
        IndicatorOutput {
            name: format!("FI({})", period),
            values: Column::F64(fi),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("FI_Raw({})", period),
            values: Column::F64(raw_force),
            style: OutputStyle::Histogram,
        },
    ])
}

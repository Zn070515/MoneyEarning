use std::collections::HashMap;
use wasm_core::{Column, DataFrame, IndError, IndicatorOutput, OutputStyle};

/// Compute Simple Moving Average for a given period.
fn sma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    let mut sum = 0.0;
    for i in 0..n {
        sum += data[i];
        if i >= period {
            sum -= data[i - period];
        }
        if i >= period - 1 {
            out[i] = sum / period as f64;
        }
    }
    out
}

/// Compute rolling standard deviation for a given period.
fn rolling_stdev(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut out = vec![f64::NAN; n];
    if n < period {
        return out;
    }
    for i in (period - 1)..n {
        let slice = &data[i + 1 - period..=i];
        let mean = slice.iter().sum::<f64>() / period as f64;
        let variance =
            slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        out[i] = variance.sqrt();
    }
    out
}

/// Volume Regime Detector.
///
/// Detects high/low volume regimes using statistical Z-score normalization.
/// Positive values indicate volume above the rolling mean (high volume regime),
/// negative values indicate volume below the rolling mean (low volume regime).
///
/// Params:
/// - `period` (default 20, range 10-100): lookback window for mean and stdev
/// - `threshold` (default 1.5, range 1-3): Z-score threshold for regime classification
///   (informational; the raw Z-score is output regardless)
pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let _threshold = params.get("threshold").copied().unwrap_or(1.5);

    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let v = volume.to_f64_vec();
    let n = v.len();

    if n < period {
        return Err(IndError::DataInsufficient(period));
    }

    let vol_sma = sma(&v, period);
    let vol_std = rolling_stdev(&v, period);

    let mut regime = vec![f64::NAN; n];
    for i in 0..n {
        if vol_sma[i].is_finite() && vol_std[i].is_finite() && vol_std[i] > 0.0 {
            regime[i] = (v[i] - vol_sma[i]) / vol_std[i];
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("VR({},{})", period, _threshold),
        values: Column::F64(regime),
        style: OutputStyle::Histogram,
    }])
}

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

/// Volatility Threshold Bands — dynamic support/resistance bands based on volatility regime.
///
/// Params: period (default 20), multiplier (default 2.0)
///
/// Formula:
///   center = SMA(close, period)
///   recent_vol = RMS of daily range over period = sqrt(average((high - low)^2))
///   upper = center + multiplier * recent_vol
///   lower = center - multiplier * recent_vol
///
/// Output: center line (Line) + Band { upper, lower }
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let multiplier = params.get("multiplier").copied().unwrap_or(2.0);

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    if n < period {
        return Err(IndError::DataInsufficient(period));
    }

    // Daily range and its squared value
    let mut range_sq = vec![0.0; n];
    for i in 0..n {
        let r = h[i] - l[i];
        range_sq[i] = r * r;
    }

    // center = SMA(close, period)
    let center = sma(&c, period);

    // recent_vol = sqrt(SMA(range_sq, period))
    let range_sq_sma = sma(&range_sq, period);

    let mut upper = vec![f64::NAN; n];
    let mut lower = vec![f64::NAN; n];

    for i in 0..n {
        if center[i].is_finite() && range_sq_sma[i].is_finite() {
            let recent_vol = range_sq_sma[i].sqrt();
            upper[i] = center[i] + multiplier * recent_vol;
            lower[i] = center[i] - multiplier * recent_vol;
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("THRESH_MID({})", period),
            values: Column::F64(center),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("THRESH_BAND({})", period),
            values: Column::F64(upper.clone()),
            style: OutputStyle::Band {
                upper: Column::F64(upper),
                lower: Column::F64(lower),
            },
        },
    ])
}

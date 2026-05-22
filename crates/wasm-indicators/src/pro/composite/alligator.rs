use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

/// Compute SMMA (Smoothed Moving Average, a.k.a. RMA / Wilder's MA).
/// Formula: first_val = SMA(vals[0..period]), then SMMA[i] = (SMMA[i-1]*(period-1) + vals[i]) / period
fn smma(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    if n < period {
        return result;
    }
    let first_sum: f64 = vals[0..period].iter().sum();
    result[period - 1] = first_sum / period as f64;
    for i in period..n {
        result[i] = (result[i - 1] * (period as f64 - 1.0) + vals[i]) / period as f64;
    }
    result
}

/// Shift a series forward (to the right) by `shift` bars: NaNs at the front.
fn shift_forward(vals: &[f64], shift: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    for i in shift..n {
        result[i] = vals[i - shift];
    }
    result
}

/// Bill Williams Alligator — three SMMA lines of median price, shifted forward.
///
/// Params:
///   jaw_period (default 13), jaw_shift (default 8)
///   teeth_period (default 8), teeth_shift (default 5)
///   lips_period (default 5), lips_shift (default 3)
///
/// Output: 3 lines — jaw, teeth, lips
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let jaw_period = params.get("jaw_period").copied().unwrap_or(13.0) as usize;
    let jaw_shift = params.get("jaw_shift").copied().unwrap_or(8.0) as usize;
    let teeth_period = params.get("teeth_period").copied().unwrap_or(8.0) as usize;
    let teeth_shift = params.get("teeth_shift").copied().unwrap_or(5.0) as usize;
    let lips_period = params.get("lips_period").copied().unwrap_or(5.0) as usize;
    let lips_shift = params.get("lips_shift").copied().unwrap_or(3.0) as usize;

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();

    let max_period = jaw_period.max(teeth_period).max(lips_period);
    if n < max_period {
        return Err(IndError::DataInsufficient(max_period));
    }

    // Median price = (high + low) / 2
    let mut median = vec![0.0; n];
    for i in 0..n {
        median[i] = (h[i] + l[i]) / 2.0;
    }

    let jaw_raw = smma(&median, jaw_period);
    let teeth_raw = smma(&median, teeth_period);
    let lips_raw = smma(&median, lips_period);

    let jaw = shift_forward(&jaw_raw, jaw_shift);
    let teeth = shift_forward(&teeth_raw, teeth_shift);
    let lips = shift_forward(&lips_raw, lips_shift);

    Ok(vec![
        IndicatorOutput {
            name: format!("ALLIGATOR_JAW({},{})", jaw_period, jaw_shift),
            values: Column::F64(jaw),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("ALLIGATOR_TEETH({},{})", teeth_period, teeth_shift),
            values: Column::F64(teeth),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("ALLIGATOR_LIPS({},{})", lips_period, lips_shift),
            values: Column::F64(lips),
            style: OutputStyle::Line,
        },
    ])
}

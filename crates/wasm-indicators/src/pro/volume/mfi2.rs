use std::collections::HashMap;
use wasm_core::{Column, DataFrame, IndError, IndicatorOutput, OutputStyle};

/// Market Facilitation Index (Bill Williams).
///
/// Measures the efficiency of price movement relative to volume.
/// Values are scaled by 1,000,000 for readability.
///
/// Classification zones:
/// - Green: MFI up, volume up (trending)
/// - Fade: MFI down, volume down (sleepy)
/// - Fake: MFI down, volume up (reversal/potential fake move)
/// - Squat: MFI up, volume down (absorption)
///
/// Params: none
pub fn compute(
    df: &DataFrame,
    _params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;

    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = h.len();

    if n < 1 {
        return Err(IndError::DataInsufficient(1));
    }

    let mut mfi2 = vec![f64::NAN; n];
    for i in 0..n {
        let range = h[i] - l[i];
        if range > 0.0 && v[i] > 0.0 {
            mfi2[i] = (range / v[i]) * 1_000_000.0;
        } else if range > 0.0 {
            // Volume is zero, avoid infinite but still compute range ratio
            mfi2[i] = range * 1_000_000.0;
        } else {
            mfi2[i] = 0.0;
        }
    }

    Ok(vec![IndicatorOutput {
        name: "MFI2".into(),
        values: Column::F64(mfi2),
        style: OutputStyle::Histogram,
    }])
}

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

/// Accelerator Oscillator (Bill Williams).
/// AC = AO - SMA(AO, 5), where AO = SMA(median,5) - SMA(median,34).
///
/// Params: none.
///
/// Output: Histogram.
pub fn compute(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();

    if n < 39 {
        // 34 for AO + 5 for SMA of AO
        return Err(IndError::DataInsufficient(39));
    }

    // Median price = (high + low) / 2
    let mut median = vec![0.0; n];
    for i in 0..n {
        median[i] = (h[i] + l[i]) / 2.0;
    }

    let sma5 = sma(&median, 5);
    let sma34 = sma(&median, 34);

    // Compute AO
    let mut ao = vec![f64::NAN; n];
    for i in 34..n {
        if sma5[i].is_finite() && sma34[i].is_finite() {
            ao[i] = sma5[i] - sma34[i];
        }
    }

    // SMA of AO with period 5
    let ao_sma5 = sma(&ao, 5);

    // AC = AO - SMA(AO, 5)
    let mut ac = vec![f64::NAN; n];
    for i in 39..n {
        if ao[i].is_finite() && ao_sma5[i].is_finite() {
            ac[i] = ao[i] - ao_sma5[i];
        }
    }

    Ok(vec![IndicatorOutput {
        name: "AC".to_string(),
        values: Column::F64(ac),
        style: OutputStyle::Histogram,
    }])
}

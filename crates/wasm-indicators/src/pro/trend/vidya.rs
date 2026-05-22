use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(9.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    if n < period + 1 {
        return Err(IndError::DataInsufficient(period + 1));
    }

    let mut result = vec![f64::NAN; n];

    // Seed VIDYA with SMA of first `period` bars
    let seed: f64 = c[0..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = seed;
    let mut vidya = seed;

    for i in period..n {
        // Calculate CMO (Chande Momentum Oscillator) over `period` bars
        let start = i - period;
        let mut sum_gains = 0.0;
        let mut sum_losses = 0.0;
        for j in (start + 1)..=i {
            let diff = c[j] - c[j - 1];
            if diff > 0.0 {
                sum_gains += diff;
            } else {
                sum_losses += -diff;
            }
        }

        let cmo = if sum_gains + sum_losses == 0.0 {
            0.0
        } else {
            ((sum_gains - sum_losses) / (sum_gains + sum_losses)) * 100.0
        };

        // Absolute CMO / 100 is used as the smoothing constant
        let cmo_abs = cmo.abs() / 100.0;
        vidya = vidya + cmo_abs * (c[i] - vidya);
        result[i] = vidya;
    }

    Ok(vec![IndicatorOutput {
        name: format!("VIDYA({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let phase = params.get("phase").copied().unwrap_or(0.0);

    if phase < -100.0 || phase > 100.0 {
        return Err(IndError::InvalidParams(
            "phase must be between -100 and 100".into(),
        ));
    }

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    if n < 2 {
        return Err(IndError::DataInsufficient(2));
    }

    // Phase ratio adjusts the alpha: 0.5 base, +/- phase/200 modulation
    let phase_ratio = 0.5 + phase / 200.0;
    let beta = 0.45 * (period as f64 - 1.0) / (0.45 * (period as f64 - 1.0) + 2.0);
    let alpha = beta.powf(period as f64) * phase_ratio;

    let mut result = vec![f64::NAN; n];

    // Initialize first two values
    result[0] = c[0];
    if n > 1 {
        result[1] = c[1];
    }

    // Simplified Jurik Moving Average
    for i in 2..n {
        let prev = result[i - 1];
        let prev_prev = result[i - 2];
        result[i] =
            prev + alpha * (c[i] - prev) + (1.0 - alpha) * (prev - prev_prev);
    }

    Ok(vec![IndicatorOutput {
        name: format!("JMA({},{})", period, phase as i64),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

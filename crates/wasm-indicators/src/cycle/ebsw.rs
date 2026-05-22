use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(40.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    if n < period { return Err(IndError::DataInsufficient(period)); }

    let mut smooth = c.clone();
    let alpha = 2.0 / (period as f64 + 1.0);
    for _ in 0..2 {
        let mut ema = vec![0.0; n];
        ema[0] = smooth[0];
        for i in 1..n { ema[i] = alpha * smooth[i] + (1.0 - alpha) * ema[i-1]; }
        smooth = ema;
    }

    let mut result = vec![f64::NAN; n];
    for i in 2..n {
        let q1 = 0.0962 * smooth[i] + 0.5769 * smooth[i-1] - 0.5769 * smooth[i-2] - 0.0962 * smooth[i-2].max(smooth[i-2]);
        result[i] = q1;
    }

    Ok(vec![IndicatorOutput {
        name: format!("EBSW({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

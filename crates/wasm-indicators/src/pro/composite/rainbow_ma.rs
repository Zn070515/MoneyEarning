use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ema(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    if n < period {
        return result;
    }
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Seed with SMA for the first value
    let first_sum: f64 = vals[0..period].iter().sum();
    result[period - 1] = first_sum / period as f64;

    for i in period..n {
        result[i] = (vals[i] - result[i - 1]) * multiplier + result[i - 1];
    }
    result
}

/// Rainbow Moving Averages — 10 EMAs with periods 2,4,6,...,20 forming a rainbow.
///
/// Params: none.
///
/// Output: 10 lines named "EMA(2)", "EMA(4)", ..., "EMA(20)".
pub fn compute(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    if n < 20 {
        return Err(IndError::DataInsufficient(20));
    }

    let periods: [usize; 10] = [2, 4, 6, 8, 10, 12, 14, 16, 18, 20];
    let mut outputs = Vec::with_capacity(10);

    for &p in &periods {
        let ema_line = ema(&c, p);
        outputs.push(IndicatorOutput {
            name: format!("EMA({})", p),
            values: Column::F64(ema_line),
            style: OutputStyle::Line,
        });
    }

    Ok(outputs)
}

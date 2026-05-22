use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(21.0) as usize;
    let annualize = params.get("annualize").copied().unwrap_or(252.0);
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();
    if n < period + 1 {
        return Err(IndError::DataInsufficient(period + 1));
    }

    const FOUR_LN2: f64 = 4.0 * std::f64::consts::LN_2;

    // Parkinson daily variance component
    // pk[i] = ln(H/L)^2 / (4 * ln(2))
    let mut pk_daily = vec![f64::NAN; n];
    for i in 0..n {
        if h[i] <= 0.0 || l[i] <= 0.0 {
            continue;
        }
        let hl_ln = (h[i] / l[i]).ln();
        pk_daily[i] = hl_ln * hl_ln / FOUR_LN2;
    }

    // Rolling SMA of daily components, then sqrt and annualize
    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        let slice = &pk_daily[i + 1 - period..=i];
        let valid: Vec<f64> = slice.iter().filter(|v| v.is_finite()).copied().collect();
        if valid.is_empty() {
            continue;
        }
        let avg = valid.iter().sum::<f64>() / valid.len() as f64;
        result[i] = avg.sqrt() * annualize.sqrt() * 100.0;
    }

    Ok(vec![IndicatorOutput {
        name: format!("PK_VOL({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

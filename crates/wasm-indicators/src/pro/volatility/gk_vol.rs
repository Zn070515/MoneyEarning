use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(21.0) as usize;
    let annualize = params.get("annualize").copied().unwrap_or(252.0);
    let open = df.column("open").ok_or(IndError::InvalidName)?;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let o = open.to_f64_vec();
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();
    if n < period + 1 {
        return Err(IndError::DataInsufficient(period + 1));
    }

    // Garman-Klass daily variance component
    // gk[i] = 0.5 * ln(H/L)^2 - 0.386 * ln(C/O)^2
    let mut gk_daily = vec![f64::NAN; n];
    for i in 0..n {
        if h[i] <= 0.0 || l[i] <= 0.0 || o[i] <= 0.0 || c[i] <= 0.0 {
            continue;
        }
        let hl_ln = (h[i] / l[i]).ln();
        let co_ln = (c[i] / o[i]).ln();
        gk_daily[i] = 0.5 * hl_ln * hl_ln - 0.386 * co_ln * co_ln;
        if gk_daily[i] < 0.0 {
            gk_daily[i] = 0.0;
        }
    }

    // Rolling SMA of daily components, then sqrt and annualize
    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        let slice = &gk_daily[i + 1 - period..=i];
        let valid: Vec<f64> = slice.iter().filter(|v| v.is_finite()).copied().collect();
        if valid.is_empty() {
            continue;
        }
        let avg = valid.iter().sum::<f64>() / valid.len() as f64;
        result[i] = avg.sqrt() * annualize.sqrt() * 100.0;
    }

    Ok(vec![IndicatorOutput {
        name: format!("GK_VOL({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

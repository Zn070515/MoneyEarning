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

    // Rogers-Satchell daily variance component
    // rs[i] = ln(H/C) * ln(H/O) + ln(L/C) * ln(L/O)
    let mut rs_daily = vec![f64::NAN; n];
    for i in 0..n {
        if h[i] <= 0.0 || l[i] <= 0.0 || o[i] <= 0.0 || c[i] <= 0.0 {
            continue;
        }
        let hc_ln = (h[i] / c[i]).ln();
        let ho_ln = (h[i] / o[i]).ln();
        let lc_ln = (l[i] / c[i]).ln();
        let lo_ln = (l[i] / o[i]).ln();
        rs_daily[i] = hc_ln * ho_ln + lc_ln * lo_ln;
        if rs_daily[i] < 0.0 {
            rs_daily[i] = 0.0;
        }
    }

    // Rolling SMA of daily components, then sqrt and annualize
    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        let slice = &rs_daily[i + 1 - period..=i];
        let valid: Vec<f64> = slice.iter().filter(|v| v.is_finite()).copied().collect();
        if valid.is_empty() {
            continue;
        }
        let avg = valid.iter().sum::<f64>() / valid.len() as f64;
        result[i] = avg.sqrt() * annualize.sqrt() * 100.0;
    }

    Ok(vec![IndicatorOutput {
        name: format!("RS_VOL({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

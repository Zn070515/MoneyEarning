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

    // Rogers-Satchell daily component for sigma_rs
    let mut rs_daily = vec![f64::NAN; n];
    for i in 0..n {
        if h[i] <= 0.0 || l[i] <= 0.0 || o[i] <= 0.0 || c[i] <= 0.0 {
            continue;
        }
        let hc_ln = (h[i] / c[i]).ln();
        let ho_ln = (h[i] / o[i]).ln();
        let lc_ln = (l[i] / c[i]).ln();
        let lo_ln = (l[i] / o[i]).ln();
        let v = hc_ln * ho_ln + lc_ln * lo_ln;
        rs_daily[i] = if v > 0.0 { v } else { 0.0 };
    }

    // Log returns for sigma_o and sigma_c
    let mut open_ret = vec![f64::NAN; n];
    let mut close_ret = vec![f64::NAN; n];
    for i in 1..n {
        if o[i] > 0.0 && o[i - 1] > 0.0 {
            open_ret[i] = (o[i] / o[i - 1]).ln();
        }
        if c[i] > 0.0 && c[i - 1] > 0.0 {
            close_ret[i] = (c[i] / c[i - 1]).ln();
        }
    }

    // k factor
    let k = 0.34 / (1.34 + (period + 1) as f64 / (period - 1) as f64);

    let mut result = vec![f64::NAN; n];
    for i in period..n {
        let slice_o = &open_ret[i + 1 - period..=i];
        let slice_c = &close_ret[i + 1 - period..=i];
        let slice_rs = &rs_daily[i + 1 - period..=i];

        // sigma_o: variance of log(open / prev_open)
        let vo: Vec<f64> = slice_o.iter().filter(|v| v.is_finite()).copied().collect();
        if vo.len() < 2 {
            continue;
        }
        let cnt_o = vo.len() as f64;
        let mean_o = vo.iter().sum::<f64>() / cnt_o;
        let sigma_o = vo.iter().map(|v| (v - mean_o) * (v - mean_o)).sum::<f64>() / (cnt_o - 1.0);

        // sigma_c: variance of log(close / prev_close)
        let vc: Vec<f64> = slice_c.iter().filter(|v| v.is_finite()).copied().collect();
        if vc.len() < 2 {
            continue;
        }
        let cnt_c = vc.len() as f64;
        let mean_c = vc.iter().sum::<f64>() / cnt_c;
        let sigma_c = vc.iter().map(|v| (v - mean_c) * (v - mean_c)).sum::<f64>() / (cnt_c - 1.0);

        // sigma_rs: average of RS daily components
        let vrs: Vec<f64> = slice_rs.iter().filter(|v| v.is_finite()).copied().collect();
        if vrs.is_empty() {
            continue;
        }
        let sigma_rs = vrs.iter().sum::<f64>() / vrs.len() as f64;

        // Yang-Zhang variance
        let yz_var = sigma_o + k * sigma_c + (1.0 - k) * sigma_rs;
        if yz_var > 0.0 {
            result[i] = yz_var.sqrt() * annualize.sqrt() * 100.0;
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("YZ_VOL({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

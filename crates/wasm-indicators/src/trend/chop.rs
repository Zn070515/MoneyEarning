use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = h.len();
    if n < period + 1 { return Err(IndError::DataInsufficient(period + 1)); }

    let mut tr_vals = vec![0.0; n];
    for i in 1..n {
        tr_vals[i] = (h[i] - l[i]).max((h[i] - c[i-1]).abs()).max((l[i] - c[i-1]).abs());
    }

    let mut result = vec![f64::NAN; n];
    let mut hh = vec![f64::NAN; n];
    let mut ll = vec![f64::NAN; n];

    for i in 0..n {
        let start = if i < period { 0 } else { i - period + 1 };
        let h_max = h[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let l_min = l[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
        hh[i] = h_max;
        ll[i] = l_min;
    }

    let mut sum_tr_all = 0.0;
    for i in 1..n {
        sum_tr_all += tr_vals[i];
        if i >= period { sum_tr_all -= tr_vals[i - period]; }
        if i >= period - 1 && sum_tr_all != 0.0 && (hh[i] - ll[i]) != 0.0 {
            let chop_val = 100.0 * (sum_tr_all / (hh[i] - ll[i])).log10() / (period as f64).log10();
            result[i] = chop_val.clamp(0.0, 100.0);
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("CHOP({})", period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

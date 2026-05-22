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

    let mut tr = vec![f64::NAN; n];
    let mut vm_plus = vec![f64::NAN; n];
    let mut vm_minus = vec![f64::NAN; n];
    for i in 1..n {
        tr[i] = (h[i] - l[i]).max((h[i] - c[i-1]).abs()).max((l[i] - c[i-1]).abs());
        vm_plus[i] = (h[i] - l[i-1]).abs();
        vm_minus[i] = (l[i] - h[i-1]).abs();
    }

    let mut sum_tr = 0.0;
    let mut sum_vp = 0.0;
    let mut sum_vm = 0.0;
    let mut vi_plus = vec![f64::NAN; n];
    let mut vi_minus = vec![f64::NAN; n];

    for i in 1..n {
        sum_tr += tr[i];
        sum_vp += vm_plus[i];
        sum_vm += vm_minus[i];
        if i >= period {
            sum_tr -= tr[i - period + 1];
            sum_vp -= vm_plus[i - period + 1];
            sum_vm -= vm_minus[i - period + 1];
            vi_plus[i] = if sum_tr != 0.0 { sum_vp / sum_tr } else { f64::NAN };
            vi_minus[i] = if sum_tr != 0.0 { sum_vm / sum_tr } else { f64::NAN };
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("VI+({})", period),
            values: Column::F64(vi_plus),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("VI-({})", period),
            values: Column::F64(vi_minus),
            style: OutputStyle::Line,
        },
    ])
}

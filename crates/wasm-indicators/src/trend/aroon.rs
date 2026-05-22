use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn llv_idx(vals: &[f64], period: usize) -> Vec<Option<usize>> {
    let n = vals.len();
    let mut result = vec![None; n];
    if n < period { return result; }
    for i in (period - 1)..n {
        let slice = &vals[i + 1 - period..=i];
        let mut min_idx = 0;
        let mut min_val = f64::MAX;
        for (j, &v) in slice.iter().enumerate() {
            if v < min_val { min_val = v; min_idx = j; }
        }
        result[i] = Some(period - 1 - min_idx);
    }
    result
}

fn hhv_idx(vals: &[f64], period: usize) -> Vec<Option<usize>> {
    let n = vals.len();
    let mut result = vec![None; n];
    if n < period { return result; }
    for i in (period - 1)..n {
        let slice = &vals[i + 1 - period..=i];
        let mut max_idx = 0;
        let mut max_val = f64::MIN;
        for (j, &v) in slice.iter().enumerate() {
            if v > max_val { max_val = v; max_idx = j; }
        }
        result[i] = Some(period - 1 - max_idx);
    }
    result
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();

    let h_idx = hhv_idx(&h, period);
    let l_idx = llv_idx(&l, period);

    let mut aroon_up = vec![f64::NAN; n];
    let mut aroon_down = vec![f64::NAN; n];
    for i in 0..n {
        if let Some(idx) = h_idx[i] { aroon_up[i] = 100.0 * (period as f64 - idx as f64 - 1.0) / (period as f64 - 1.0); }
        if let Some(idx) = l_idx[i] { aroon_down[i] = 100.0 * (period as f64 - idx as f64 - 1.0) / (period as f64 - 1.0); }
    }

    Ok(vec![
        IndicatorOutput { name: "AROON_UP".into(), values: Column::F64(aroon_up), style: OutputStyle::Line },
        IndicatorOutput { name: "AROON_DOWN".into(), values: Column::F64(aroon_down), style: OutputStyle::Line },
    ])
}

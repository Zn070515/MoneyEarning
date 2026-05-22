use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn llv(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    let mut min_val = f64::MAX;
    for i in 0..n {
        min_val = min_val.min(vals[i]);
        if i >= period && vals[i - period] == min_val {
            min_val = vals[i - period + 1..=i].iter().cloned().fold(f64::MAX, f64::min);
        }
        if i >= period - 1 { result[i] = min_val; }
    }
    result
}

fn hhv(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    let mut max_val = f64::MIN;
    for i in 0..n {
        max_val = max_val.max(vals[i]);
        if i >= period && vals[i - period] == max_val {
            max_val = vals[i - period + 1..=i].iter().cloned().fold(f64::MIN, f64::max);
        }
        if i >= period - 1 { result[i] = max_val; }
    }
    result
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(10.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();

    let mut median = vec![0.0; n];
    for i in 0..n { median[i] = (h[i] + l[i]) / 2.0; }

    let hhv_p = hhv(&median, period);
    let llv_p = llv(&median, period);

    let mut v1 = vec![f64::NAN; n];
    for i in period - 1..n {
        if (hhv_p[i] - llv_p[i]) > 0.0 {
            v1[i] = 0.66 * 2.0 * ((median[i] - llv_p[i]) / (hhv_p[i] - llv_p[i]) - 0.5);
        }
    }

    // Recursive smoothing with 0.5 cap
    let mut fish = vec![f64::NAN; n];
    let mut prev_v = 0.0;
    let mut prev_fish = 0.0;
    for i in period..n {
        if v1[i].is_finite() {
            let v = 0.66 * v1[i] + 0.34 * prev_v;
            let v_clamped = v.max(-0.999).min(0.999);
            let f = 0.5 * ((1.0 + v_clamped) / (1.0 - v_clamped)).ln() + 0.5 * prev_fish;
            prev_v = v;
            prev_fish = f;
            fish[i] = f;
        }
    }
    Ok(vec![IndicatorOutput {
        name: format!("FISHER({})", period), values: Column::F64(fish), style: OutputStyle::Line,
    }])
}

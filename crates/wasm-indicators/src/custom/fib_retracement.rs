use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();

    let period = params.get("period").copied().unwrap_or(100.0) as usize;
    let start = if n > period { n - period } else { 0 };

    let mut swing_h = f64::NEG_INFINITY;
    let mut swing_l = f64::INFINITY;
    for i in start..n {
        if h[i] > swing_h { swing_h = h[i]; }
        if l[i] < swing_l { swing_l = l[i]; }
    }

    let range = swing_h - swing_l;
    let fib_levels = [0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0, 1.272, 1.618];
    let mut outputs = Vec::new();
    let base = vec![f64::NAN; n];

    for &level in fib_levels.iter() {
        let mut vals = base.clone();
        for i in start..n { vals[i] = swing_h - range * level; }
        outputs.push(IndicatorOutput {
            name: format!("FIB_{:.3}", level),
            values: Column::F64(vals),
            style: OutputStyle::Line,
        });
    }
    Ok(outputs)
}

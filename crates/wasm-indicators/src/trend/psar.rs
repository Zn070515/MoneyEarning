use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let af_step = params.get("af_step").copied().unwrap_or(0.02);
    let af_max = params.get("af_max").copied().unwrap_or(0.2);
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let n = h.len();

    let mut result = vec![f64::NAN; n];
    let mut is_long = true;
    let mut af = af_step;
    let mut ep = h[0]; // extreme point
    let mut sar = l[0];

    for i in 1..n {
        result[i] = sar;

        if is_long {
            if h[i] > ep { ep = h[i]; af = (af + af_step).min(af_max); }
            sar = sar + af * (ep - sar);
            if sar > l[i - 1] { sar = l[i - 1]; }
            if sar > l[i] { sar = l[i]; }
            if l[i] < sar {
                is_long = false;
                sar = ep;
                ep = l[i];
                af = af_step;
            }
        } else {
            if l[i] < ep { ep = l[i]; af = (af + af_step).min(af_max); }
            sar = sar + af * (ep - sar);
            if sar < h[i - 1] { sar = h[i - 1]; }
            if sar < h[i] { sar = h[i]; }
            if h[i] > sar {
                is_long = true;
                sar = ep;
                ep = h[i];
                af = af_step;
            }
        }
    }

    Ok(vec![IndicatorOutput {
        name: "PSAR".into(), values: Column::F64(result), style: OutputStyle::Dots,
    }])
}

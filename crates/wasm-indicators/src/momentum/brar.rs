use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(26.0) as usize;
    let open = df.column("open").ok_or(IndError::InvalidName)?;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let o = open.to_f64_vec();
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    let mut br = vec![f64::NAN; n];
    let mut ar = vec![f64::NAN; n];

    for i in 0..n {
        let start = if i < period { 0 } else { i - period + 1 };
        let mut h_minus_c_sum = 0.0;
        let mut c_minus_l_sum = 0.0;
        let mut h_minus_o_sum = 0.0;
        let mut o_minus_l_sum = 0.0;
        for j in start..=i {
            if j > 0 {
                h_minus_c_sum += (h[j] - c[j - 1]).max(0.0);
                c_minus_l_sum += (c[j - 1] - l[j]).max(0.0);
            }
            h_minus_o_sum += (h[j] - o[j]).max(0.0);
            o_minus_l_sum += (o[j] - l[j]).max(0.0);
        }
        br[i] = if c_minus_l_sum != 0.0 { h_minus_c_sum / c_minus_l_sum * 100.0 } else { f64::NAN };
        ar[i] = if o_minus_l_sum != 0.0 { h_minus_o_sum / o_minus_l_sum * 100.0 } else { f64::NAN };
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("BR({})", period),
            values: Column::F64(br),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("AR({})", period),
            values: Column::F64(ar),
            style: OutputStyle::Line,
        },
    ])
}

use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let stddev = params.get("stddev").copied().unwrap_or(2.0);
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    let mut mid = vec![f64::NAN; n];
    let mut upper = vec![f64::NAN; n];
    let mut lower = vec![f64::NAN; n];

    for i in (period - 1)..n {
        let slice = &c[i + 1 - period..=i];
        let mean = slice.iter().sum::<f64>() / period as f64;
        let var = slice.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / period as f64;
        let sd = var.sqrt();
        mid[i] = mean;
        upper[i] = mean + stddev * sd;
        lower[i] = mean - stddev * sd;
    }

    Ok(vec![
        IndicatorOutput { name: "MID".into(), values: Column::F64(mid), style: OutputStyle::Line },
        IndicatorOutput {
            name: "UPPER".into(), values: Column::F64(upper.clone()), style: OutputStyle::Band {
                upper: Column::F64(upper), lower: Column::F64(lower),
            },
        },
    ])
}

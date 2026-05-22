use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(9.0) as usize;
    let offset = params.get("offset").copied().unwrap_or(0.85);
    let sigma = params.get("sigma").copied().unwrap_or(6.0);

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    if n < period {
        return Err(IndError::DataInsufficient(period));
    }

    // Precompute Gaussian distribution weights with offset
    let m = offset * (period - 1) as f64;
    let s = period as f64 / sigma;
    let weights: Vec<f64> = (0..period)
        .map(|i| {
            let x = i as f64;
            (-((x - m) * (x - m)) / (2.0 * s * s)).exp()
        })
        .collect();
    let weight_sum: f64 = weights.iter().sum();

    let mut result = vec![f64::NAN; n];
    for i in (period - 1)..n {
        let mut alma_val = 0.0;
        for j in 0..period {
            alma_val += weights[j] * c[i - (period - 1 - j)];
        }
        result[i] = alma_val / weight_sum;
    }

    Ok(vec![IndicatorOutput {
        name: format!("ALMA({},{},{})", period, offset, sigma),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

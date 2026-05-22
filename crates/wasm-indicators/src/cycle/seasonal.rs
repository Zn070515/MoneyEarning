use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let mode = params.get("mode").copied().unwrap_or(0.0) as usize;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();
    let mut result = vec![f64::NAN; n];

    let mut rets = vec![f64::NAN; n];
    for i in 1..n {
        if c[i-1] != 0.0 { rets[i] = (c[i] / c[i-1] - 1.0) * 100.0; }
    }

    let bucket_size = if mode == 0 { 5 } else { 21 };
    let mut bucket_sum = vec![0.0; bucket_size];
    let mut bucket_cnt = vec![0usize; bucket_size];

    for i in 1..n {
        if rets[i].is_finite() {
            let bucket = i % bucket_size;
            bucket_sum[bucket] += rets[i];
            bucket_cnt[bucket] += 1;
        }
    }

    for i in 0..n {
        let bucket = i % bucket_size;
        if bucket_cnt[bucket] > 0 {
            result[i] = bucket_sum[bucket] / bucket_cnt[bucket] as f64;
        }
    }

    Ok(vec![IndicatorOutput {
        name: if mode == 0 { "SEASONAL_W".into() } else { "SEASONAL_M".into() },
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

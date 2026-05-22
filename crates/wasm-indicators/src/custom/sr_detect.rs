use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let threshold = params.get("threshold").copied().unwrap_or(3.0);
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let volume = df.column("volume").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let v = volume.to_f64_vec();
    let n = h.len();

    let mut pivot_h = vec![f64::NAN; n];
    let mut pivot_l = vec![f64::NAN; n];
    let mut vol_zone = vec![f64::NAN; n];

    for i in period..n - period {
        let is_pivot_high = (0..period).all(|j| h[i] >= h[i - j - 1]) && (0..period).all(|j| h[i] >= h[i + j + 1]);
        let is_pivot_low = (0..period).all(|j| l[i] <= l[i - j - 1]) && (0..period).all(|j| l[i] <= l[i + j + 1]);
        if is_pivot_high { pivot_h[i] = h[i]; }
        if is_pivot_low { pivot_l[i] = l[i]; }

        let avg_vol: f64 = v[i - period..=i + period].iter().sum::<f64>() / (2 * period + 1) as f64;
        if v[i] > avg_vol * threshold {
            vol_zone[i] = (h[i] + l[i]) / 2.0;
        }
    }

    Ok(vec![
        IndicatorOutput { name: "SR_RESIST".into(), values: Column::F64(pivot_h), style: OutputStyle::Dots },
        IndicatorOutput { name: "SR_SUPPORT".into(), values: Column::F64(pivot_l), style: OutputStyle::Dots },
        IndicatorOutput { name: "SR_VOLZONE".into(), values: Column::F64(vol_zone), style: OutputStyle::Line },
    ])
}

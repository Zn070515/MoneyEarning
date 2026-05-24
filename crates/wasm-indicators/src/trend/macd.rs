use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn calc_ema(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    if n < period {
        return vec![f64::NAN; n];
    }
    let mut result = vec![f64::NAN; n];
    let multiplier = 2.0 / (period as f64 + 1.0);
    let sma_init: f64 = vals[0..period].iter().sum::<f64>() / period as f64;
    let mut ema_val = sma_init;
    for i in 0..n {
        if i < period - 1 {
            continue;
        } else if i == period - 1 {
            result[i] = ema_val;
        } else {
            ema_val = (vals[i] - ema_val) * multiplier + ema_val;
            result[i] = ema_val;
        }
    }
    result
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let fast = params.get("fast").copied().unwrap_or(12.0) as usize;
    let slow = params.get("slow").copied().unwrap_or(26.0) as usize;
    let signal = params.get("signal").copied().unwrap_or(9.0) as usize;
    if fast == 0 || slow == 0 || signal == 0 { return Err(IndError::DataInsufficient(slow)); }

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let vals = close.to_f64_vec();
    let n = vals.len();

    if n < slow + signal {
        return Err(IndError::DataInsufficient(slow + signal));
    }

    let ema_fast = calc_ema(&vals, fast);
    let ema_slow = calc_ema(&vals, slow);

    let mut dif = vec![f64::NAN; n];
    for i in 0..n {
        if ema_fast[i].is_finite() && ema_slow[i].is_finite() {
            dif[i] = ema_fast[i] - ema_slow[i];
        }
    }

    let dea = calc_ema(&dif.iter().skip(slow - 1).copied().collect::<Vec<_>>(), signal);
    let mut dea_full = vec![f64::NAN; n];
    let max_i = dea.len().min(n.saturating_sub(slow + signal - 2));
    for i in 0..max_i {
        dea_full[slow + signal - 2 + i] = dea[i];
    }

    let mut hist = vec![f64::NAN; n];
    for i in 0..n {
        if dif[i].is_finite() && dea_full[i].is_finite() {
            hist[i] = (dif[i] - dea_full[i]) * 2.0;
        }
    }

    Ok(vec![
        IndicatorOutput {
            name: "DIF".into(),
            values: Column::F64(dif),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: "DEA".into(),
            values: Column::F64(dea_full),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: "MACD".into(),
            values: Column::F64(hist),
            style: OutputStyle::Histogram,
        },
    ])
}

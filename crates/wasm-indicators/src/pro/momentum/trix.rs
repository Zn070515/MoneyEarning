use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ema(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![f64::NAN; n];
    if n == 0 {
        return out;
    }
    let mut seed_idx = None;
    for i in 0..n {
        if vals[i].is_finite() {
            out[i] = vals[i];
            seed_idx = Some(i);
            break;
        }
    }
    let start = match seed_idx {
        Some(idx) => idx,
        None => return out,
    };
    for i in start + 1..n {
        if vals[i].is_finite() {
            out[i] = alpha * vals[i] + (1.0 - alpha) * out[i - 1];
        } else {
            out[i] = out[i - 1];
        }
    }
    out
}

pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(15.0) as usize;
    let signal = params.get("signal").copied().unwrap_or(9.0) as usize;

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    // Triple EMA requires roughly 3*period data points to stabilize,
    // plus 1 for ROC, plus signal period
    let min_required = period * 3 + signal + 1;
    if n < min_required {
        return Err(IndError::DataInsufficient(min_required));
    }

    // Triple exponential smoothing
    let ema1 = ema(&c, period);
    let ema2 = ema(&ema1, period);
    let ema3 = ema(&ema2, period);

    // TRIX = ROC(ema3, 1) as percent
    let mut trix = vec![f64::NAN; n];
    for i in 1..n {
        if ema3[i].is_finite() && ema3[i - 1].is_finite() && ema3[i - 1] != 0.0 {
            trix[i] = (ema3[i] - ema3[i - 1]) / ema3[i - 1] * 100.0;
        }
    }

    // Signal line = EMA(trix, signal)
    let signal_line = ema(&trix, signal);

    Ok(vec![
        IndicatorOutput {
            name: format!("TRIX({})", period),
            values: Column::F64(trix),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("TRIX_SIG({})", signal),
            values: Column::F64(signal_line),
            style: OutputStyle::Line,
        },
    ])
}

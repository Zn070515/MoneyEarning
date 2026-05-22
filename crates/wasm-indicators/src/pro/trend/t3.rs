use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn ema_series(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut out = vec![f64::NAN; n];
    if n == 0 {
        return out;
    }
    out[0] = data[0];
    for i in 1..n {
        if data[i].is_finite() && out[i - 1].is_finite() {
            out[i] = alpha * data[i] + (1.0 - alpha) * out[i - 1];
        } else if data[i].is_finite() {
            out[i] = data[i];
        }
    }
    out
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(10.0) as usize;
    let v = params.get("v").copied().unwrap_or(0.7);

    if v < 0.1 || v > 1.0 {
        return Err(IndError::InvalidParams(
            "v must be between 0.1 and 1.0".into(),
        ));
    }

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    if n == 0 {
        return Err(IndError::DataInsufficient(1));
    }

    // Six iterations of EMA smoothing
    let e1 = ema_series(&c, period);
    let e2 = ema_series(&e1, period);
    let e3 = ema_series(&e2, period);
    let e4 = ema_series(&e3, period);
    let e5 = ema_series(&e4, period);
    let e6 = ema_series(&e5, period);

    // T3 coefficients (Tilson's generalized DEMA/TEMA)
    let v2 = v * v;
    let v3 = v2 * v;
    let c1 = -v3;
    let c2 = 3.0 * v2 + 3.0 * v3;
    let c3 = -6.0 * v2 - 3.0 * v - 3.0 * v3;
    let c4 = 1.0 + 3.0 * v + v3 + 3.0 * v2;

    let mut result = vec![f64::NAN; n];
    for i in 0..n {
        if e3[i].is_finite() && e4[i].is_finite() && e5[i].is_finite() && e6[i].is_finite() {
            result[i] = c1 * e6[i] + c2 * e5[i] + c3 * e4[i] + c4 * e3[i];
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("T3({},{})", period, v),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

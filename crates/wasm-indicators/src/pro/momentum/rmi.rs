use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let momentum_period = params.get("momentum_period").copied().unwrap_or(5.0) as usize;

    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let n = c.len();

    let min_required = period + momentum_period + 1;
    if n < min_required {
        return Err(IndError::DataInsufficient(min_required));
    }

    // Momentum: close[i] - close[i - momentum_period]
    let mut mom = vec![f64::NAN; n];
    for i in momentum_period..n {
        mom[i] = c[i] - c[i - momentum_period];
    }

    // momentum-based RSI: use same Wilder's smoothing on the momentum series
    let mut gains = Vec::with_capacity(n);
    let mut losses = Vec::with_capacity(n);
    for i in momentum_period + 1..n {
        if mom[i].is_finite() && mom[i - 1].is_finite() {
            let change = mom[i] - mom[i - 1];
            if change >= 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        } else {
            gains.push(0.0);
            losses.push(0.0);
        }
    }

    let changes_len = gains.len();
    if changes_len < period {
        return Err(IndError::DataInsufficient(period + momentum_period + 1));
    }

    // Seed with SMA
    let avg_gain_init = gains[0..period].iter().sum::<f64>() / period as f64;
    let avg_loss_init = losses[0..period].iter().sum::<f64>() / period as f64;

    let mut avg_gain = avg_gain_init;
    let mut avg_loss = avg_loss_init;

    // The RMI values align to the original close array
    // gains[0] is the change from mom[momentum_period+1] to mom[momentum_period] which maps to index momentum_period+1 in original
    // After period changes, the first RMI maps to index (momentum_period+1+period) in original
    let mut result = vec![f64::NAN; n];

    for i in period..changes_len {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;
        let rs = if avg_loss == 0.0 {
            100.0
        } else {
            avg_gain / avg_loss
        };
        // This RMI maps to close index: momentum_period + 1 + i + 1 = momentum_period + i + 2
        let idx = momentum_period + i + 2;
        if idx < n {
            result[idx] = 100.0 - (100.0 / (1.0 + rs));
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("RMI({},{})", period, momentum_period),
        values: Column::F64(result),
        style: OutputStyle::Line,
    }])
}

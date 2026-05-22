use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute(
    df: &DataFrame,
    params: &HashMap<String, f64>,
) -> Result<Vec<IndicatorOutput>, IndError> {
    let short = params.get("short").copied().unwrap_or(7.0) as usize;
    let mid = params.get("mid").copied().unwrap_or(14.0) as usize;
    let long = params.get("long").copied().unwrap_or(28.0) as usize;

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    if n < long + 1 {
        return Err(IndError::DataInsufficient(long + 1));
    }

    // Buying pressure (BP) and True Range (TR) for each bar
    let mut bp = vec![0.0; n];
    let mut tr = vec![0.0; n];
    for i in 0..n {
        let prev_close = if i > 0 { c[i - 1] } else { c[i] };
        let true_low = l[i].min(prev_close);
        let true_high = h[i].max(prev_close);
        bp[i] = c[i] - true_low;
        tr[i] = true_high - true_low;
    }

    // Sum BP and TR for a given period using sliding window
    let avg_period = |period: usize| -> Vec<f64> {
        let mut result = vec![f64::NAN; n];
        let mut bp_sum = 0.0;
        let mut tr_sum = 0.0;
        for i in 0..n {
            bp_sum += bp[i];
            tr_sum += tr[i];
            if i >= period - 1 {
                if tr_sum > 0.0 {
                    result[i] = bp_sum / tr_sum;
                }
                bp_sum -= bp[i + 1 - period];
                tr_sum -= tr[i + 1 - period];
            }
        }
        result
    };

    let avg_short = avg_period(short);
    let avg_mid = avg_period(mid);
    let avg_long = avg_period(long);

    // Ultimate Oscillator = 100 * (4*avg_short + 2*avg_mid + 1*avg_long) / 7
    let mut uo = vec![f64::NAN; n];
    for i in 0..n {
        if avg_short[i].is_finite() && avg_mid[i].is_finite() && avg_long[i].is_finite() {
            uo[i] = 100.0 * (4.0 * avg_short[i] + 2.0 * avg_mid[i] + avg_long[i]) / 7.0;
        }
    }

    Ok(vec![IndicatorOutput {
        name: format!("UO({},{},{})", short, mid, long),
        values: Column::F64(uo),
        style: OutputStyle::Line,
    }])
}

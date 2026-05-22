use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn calc_atr(h: &[f64], l: &[f64], c: &[f64], period: usize) -> Vec<f64> {
    let n = c.len();
    let mut tr = vec![0.0; n];
    for i in 1..n {
        let a = h[i] - l[i];
        let b = (h[i] - c[i - 1]).abs();
        let d = (l[i] - c[i - 1]).abs();
        tr[i] = a.max(b).max(d);
    }
    tr[0] = h[0] - l[0];
    let mut result = vec![f64::NAN; n];
    let mut atr = tr[0..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = atr;
    for i in period..n {
        atr = (atr * (period - 1) as f64 + tr[i]) / period as f64;
        result[i] = atr;
    }
    result
}

fn wilder_smooth(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    let mut started = false;
    let mut sm_val = 0.0;
    let mut sum = 0.0;
    let mut count = 0;
    for i in 0..n {
        if vals[i].is_finite() {
            if !started {
                sum += vals[i]; count += 1;
                if count >= period {
                    sm_val = sum / period as f64;
                    started = true;
                    result[i] = sm_val;
                }
            } else {
                sm_val = (sm_val * (period - 1) as f64 + vals[i]) / period as f64;
                result[i] = sm_val;
            }
        }
    }
    result
}

pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    let atr = calc_atr(&h, &l, &c, period);

    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];
    for i in 1..n {
        let up_move = h[i] - h[i - 1];
        let down_move = l[i - 1] - l[i];
        plus_dm[i] = if up_move > down_move && up_move > 0.0 { up_move } else { 0.0 };
        minus_dm[i] = if down_move > up_move && down_move > 0.0 { down_move } else { 0.0 };
    }

    let plus_di_smooth = wilder_smooth(&plus_dm, period);
    let minus_di_smooth = wilder_smooth(&minus_dm, period);

    let mut plus_di = vec![f64::NAN; n];
    let mut minus_di = vec![f64::NAN; n];
    for i in 0..n {
        if atr[i].is_finite() && plus_di_smooth[i].is_finite() {
            plus_di[i] = 100.0 * plus_di_smooth[i] / atr[i];
        }
        if atr[i].is_finite() && minus_di_smooth[i].is_finite() {
            minus_di[i] = 100.0 * minus_di_smooth[i] / atr[i];
        }
    }

    let mut dx = vec![f64::NAN; n];
    for i in 0..n {
        if plus_di[i].is_finite() && minus_di[i].is_finite() && (plus_di[i] + minus_di[i]) > 0.0 {
            dx[i] = 100.0 * (plus_di[i] - minus_di[i]).abs() / (plus_di[i] + minus_di[i]);
        }
    }

    let adx = wilder_smooth(&dx, period);

    Ok(vec![
        IndicatorOutput { name: "ADX".into(), values: Column::F64(adx), style: OutputStyle::Line },
        IndicatorOutput { name: "PDI".into(), values: Column::F64(plus_di), style: OutputStyle::Line },
        IndicatorOutput { name: "MDI".into(), values: Column::F64(minus_di), style: OutputStyle::Line },
    ])
}

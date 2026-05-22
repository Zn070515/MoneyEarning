use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn hhv(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    if n == 0 || period == 0 {
        return result;
    }
    let p = period.min(n);
    let mut max_val = f64::MIN;
    for i in 0..n {
        max_val = max_val.max(vals[i]);
        if i >= p && (vals[i - p] - max_val).abs() < 1e-12 {
            max_val = vals[i - p + 1..=i].iter().cloned().fold(f64::MIN, f64::max);
        }
        if i >= p - 1 {
            result[i] = max_val;
        }
    }
    result
}

fn llv(vals: &[f64], period: usize) -> Vec<f64> {
    let n = vals.len();
    let mut result = vec![f64::NAN; n];
    if n == 0 || period == 0 {
        return result;
    }
    let p = period.min(n);
    let mut min_val = f64::MAX;
    for i in 0..n {
        min_val = min_val.min(vals[i]);
        if i >= p && (vals[i - p] - min_val).abs() < 1e-12 {
            min_val = vals[i - p + 1..=i].iter().cloned().fold(f64::MAX, f64::min);
        }
        if i >= p - 1 {
            result[i] = min_val;
        }
    }
    result
}

/// Ichimoku Kinko Hyo (一目均衡表)
///
/// Params: tenkan (default 9), kijun (default 26), senkou_b (default 52)
///
/// Output: 5 lines — tenkan_sen, kijun_sen, senkou_span_a, senkou_span_b, chikou_span
pub fn compute(df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let tenkan = params.get("tenkan").copied().unwrap_or(9.0) as usize;
    let kijun = params.get("kijun").copied().unwrap_or(26.0) as usize;
    let senkou_b_period = params.get("senkou_b").copied().unwrap_or(52.0) as usize;

    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();
    let c = close.to_f64_vec();
    let n = c.len();

    if n < 2 {
        return Err(IndError::DataInsufficient(2));
    }

    // Compute highest high and lowest low for tenkan and kijun
    let hh_tenkan = hhv(&h, tenkan);
    let ll_tenkan = llv(&l, tenkan);
    let hh_kijun = hhv(&h, kijun);
    let ll_kijun = llv(&l, kijun);
    let hh_senkou_b = hhv(&h, senkou_b_period);
    let ll_senkou_b = llv(&l, senkou_b_period);

    // tenkan_sen = (highest(high, tenkan) + lowest(low, tenkan)) / 2
    let mut tenkan_sen = vec![f64::NAN; n];
    let mut kijun_sen = vec![f64::NAN; n];
    let mut senkou_a_raw = vec![f64::NAN; n];
    let mut senkou_b_raw = vec![f64::NAN; n];

    for i in 0..n {
        if hh_tenkan[i].is_finite() && ll_tenkan[i].is_finite() {
            tenkan_sen[i] = (hh_tenkan[i] + ll_tenkan[i]) / 2.0;
        }
        if hh_kijun[i].is_finite() && ll_kijun[i].is_finite() {
            kijun_sen[i] = (hh_kijun[i] + ll_kijun[i]) / 2.0;
        }
        if tenkan_sen[i].is_finite() && kijun_sen[i].is_finite() {
            senkou_a_raw[i] = (tenkan_sen[i] + kijun_sen[i]) / 2.0;
        }
        if hh_senkou_b[i].is_finite() && ll_senkou_b[i].is_finite() {
            senkou_b_raw[i] = (hh_senkou_b[i] + ll_senkou_b[i]) / 2.0;
        }
    }

    // Shift senkou_a and senkou_b forward by kijun periods:
    // Plot value computed at index i at index i+kijun.
    // In the output array: result[i] = computed[i-kijun] for i >= kijun, NaN otherwise.
    let mut senkou_a = vec![f64::NAN; n];
    let mut senkou_b = vec![f64::NAN; n];
    for i in kijun..n {
        senkou_a[i] = senkou_a_raw[i - kijun];
        senkou_b[i] = senkou_b_raw[i - kijun];
    }

    // chikou_span: close shifted backward by kijun periods.
    // Value at position i is close[i-kijun].
    let mut chikou_span = vec![f64::NAN; n];
    for i in kijun..n {
        chikou_span[i] = c[i - kijun];
    }

    Ok(vec![
        IndicatorOutput {
            name: format!("ICHIMOKU_TENKAN({})", tenkan),
            values: Column::F64(tenkan_sen),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("ICHIMOKU_KIJUN({})", kijun),
            values: Column::F64(kijun_sen),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("ICHIMOKU_SENKOU_A({},{})", tenkan, kijun),
            values: Column::F64(senkou_a.clone()),
            style: OutputStyle::Band {
                upper: Column::F64(senkou_a),
                lower: Column::F64(senkou_b),
            },
        },
        IndicatorOutput {
            name: format!("ICHIMOKU_SENKOU_B({})", senkou_b_period),
            values: Column::F64(vec![f64::NAN; n]),
            style: OutputStyle::Line,
        },
        IndicatorOutput {
            name: format!("ICHIMOKU_CHIKOU({})", kijun),
            values: Column::F64(chikou_span),
            style: OutputStyle::Line,
        },
    ])
}

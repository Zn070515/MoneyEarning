use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

pub fn compute_range_stats(df: &DataFrame, start_idx: usize, end_idx: usize) -> Result<Vec<IndicatorOutput>, IndError> {
    let close = df.column("close").ok_or(IndError::InvalidName)?;
    let high = df.column("high").ok_or(IndError::InvalidName)?;
    let low = df.column("low").ok_or(IndError::InvalidName)?;
    let c = close.to_f64_vec();
    let h = high.to_f64_vec();
    let l = low.to_f64_vec();

    if c.is_empty() { return Err(IndError::DataInsufficient(1)); }
    let end = end_idx.min(c.len() - 1);
    if start_idx > end { return Err(IndError::DataInsufficient(2)); }

    let pct_change = if c[start_idx] != 0.0 { (c[end] / c[start_idx] - 1.0) * 100.0 } else { 0.0 };
    let mut max_dd = 0.0;
    let mut peak = c[start_idx];
    for i in start_idx..=end {
        if c[i] > peak { peak = c[i]; }
        if peak != 0.0 {
            let dd = (peak - c[i]) / peak * 100.0;
            if dd > max_dd { max_dd = dd; }
        }
    }

    let mut atr_sum = 0.0;
    let mut count = 0;
    for i in start_idx.max(1)..=end {
        let tr = (h[i] - l[i]).max((h[i] - c[i-1]).abs()).max((l[i] - c[i-1]).abs());
        atr_sum += tr;
        count += 1;
    }
    let avg_atr = if count > 0 { atr_sum / count as f64 } else { 0.0 };

    let n = c.len();
    let mut pct_arr = vec![f64::NAN; n];
    let mut dd_arr = vec![f64::NAN; n];
    let mut atr_arr = vec![f64::NAN; n];
    pct_arr[end] = pct_change;
    dd_arr[end] = max_dd;
    atr_arr[end] = avg_atr;

    Ok(vec![
        IndicatorOutput { name: "RANGE_CHG".into(), values: Column::F64(pct_arr), style: OutputStyle::Line },
        IndicatorOutput { name: "RANGE_MAXDD".into(), values: Column::F64(dd_arr), style: OutputStyle::Line },
        IndicatorOutput { name: "RANGE_ATR".into(), values: Column::F64(atr_arr), style: OutputStyle::Line },
    ])
}

pub fn compute(_df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    Err(IndError::InvalidParams("区间统计需要指定起止索引".into()))
}

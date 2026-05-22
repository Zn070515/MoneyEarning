use std::collections::HashMap;
use wasm_core::{DataFrame, IndError, IndicatorOutput, Column, OutputStyle};

fn get_ohlc(df: &DataFrame) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let o = df.column("open").map(|c| c.to_f64_vec()).unwrap_or_default();
    let h = df.column("high").map(|c| c.to_f64_vec()).unwrap_or_default();
    let l = df.column("low").map(|c| c.to_f64_vec()).unwrap_or_default();
    let c = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    (o, h, l, c)
}

fn body(open: f64, close: f64) -> f64 { (close - open).abs() }
fn upper_shadow(open: f64, close: f64, high: f64) -> f64 { high - open.max(close) }
fn lower_shadow(open: f64, close: f64, low: f64) -> f64 { open.min(close) - low }
fn is_bullish(open: f64, close: f64) -> bool { close > open }
fn body_pct(open: f64, close: f64) -> f64 { if open != 0.0 { body(open, close) / open * 100.0 } else { 0.0 } }

// 1. Doji — body is very small
pub fn cdl_doji(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, h, l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let b = body(o[i], c[i]);
        let range = h[i] - l[i];
        if range > 0.0 && b / range < 0.1 { result[i] = 1.0; }
    }
    Ok(vec![IndicatorOutput { name: "DOJI".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 2. Hammer — small body at top, long lower shadow
pub fn cdl_hammer(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, h, l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 1..n {
        let b = body(o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let us = upper_shadow(o[i], c[i], h[i]);
        let range = h[i] - l[i];
        let downtrend = c[i - 1] > c[i];
        if downtrend && b > 0.0 && ls > b * 2.0 && us < b * 0.3 && range > 0.0 { result[i] = 1.0; }
    }
    Ok(vec![IndicatorOutput { name: "HAMMER".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 3. Inverted Hammer — small body at bottom, long upper shadow, after downtrend
pub fn cdl_inverted_hammer(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, h, l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 1..n {
        let b = body(o[i], c[i]);
        let us = upper_shadow(o[i], c[i], h[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let range = h[i] - l[i];
        let downtrend = c[i - 1] > c[i];
        if downtrend && b > 0.0 && us > b * 2.0 && ls < b * 0.3 && range > 0.0 { result[i] = 1.0; }
    }
    Ok(vec![IndicatorOutput { name: "INV_HAMMER".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 4. Hanging Man — hammer at top of uptrend (warning)
pub fn cdl_hanging_man(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, h, l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 1..n {
        let b = body(o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let us = upper_shadow(o[i], c[i], h[i]);
        let range = h[i] - l[i];
        let uptrend = c[i - 1] < c[i];
        if uptrend && b > 0.0 && ls > b * 2.0 && us < b * 0.3 && range > 0.0 { result[i] = -1.0; }
    }
    Ok(vec![IndicatorOutput { name: "HANGING_MAN".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 5. Shooting Star — inverted hammer at top of uptrend (warning)
pub fn cdl_shooting_star(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, h, l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 1..n {
        let b = body(o[i], c[i]);
        let us = upper_shadow(o[i], c[i], h[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let range = h[i] - l[i];
        let uptrend = c[i - 1] < c[i];
        if uptrend && b > 0.0 && us > b * 2.0 && ls < b * 0.3 && range > 0.0 { result[i] = -1.0; }
    }
    Ok(vec![IndicatorOutput { name: "SHOOTING_STAR".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 6. Engulfing — body engulfs previous body
pub fn cdl_engulfing(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, _h, _l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 1..n {
        let prev_bull = is_bullish(o[i-1], c[i-1]);
        let curr_bull = is_bullish(o[i], c[i]);
        let curr_body = body(o[i], c[i]);
        let prev_body = body(o[i-1], c[i-1]);
        if curr_bull != prev_bull && curr_body > prev_body {
            result[i] = if curr_bull { 1.0 } else { -1.0 };
        }
    }
    Ok(vec![IndicatorOutput { name: "ENGULFING".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 7. Harami — body contained within previous body
pub fn cdl_harami(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, _h, _l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 1..n {
        let prev_bull = is_bullish(o[i-1], c[i-1]);
        let curr_bull = is_bullish(o[i], c[i]);
        let curr_body = body(o[i], c[i]);
        let prev_body = body(o[i-1], c[i-1]);
        if curr_bull != prev_bull && curr_body < prev_body * 0.7 {
            result[i] = if curr_bull { 1.0 } else { -1.0 };
        }
    }
    Ok(vec![IndicatorOutput { name: "HARAMI".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 8. Piercing — bull opens below prev low, closes above 50% of prev body
pub fn cdl_piercing(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, _h, l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 1..n {
        let _prev_body = body(o[i-1], c[i-1]);
        let prev_mid = (o[i-1] + c[i-1]) / 2.0;
        if !is_bullish(o[i-1], c[i-1]) && is_bullish(o[i], c[i])
            && o[i] < l[i-1] && c[i] > prev_mid && c[i] < o[i-1] {
            result[i] = 1.0;
        }
    }
    Ok(vec![IndicatorOutput { name: "PIERCING".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 9. Dark Cloud Cover — bear opens above prev high, closes below 50% of prev body
pub fn cdl_dark_cloud_cover(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, h, _l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 1..n {
        let prev_mid = (o[i-1] + c[i-1]) / 2.0;
        if is_bullish(o[i-1], c[i-1]) && !is_bullish(o[i], c[i])
            && o[i] > h[i-1] && c[i] < prev_mid && c[i] > o[i-1] {
            result[i] = -1.0;
        }
    }
    Ok(vec![IndicatorOutput { name: "DARK_CLOUD".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 10. Morning Star — 3-candle: bear + small body + bull
pub fn cdl_morning_star(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, _h, _l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 2..n {
        let bearish1 = !is_bullish(o[i-2], c[i-2]) && body(o[i-2], c[i-2]) > 0.0;
        let small2 = body_pct(o[i-1], c[i-1]) < 1.5;
        let bullish3 = is_bullish(o[i], c[i]) && c[i] > (o[i-2] + c[i-2]) / 2.0;
        if bearish1 && small2 && bullish3 { result[i] = 1.0; }
    }
    Ok(vec![IndicatorOutput { name: "MORNING_STAR".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 11. Evening Star — 3-candle: bull + small body + bear
pub fn cdl_evening_star(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, _h, _l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 2..n {
        let bullish1 = is_bullish(o[i-2], c[i-2]) && body(o[i-2], c[i-2]) > 0.0;
        let small2 = body_pct(o[i-1], c[i-1]) < 1.5;
        let bearish3 = !is_bullish(o[i], c[i]) && c[i] < (o[i-2] + c[i-2]) / 2.0;
        if bullish1 && small2 && bearish3 { result[i] = -1.0; }
    }
    Ok(vec![IndicatorOutput { name: "EVENING_STAR".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 12. Three White Soldiers — 3 consecutive bullish, each closing higher
pub fn cdl_three_white_soldiers(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, _h, _l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 2..n {
        let all_bull = is_bullish(o[i-2], c[i-2]) && is_bullish(o[i-1], c[i-1]) && is_bullish(o[i], c[i]);
        let rising = c[i] > c[i-1] && c[i-1] > c[i-2];
        if all_bull && rising { result[i] = 1.0; }
    }
    Ok(vec![IndicatorOutput { name: "3WHITE_SOLDIERS".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 13. Three Black Crows — 3 consecutive bearish, each closing lower
pub fn cdl_three_black_crows(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, _h, _l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 2..n {
        let all_bear = !is_bullish(o[i-2], c[i-2]) && !is_bullish(o[i-1], c[i-1]) && !is_bullish(o[i], c[i]);
        let falling = c[i] < c[i-1] && c[i-1] < c[i-2];
        if all_bear && falling { result[i] = -1.0; }
    }
    Ok(vec![IndicatorOutput { name: "3BLACK_CROWS".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 14. Marubozu — full body, tiny shadows (strong conviction)
pub fn cdl_marubozu(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (o, h, l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let b = body(o[i], c[i]);
        let us = upper_shadow(o[i], c[i], h[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        if b > 0.0 && us < b * 0.05 && ls < b * 0.05 {
            result[i] = if is_bullish(o[i], c[i]) { 1.0 } else { -1.0 };
        }
    }
    Ok(vec![IndicatorOutput { name: "MARUBOZU".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

// 15. Inside Bar — entire candle contained within previous candle's range
pub fn cdl_inside(df: &DataFrame, _params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    let mut result = vec![0.0; n];
    for i in 1..n {
        if h[i] < h[i-1] && l[i] > l[i-1] { result[i] = 1.0; }
    }
    Ok(vec![IndicatorOutput { name: "INSIDE".into(), values: Column::F64(result), style: OutputStyle::Histogram }])
}

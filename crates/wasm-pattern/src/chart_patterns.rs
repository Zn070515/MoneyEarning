use wasm_core::DataFrame;
use crate::PatternMatch;

pub fn list_chart() -> Vec<crate::PatternMeta> {
    vec![
        crate::PatternMeta { name: "double_top".into(), name_cn: "双顶".into(), category: "形态".into() },
        crate::PatternMeta { name: "double_bottom".into(), name_cn: "双底".into(), category: "形态".into() },
        crate::PatternMeta { name: "head_shoulders".into(), name_cn: "头肩顶".into(), category: "形态".into() },
        crate::PatternMeta { name: "inv_head_shoulders".into(), name_cn: "头肩底".into(), category: "形态".into() },
        crate::PatternMeta { name: "asc_triangle".into(), name_cn: "上升三角形".into(), category: "形态".into() },
        crate::PatternMeta { name: "desc_triangle".into(), name_cn: "下降三角形".into(), category: "形态".into() },
        crate::PatternMeta { name: "sym_triangle".into(), name_cn: "对称三角形".into(), category: "形态".into() },
        crate::PatternMeta { name: "flag".into(), name_cn: "旗形".into(), category: "形态".into() },
        crate::PatternMeta { name: "wedge".into(), name_cn: "楔形".into(), category: "形态".into() },
        crate::PatternMeta { name: "rectangle".into(), name_cn: "矩形整理".into(), category: "形态".into() },
        crate::PatternMeta { name: "cup_handle".into(), name_cn: "杯柄形态".into(), category: "形态".into() },
        crate::PatternMeta { name: "rising_three".into(), name_cn: "上升三步曲".into(), category: "形态".into() },
        crate::PatternMeta { name: "falling_three".into(), name_cn: "下降三步曲".into(), category: "形态".into() },
        crate::PatternMeta { name: "lsd_top".into(), name_cn: "扩散顶".into(), category: "形态".into() },
        crate::PatternMeta { name: "diamond".into(), name_cn: "菱形".into(), category: "形态".into() },
        crate::PatternMeta { name: "rounding_bottom".into(), name_cn: "圆弧底".into(), category: "形态".into() },
        crate::PatternMeta { name: "mat_hold".into(), name_cn: "垫形持有".into(), category: "形态".into() },
    ]
}

pub fn recognize_chart(df: &DataFrame, pattern: &str) -> Vec<PatternMatch> {
    match pattern {
        "double_top" => find_double_top(df),
        "double_bottom" => find_double_bottom(df),
        "head_shoulders" => find_head_shoulders(df),
        "inv_head_shoulders" => find_inv_head_shoulders(df),
        "asc_triangle" => find_triangle(df, true),
        "desc_triangle" => find_triangle(df, false),
        "sym_triangle" => find_sym_triangle(df),
        "flag" => find_flag(df, false),
        "wedge" => find_flag(df, true),
        "rectangle" => find_rectangle(df),
        "cup_handle" => find_cup_handle(df),
        "rising_three" => find_three_methods(df, true),
        "falling_three" => find_three_methods(df, false),
        "lsd_top" => find_broadening(df),
        "diamond" => find_diamond(df),
        "rounding_bottom" => find_rounding_bottom(df),
        "mat_hold" => find_mat_hold(df),
        _ => vec![],
    }
}

fn get_ohlc(df: &DataFrame) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let o = df.column("open").map(|c| c.to_f64_vec()).unwrap_or_default();
    let h = df.column("high").map(|c| c.to_f64_vec()).unwrap_or_default();
    let l = df.column("low").map(|c| c.to_f64_vec()).unwrap_or_default();
    let c = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    (o, h, l, c)
}

fn find_swings(high: &[f64], low: &[f64], n: usize) -> (Vec<usize>, Vec<usize>) {
    let mut peaks = Vec::new();
    let mut troughs = Vec::new();
    let lookback = 5;
    for i in lookback..n.saturating_sub(lookback) {
        let is_peak = (i - lookback..=i + lookback).all(|j| high[i] >= high[j]);
        let is_trough = (i - lookback..=i + lookback).all(|j| low[i] <= low[j]);
        if is_peak { peaks.push(i); }
        if is_trough { troughs.push(i); }
    }
    (peaks, troughs)
}

// ── Double Top / Bottom ──

fn find_double_top(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 40 { return vec![]; }
    let (peaks, _) = find_swings(&h, &l, n);
    let mut results = Vec::new();
    for w in 0..peaks.len().saturating_sub(1) {
        let p1 = peaks[w];
        let p2 = peaks[w + 1];
        let dist = p2 - p1;
        if dist < 8 || dist > 60 { continue; }
        let avg_peak_h = (h[p1] + h[p2]) / 2.0;
        if (h[p1] - h[p2]).abs() / avg_peak_h.max(0.01) < 0.03 {
            let _valley_low = (p1..=p2).map(|i| l[i]).fold(f64::INFINITY, f64::min);
            let valley_idx = (p1..=p2)
                .min_by(|&a, &b| l[a].partial_cmp(&l[b]).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(p1);
            let neckline = l[valley_idx];
            let has_breakdown = (p2 + 1..n).any(|i| c[i] < neckline);
            if has_breakdown {
                results.push(PatternMatch {
                    name: "double_top".into(), name_cn: "双顶".into(),
                    start_idx: p1, end_idx: p2, confidence: 0.8,
                    direction: "bearish".into(),
                    description: format!("双顶 @ 颈线{:.2}", neckline),
                });
            }
        }
    }
    results
}

fn find_double_bottom(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 40 { return vec![]; }
    let (_, troughs) = find_swings(&h, &l, n);
    let mut results = Vec::new();
    for w in 0..troughs.len().saturating_sub(1) {
        let t1 = troughs[w];
        let t2 = troughs[w + 1];
        let dist = t2 - t1;
        if dist < 8 || dist > 60 { continue; }
        let avg_trough = (l[t1] + l[t2]) / 2.0;
        if (l[t1] - l[t2]).abs() / avg_trough.max(0.01) < 0.03 {
            let neckline = (t1..=t2).map(|i| h[i]).fold(f64::NEG_INFINITY, f64::max);
            let has_breakout = (t2 + 1..n).any(|i| c[i] > neckline);
            if has_breakout {
                results.push(PatternMatch {
                    name: "double_bottom".into(), name_cn: "双底".into(),
                    start_idx: t1, end_idx: t2, confidence: 0.8,
                    direction: "bullish".into(),
                    description: format!("双底 @ 颈线{:.2}", neckline),
                });
            }
        }
    }
    results
}

// ── Head & Shoulders ──

fn find_head_shoulders(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 60 { return vec![]; }
    let (peaks, _) = find_swings(&h, &l, n);
    let mut results = Vec::new();
    for w in 0..peaks.len().saturating_sub(2) {
        let ls = peaks[w];
        let head = peaks[w + 1];
        let rs = peaks[w + 2];
        if rs - ls > 60 || h[head] <= h[ls] || h[head] <= h[rs] { continue; }
        let shoulder_h = (h[ls] + h[rs]) / 2.0;
        if (h[ls] - h[rs]).abs() / shoulder_h.max(0.01) > 0.05 { continue; }
        if h[head] < shoulder_h * 1.02 { continue; }
        let neckline = (ls..=rs).map(|i| l[i]).fold(f64::INFINITY, f64::min);
        let has_breakdown = (rs + 1..n).any(|i| c[i] < neckline);
        if has_breakdown {
            results.push(PatternMatch {
                name: "head_shoulders".into(), name_cn: "头肩顶".into(),
                start_idx: ls, end_idx: rs, confidence: 0.75,
                direction: "bearish".into(),
                description: format!("头肩顶 @ 颈线{:.2}", neckline),
            });
        }
    }
    results
}

fn find_inv_head_shoulders(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 60 { return vec![]; }
    let (_, troughs) = find_swings(&h, &l, n);
    let mut results = Vec::new();
    for w in 0..troughs.len().saturating_sub(2) {
        let ls = troughs[w];
        let head = troughs[w + 1];
        let rs = troughs[w + 2];
        if rs - ls > 60 || l[head] >= l[ls] || l[head] >= l[rs] { continue; }
        let shoulder_l = (l[ls] + l[rs]) / 2.0;
        if (l[ls] - l[rs]).abs() / shoulder_l.max(0.01) > 0.05 { continue; }
        if l[head] > shoulder_l * 0.98 { continue; }
        let neckline = (ls..=rs).map(|i| h[i]).fold(f64::NEG_INFINITY, f64::max);
        let has_breakout = (rs + 1..n).any(|i| c[i] > neckline);
        if has_breakout {
            results.push(PatternMatch {
                name: "inv_head_shoulders".into(), name_cn: "头肩底".into(),
                start_idx: ls, end_idx: rs, confidence: 0.75,
                direction: "bullish".into(),
                description: format!("头肩底 @ 颈线{:.2}", neckline),
            });
        }
    }
    results
}

// ── Triangles ──

fn find_triangle(df: &DataFrame, ascending: bool) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 20 { return vec![]; }
    let window = 30.min(n);
    let recent_h = &h[n - window..];
    let recent_l = &l[n - window..];
    let first_h = recent_h.iter().take(10).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let last_h = recent_h.iter().rev().take(10).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let first_l = recent_l.iter().take(10).fold(f64::INFINITY, |a, &b| a.min(b));
    let last_l = recent_l.iter().rev().take(10).fold(f64::INFINITY, |a, &b| a.min(b));
    let h_range = (first_h - first_l).abs();
    let high_trend = last_h - first_h;
    let low_trend = last_l - first_l;

    if ascending && high_trend < h_range * 0.05 && low_trend > h_range * 0.05 {
        let conf = ((low_trend - high_trend) / h_range.max(0.01)).min(1.0);
        results(PatternMatch {
            name: "asc_triangle".into(), name_cn: "上升三角形".into(),
            start_idx: n - window, end_idx: n - 1, confidence: conf.max(0.5),
            direction: "bullish".into(),
            description: "上升三角形突破".into(),
        })
    } else if !ascending && low_trend > -h_range * 0.05 && high_trend < -h_range * 0.05 {
        let conf = ((low_trend - high_trend) / h_range.max(0.01)).min(1.0);
        results(PatternMatch {
            name: "desc_triangle".into(), name_cn: "下降三角形".into(),
            start_idx: n - window, end_idx: n - 1, confidence: conf.max(0.5),
            direction: "bearish".into(),
            description: "下降三角形突破".into(),
        })
    } else {
        vec![]
    }
}

fn find_sym_triangle(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 20 { return vec![]; }
    let window = 30.min(n);
    let recent_h = &h[n - window..];
    let recent_l = &l[n - window..];
    let first_h = recent_h.iter().take(10).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let last_h = recent_h.iter().rev().take(10).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let first_l = recent_l.iter().take(10).fold(f64::INFINITY, |a, &b| a.min(b));
    let last_l = recent_l.iter().rev().take(10).fold(f64::INFINITY, |a, &b| a.min(b));
    let h_range = (first_h - first_l).abs();
    let high_trend = last_h - first_h;
    let low_trend = last_l - first_l;
    // Symmetric: both converging
    if high_trend < -h_range * 0.03 && low_trend > h_range * 0.03 {
        let conf = ((low_trend - high_trend) / h_range.max(0.01)).min(1.0);
        results(PatternMatch {
            name: "sym_triangle".into(), name_cn: "对称三角形".into(),
            start_idx: n - window, end_idx: n - 1, confidence: conf.max(0.5),
            direction: "neutral".into(),
            description: "对称三角形".into(),
        })
    } else { vec![] }
}

// ── Flag / Wedge ──

fn find_flag(df: &DataFrame, is_wedge: bool) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 30 { return vec![]; }
    let pole_end = n - 10;
    if pole_end < 10 { return vec![]; }
    // Check for preceding sharp move (pole)
    let start = (pole_end - 10).max(0);
    let pole_move = (c[pole_end] - c[start]) / c[start].max(0.01);
    if pole_move.abs() < 0.05 { return vec![]; }
    // Recent range should be consolidating
    let recent_h = &h[pole_end..];
    let recent_l = &l[pole_end..];
    let avg_range = recent_h.iter().zip(recent_l.iter()).map(|(a, b)| a - b).sum::<f64>() / recent_h.len() as f64;
    if avg_range < 0.01 { return vec![]; }
    let name_cn = if is_wedge { "楔形" } else { "旗形" };
    let dir = if pole_move > 0.0 { "bullish" } else { "bearish" };
    results(PatternMatch {
        name: if is_wedge { "wedge" } else { "flag" }.into(),
        name_cn: format!("{}({})", name_cn, if dir == "bullish" { "看涨" } else { "看跌" }).into(),
        start_idx: start, end_idx: n - 1, confidence: 0.65,
        direction: dir.into(),
        description: format!("{}{} @ {:.2}", name_cn, if dir == "bullish" { "看涨" } else { "看跌" }, c[n - 1]),
    })
}

fn find_rectangle(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 20 { return vec![]; }
    let window = 20.min(n);
    let recent_h = &h[n - window..];
    let recent_l = &l[n - window..];
    let h_mean = recent_h.iter().sum::<f64>() / window as f64;
    let l_mean = recent_l.iter().sum::<f64>() / window as f64;
    let range = h_mean - l_mean;
    let h_std = (recent_h.iter().map(|&x| (x - h_mean).powi(2)).sum::<f64>() / window as f64).sqrt();
    let l_std = (recent_l.iter().map(|&x| (x - l_mean).powi(2)).sum::<f64>() / window as f64).sqrt();
    if h_std < range * 0.2 && l_std < range * 0.2 && range > 0.01 {
        results(PatternMatch {
            name: "rectangle".into(), name_cn: "矩形整理".into(),
            start_idx: n - window, end_idx: n - 1, confidence: 0.6,
            direction: "neutral".into(),
            description: format!("矩形整理 H:{:.2} L:{:.2}", h_mean, l_mean),
        })
    } else { vec![] }
}

fn find_cup_handle(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, _h, _l, c) = get_ohlc(df);
    let n = c.len();
    if n < 40 { return vec![]; }
    let cup_end = n - 5;
    let cup_start = (n - 35).max(0);
    // Simplified: look for a rounded U-shape in the cup period
    let cup_slice = &c[cup_start..cup_end];
    let mid = cup_slice.len() / 2;
    let left = cup_slice[..mid].iter().sum::<f64>() / mid as f64;
    let right = cup_slice[mid..].iter().sum::<f64>() / (cup_slice.len() - mid) as f64;
    let cup_min = cup_slice.iter().cloned().fold(f64::INFINITY, f64::min);
    // Left and right rims near same level, middle lower = U shape
    if (left - right).abs() / left.max(0.01) < 0.05 && cup_min < left * 0.85 {
        results(PatternMatch {
            name: "cup_handle".into(), name_cn: "杯柄形态".into(),
            start_idx: cup_start, end_idx: n - 1, confidence: 0.6,
            direction: "bullish".into(),
            description: format!("杯柄形态 @ {:.2}", c[n - 1]),
        })
    } else { vec![] }
}

fn find_three_methods(df: &DataFrame, rising: bool) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 4..o.len() {
        let avg = crate::avg_body(&o, &c, i);
        if rising {
            if crate::is_bullish(o[i-4], c[i-4]) && crate::body(o[i-4], c[i-4]) > avg * 1.2
                && (1..4).all(|j| crate::is_bearish(o[i-j], c[i-j]) && crate::body(o[i-j], c[i-j]) < avg)
                && (1..4).all(|j| l[i-j] > l[i-4] && h[i-j] < h[i-4])
                && crate::is_bullish(o[i], c[i]) && c[i] > h[i-4]
            {
                results.push(PatternMatch {
                    name: "rising_three".into(), name_cn: "上升三步曲".into(),
                    start_idx: i - 4, end_idx: i, confidence: 0.85,
                    direction: "bullish".into(),
                    description: format!("上升三步曲 @ {}", c[i]),
                });
            }
        } else {
            if crate::is_bearish(o[i-4], c[i-4]) && crate::body(o[i-4], c[i-4]) > avg * 1.2
                && (1..4).all(|j| crate::is_bullish(o[i-j], c[i-j]) && crate::body(o[i-j], c[i-j]) < avg)
                && (1..4).all(|j| l[i-j] > l[i-4] && h[i-j] < h[i-4])
                && crate::is_bearish(o[i], c[i]) && c[i] < l[i-4]
            {
                results.push(PatternMatch {
                    name: "falling_three".into(), name_cn: "下降三步曲".into(),
                    start_idx: i - 4, end_idx: i, confidence: 0.85,
                    direction: "bearish".into(),
                    description: format!("下降三步曲 @ {}", c[i]),
                });
            }
        }
    }
    results
}

fn find_broadening(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, _c) = get_ohlc(df);
    let n = h.len();
    if n < 20 { return vec![]; }
    let window = 20.min(n);
    let recent_h = &h[n - window..];
    let recent_l = &l[n - window..];
    let first_h = recent_h.iter().take(5).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let last_h = recent_h.iter().rev().take(5).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let first_l = recent_l.iter().take(5).fold(f64::INFINITY, |a, &b| a.min(b));
    let last_l = recent_l.iter().rev().take(5).fold(f64::INFINITY, |a, &b| a.min(b));
    // Broadening = highs getting higher, lows getting lower
    if last_h > first_h && last_l < first_l {
        results(PatternMatch {
            name: "lsd_top".into(), name_cn: "扩散顶".into(),
            start_idx: n - window, end_idx: n - 1, confidence: 0.55,
            direction: "bearish".into(),
            description: "扩散顶——波动加剧".into(),
        })
    } else { vec![] }
}

fn find_diamond(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, _c) = get_ohlc(df);
    let n = h.len();
    if n < 30 { return vec![]; }
    let mid = n - 15;
    let left_h = &h[mid - 10..mid];
    let right_h = &h[mid..n];
    let left_l = &l[mid - 10..mid];
    let right_l = &l[mid..n];
    let left_range = left_h.iter().zip(left_l).map(|(a, b)| a - b).sum::<f64>() / 10.0;
    let right_range = right_h.iter().zip(right_l).map(|(a, b)| a - b).sum::<f64>() / right_h.len() as f64;
    // Diamond: expanding then contracting
    if left_range > 0.01 && right_range < left_range * 0.6 {
        results(PatternMatch {
            name: "diamond".into(), name_cn: "菱形".into(),
            start_idx: mid - 10, end_idx: n - 1, confidence: 0.5,
            direction: "bearish".into(),
            description: "菱形顶部".into(),
        })
    } else { vec![] }
}

fn find_rounding_bottom(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, _h, _l, c) = get_ohlc(df);
    let n = c.len();
    if n < 40 { return vec![]; }
    let end = n - 1;
    let start = (n - 40).max(0);
    let slice = &c[start..=end];
    let mid = slice.len() / 2;
    let left_avg = slice[..mid].iter().sum::<f64>() / mid as f64;
    let right_avg = slice[mid..].iter().sum::<f64>() / (slice.len() - mid) as f64;
    let min_val = slice.iter().cloned().fold(f64::INFINITY, f64::min);
    let min_idx = slice.iter().position(|&x| x == min_val).unwrap_or(mid);
    // Rounded bottom: decline → flat bottom → recovery
    if min_idx > 5 && min_idx < slice.len() - 5 && min_val < left_avg * 0.85
        && (left_avg - right_avg).abs() / left_avg.max(0.01) < 0.08
    {
        results(PatternMatch {
            name: "rounding_bottom".into(), name_cn: "圆弧底".into(),
            start_idx: start, end_idx: end, confidence: 0.6,
            direction: "bullish".into(),
            description: format!("圆弧底 {:.2}→{:.2}", c[start], c[end]),
        })
    } else { vec![] }
}

// ── Mat Hold (垫形持有) ──
// Bullish: long white day1, small pullback days 2-4 within day1 body, then strong white day5

fn find_mat_hold(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 4..o.len() {
        let avg = crate::avg_body(&o, &c, i);
        // Day 1: strong bullish, long body
        if !crate::is_bullish(o[i-4], c[i-4]) || crate::body(o[i-4], c[i-4]) < avg * 1.5 {
            continue;
        }
        // Days 2-4: small bodies, stay within day1 range (no gap down)
        let pullback_ok = (1..4).all(|j| {
            let body_j = crate::body(o[i-j], c[i-j]);
            body_j < avg
                && l[i-j] > l[i-4]
                && h[i-j] <= h[i-4]
        });
        if !pullback_ok { continue; }
        // Day 5: strong bullish, makes new high above day1
        if crate::is_bullish(o[i], c[i]) && c[i] > h[i-4] {
            results.push(PatternMatch {
                name: "mat_hold".into(), name_cn: "垫形持有".into(),
                start_idx: i - 4, end_idx: i, confidence: 0.8,
                direction: "bullish".into(),
                description: format!("垫形持有 @ {}", c[i]),
            });
        }
    }
    results
}

fn results(pm: PatternMatch) -> Vec<PatternMatch> { vec![pm] }

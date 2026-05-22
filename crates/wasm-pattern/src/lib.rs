use wasm_core::{DataFrame, IndicatorOutput, OutputStyle, Column};

/// Recognized pattern with location and confidence
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub name: String,
    pub name_cn: String,
    pub start_idx: usize,
    pub end_idx: usize,
    pub confidence: f64,   // 0.0 ~ 1.0
    pub direction: String, // "bullish", "bearish", "neutral"
    pub description: String,
}

/// Available patterns list
pub fn list_patterns() -> Vec<PatternMeta> {
    vec![
        // Single candle
        PatternMeta { name: "doji".into(), name_cn: "十字星".into(), category: "单K线".into() },
        PatternMeta { name: "hammer".into(), name_cn: "锤子线".into(), category: "单K线".into() },
        PatternMeta { name: "inv_hammer".into(), name_cn: "倒锤子".into(), category: "单K线".into() },
        PatternMeta { name: "marubozu".into(), name_cn: "光头光脚".into(), category: "单K线".into() },
        PatternMeta { name: "spinning_top".into(), name_cn: "纺锤线".into(), category: "单K线".into() },
        // Double candle
        PatternMeta { name: "engulfing".into(), name_cn: "吞没形态".into(), category: "双K线".into() },
        PatternMeta { name: "harami".into(), name_cn: "孕线".into(), category: "双K线".into() },
        PatternMeta { name: "piercing".into(), name_cn: "刺透形态".into(), category: "双K线".into() },
        PatternMeta { name: "dark_cloud".into(), name_cn: "乌云盖顶".into(), category: "双K线".into() },
        PatternMeta { name: "tweezer".into(), name_cn: "平头形态".into(), category: "双K线".into() },
        // Triple candle
        PatternMeta { name: "morning_star".into(), name_cn: "启明星".into(), category: "三K线".into() },
        PatternMeta { name: "evening_star".into(), name_cn: "黄昏星".into(), category: "三K线".into() },
        PatternMeta { name: "three_white_soldiers".into(), name_cn: "红三兵".into(), category: "三K线".into() },
        PatternMeta { name: "three_black_crows".into(), name_cn: "三只乌鸦".into(), category: "三K线".into() },
        PatternMeta { name: "three_inside".into(), name_cn: "三重推进".into(), category: "三K线".into() },
        // Chart patterns
        PatternMeta { name: "double_top".into(), name_cn: "双顶".into(), category: "形态".into() },
        PatternMeta { name: "double_bottom".into(), name_cn: "双底".into(), category: "形态".into() },
        PatternMeta { name: "head_shoulders".into(), name_cn: "头肩顶".into(), category: "形态".into() },
        PatternMeta { name: "inv_head_shoulders".into(), name_cn: "头肩底".into(), category: "形态".into() },
        PatternMeta { name: "asc_triangle".into(), name_cn: "上升三角形".into(), category: "形态".into() },
        PatternMeta { name: "desc_triangle".into(), name_cn: "下降三角形".into(), category: "形态".into() },
        PatternMeta { name: "sym_triangle".into(), name_cn: "对称三角形".into(), category: "形态".into() },
        PatternMeta { name: "flag".into(), name_cn: "旗形".into(), category: "形态".into() },
        PatternMeta { name: "wedge".into(), name_cn: "楔形".into(), category: "形态".into() },
        PatternMeta { name: "rectangle".into(), name_cn: "矩形整理".into(), category: "形态".into() },
        PatternMeta { name: "cup_handle".into(), name_cn: "杯柄形态".into(), category: "形态".into() },
    ]
}

#[derive(Debug, Clone)]
pub struct PatternMeta {
    pub name: String,
    pub name_cn: String,
    pub category: String,
}

/// Recognize a specific pattern across the DataFrame
pub fn recognize(df: &DataFrame, pattern: &str) -> Vec<PatternMatch> {
    match pattern {
        // Single candle
        "doji" => find_doji(df),
        "hammer" => find_hammer(df),
        "inv_hammer" => find_inv_hammer(df),
        "marubozu" => find_marubozu(df),
        "spinning_top" => find_spinning_top(df),
        // Double candle
        "engulfing" => find_engulfing(df),
        "harami" => find_harami(df),
        "piercing" => find_piercing(df),
        "dark_cloud" => find_dark_cloud(df),
        "tweezer" => find_tweezer(df),
        // Triple candle
        "morning_star" => find_morning_star(df),
        "evening_star" => find_evening_star(df),
        "three_white_soldiers" => find_three_white_soldiers(df),
        "three_black_crows" => find_three_black_crows(df),
        "three_inside" => find_three_inside(df),
        // Chart patterns
        "double_top" => find_double_top(df),
        "double_bottom" => find_double_bottom(df),
        "head_shoulders" => find_head_shoulders(df),
        "inv_head_shoulders" => find_inv_head_shoulders(df),
        "asc_triangle" => find_triangle(df, true),
        "desc_triangle" => find_triangle(df, false),
        _ => vec![],
    }
}

/// Scan all patterns and return matches
pub fn scan_all(df: &DataFrame) -> Vec<PatternMatch> {
    let patterns = list_patterns();
    let mut results = Vec::new();
    for p in &patterns {
        let matches = recognize(df, &p.name);
        results.extend(matches);
    }
    results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Return pattern signals as IndicatorOutput for chart overlay
pub fn pattern_signal(df: &DataFrame, pattern: &str) -> IndicatorOutput {
    let matches = recognize(df, pattern);
    let n = df.len();
    let mut values = vec![0.0f64; n];

    for m in &matches {
        values[m.end_idx] = if m.direction == "bullish" { 1.0 }
            else if m.direction == "bearish" { -1.0 }
            else { 0.5 };
    }

    IndicatorOutput {
        name: format!("pattern_{}", pattern),
        values: Column::F64(values),
        style: OutputStyle::Dots,
    }
}

// ── Helpers ──

fn get_ohlc(df: &DataFrame) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let o = df.column("open").map(|c| c.to_f64_vec()).unwrap_or_default();
    let h = df.column("high").map(|c| c.to_f64_vec()).unwrap_or_default();
    let l = df.column("low").map(|c| c.to_f64_vec()).unwrap_or_default();
    let c = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    (o, h, l, c)
}

fn body(open: f64, close: f64) -> f64 { (close - open).abs() }
fn upper_shadow(h: f64, o: f64, c: f64) -> f64 { h - o.max(c) }
fn lower_shadow(o: f64, c: f64, l: f64) -> f64 { o.min(c) - l }
fn is_bullish(o: f64, c: f64) -> bool { c > o }
fn is_bearish(o: f64, c: f64) -> bool { c < o }

fn avg_body(o: &[f64], c: &[f64], n: usize) -> f64 {
    let start = if n > 20 { n - 20 } else { 0 };
    let count = n - start;
    if count == 0 { return 0.01; }
    (start..n).map(|i| body(o[i], c[i])).sum::<f64>() / count as f64
}

fn avg_range(h: &[f64], l: &[f64], n: usize) -> f64 {
    let start = if n > 20 { n - 20 } else { 0 };
    let count = n - start;
    if count == 0 { return 0.01; }
    (start..n).map(|i| h[i] - l[i]).sum::<f64>() / count as f64
}

// ── Single Candle Patterns ──

fn find_doji(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 0..o.len() {
        let b = body(o[i], c[i]);
        let range = h[i] - l[i];
        if range > 0.0 && b / range < 0.1 {
            let confidence = 1.0 - b / range;
            let us = upper_shadow(h[i], o[i], c[i]);
            let ls = lower_shadow(o[i], c[i], l[i]);
            let kind = if us > ls * 2.0 { "墓碑十字" }
                else if ls > us * 2.0 { "蜻蜓十字" }
                else { "标准十字星" };
            results.push(PatternMatch {
                name: "doji".into(), name_cn: kind.into(),
                start_idx: i, end_idx: i, confidence,
                direction: "neutral".into(),
                description: format!("{} @ {}", kind, c[i]),
            });
        }
    }
    results
}

fn find_hammer(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let b = body(o[i], c[i]);
        let range = h[i] - l[i];
        let ls = lower_shadow(o[i], c[i], l[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let avg = avg_body(&o, &c, i);
        // Hammer: small body at top, long lower shadow (2x+ body), little/no upper shadow
        if range > 0.0 && b > avg * 0.3 && ls > b * 2.0 && us < b * 0.5 {
            let conf = (ls / b.max(0.0001)).min(5.0) / 5.0;
            results.push(PatternMatch {
                name: "hammer".into(), name_cn: "锤子线".into(),
                start_idx: i, end_idx: i, confidence: conf,
                direction: "bullish".into(),
                description: format!("锤子线 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_inv_hammer(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let b = body(o[i], c[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let avg = avg_body(&o, &c, i);
        if b > avg * 0.3 && us > b * 2.0 && ls < b * 0.5 {
            let conf = (us / b.max(0.0001)).min(5.0) / 5.0;
            results.push(PatternMatch {
                name: "inv_hammer".into(), name_cn: "倒锤子".into(),
                start_idx: i, end_idx: i, confidence: conf,
                direction: "bearish".into(),
                description: format!("倒锤子 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_marubozu(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 0..o.len() {
        let b = body(o[i], c[i]);
        let range = h[i] - l[i];
        if range > 0.0 && b / range > 0.9 {
            let dir = if is_bullish(o[i], c[i]) { "bullish" } else { "bearish" };
            let cn = if dir == "bullish" { "光头光脚阳线" } else { "光头光脚阴线" };
            results.push(PatternMatch {
                name: "marubozu".into(), name_cn: cn.into(),
                start_idx: i, end_idx: i, confidence: b / range,
                direction: dir.into(),
                description: format!("{} @ {}", cn, c[i]),
            });
        }
    }
    results
}

fn find_spinning_top(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 0..o.len() {
        let b = body(o[i], c[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let range = h[i] - l[i];
        if range > 0.0 && b / range < 0.3 && us > b && ls > b {
            results.push(PatternMatch {
                name: "spinning_top".into(), name_cn: "纺锤线".into(),
                start_idx: i, end_idx: i, confidence: 0.7,
                direction: "neutral".into(),
                description: format!("纺锤线 @ {}", c[i]),
            });
        }
    }
    results
}

// ── Double Candle Patterns ──

fn find_engulfing(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let prev_bull = is_bullish(o[i-1], c[i-1]);
        let curr_bull = is_bullish(o[i], c[i]);
        let prev_body = body(o[i-1], c[i-1]);
        let curr_body = body(o[i], c[i]);
        let avg = avg_body(&o, &c, i);

        // Bullish engulfing: prev bearish, current bullish, current body engulfs prev
        if !prev_bull && curr_bull && curr_body > prev_body * 1.2
            && o[i] < c[i-1] && c[i] > o[i-1] && prev_body > avg * 0.5
        {
            let conf = (curr_body / prev_body.max(0.0001)).min(3.0) / 3.0;
            results.push(PatternMatch {
                name: "engulfing".into(), name_cn: "看涨吞没".into(),
                start_idx: i-1, end_idx: i, confidence: conf,
                direction: "bullish".into(),
                description: format!("看涨吞没 @ {}", c[i]),
            });
        }
        // Bearish engulfing
        if prev_bull && !curr_bull && curr_body > prev_body * 1.2
            && o[i] > c[i-1] && c[i] < o[i-1] && prev_body > avg * 0.5
        {
            let conf = (curr_body / prev_body.max(0.0001)).min(3.0) / 3.0;
            results.push(PatternMatch {
                name: "engulfing".into(), name_cn: "看跌吞没".into(),
                start_idx: i-1, end_idx: i, confidence: conf,
                direction: "bearish".into(),
                description: format!("看跌吞没 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_harami(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let prev_body = body(o[i-1], c[i-1]);
        let curr_body = body(o[i], c[i]);
        let avg = avg_body(&o, &c, i);
        // Harami: big body then small body contained within prev body
        if prev_body > avg * 1.5 && curr_body < prev_body * 0.5
            && c[i].min(o[i]) >= c[i-1].min(o[i-1])
            && c[i].max(o[i]) <= c[i-1].max(o[i-1])
        {
            let dir = if c[i] > o[i] { "bullish" } else { "bearish" };
            let cn = if dir == "bullish" { "看涨孕线" } else { "看跌孕线" };
            let conf = 1.0 - curr_body / prev_body.max(0.0001);
            results.push(PatternMatch {
                name: "harami".into(), name_cn: cn.into(),
                start_idx: i-1, end_idx: i, confidence: conf,
                direction: dir.into(),
                description: format!("{} @ {}", cn, c[i]),
            });
        }
    }
    results
}

fn find_piercing(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let prev_body = body(o[i-1], c[i-1]);
        let avg = avg_body(&o, &c, i);
        // Piercing: prev bearish, curr bullish, opens below prev low, closes above prev midpoint
        if is_bearish(o[i-1], c[i-1]) && is_bullish(o[i], c[i])
            && o[i] < l[i-1] && c[i] > (o[i-1] + c[i-1]) / 2.0
            && c[i] < o[i-1] && prev_body > avg * 0.5
        {
            let penetration = (c[i] - (o[i-1] + c[i-1]) / 2.0) / prev_body.max(0.0001);
            let conf = (penetration + 0.5).min(1.0);
            results.push(PatternMatch {
                name: "piercing".into(), name_cn: "刺透形态".into(),
                start_idx: i-1, end_idx: i, confidence: conf,
                direction: "bullish".into(),
                description: format!("刺透形态 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_dark_cloud(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let prev_body = body(o[i-1], c[i-1]);
        let avg = avg_body(&o, &c, i);
        // Dark cloud cover: prev bullish, curr bearish, opens above prev high, closes below prev midpoint
        if is_bullish(o[i-1], c[i-1]) && is_bearish(o[i], c[i])
            && o[i] > h[i-1] && c[i] < (o[i-1] + c[i-1]) / 2.0
            && c[i] > o[i-1] && prev_body > avg * 0.5
        {
            let penetration = ((o[i-1] + c[i-1]) / 2.0 - c[i]) / prev_body.max(0.0001);
            let conf = (penetration + 0.5).min(1.0);
            results.push(PatternMatch {
                name: "dark_cloud".into(), name_cn: "乌云盖顶".into(),
                start_idx: i-1, end_idx: i, confidence: conf,
                direction: "bearish".into(),
                description: format!("乌云盖顶 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_tweezer(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let avg_r = avg_range(&h, &l, i);
        // Tweezer top: matching highs
        if (h[i] - h[i-1]).abs() < avg_r * 0.1 {
            let dir = if is_bearish(o[i], c[i]) { "bearish" } else { "neutral" };
            results.push(PatternMatch {
                name: "tweezer".into(), name_cn: "平头顶".into(),
                start_idx: i-1, end_idx: i, confidence: 0.8,
                direction: dir.into(),
                description: format!("平头顶 @ {}", h[i]),
            });
        }
        // Tweezer bottom: matching lows
        if (l[i] - l[i-1]).abs() < avg_r * 0.1 {
            let dir = if is_bullish(o[i], c[i]) { "bullish" } else { "neutral" };
            results.push(PatternMatch {
                name: "tweezer".into(), name_cn: "平头底".into(),
                start_idx: i-1, end_idx: i, confidence: 0.8,
                direction: dir.into(),
                description: format!("平头底 @ {}", l[i]),
            });
        }
    }
    results
}

// ── Triple Candle Patterns ──

fn find_morning_star(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        let b2 = body(o[i], c[i]);
        // Day1: big bearish, Day2: small body (gap down), Day3: big bullish (gap up, closes into day1 body)
        if is_bearish(o[i-2], c[i-2]) && b0 > avg * 1.2
            && b1 < avg * 0.6 && c[i-1].max(o[i-1]) < c[i-2]
            && is_bullish(o[i], c[i]) && b2 > avg * 1.2
            && c[i] > (o[i-2] + c[i-2]) / 2.0
        {
            results.push(PatternMatch {
                name: "morning_star".into(), name_cn: "启明星".into(),
                start_idx: i-2, end_idx: i, confidence: 0.85,
                direction: "bullish".into(),
                description: format!("启明星 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_evening_star(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        let b2 = body(o[i], c[i]);
        // Day1: big bullish, Day2: small body (gap up), Day3: big bearish (gap down, closes into day1 body)
        if is_bullish(o[i-2], c[i-2]) && b0 > avg * 1.2
            && b1 < avg * 0.6 && c[i-1].min(o[i-1]) > c[i-2]
            && is_bearish(o[i], c[i]) && b2 > avg * 1.2
            && c[i] < (o[i-2] + c[i-2]) / 2.0
        {
            results.push(PatternMatch {
                name: "evening_star".into(), name_cn: "黄昏星".into(),
                start_idx: i-2, end_idx: i, confidence: 0.85,
                direction: "bearish".into(),
                description: format!("黄昏星 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_three_white_soldiers(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        let b2 = body(o[i], c[i]);
        // Three consecutive bullish candles, each closing higher, similar body sizes
        if is_bullish(o[i-2], c[i-2]) && is_bullish(o[i-1], c[i-1]) && is_bullish(o[i], c[i])
            && c[i-2] < c[i-1] && c[i-1] < c[i]
            && b0 > avg * 0.7 && b1 > avg * 0.7 && b2 > avg * 0.7
            && o[i-1] > o[i-2] && o[i-1] < c[i-2]  // opens within prev body
            && o[i] > o[i-1] && o[i] < c[i-1]
        {
            let consistency = 1.0 - (b0.max(b1.max(b2)) - b0.min(b1.min(b2))) / b0.max(b1.max(b2)).max(0.0001);
            results.push(PatternMatch {
                name: "three_white_soldiers".into(), name_cn: "红三兵".into(),
                start_idx: i-2, end_idx: i, confidence: consistency,
                direction: "bullish".into(),
                description: format!("红三兵 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_three_black_crows(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        let b2 = body(o[i], c[i]);
        if is_bearish(o[i-2], c[i-2]) && is_bearish(o[i-1], c[i-1]) && is_bearish(o[i], c[i])
            && c[i-2] > c[i-1] && c[i-1] > c[i]
            && b0 > avg * 0.7 && b1 > avg * 0.7 && b2 > avg * 0.7
            && o[i-1] < o[i-2] && o[i-1] > c[i-2]
            && o[i] < o[i-1] && o[i] > c[i-1]
        {
            let consistency = 1.0 - (b0.max(b1.max(b2)) - b0.min(b1.min(b2))) / b0.max(b1.max(b2)).max(0.0001);
            results.push(PatternMatch {
                name: "three_black_crows".into(), name_cn: "三只乌鸦".into(),
                start_idx: i-2, end_idx: i, confidence: consistency,
                direction: "bearish".into(),
                description: format!("三只乌鸦 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_three_inside(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        let _b2 = body(o[i], c[i]);
        // Three inside up: bearish, harami (bullish contained), bullish breakout
        if is_bearish(o[i-2], c[i-2]) && b0 > avg
            && b1 < b0 * 0.5 && c[i-1].min(o[i-1]) >= c[i-2] && c[i-1].max(o[i-1]) <= o[i-2]
            && is_bullish(o[i], c[i]) && c[i] > o[i-2]
        {
            results.push(PatternMatch {
                name: "three_inside".into(), name_cn: "上涨三重推进".into(),
                start_idx: i-2, end_idx: i, confidence: 0.8,
                direction: "bullish".into(),
                description: format!("上涨三重推进 @ {}", c[i]),
            });
        }
        // Three inside down: bullish, harami (bearish contained), bearish breakdown
        if is_bullish(o[i-2], c[i-2]) && b0 > avg
            && b1 < b0 * 0.5 && c[i-1].min(o[i-1]) >= o[i-2] && c[i-1].max(o[i-1]) <= c[i-2]
            && is_bearish(o[i], c[i]) && c[i] < o[i-2]
        {
            results.push(PatternMatch {
                name: "three_inside".into(), name_cn: "下跌三重推进".into(),
                start_idx: i-2, end_idx: i, confidence: 0.8,
                direction: "bearish".into(),
                description: format!("下跌三重推进 @ {}", c[i]),
            });
        }
    }
    results
}

// ── Chart Patterns ──

fn find_swings(high: &[f64], low: &[f64], n: usize) -> (Vec<usize>, Vec<usize>) {
    let mut peaks = Vec::new();
    let mut troughs = Vec::new();
    let lookback = 5;

    for i in lookback..n.saturating_sub(lookback) {
        let is_peak = (i-lookback..=i+lookback).all(|j| high[i] >= high[j]);
        let is_trough = (i-lookback..=i+lookback).all(|j| low[i] <= low[j]);

        if is_peak { peaks.push(i); }
        if is_trough { troughs.push(i); }
    }
    (peaks, troughs)
}

fn find_double_top(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 40 { return vec![]; }
    let (peaks, _) = find_swings(&h, &l, n);
    let mut results = Vec::new();

    for w in 0..peaks.len().saturating_sub(1) {
        let p1 = peaks[w];
        let p2 = peaks[w+1];
        let dist = p2 - p1;
        if dist < 8 || dist > 60 { continue; }

        let _ph = h[p1].max(h[p2]);
        let _pl = h[p1].min(h[p2]);
        let avg_peak_h = (h[p1] + h[p2]) / 2.0;

        // Two peaks at similar levels
        if (h[p1] - h[p2]).abs() / avg_peak_h.max(0.01) < 0.03 {
            // Check valley between peaks
            let valley_low = (p1..=p2).map(|i| l[i]).fold(f64::INFINITY, f64::min);
            let valley_idx = (p1..=p2).min_by(|&a, &b| l[a].partial_cmp(&l[b]).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(p1);
            let retrace = (avg_peak_h - valley_low) / avg_peak_h.max(0.01);

            // Neckline break
            let neckline = l[valley_idx];
            let has_breakdown = (p2+1..n).any(|i| c[i] < neckline);

            if retrace > 0.05 && has_breakdown {
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
        let t2 = troughs[w+1];
        let dist = t2 - t1;
        if dist < 8 || dist > 60 { continue; }

        let avg_trough = (l[t1] + l[t2]) / 2.0;
        if (l[t1] - l[t2]).abs() / avg_trough.max(0.01) < 0.03 {
            let valley_high = (t1..=t2).map(|i| h[i]).fold(f64::NEG_INFINITY, f64::max);
            let neckline = valley_high;
            let has_breakout = (t2+1..n).any(|i| c[i] > neckline);

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

fn find_head_shoulders(df: &DataFrame) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 60 { return vec![]; }
    let (peaks, _) = find_swings(&h, &l, n);
    let mut results = Vec::new();

    // Look for 3-peak pattern: left shoulder, head, right shoulder
    for w in 0..peaks.len().saturating_sub(2) {
        let ls = peaks[w];
        let head = peaks[w+1];
        let rs = peaks[w+2];
        if rs - ls > 60 { continue; }
        if h[head] <= h[ls] || h[head] <= h[rs] { continue; }

        // Shoulders should be at similar levels
        let shoulder_h = (h[ls] + h[rs]) / 2.0;
        if (h[ls] - h[rs]).abs() / shoulder_h.max(0.01) > 0.05 { continue; }

        // Head must be significantly higher
        if h[head] < shoulder_h * 1.02 { continue; }

        // Neckline between LS and RS troughs
        let between_trough = (ls..=rs).map(|i| l[i]).fold(f64::INFINITY, f64::min);
        let has_breakdown = (rs+1..n).any(|i| c[i] < between_trough);

        if has_breakdown {
            let conf = ((h[head] - shoulder_h) / shoulder_h.max(0.01)).min(0.3) / 0.3;
            results.push(PatternMatch {
                name: "head_shoulders".into(), name_cn: "头肩顶".into(),
                start_idx: ls, end_idx: rs, confidence: conf,
                direction: "bearish".into(),
                description: format!("头肩顶 @ 颈线{:.2}", between_trough),
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
        let head = troughs[w+1];
        let rs = troughs[w+2];
        if rs - ls > 60 { continue; }
        if l[head] >= l[ls] || l[head] >= l[rs] { continue; }

        let shoulder_l = (l[ls] + l[rs]) / 2.0;
        if (l[ls] - l[rs]).abs() / shoulder_l.max(0.01) > 0.05 { continue; }
        if l[head] > shoulder_l * 0.98 { continue; }

        let neckline = (ls..=rs).map(|i| h[i]).fold(f64::NEG_INFINITY, f64::max);
        let has_breakout = (rs+1..n).any(|i| c[i] > neckline);

        if has_breakout {
            let conf = ((shoulder_l - l[head]) / shoulder_l.max(0.01)).min(0.3) / 0.3;
            results.push(PatternMatch {
                name: "inv_head_shoulders".into(), name_cn: "头肩底".into(),
                start_idx: ls, end_idx: rs, confidence: conf,
                direction: "bullish".into(),
                description: format!("头肩底 @ 颈线{:.2}", neckline),
            });
        }
    }
    results
}

fn find_triangle(df: &DataFrame, ascending: bool) -> Vec<PatternMatch> {
    let (_o, h, l, c) = get_ohlc(df);
    let n = c.len();
    if n < 20 { return vec![]; }
    let (_peaks, _troughs) = find_swings(&h, &l, n);
    let mut results = Vec::new();

    // Simplified: look for convergence of peaks and troughs in recent window
    let window = 30.min(n);
    let recent_h = &h[n-window..];
    let recent_l = &l[n-window..];

    let first_h = recent_h.iter().take(10).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let last_h = recent_h.iter().rev().take(10).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let first_l = recent_l.iter().take(10).fold(f64::INFINITY, |a, &b| a.min(b));
    let last_l = recent_l.iter().rev().take(10).fold(f64::INFINITY, |a, &b| a.min(b));

    let high_trend = last_h - first_h;
    let low_trend = last_l - first_l;
    let h_range = (first_h - first_l).abs();

    if ascending && high_trend < h_range * 0.05 && low_trend > h_range * 0.05 {
        let conf = ((low_trend - high_trend) / h_range.max(0.01)).min(1.0);
        results.push(PatternMatch {
            name: "asc_triangle".into(), name_cn: "上升三角形".into(),
            start_idx: n - window, end_idx: n - 1, confidence: conf.max(0.5),
            direction: "bullish".into(),
            description: "上升三角形突破".into(),
        });
    }
    if !ascending && low_trend > -h_range * 0.05 && high_trend < -h_range * 0.05 {
        let conf = ((low_trend - high_trend) / h_range.max(0.01)).min(1.0);
        results.push(PatternMatch {
            name: "desc_triangle".into(), name_cn: "下降三角形".into(),
            start_idx: n - window, end_idx: n - 1, confidence: conf.max(0.5),
            direction: "bearish".into(),
            description: "下降三角形突破".into(),
        });
    }

    results
}

use wasm_core::DataFrame;
use crate::{PatternMatch, body, is_bullish, is_bearish, avg_body};

pub fn list_double() -> Vec<crate::PatternMeta> {
    vec![
        crate::PatternMeta { name: "engulfing".into(), name_cn: "吞没形态".into(), category: "双K线".into() },
        crate::PatternMeta { name: "harami".into(), name_cn: "孕线".into(), category: "双K线".into() },
        crate::PatternMeta { name: "harami_cross".into(), name_cn: "十字孕线".into(), category: "双K线".into() },
        crate::PatternMeta { name: "piercing".into(), name_cn: "刺透形态".into(), category: "双K线".into() },
        crate::PatternMeta { name: "dark_cloud".into(), name_cn: "乌云盖顶".into(), category: "双K线".into() },
        crate::PatternMeta { name: "tweezer".into(), name_cn: "平头形态".into(), category: "双K线".into() },
        crate::PatternMeta { name: "kicking".into(), name_cn: "踢击".into(), category: "双K线".into() },
        crate::PatternMeta { name: "meeting_lines".into(), name_cn: " встречные线".into(), category: "双K线".into() },
        crate::PatternMeta { name: "separating_lines".into(), name_cn: "分离线".into(), category: "双K线".into() },
        crate::PatternMeta { name: "matching_low".into(), name_cn: "匹配低点".into(), category: "双K线".into() },
        crate::PatternMeta { name: "on_neck".into(), name_cn: "颈上线".into(), category: "双K线".into() },
        crate::PatternMeta { name: "in_neck".into(), name_cn: "颈内线".into(), category: "双K线".into() },
        crate::PatternMeta { name: "thrusting".into(), name_cn: "穿刺线".into(), category: "双K线".into() },
        crate::PatternMeta { name: "tasuki_gap".into(), name_cn: "田足缺口".into(), category: "双K线".into() },
        crate::PatternMeta { name: "side_by_side_white".into(), name_cn: "并列白线".into(), category: "双K线".into() },
    ]
}

pub fn recognize_double(df: &DataFrame, pattern: &str) -> Vec<PatternMatch> {
    match pattern {
        "engulfing" => find_engulfing(df),
        "harami" => find_harami(df),
        "harami_cross" => find_harami_cross(df),
        "piercing" => find_piercing(df),
        "dark_cloud" => find_dark_cloud(df),
        "tweezer" => find_tweezer(df),
        "kicking" => find_kicking(df),
        "meeting_lines" => find_meeting_lines(df),
        "separating_lines" => find_separating_lines(df),
        "matching_low" => find_matching_low(df),
        "on_neck" => find_counter_lines(df, "on_neck"),
        "in_neck" => find_counter_lines(df, "in_neck"),
        "thrusting" => find_counter_lines(df, "thrusting"),
        "tasuki_gap" => find_tasuki_gap(df),
        "side_by_side_white" => find_side_by_side_white(df),
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

fn find_engulfing(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let prev_bull = is_bullish(o[i-1], c[i-1]);
        let curr_bull = is_bullish(o[i], c[i]);
        let prev_body = body(o[i-1], c[i-1]);
        let curr_body = body(o[i], c[i]);
        let avg = avg_body(&o, &c, i);

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

fn find_harami_cross(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let prev_body = body(o[i-1], c[i-1]);
        let curr_body = body(o[i], c[i]);
        let curr_range = h[i] - l[i];
        let avg = avg_body(&o, &c, i);
        // Day1: long body, Day2: doji contained within Day1 body
        if prev_body > avg * 1.5 && curr_body / curr_range.max(0.0001) < 0.1
            && c[i].min(o[i]) >= c[i-1].min(o[i-1])
            && c[i].max(o[i]) <= c[i-1].max(o[i-1])
        {
            let dir = if is_bullish(o[i-1], c[i-1]) { "bearish" } else { "bullish" };
            let cn = if dir == "bullish" { "看涨十字孕线" } else { "看跌十字孕线" };
            results.push(PatternMatch {
                name: "harami_cross".into(), name_cn: cn.into(),
                start_idx: i-1, end_idx: i, confidence: 0.9,
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
        let avg_r = crate::avg_range(&h, &l, i);
        if (h[i] - h[i-1]).abs() < avg_r * 0.1 {
            let dir = if is_bearish(o[i], c[i]) { "bearish" } else { "neutral" };
            results.push(PatternMatch {
                name: "tweezer".into(), name_cn: "平头顶".into(),
                start_idx: i-1, end_idx: i, confidence: 0.8,
                direction: dir.into(),
                description: format!("平头顶 @ {}", h[i]),
            });
        }
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

fn find_kicking(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let b0 = body(o[i-1], c[i-1]);
        let b1 = body(o[i], c[i]);
        let avg = avg_body(&o, &c, i);
        // Bullish kicking: marubozu bearish + gap up + marubozu bullish
        if is_bearish(o[i-1], c[i-1]) && b0 > avg * 1.3
            && is_bullish(o[i], c[i]) && b1 > avg * 1.3
            && o[i] > o[i-1]
        {
            results.push(PatternMatch {
                name: "kicking".into(), name_cn: "看涨踢击".into(),
                start_idx: i-1, end_idx: i, confidence: 0.9,
                direction: "bullish".into(),
                description: format!("看涨踢击 @ {}", c[i]),
            });
        }
        if is_bullish(o[i-1], c[i-1]) && b0 > avg * 1.3
            && is_bearish(o[i], c[i]) && b1 > avg * 1.3
            && o[i] < o[i-1]
        {
            results.push(PatternMatch {
                name: "kicking".into(), name_cn: "看跌踢击".into(),
                start_idx: i-1, end_idx: i, confidence: 0.9,
                direction: "bearish".into(),
                description: format!("看跌踢击 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_meeting_lines(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let b0 = body(o[i-1], c[i-1]);
        let b1 = body(o[i], c[i]);
        let avg = avg_body(&o, &c, i);
        // Meeting lines: contrasting directions, close at similar price
        if b0 > avg && b1 > avg &&
            is_bearish(o[i-1], c[i-1]) && is_bullish(o[i], c[i]) &&
            (c[i] - c[i-1]).abs() / avg.max(0.0001) < 0.1
        {
            results.push(PatternMatch {
                name: "meeting_lines".into(), name_cn: " встречные线(看涨)".into(),
                start_idx: i-1, end_idx: i, confidence: 0.8,
                direction: "bullish".into(),
                description: format!(" встречные线看涨 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_separating_lines(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let b0 = body(o[i-1], c[i-1]);
        let b1 = body(o[i], c[i]);
        let avg = avg_body(&o, &c, i);
        // Bullish separating: bearish day1 + bullish day2, same open
        if b0 > avg && b1 > avg &&
            is_bearish(o[i-1], c[i-1]) && is_bullish(o[i], c[i]) &&
            (o[i] - o[i-1]).abs() / avg.max(0.0001) < 0.1
        {
            results.push(PatternMatch {
                name: "separating_lines".into(), name_cn: "分离线(看涨)".into(),
                start_idx: i-1, end_idx: i, confidence: 0.75,
                direction: "bullish".into(),
                description: format!("分离线看涨 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_matching_low(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let b0 = body(o[i-1], c[i-1]);
        let b1 = body(o[i], c[i]);
        let avg = avg_body(&o, &c, i);
        if is_bearish(o[i-1], c[i-1]) && is_bearish(o[i], c[i])
            && b0 > avg && b1 > avg && (c[i] - c[i-1]).abs() / avg.max(0.0001) < 0.1
        {
            results.push(PatternMatch {
                name: "matching_low".into(), name_cn: "匹配低点".into(),
                start_idx: i-1, end_idx: i, confidence: 0.7,
                direction: "bullish".into(),
                description: format!("匹配低点 @ {}", c[i]),
            });
        }
    }
    results
}

// On neck / In neck / Thrusting — counterattack lines
fn find_counter_lines(df: &DataFrame, pattern: &str) -> Vec<PatternMatch> {
    let (o, _h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let prev_body = body(o[i-1], c[i-1]);
        let avg = avg_body(&o, &c, i);
        if !is_bearish(o[i-1], c[i-1]) || !is_bullish(o[i], c[i]) || prev_body < avg * 0.5 {
            continue;
        }
        let low1 = l[i-1];
        let body1_range = o[i-1] - c[i-1]; // bearish body
        let target = match pattern {
            "on_neck" => low1,
            "in_neck" => low1 + body1_range * 0.1,
            "thrusting" => low1 + body1_range * 0.3,
            _ => low1,
        };
        let name = match pattern {
            "on_neck" => "颈上线", "in_neck" => "颈内线", "thrusting" => "穿刺线", _ => "",
        };
        if (c[i] - target).abs() / avg.max(0.0001) < 0.15 {
            let dir = if pattern == "thrusting" { "bullish" } else { "bearish" };
            results.push(PatternMatch {
                name: pattern.into(), name_cn: name.into(),
                start_idx: i-1, end_idx: i, confidence: 0.65,
                direction: dir.into(),
                description: format!("{} @ {}", name, c[i]),
            });
        }
    }
    results
}

// ── Tasuki Gap (田足缺口) ──

fn find_tasuki_gap(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-1], c[i-1]);
        // Upside Tasuki: bullish day1, gap up day2 (bullish), then day3 bearish closing into gap
        if is_bullish(o[i-2], c[i-2]) && b0 > avg
            && is_bullish(o[i-1], c[i-1]) && l[i-1] > h[i-2] // gap up
            && is_bearish(o[i], c[i]) && c[i] < o[i-1] && c[i] > c[i-2] // closes into gap
        {
            results.push(PatternMatch {
                name: "tasuki_gap".into(), name_cn: "上行田足缺口".into(),
                start_idx: i-2, end_idx: i, confidence: 0.7,
                direction: "bullish".into(),
                description: format!("上行田足缺口 @ {}", c[i]),
            });
        }
        // Downside Tasuki: bearish day1, gap down day2 (bearish), then day3 bullish closing into gap
        if is_bearish(o[i-2], c[i-2]) && b0 > avg
            && is_bearish(o[i-1], c[i-1]) && h[i-1] < l[i-2] // gap down
            && is_bullish(o[i], c[i]) && c[i] > o[i-1] && c[i] < c[i-2] // closes into gap
        {
            results.push(PatternMatch {
                name: "tasuki_gap".into(), name_cn: "下行田足缺口".into(),
                start_idx: i-2, end_idx: i, confidence: 0.7,
                direction: "bearish".into(),
                description: format!("下行田足缺口 @ {}", c[i]),
            });
        }
    }
    results
}

// ── Side by Side White Lines (并列白线) ──

fn find_side_by_side_white(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        // Bullish: gap down then two equal-sized white lines
        if is_bearish(o[i-2], c[i-2]) && body(o[i-2], c[i-2]) > avg
            && is_bullish(o[i-1], c[i-1]) && is_bullish(o[i], c[i])
            && h[i-1] < l[i-2] // gap down after bearish
            && (c[i-1] - c[i]).abs() / avg.max(0.0001) < 0.15 // equal closes
            && (o[i-1] - o[i]).abs() / avg.max(0.0001) < 0.15 // equal opens
        {
            results.push(PatternMatch {
                name: "side_by_side_white".into(), name_cn: "并列白线(看涨)".into(),
                start_idx: i-1, end_idx: i, confidence: 0.7,
                direction: "bullish".into(),
                description: format!("并列白线看涨 @ {}", c[i]),
            });
        }
    }
    results
}

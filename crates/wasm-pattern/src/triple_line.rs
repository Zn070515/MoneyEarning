use wasm_core::DataFrame;
use crate::{PatternMatch, body, is_bullish, is_bearish, avg_body};

pub fn list_triple() -> Vec<crate::PatternMeta> {
    vec![
        crate::PatternMeta { name: "morning_star".into(), name_cn: "启明星".into(), category: "三K线".into() },
        crate::PatternMeta { name: "evening_star".into(), name_cn: "黄昏星".into(), category: "三K线".into() },
        crate::PatternMeta { name: "morning_doji_star".into(), name_cn: "晨十字星".into(), category: "三K线".into() },
        crate::PatternMeta { name: "evening_doji_star".into(), name_cn: "昏十字星".into(), category: "三K线".into() },
        crate::PatternMeta { name: "three_white_soldiers".into(), name_cn: "红三兵".into(), category: "三K线".into() },
        crate::PatternMeta { name: "three_black_crows".into(), name_cn: "三只乌鸦".into(), category: "三K线".into() },
        crate::PatternMeta { name: "three_inside_up".into(), name_cn: "三内升".into(), category: "三K线".into() },
        crate::PatternMeta { name: "three_inside_down".into(), name_cn: "三内降".into(), category: "三K线".into() },
        crate::PatternMeta { name: "three_outside_up".into(), name_cn: "三外升".into(), category: "三K线".into() },
        crate::PatternMeta { name: "three_outside_down".into(), name_cn: "三外降".into(), category: "三K线".into() },
        crate::PatternMeta { name: "abandoned_baby".into(), name_cn: "弃婴".into(), category: "三K线".into() },
        crate::PatternMeta { name: "tristar".into(), name_cn: "三星".into(), category: "三K线".into() },
        crate::PatternMeta { name: "unique_three_river".into(), name_cn: "独立三河".into(), category: "三K线".into() },
        crate::PatternMeta { name: "three_stars_south".into(), name_cn: "南方三星".into(), category: "三K线".into() },
        crate::PatternMeta { name: "squeeze_alert".into(), name_cn: "挤压预警".into(), category: "三K线".into() },
        crate::PatternMeta { name: "ladder_bottom".into(), name_cn: "阶梯底".into(), category: "三K线".into() },
    ]
}

pub fn recognize_triple(df: &DataFrame, pattern: &str) -> Vec<PatternMatch> {
    match pattern {
        "morning_star" => find_morning_star(df),
        "evening_star" => find_evening_star(df),
        "morning_doji_star" => find_doji_star(df, true),
        "evening_doji_star" => find_doji_star(df, false),
        "three_white_soldiers" => find_three_white_soldiers(df),
        "three_black_crows" => find_three_black_crows(df),
        "three_inside_up" => find_three_inside(df, true),
        "three_inside_down" => find_three_inside(df, false),
        "three_outside_up" => find_three_outside(df, true),
        "three_outside_down" => find_three_outside(df, false),
        "abandoned_baby" => find_abandoned_baby(df),
        "tristar" => find_tristar(df),
        "unique_three_river" => find_unique_three_river(df),
        "three_stars_south" => find_three_stars_south(df),
        "squeeze_alert" => find_squeeze_alert(df),
        "ladder_bottom" => find_ladder_bottom(df),
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

fn find_morning_star(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        let b2 = body(o[i], c[i]);
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

fn find_doji_star(df: &DataFrame, morning: bool) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        let b2 = body(o[i], c[i]);
        let range1 = h[i-1] - l[i-1];
        let is_doji = b1 / range1.max(0.0001) < 0.1;

        if morning {
            if is_bearish(o[i-2], c[i-2]) && b0 > avg * 1.2
                && is_doji && c[i-1] < c[i-2] // gap down
                && is_bullish(o[i], c[i]) && b2 > avg * 1.2
                && c[i] > (o[i-2] + c[i-2]) / 2.0
            {
                results.push(PatternMatch {
                    name: "morning_doji_star".into(), name_cn: "晨十字星".into(),
                    start_idx: i-2, end_idx: i, confidence: 0.9,
                    direction: "bullish".into(),
                    description: format!("晨十字星 @ {}", c[i]),
                });
            }
        } else {
            if is_bullish(o[i-2], c[i-2]) && b0 > avg * 1.2
                && is_doji && c[i-1] > c[i-2] // gap up
                && is_bearish(o[i], c[i]) && b2 > avg * 1.2
                && c[i] < (o[i-2] + c[i-2]) / 2.0
            {
                results.push(PatternMatch {
                    name: "evening_doji_star".into(), name_cn: "昏十字星".into(),
                    start_idx: i-2, end_idx: i, confidence: 0.9,
                    direction: "bearish".into(),
                    description: format!("昏十字星 @ {}", c[i]),
                });
            }
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
        if is_bullish(o[i-2], c[i-2]) && is_bullish(o[i-1], c[i-1]) && is_bullish(o[i], c[i])
            && c[i-2] < c[i-1] && c[i-1] < c[i]
            && b0 > avg * 0.7 && b1 > avg * 0.7 && b2 > avg * 0.7
            && o[i-1] > o[i-2] && o[i-1] < c[i-2]
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

fn find_three_inside(df: &DataFrame, up: bool) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        let b2 = body(o[i], c[i]);

        if up {
            // Bearish → small bullish harami → strong bullish breakout
            if is_bearish(o[i-2], c[i-2]) && b0 > avg
                && is_bullish(o[i-1], c[i-1]) && b1 < b0 * 0.5
                && c[i-1].min(o[i-1]) >= c[i-2] && c[i-1].max(o[i-1]) <= o[i-2]
                && is_bullish(o[i], c[i]) && b2 > b1 * 2.0 && c[i] > o[i-2]
            {
                results.push(PatternMatch {
                    name: "three_inside_up".into(), name_cn: "三内升".into(),
                    start_idx: i-2, end_idx: i, confidence: 0.8,
                    direction: "bullish".into(),
                    description: format!("三内升 @ {}", c[i]),
                });
            }
        } else {
            if is_bullish(o[i-2], c[i-2]) && b0 > avg
                && is_bearish(o[i-1], c[i-1]) && b1 < b0 * 0.5
                && c[i-1].min(o[i-1]) >= o[i-2] && c[i-1].max(o[i-1]) <= c[i-2]
                && is_bearish(o[i], c[i]) && b2 > b1 * 2.0 && c[i] < o[i-2]
            {
                results.push(PatternMatch {
                    name: "three_inside_down".into(), name_cn: "三内降".into(),
                    start_idx: i-2, end_idx: i, confidence: 0.8,
                    direction: "bearish".into(),
                    description: format!("三内降 @ {}", c[i]),
                });
            }
        }
    }
    results
}

fn find_three_outside(df: &DataFrame, up: bool) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        if up {
            // Small bearish → bullish engulfing → bullish continuation
            if is_bearish(o[i-2], c[i-2]) && b0 < avg
                && is_bullish(o[i-1], c[i-1]) && b1 > b0 * 1.5
                && o[i-1] < c[i-2] && c[i-1] > o[i-2]
                && is_bullish(o[i], c[i]) && c[i] > c[i-1]
            {
                results.push(PatternMatch {
                    name: "three_outside_up".into(), name_cn: "三外升".into(),
                    start_idx: i-2, end_idx: i, confidence: 0.8,
                    direction: "bullish".into(),
                    description: format!("三外升 @ {}", c[i]),
                });
            }
        } else {
            if is_bullish(o[i-2], c[i-2]) && b0 < avg
                && is_bearish(o[i-1], c[i-1]) && b1 > b0 * 1.5
                && o[i-1] > c[i-2] && c[i-1] < o[i-2]
                && is_bearish(o[i], c[i]) && c[i] < c[i-1]
            {
                results.push(PatternMatch {
                    name: "three_outside_down".into(), name_cn: "三外降".into(),
                    start_idx: i-2, end_idx: i, confidence: 0.8,
                    direction: "bearish".into(),
                    description: format!("三外降 @ {}", c[i]),
                });
            }
        }
    }
    results
}

fn find_abandoned_baby(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        let b1 = body(o[i-1], c[i-1]);
        let b2 = body(o[i], c[i]);
        let range1 = h[i-1] - l[i-1];
        let is_doji = b1 / range1.max(0.0001) < 0.1;

        // Bullish abandoned baby: bearish → doji gap down (shadows also gapped) → bullish gap up
        if is_bearish(o[i-2], c[i-2]) && b0 > avg * 1.2
            && is_doji && h[i-1] < l[i-2] // complete gap down (including shadows)
            && is_bullish(o[i], c[i]) && b2 > avg * 1.2
            && l[i] > h[i-1] // complete gap up (including shadows)
        {
            results.push(PatternMatch {
                name: "abandoned_baby".into(), name_cn: "弃婴(看涨)".into(),
                start_idx: i-2, end_idx: i, confidence: 0.95,
                direction: "bullish".into(),
                description: format!("弃婴看涨 @ {}", c[i]),
            });
        }
        // Bearish abandoned baby
        if is_bullish(o[i-2], c[i-2]) && b0 > avg * 1.2
            && is_doji && l[i-1] > h[i-2] // gap up
            && is_bearish(o[i], c[i]) && b2 > avg * 1.2
            && h[i] < l[i-1] // gap down
        {
            results.push(PatternMatch {
                name: "abandoned_baby".into(), name_cn: "弃婴(看跌)".into(),
                start_idx: i-2, end_idx: i, confidence: 0.95,
                direction: "bearish".into(),
                description: format!("弃婴看跌 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_tristar(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let is_doji = |idx: usize| {
            let b = body(o[idx], c[idx]);
            let range = h[idx] - l[idx];
            range > 0.0 && b / range < 0.1
        };
        if is_doji(i-2) && is_doji(i-1) && is_doji(i) {
            // Middle doji must gap from the others
            if c[i-1] > c[i-2] && c[i] < c[i-1] {
                results.push(PatternMatch {
                    name: "tristar".into(), name_cn: "三星(看跌)".into(),
                    start_idx: i-2, end_idx: i, confidence: 0.8,
                    direction: "bearish".into(),
                    description: format!("三星看跌 @ {}", c[i]),
                });
            } else if c[i-1] < c[i-2] && c[i] > c[i-1] {
                results.push(PatternMatch {
                    name: "tristar".into(), name_cn: "三星(看涨)".into(),
                    start_idx: i-2, end_idx: i, confidence: 0.8,
                    direction: "bullish".into(),
                    description: format!("三星看涨 @ {}", c[i]),
                });
            }
        }
    }
    results
}

fn find_unique_three_river(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        // Day1: bearish long body, Day2: harami-like narrow body, Day3: small bullish within day2 range
        if is_bearish(o[i-2], c[i-2]) && body(o[i-2], c[i-2]) > avg * 1.2
            && body(o[i-1], c[i-1]) < body(o[i-2], c[i-2]) * 0.5
            && is_bullish(o[i], c[i]) && body(o[i], c[i]) < body(o[i-1], c[i-1])
            && l[i] > l[i-1]
        {
            results.push(PatternMatch {
                name: "unique_three_river".into(), name_cn: "独立三河".into(),
                start_idx: i-2, end_idx: i, confidence: 0.7,
                direction: "bullish".into(),
                description: format!("独立三河 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_three_stars_south(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 2..o.len() {
        let avg = avg_body(&o, &c, i);
        let b0 = body(o[i-2], c[i-2]);
        // Day1: long bearish, Day2: smaller bearish with lower low, Day3: minimal bearish marubozu
        if is_bearish(o[i-2], c[i-2]) && b0 > avg * 1.2
            && is_bearish(o[i-1], c[i-1]) && body(o[i-1], c[i-1]) < b0
            && l[i-1] < l[i-2]
            && is_bearish(o[i], c[i]) && body(o[i], c[i]) < body(o[i-1], c[i-1])
            && l[i] > l[i-1] // higher low = selling exhausted
        {
            results.push(PatternMatch {
                name: "three_stars_south".into(), name_cn: "南方三星".into(),
                start_idx: i-2, end_idx: i, confidence: 0.75,
                direction: "bullish".into(),
                description: format!("南方三星 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_squeeze_alert(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 3..o.len() {
        let ranges: Vec<f64> = (i-3..=i).map(|j| h[j] - l[j]).collect();
        // Narrowing ranges over 3 bars = squeeze
        if ranges[3] < ranges[2] && ranges[2] < ranges[1] && ranges[1] < ranges[0] {
            let total_compression = 1.0 - ranges[3] / ranges[0].max(0.0001);
            if total_compression > 0.5 && is_bullish(o[i], c[i]) {
                results.push(PatternMatch {
                    name: "squeeze_alert".into(), name_cn: "挤压预警".into(),
                    start_idx: i-3, end_idx: i, confidence: total_compression,
                    direction: "bullish".into(),
                    description: format!("挤压预警 @ {}", c[i]),
                });
            }
        }
    }
    results
}

// ── Ladder Bottom (阶梯底) ──
// 4 consecutive lower closes followed by a bullish reversal

fn find_ladder_bottom(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 5..o.len() {
        // Days 1-4: consecutive lower closes (bearish or mixed)
        let lower1 = c[i-4] < c[i-5].max(c[i-4] + 1.0);
        let lower2 = c[i-3] < c[i-4];
        let lower3 = c[i-2] < c[i-3];
        let lower4 = c[i-1] < c[i-2];
        let descending = lower1 && lower2 && lower3 && lower4;

        // Day 5: gap up open above previous close, strong bullish close
        let gap_up = o[i] > c[i-1];
        let strong_close = is_bullish(o[i], c[i]) && c[i] > o[i-2];

        if descending && gap_up && strong_close {
            let conf = 0.75;
            results.push(PatternMatch {
                name: "ladder_bottom".into(), name_cn: "阶梯底".into(),
                start_idx: i - 4, end_idx: i, confidence: conf,
                direction: "bullish".into(),
                description: format!("阶梯底 @ {}", c[i]),
            });
        }
    }
    results
}

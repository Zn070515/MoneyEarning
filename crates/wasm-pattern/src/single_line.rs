use wasm_core::DataFrame;
use crate::{PatternMatch, body, upper_shadow, lower_shadow, is_bullish, is_bearish, avg_body};

pub fn list_single() -> Vec<crate::PatternMeta> {
    vec![
        crate::PatternMeta { name: "doji".into(), name_cn: "十字星".into(), category: "单K线".into() },
        crate::PatternMeta { name: "hammer".into(), name_cn: "锤子线".into(), category: "单K线".into() },
        crate::PatternMeta { name: "inv_hammer".into(), name_cn: "倒锤子".into(), category: "单K线".into() },
        crate::PatternMeta { name: "hanging_man".into(), name_cn: "吊颈线".into(), category: "单K线".into() },
        crate::PatternMeta { name: "shooting_star".into(), name_cn: "射击之星".into(), category: "单K线".into() },
        crate::PatternMeta { name: "marubozu".into(), name_cn: "光头光脚".into(), category: "单K线".into() },
        crate::PatternMeta { name: "spinning_top".into(), name_cn: "纺锤线".into(), category: "单K线".into() },
        crate::PatternMeta { name: "dragonfly_doji".into(), name_cn: "蜻蜓十字".into(), category: "单K线".into() },
        crate::PatternMeta { name: "gravestone_doji".into(), name_cn: "墓碑十字".into(), category: "单K线".into() },
        crate::PatternMeta { name: "long_body".into(), name_cn: "长实体".into(), category: "单K线".into() },
        crate::PatternMeta { name: "long_upper".into(), name_cn: "长上影".into(), category: "单K线".into() },
        crate::PatternMeta { name: "long_lower".into(), name_cn: "长下影".into(), category: "单K线".into() },
        crate::PatternMeta { name: "belt_hold".into(), name_cn: "捉腰带线".into(), category: "单K线".into() },
    ]
}

pub fn recognize_single(df: &DataFrame, pattern: &str) -> Vec<PatternMatch> {
    match pattern {
        "doji" => find_doji(df),
        "hammer" => find_hammer(df),
        "inv_hammer" => find_inv_hammer(df),
        "hanging_man" => find_hanging_man(df),
        "shooting_star" => find_shooting_star(df),
        "marubozu" => find_marubozu(df),
        "spinning_top" => find_spinning_top(df),
        "dragonfly_doji" => find_dragonfly_doji(df),
        "gravestone_doji" => find_gravestone_doji(df),
        "long_body" => find_long_body(df),
        "long_upper" => find_long_upper(df),
        "long_lower" => find_long_lower(df),
        "belt_hold" => find_belt_hold(df),
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
            let (kind, direction) = if us > ls * 2.0 { ("墓碑十字", "bearish") }
                else if ls > us * 2.0 { ("蜻蜓十字", "bullish") }
                else { ("标准十字星", "neutral") };
            results.push(PatternMatch {
                name: "doji".into(), name_cn: kind.into(),
                start_idx: i, end_idx: i, confidence,
                direction: direction.into(),
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
        let ls = lower_shadow(o[i], c[i], l[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let avg = avg_body(&o, &c, i);
        if b > avg * 0.3 && ls > b * 2.0 && us < b * 0.5 {
            let conf = (ls / b.max(0.0001)).min(5.0) / 5.0;
            // Confirmation: prior downtrend
            let prior_bearish = i >= 3 && (0..3).all(|j| c[i-1-j] < o[i-1-j] || c[i-1-j] < c[i-2-j]);
            let conf = if prior_bearish { conf } else { conf * 0.6 };
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
            let prior_bearish = i >= 3 && (0..3).all(|j| c[i-1-j] < o[i-1-j]);
            let conf = if prior_bearish { conf } else { conf * 0.6 };
            results.push(PatternMatch {
                name: "inv_hammer".into(), name_cn: "倒锤子".into(),
                start_idx: i, end_idx: i, confidence: conf,
                direction: "bullish".into(),
                description: format!("倒锤子 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_hanging_man(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let b = body(o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let avg = avg_body(&o, &c, i);
        if b > avg * 0.3 && ls > b * 2.0 && us < b * 0.5 {
            let conf = (ls / b.max(0.0001)).min(5.0) / 5.0;
            let prior_bullish = i >= 3 && (0..3).all(|j| c[i-1-j] > o[i-1-j]);
            if prior_bullish {
                results.push(PatternMatch {
                    name: "hanging_man".into(), name_cn: "吊颈线".into(),
                    start_idx: i, end_idx: i, confidence: conf,
                    direction: "bearish".into(),
                    description: format!("吊颈线 @ {}", c[i]),
                });
            }
        }
    }
    results
}

fn find_shooting_star(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let b = body(o[i], c[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let avg = avg_body(&o, &c, i);
        if b > avg * 0.3 && us > b * 2.0 && ls < b * 0.5 {
            let conf = (us / b.max(0.0001)).min(5.0) / 5.0;
            let prior_bullish = i >= 3 && (0..3).all(|j| c[i-1-j] > o[i-1-j]);
            if prior_bullish {
                results.push(PatternMatch {
                    name: "shooting_star".into(), name_cn: "射击之星".into(),
                    start_idx: i, end_idx: i, confidence: conf,
                    direction: "bearish".into(),
                    description: format!("射击之星 @ {}", c[i]),
                });
            }
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

fn find_dragonfly_doji(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 0..o.len() {
        let b = body(o[i], c[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let range = h[i] - l[i];
        if range > 0.0 && b / range < 0.1 && ls > range * 0.6 && us < range * 0.1 {
            results.push(PatternMatch {
                name: "dragonfly_doji".into(), name_cn: "蜻蜓十字".into(),
                start_idx: i, end_idx: i, confidence: 0.85,
                direction: "bullish".into(),
                description: format!("蜻蜓十字 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_gravestone_doji(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 0..o.len() {
        let b = body(o[i], c[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let range = h[i] - l[i];
        if range > 0.0 && b / range < 0.1 && us > range * 0.6 && ls < range * 0.1 {
            results.push(PatternMatch {
                name: "gravestone_doji".into(), name_cn: "墓碑十字".into(),
                start_idx: i, end_idx: i, confidence: 0.85,
                direction: "bearish".into(),
                description: format!("墓碑十字 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_long_body(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, _h, _l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 5..o.len() {
        let b = body(o[i], c[i]);
        let avg = avg_body(&o, &c, i);
        if b > avg * 2.0 {
            let dir = if is_bullish(o[i], c[i]) { "bullish" } else { "bearish" };
            let cn = if dir == "bullish" { "长阳线" } else { "长阴线" };
            results.push(PatternMatch {
                name: "long_body".into(), name_cn: cn.into(),
                start_idx: i, end_idx: i, confidence: (b / avg.max(0.0001) - 1.0).min(1.0),
                direction: dir.into(),
                description: format!("{} @ {}", cn, c[i]),
            });
        }
    }
    results
}

fn find_long_upper(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 5..o.len() {
        let b = body(o[i], c[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        if us > b * 2.0 && us > ls {
            results.push(PatternMatch {
                name: "long_upper".into(), name_cn: "长上影".into(),
                start_idx: i, end_idx: i, confidence: (us / b.max(0.0001) / 3.0).min(1.0),
                direction: "bearish".into(),
                description: format!("长上影 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_long_lower(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 5..o.len() {
        let b = body(o[i], c[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        if ls > b * 2.0 && ls > us {
            results.push(PatternMatch {
                name: "long_lower".into(), name_cn: "长下影".into(),
                start_idx: i, end_idx: i, confidence: (ls / b.max(0.0001) / 3.0).min(1.0),
                direction: "bullish".into(),
                description: format!("长下影 @ {}", c[i]),
            });
        }
    }
    results
}

fn find_belt_hold(df: &DataFrame) -> Vec<PatternMatch> {
    let (o, h, l, c) = get_ohlc(df);
    let mut results = Vec::new();
    for i in 1..o.len() {
        let b = body(o[i], c[i]);
        let us = upper_shadow(h[i], o[i], c[i]);
        let ls = lower_shadow(o[i], c[i], l[i]);
        let avg = avg_body(&o, &c, i);
        // Belt hold: opens at/near high (bullish) or low (bearish), long body, no shadow on open side
        if b > avg * 1.5 {
            if is_bullish(o[i], c[i]) && ls < b * 0.1 && o[i] <= l[i] * 1.01 {
                results.push(PatternMatch {
                    name: "belt_hold".into(), name_cn: "看涨捉腰带线".into(),
                    start_idx: i, end_idx: i, confidence: 0.75,
                    direction: "bullish".into(),
                    description: format!("看涨捉腰带线 @ {}", c[i]),
                });
            }
            if is_bearish(o[i], c[i]) && us < b * 0.1 && o[i] >= h[i] * 0.99 {
                results.push(PatternMatch {
                    name: "belt_hold".into(), name_cn: "看跌捉腰带线".into(),
                    start_idx: i, end_idx: i, confidence: 0.75,
                    direction: "bearish".into(),
                    description: format!("看跌捉腰带线 @ {}", c[i]),
                });
            }
        }
    }
    results
}

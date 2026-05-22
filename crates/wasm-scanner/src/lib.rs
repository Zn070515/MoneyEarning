use std::collections::HashMap;
use wasm_core::{DataFrame, ScanExpr};

/// Scan result for one stock
#[derive(Debug, Clone)]
pub struct ScanMatch {
    pub stock_index: usize,
    pub score: f64,       // higher = stronger signal
    pub signals: Vec<String>, // descriptions of matching conditions
}

/// Run scanner over multiple stocks with a scan expression tree
pub fn scan(dfs: &[(usize, DataFrame)], expr: &ScanExpr) -> Vec<ScanMatch> {
    let mut matches = Vec::new();

    for (stock_index, df) in dfs {
        if let Some(score) = eval_expr(expr, df) {
            matches.push(ScanMatch {
                stock_index: *stock_index,
                score,
                signals: extract_signals(expr, df),
            });
        }
    }

    // Sort by score descending
    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    matches
}

fn eval_expr(expr: &ScanExpr, df: &DataFrame) -> Option<f64> {
    match expr.op.as_str() {
        "AND" | "and" => {
            let mut best = None;
            for child in &expr.children {
                let v = eval_expr(child, df)?;
                // score = product to emphasize all conditions must be strong
                best = Some(best.map_or(v, |b: f64| b.min(v).max(1.0) * (b + v) / 20.0));
            }
            best
        }
        "OR" | "or" => {
            let mut best: Option<f64> = None;
            for child in &expr.children {
                if let Some(v) = eval_expr(child, df) {
                    best = Some(best.map_or(v, |b| b.max(v)));
                }
            }
            best
        }
        "COMPARE" | "compare" => {
            eval_compare(expr, df)
        }
        "CROSS" | "cross" => {
            eval_cross(expr, df)
        }
        _ => None,
    }
}

fn eval_compare(expr: &ScanExpr, df: &DataFrame) -> Option<f64> {
    let indicator = expr.indicator.as_ref()?;
    let params = expr.params.clone().unwrap_or_default();
    let compare_op = expr.compare_op.as_ref()?;
    let threshold = expr.value?;

    let outputs = wasm_indicators::compute(indicator, df, &params).ok()?;
    let main = outputs.first()?;
    let vals = main.values.to_f64_vec();
    let last = *vals.last()?;

    match compare_op.as_str() {
        ">" | "gt" => if last > threshold { Some((last - threshold).abs() / threshold.max(1e-6)) } else { None },
        "<" | "lt" => if last < threshold { Some((threshold - last).abs() / threshold.max(1e-6)) } else { None },
        ">=" | "gte" => if last >= threshold { Some((last - threshold).abs() / threshold.max(1e-6)) } else { None },
        "<=" | "lte" => if last <= threshold { Some((threshold - last).abs() / threshold.max(1e-6)) } else { None },
        "==" | "eq" => if (last - threshold).abs() < 1e-6 { Some(1.0) } else { None },
        "!=" | "ne" => if (last - threshold).abs() >= 1e-6 { Some(1.0) } else { None },
        _ => None,
    }
}

fn eval_cross(expr: &ScanExpr, df: &DataFrame) -> Option<f64> {
    // Cross: fast indicator crosses above/below slow indicator
    let params = expr.params.clone().unwrap_or_default();
    let indicator = expr.indicator.as_ref()?;
    let direction = expr.compare_op.as_deref().unwrap_or("above");

    // For crossover, we need two indicator lines
    let outputs = wasm_indicators::compute(indicator, df, &params).ok()?;

    if outputs.len() >= 2 {
        let fast = outputs[0].values.to_f64_vec();
        let slow = outputs[1].values.to_f64_vec();
        let n = fast.len();
        if n < 2 { return None; }
        let prev_f = fast[n - 2];
        let prev_s = slow[n - 2];
        let curr_f = fast[n - 1];
        let curr_s = slow[n - 1];

        match direction {
            "above" | "up" => {
                if prev_f <= prev_s && curr_f > curr_s {
                    Some((curr_f - curr_s).abs() / curr_s.max(1e-6))
                } else { None }
            }
            "below" | "down" => {
                if prev_f >= prev_s && curr_f < curr_s {
                    Some((curr_s - curr_f).abs() / curr_s.max(1e-6))
                } else { None }
            }
            _ => None,
        }
    } else {
        // Single line: check if it crosses a threshold
        let vals = outputs[0].values.to_f64_vec();
        let threshold = expr.value.unwrap_or(0.0);
        let n = vals.len();
        if n < 2 { return None; }
        let prev = vals[n - 2];
        let curr = vals[n - 1];

        match direction {
            "above" | "up" => {
                if prev <= threshold && curr > threshold { Some(1.0) } else { None }
            }
            "below" | "down" => {
                if prev >= threshold && curr < threshold { Some(1.0) } else { None }
            }
            _ => None,
        }
    }
}

fn extract_signals(expr: &ScanExpr, df: &DataFrame) -> Vec<String> {
    let mut signals = Vec::new();
    match expr.op.as_str() {
        "AND" | "and" | "OR" | "or" => {
            for child in &expr.children {
                signals.extend(extract_signals(child, df));
            }
        }
        "COMPARE" | "compare" => {
            if let Some(ind) = &expr.indicator {
                if let Some(op) = &expr.compare_op {
                    if let Some(v) = expr.value {
                        signals.push(format!("{}({}) {} {}",
                            ind, params_str(&expr.params), op, v));
                    }
                }
            }
        }
        "CROSS" | "cross" => {
            if let Some(ind) = &expr.indicator {
                signals.push(format!("CROSS({}({}), {})",
                    ind, params_str(&expr.params),
                    expr.compare_op.as_deref().unwrap_or("above")));
            }
        }
        _ => {}
    }
    signals
}

fn params_str(params: &Option<HashMap<String, f64>>) -> String {
    match params {
        Some(p) if !p.is_empty() => {
            p.iter().map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>().join(",")
        }
        _ => String::new(),
    }
}

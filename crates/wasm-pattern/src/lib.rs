use wasm_core::{DataFrame, IndicatorOutput, OutputStyle, Column};

pub mod single_line;
pub mod double_line;
pub mod triple_line;
pub mod chart_patterns;

/// Recognized pattern with location and confidence
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub name: String,
    pub name_cn: String,
    pub start_idx: usize,
    pub end_idx: usize,
    pub confidence: f64,
    pub direction: String, // "bullish", "bearish", "neutral"
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct PatternMeta {
    pub name: String,
    pub name_cn: String,
    pub category: String,
}

// ── Helpers (re-exported for module use) ──

pub fn body(open: f64, close: f64) -> f64 { (close - open).abs() }
pub fn upper_shadow(h: f64, o: f64, c: f64) -> f64 { h - o.max(c) }
pub fn lower_shadow(o: f64, c: f64, l: f64) -> f64 { o.min(c) - l }
pub fn is_bullish(o: f64, c: f64) -> bool { c > o }
pub fn is_bearish(o: f64, c: f64) -> bool { c < o }

pub fn avg_body(o: &[f64], c: &[f64], n: usize) -> f64 {
    let start = if n > 20 { n - 20 } else { 0 };
    let count = n - start;
    if count == 0 { return 0.01; }
    (start..n).map(|i| body(o[i], c[i])).sum::<f64>() / count as f64
}

pub fn avg_range(h: &[f64], l: &[f64], n: usize) -> f64 {
    let start = if n > 20 { n - 20 } else { 0 };
    let count = n - start;
    if count == 0 { return 0.01; }
    (start..n).map(|i| h[i] - l[i]).sum::<f64>() / count as f64
}

// ── Aggregated API ──

/// List all available patterns across all categories
pub fn list_patterns() -> Vec<PatternMeta> {
    let mut all = Vec::new();
    all.extend(single_line::list_single());
    all.extend(double_line::list_double());
    all.extend(triple_line::list_triple());
    all.extend(chart_patterns::list_chart());
    all
}

/// Recognize a specific pattern across the DataFrame
pub fn recognize(df: &DataFrame, pattern: &str) -> Vec<PatternMatch> {
    // Try each module; patterns are unique across modules
    let r = single_line::recognize_single(df, pattern);
    if !r.is_empty() { return r; }
    let r = double_line::recognize_double(df, pattern);
    if !r.is_empty() { return r; }
    let r = triple_line::recognize_triple(df, pattern);
    if !r.is_empty() { return r; }
    let r = chart_patterns::recognize_chart(df, pattern);
    if !r.is_empty() { return r; }
    vec![]
}

/// Scan all patterns and return matches sorted by confidence
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

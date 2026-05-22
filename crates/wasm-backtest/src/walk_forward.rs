use std::collections::HashMap;
use wasm_core::{DataFrame, BtResult};


/// Walk-Forward CV configuration
#[derive(Debug, Clone)]
pub struct WalkForwardConfig {
    pub in_sample_size: usize,    // training window (e.g., 252 = 1 year)
    pub out_sample_size: usize,   // test window (e.g., 63 = 1 quarter)
    pub anchor_mode: AnchorMode,
    pub step_size: usize,         // how many bars to advance each iteration
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnchorMode {
    Rolling,  // window slides forward
    Anchored, // start date fixed, window expands
}

impl Default for WalkForwardConfig {
    fn default() -> Self {
        WalkForwardConfig {
            in_sample_size: 252,
            out_sample_size: 63,
            anchor_mode: AnchorMode::Rolling,
            step_size: 63,
        }
    }
}

/// Result from a single Walk-Forward window
#[derive(Debug, Clone)]
pub struct WfWindowResult {
    pub window_idx: usize,
    pub is_start: usize,
    pub is_end: usize,
    pub oos_start: usize,
    pub oos_end: usize,
    pub best_params: HashMap<String, f64>,
    pub is_metrics: BtResult,
    pub oos_metrics: BtResult,
    pub is_annual_return: f64,
    pub oos_annual_return: f64,
    pub is_sharpe: f64,
    pub oos_sharpe: f64,
}

/// Complete Walk-Forward analysis result
#[derive(Debug, Clone)]
pub struct WalkForwardResult {
    pub windows: Vec<WfWindowResult>,
    pub avg_oos_return: f64,
    pub avg_oos_sharpe: f64,
    pub avg_is_return: f64,
    pub avg_is_sharpe: f64,
    /// Parameter stability: lower = more stable
    pub param_stability_score: f64,
    /// IS/OOS correlation (higher = more robust)
    pub is_oos_correlation: f64,
    /// Overall assessment
    pub assessment: String,
}

/// Run Walk-Forward Cross Validation for a given strategy
///
/// For each window:
/// 1. Optimize parameters on in-sample data
/// 2. Test optimized parameters on out-of-sample data
/// 3. Slide forward and repeat
pub fn walk_forward_analysis(
    df: &DataFrame,
    strategy: &str,
    param_grid: &HashMap<String, Vec<f64>>,
    config: &WalkForwardConfig,
    bt_config: &crate::BacktestConfig,
) -> WalkForwardResult {
    let n = df.len();
    let min_data = config.in_sample_size + config.out_sample_size;
    if n < min_data {
        return WalkForwardResult {
            windows: vec![],
            avg_oos_return: 0.0,
            avg_oos_sharpe: 0.0,
            avg_is_return: 0.0,
            avg_is_sharpe: 0.0,
            param_stability_score: 1.0,
            is_oos_correlation: 0.0,
            assessment: format!("数据不足：需要至少{}根K线，当前仅{}根", min_data, n),
        };
    }

    let mut windows = Vec::new();
    let mut current_start = 0;
    let mut param_history: Vec<HashMap<String, f64>> = Vec::new();
    let mut in_sample_size = config.in_sample_size;

    while current_start + in_sample_size + config.out_sample_size <= n {
        let is_start = current_start;
        let is_end = is_start + in_sample_size;
        let oos_start = is_end;
        let oos_end = (oos_start + config.out_sample_size).min(n);

        // In-sample optimization
        let is_slice = df.slice(is_start, is_end);
        let best_params = if param_grid.is_empty() {
            HashMap::new()
        } else {
            grid_search(&is_slice, strategy, param_grid, bt_config)
        };

        // In-sample performance
        let is_metrics = crate::run_with_template(&is_slice, strategy, &best_params, bt_config);

        // Out-of-sample performance
        let oos_slice = df.slice(oos_start, oos_end);
        let oos_metrics = crate::run_with_template(&oos_slice, strategy, &best_params, bt_config);

        param_history.push(best_params.clone());

        windows.push(WfWindowResult {
            window_idx: windows.len(),
            is_start, is_end, oos_start, oos_end,
            is_annual_return: is_metrics.annual_return,
            oos_annual_return: oos_metrics.annual_return,
            is_sharpe: is_metrics.sharpe_ratio,
            oos_sharpe: oos_metrics.sharpe_ratio,
            best_params,
            is_metrics,
            oos_metrics,
        });

        match config.anchor_mode {
            AnchorMode::Rolling => current_start += config.step_size,
            AnchorMode::Anchored => {
                // Expanding window: in_sample grows
                in_sample_size += config.step_size;
                if current_start + in_sample_size + config.out_sample_size > n {
                    break;
                }
            }
        }
    }

    // ── Aggregate metrics ──
    let avg_oos_return = windows.iter().map(|w| w.oos_annual_return).sum::<f64>()
        / windows.len().max(1) as f64;
    let avg_oos_sharpe = windows.iter().map(|w| w.oos_sharpe).sum::<f64>()
        / windows.len().max(1) as f64;
    let avg_is_return = windows.iter().map(|w| w.is_annual_return).sum::<f64>()
        / windows.len().max(1) as f64;
    let avg_is_sharpe = windows.iter().map(|w| w.is_sharpe).sum::<f64>()
        / windows.len().max(1) as f64;

    // IS/OOS correlation
    let is_returns: Vec<f64> = windows.iter().map(|w| w.is_annual_return).collect();
    let oos_returns: Vec<f64> = windows.iter().map(|w| w.oos_annual_return).collect();
    let is_oos_correlation = pearson_r(&is_returns, &oos_returns);

    // Parameter stability
    let param_stability = compute_param_stability(&param_history);

    // Assessment
    let assessment = if avg_oos_sharpe > 1.0 && param_stability < 0.3 {
        format!("优秀：策略稳健，样本外Sharpe={:.2}，参数稳定。适合实盘。", avg_oos_sharpe)
    } else if avg_oos_sharpe > 0.5 && param_stability < 0.5 {
        format!("良好：策略有效，样本外Sharpe={:.2}。建议监控参数漂移。", avg_oos_sharpe)
    } else if avg_oos_sharpe > 0.0 {
        format!("一般：样本外Sharpe={:.2}，参数不稳定。需重新评估因子逻辑。", avg_oos_sharpe)
    } else {
        format!("警告：样本外收益为负({:.2}%)。策略可能过拟合或失效。", avg_oos_return * 100.0)
    };

    WalkForwardResult {
        windows,
        avg_oos_return, avg_oos_sharpe,
        avg_is_return, avg_is_sharpe,
        param_stability_score: param_stability,
        is_oos_correlation,
        assessment,
    }
}

/// Grid search over parameter space
fn grid_search(
    df: &DataFrame,
    strategy: &str,
    param_grid: &HashMap<String, Vec<f64>>,
    bt_config: &crate::BacktestConfig,
) -> HashMap<String, f64> {
    let param_names: Vec<&String> = param_grid.keys().collect();
    if param_names.is_empty() {
        return HashMap::new();
    }

    let mut best_params = HashMap::new();
    let mut best_sharpe = f64::NEG_INFINITY;

    // Generate all combinations
    let combinations = generate_combinations(param_grid);
    for combo in &combinations {
        let result = crate::run_with_template(df, strategy, combo, bt_config);
        let score = result.sharpe_ratio;
        if score > best_sharpe {
            best_sharpe = score;
            best_params = combo.clone();
        }
    }

    best_params
}

fn generate_combinations(grid: &HashMap<String, Vec<f64>>) -> Vec<HashMap<String, f64>> {
    let keys: Vec<&String> = grid.keys().collect();
    if keys.is_empty() {
        return vec![HashMap::new()];
    }

    let mut results = vec![HashMap::new()];
    for key in &keys {
        let values = grid.get(*key).cloned().unwrap_or_default();
        let mut new_results = Vec::new();
        for existing in &results {
            for &val in &values {
                let mut combo = existing.clone();
                combo.insert((*key).clone(), val);
                new_results.push(combo);
            }
        }
        results = new_results;
    }
    results
}

/// Compute parameter stability across windows (coefficient of variation of each param)
fn compute_param_stability(param_history: &[HashMap<String, f64>]) -> f64 {
    if param_history.len() < 2 {
        return 0.0;
    }

    let all_keys: Vec<&String> = param_history[0].keys().collect();
    if all_keys.is_empty() {
        return 0.0;
    }

    let mut total_cv = 0.0;
    for key in &all_keys {
        let values: Vec<f64> = param_history.iter()
            .filter_map(|h| h.get(*key).copied())
            .collect();
        if values.len() >= 2 {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let var = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let std = var.sqrt();
            let cv = if mean.abs() > 0.0001 { std / mean.abs() } else { std };
            total_cv += cv;
        }
    }
    total_cv / all_keys.len().max(1) as f64
}

fn pearson_r(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 { return 0.0; }
    let mx = x.iter().sum::<f64>() / n as f64;
    let my = y.iter().sum::<f64>() / n as f64;
    let mut cov = 0.0;
    let mut sx2 = 0.0;
    let mut sy2 = 0.0;
    for i in 0..n {
        let dx = x[i] - mx;
        let dy = y[i] - my;
        cov += dx * dy;
        sx2 += dx * dx;
        sy2 += dy * dy;
    }
    let denom = (sx2 * sy2).sqrt();
    if denom < 1e-10 { 0.0 } else { cov / denom }
}

/// Compute Walk-Forward Efficiency (WFE) = OOS return / IS return
pub fn walk_forward_efficiency(wf_result: &WalkForwardResult) -> f64 {
    if wf_result.avg_is_return.abs() < 0.0001 {
        return 0.0;
    }
    wf_result.avg_oos_return / wf_result.avg_is_return
}

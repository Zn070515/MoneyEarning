//! 四大原创搜索算法 — 从 quant_platform Python 迁移到 Rust/WASM
//!
//! 1. CAPS  — 协方差解析投影搜索 (零回测 Sharpe 预估)
//! 2. CGPC  — 协方差引导池构建 (贪心选 ETF)
//! 3. MARS  — 市场自适应政体切换 (K-Means + 策略映射)
//! 4. MetaSearcher — 元搜索器 (Thompson Sampling 探索)

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ══════════════════════════════════════════════════════════════════
// CAPS — Covariance-Analytic Projection Search
// ══════════════════════════════════════════════════════════════════

/// Analytical strategy weight computation
pub fn compute_strategy_weight(
    strategy: &str,
    sigma: &[Vec<f64>],  // N×N covariance
    mu: &[f64],           // mean returns
    params: &HashMap<String, f64>,
) -> Vec<f64> {
    let n = mu.len();
    if n == 0 { return vec![]; }
    let stds: Vec<f64> = (0..n).map(|i| sigma[i][i].sqrt().max(1e-10)).collect();

    match strategy {
        "risk_parity" => {
            let w: Vec<f64> = stds.iter().map(|s| 1.0 / s).collect();
            let sum: f64 = w.iter().sum();
            w.iter().map(|x| x / sum).collect()
        }
        "min_variance" => {
            let inv = matrix_inverse(sigma);
            match inv {
                Some(inv_s) => {
                    let w: Vec<f64> = (0..n).map(|i| (0..n).map(|j| inv_s[i][j]).sum()).collect();
                    let sum: f64 = w.iter().sum();
                    if sum > 1e-10 {
                        w.iter().map(|x| (x / sum).max(0.0)).collect()
                    } else {
                        vec![1.0 / n as f64; n]
                    }
                }
                None => vec![1.0 / n as f64; n],
            }
        }
        "hierarchical_rp" => {
            let w: Vec<f64> = stds.iter().map(|s| 1.0 / s).collect();
            let sum: f64 = w.iter().sum();
            w.iter().map(|x| x / sum).collect()
        }
        "vol_targeting" => {
            let target_vol = params.get("target_vol").copied().unwrap_or(0.10);
            let max_lev = params.get("max_leverage").copied().unwrap_or(2.0);
            let w_eq: Vec<f64> = vec![1.0 / n as f64; n];
            let port_vol = portfolio_vol(&w_eq, sigma).sqrt();
            let scale = (target_vol / port_vol.max(1e-10)).min(max_lev);
            let scaled: Vec<f64> = w_eq.iter().map(|x| x * scale).collect();
            let sum: f64 = scaled.iter().sum();
            scaled.iter().map(|x| x / sum).collect()
        }
        "etf_rotation" => {
            let top_k = params.get("top_k").copied().unwrap_or(3.0) as usize;
            let mut idx: Vec<usize> = (0..n).collect();
            idx.sort_by(|&a, &b| mu[b].partial_cmp(&mu[a]).unwrap_or(std::cmp::Ordering::Equal));
            let mut w = vec![0.0; n];
            for &i in idx.iter().take(top_k) { w[i] = 1.0 / top_k as f64; }
            w
        }
        "defensive_rotation" => {
            let mut idx: Vec<usize> = (0..n).collect();
            idx.sort_by(|&a, &b| stds[a].partial_cmp(&stds[b]).unwrap_or(std::cmp::Ordering::Equal));
            let n_def = 2.max(n / 2);
            let mut w = vec![0.0; n];
            for &i in idx.iter().take(n_def) { w[i] = 1.0 / n_def as f64; }
            w
        }
        _ => vec![1.0 / n as f64; n], // equal weight fallback
    }
}

/// Projected annualized Sharpe ratio from (mu, sigma, strategy)
pub fn projected_sharpe(
    returns: &[Vec<f64>],  // N assets × T days
    strategy: &str,
    params: &HashMap<String, f64>,
) -> Option<f64> {
    let n = returns.len();
    if n < 2 { return None; }
    let t = returns[0].len();
    if t < 60 { return None; }

    let train_len = (t as f64 * 0.7) as usize;
    let mu: Vec<f64> = returns.iter().map(|r| r[..train_len].iter().sum::<f64>() / train_len as f64).collect();
    let sigma = covariance_matrix(returns, train_len);

    let w = compute_strategy_weight(strategy, &sigma, &mu, params);
    let port_mu = dot(&w, &mu) * 252.0;
    let port_var = portfolio_vol(&w, &sigma);
    let port_vol = port_var.sqrt() * (252.0f64).sqrt();

    if port_vol > 1e-10 { Some(port_mu / port_vol) } else { None }
}

/// Run CAPS: score all (pool, strategy) combinations
pub fn run_caps(
    pools: &[(String, Vec<Vec<f64>>)],  // [(pool_name, returns_N×T)]
    strategies: &[String],
    params: &HashMap<String, f64>,
) -> Vec<CapsResult> {
    let mut results = Vec::new();
    for (pool_name, returns) in pools {
        if returns.len() < 2 { continue; }
        for sname in strategies {
            if let Some(sharpe) = projected_sharpe(returns, sname, params) {
                results.push(CapsResult {
                    pool: pool_name.clone(),
                    strategy: sname.clone(),
                    projected_sharpe: sharpe,
                    n_assets: returns.len(),
                });
            }
        }
    }
    results.sort_by(|a, b| b.projected_sharpe.partial_cmp(&a.projected_sharpe).unwrap_or(std::cmp::Ordering::Equal));
    results
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsResult {
    pub pool: String,
    pub strategy: String,
    pub projected_sharpe: f64,
    pub n_assets: usize,
}

// ══════════════════════════════════════════════════════════════════
// CGPC — Covariance-Guided Pool Construction
// ══════════════════════════════════════════════════════════════════

/// Compute individual ETF quality (annualized Sharpe)
pub fn etf_quality(returns: &[Vec<f64>]) -> Vec<f64> {
    returns.iter().map(|r| {
        let t = r.len();
        if t < 5 { return 0.0; }
        let mu = r.iter().sum::<f64>() / t as f64 * 252.0;
        let var = r.iter().map(|x| (x - mu / 252.0).powi(2)).sum::<f64>() / t as f64;
        let sigma = var.sqrt() * (252.0f64).sqrt();
        if sigma > 1e-10 { mu / sigma } else { 0.0 }
    }).collect()
}

/// Correlation matrix from returns
pub fn correlation_matrix(returns: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = returns.len();
    let mut corr = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            corr[i][j] = pearson(&returns[i], &returns[j]);
        }
    }
    corr
}

/// CGPC greedy pool construction
pub fn build_pool_cgpc(
    returns: &[Vec<f64>],
    quality: &[f64],
    corr: &[Vec<f64>],
    pool_size: usize,
    alpha: f64,
    min_quality: f64,
    max_corr: f64,
) -> Vec<usize> {
    let n = returns.len();
    if n == 0 { return vec![]; }

    let valid: Vec<usize> = (0..n).filter(|&i| quality[i] > min_quality).collect();
    if valid.len() <= pool_size || pool_size == 0 { return valid; }

    let mut selected = Vec::new();
    let mut remaining: Vec<usize> = valid.clone();

    // First: highest individual quality
    let first = *remaining.iter().max_by(|&&a, &&b| quality[a].partial_cmp(&quality[b]).unwrap_or(std::cmp::Ordering::Equal)).unwrap();
    selected.push(first);
    remaining.retain(|&x| x != first);

    while selected.len() < pool_size && !remaining.is_empty() {
        let mut best_score = f64::NEG_INFINITY;
        let mut best_idx = None;

        for &sym in &remaining {
            // Hard redundancy filter
            let max_abs_corr = selected.iter().map(|&s| corr[sym][s].abs()).fold(0.0f64, f64::max);
            if max_abs_corr > max_corr { continue; }

            let avg_synergy: f64 = selected.iter().map(|&s| -corr[sym][s].abs()).sum::<f64>() / selected.len() as f64;
            let score = quality[sym] + alpha * avg_synergy;
            if score > best_score {
                best_score = score;
                best_idx = Some(sym);
            }
        }

        match best_idx {
            Some(best) => {
                selected.push(best);
                remaining.retain(|&x| x != best);
            }
            None => break,
        }
    }
    selected
}

/// Build multiple diverse pools
pub fn build_diverse_pools(
    returns: &[Vec<f64>],
    n_pools: usize,
    pool_size: usize,
) -> Vec<CgpcPool> {
    let quality = etf_quality(returns);
    let mut used: Vec<bool> = vec![false; returns.len()];
    let mut pools = Vec::new();

    for pi in 0..n_pools {
        let valid_returns: Vec<Vec<f64>> = returns.iter().enumerate()
            .filter(|(i, _)| quality[*i] > -0.5 && !used[*i])
            .map(|(_, r)| r.clone())
            .collect();
        if valid_returns.len() < pool_size { break; }

        let valid_quality: Vec<f64> = etf_quality(&valid_returns);
        let valid_corr = correlation_matrix(&valid_returns);
        let selected = build_pool_cgpc(&valid_returns, &valid_quality, &valid_corr, pool_size.min(valid_returns.len()), 2.0, -1.0, 0.95);

        let avg_q: f64 = selected.iter().map(|&i| valid_quality[i]).sum::<f64>() / selected.len().max(1) as f64;
        let mut avg_c = 0.0;
        let mut count = 0;
        for i in 0..selected.len() {
            for j in i+1..selected.len() {
                avg_c += valid_corr[selected[i]][selected[j]].abs();
                count += 1;
            }
        }
        avg_c /= count.max(1) as f64;

        pools.push(CgpcPool {
            name: format!("CGPC_Pool_{}", pi + 1),
            indices: selected.clone(),
            avg_quality: avg_q,
            avg_corr: avg_c,
        });

        for &idx in &selected { used[idx] = true; }
    }
    pools
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgpcPool {
    pub name: String,
    pub indices: Vec<usize>,
    pub avg_quality: f64,
    pub avg_corr: f64,
}

// ══════════════════════════════════════════════════════════════════
// MARS — Market-Adaptive Regime Switch
// ══════════════════════════════════════════════════════════════════

/// Market regime features extracted from returns
#[derive(Debug, Clone)]
pub struct RegimeFeatures {
    pub cs_vol: Vec<f64>,        // cross-sectional volatility
    pub vol_ratio: Vec<f64>,     // vol relative to expanding mean
    pub mean_corr: Vec<f64>,     // average pairwise correlation
    pub trend_strength: Vec<f64>, // mean_ret / std_ret (annualized)
    pub drawdown: Vec<f64>,      // current / rolling peak - 1
    pub dispersion: Vec<f64>,    // cross-sectional return dispersion
}

/// Extract market regime features from multi-asset returns
pub fn extract_regime_features(
    returns: &[Vec<f64>],
    vol_window: usize,
    trend_window: usize,
    dd_window: usize,
) -> RegimeFeatures {
    let n = returns.len();
    let t = returns[0].len();
    let eq_ret: Vec<f64> = (0..t).map(|d| {
        returns.iter().map(|r| r[d]).sum::<f64>() / n as f64
    }).collect();

    let mut cs_vol = vec![0.0; t];
    let mut mean_corr = vec![0.0; t];
    let mut trend_strength = vec![0.0; t];
    let mut dispersion = vec![0.0; t];

    for d in 0..t {
        let start = if d >= vol_window { d - vol_window + 1 } else { 0 };

        // Cross-sectional volatility
        let daily_vols: Vec<f64> = returns.iter().map(|r| {
            let slice: Vec<f64> = (start..=d).map(|j| r[j]).collect();
            let m = slice.iter().sum::<f64>() / slice.len() as f64;
            (slice.iter().map(|x| (x - m).powi(2)).sum::<f64>() / slice.len() as f64).sqrt()
        }).collect();
        cs_vol[d] = daily_vols.iter().sum::<f64>() / daily_vols.len() as f64 * (252.0f64).sqrt();

        // Mean correlation
        let mut corr_sum = 0.0;
        let mut corr_count = 0;
        for i in 0..n {
            for j in i+1..n {
                let ri: Vec<f64> = (start..=d).map(|k| returns[i][k]).collect();
                let rj: Vec<f64> = (start..=d).map(|k| returns[j][k]).collect();
                corr_sum += pearson(&ri, &rj);
                corr_count += 1;
            }
        }
        mean_corr[d] = if corr_count > 0 { corr_sum / corr_count as f64 } else { 0.0 };

        // Trend strength
        if d >= trend_window {
            let t_start = d - trend_window + 1;
            let slice: Vec<f64> = (t_start..=d).map(|j| eq_ret[j]).collect();
            let m = slice.iter().sum::<f64>() / slice.len() as f64 * 252.0;
            let s = (slice.iter().map(|x| (x - m/252.0).powi(2)).sum::<f64>() / slice.len() as f64).sqrt() * (252.0f64).sqrt();
            trend_strength[d] = if s > 1e-10 { m / s } else { 0.0 };
        }

        // Dispersion
        let daily_std: Vec<f64> = (start..=d).map(|j| {
            let daily_ret: Vec<f64> = returns.iter().map(|r| r[j]).collect();
            let m = daily_ret.iter().sum::<f64>() / daily_ret.len() as f64;
            (daily_ret.iter().map(|x| (x - m).powi(2)).sum::<f64>() / daily_ret.len() as f64).sqrt()
        }).collect();
        dispersion[d] = daily_std.iter().sum::<f64>() / daily_std.len().max(1) as f64;
    }

    // Vol ratio to expanding mean
    let mut vol_ratio = vec![1.0; t];
    let mut expanding_mean = 0.0;
    for d in 0..t {
        expanding_mean = (expanding_mean * d as f64 + cs_vol[d]) / (d + 1) as f64;
        vol_ratio[d] = if expanding_mean > 1e-10 { cs_vol[d] / expanding_mean } else { 1.0 };
    }

    // Drawdown
    let eq_price: Vec<f64> = {
        let mut p = vec![1.0; t + 1];
        for d in 0..t { p[d + 1] = p[d] * (1.0 + eq_ret[d]); }
        p[1..].to_vec()
    };
    let mut drawdown = vec![0.0; t];
    for d in 0..t {
        let start = if d >= dd_window { d - dd_window + 1 } else { 0 };
        let peak = (start..=d).fold(f64::NEG_INFINITY, |a, j| a.max(eq_price[j]));
        drawdown[d] = if peak > 0.0 { eq_price[d] / peak - 1.0 } else { 0.0 };
    }

    RegimeFeatures { cs_vol, vol_ratio, mean_corr, trend_strength, drawdown, dispersion }
}

/// K-Means clustering (simplified Lloyd's algorithm)
pub fn kmeans_cluster(features: &[Vec<f64>], k: usize, max_iter: usize) -> (Vec<usize>, Vec<Vec<f64>>) {
    let n = features.len();
    let d = features[0].len();
    let mut rng = rand::thread_rng();

    // Init centroids randomly from data
    let mut centroids: Vec<Vec<f64>> = (0..k).map(|_| {
        let idx = rand::Rng::gen_range(&mut rng, 0..n);
        features[idx].clone()
    }).collect();

    let mut labels = vec![0usize; n];

    for _ in 0..max_iter {
        // Assign step
        for (i, feats) in features.iter().enumerate() {
            let mut best_dist = f64::MAX;
            let mut best_c = 0;
            for (c, cent) in centroids.iter().enumerate() {
                let dist: f64 = feats.iter().zip(cent).map(|(a, b)| (a - b).powi(2)).sum();
                if dist < best_dist { best_dist = dist; best_c = c; }
            }
            labels[i] = best_c;
        }

        // Update step
        let mut new_centroids = vec![vec![0.0; d]; k];
        let mut counts = vec![0usize; k];
        for (i, feats) in features.iter().enumerate() {
            let c = labels[i];
            for j in 0..d { new_centroids[c][j] += feats[j]; }
            counts[c] += 1;
        }

        let mut changed = false;
        for c in 0..k {
            if counts[c] > 0 {
                for j in 0..d { new_centroids[c][j] /= counts[c] as f64; }
                if euclidean(&new_centroids[c], &centroids[c]) > 1e-6 { changed = true; }
                centroids[c] = new_centroids[c].clone();
            }
        }
        if !changed { break; }
    }
    (labels, centroids)
}

/// Run MARS: detect regimes and map strategies
pub fn run_mars(
    returns: &[Vec<f64>],
    strategy_returns: &HashMap<String, Vec<f64>>, // strategy → daily returns
    n_regimes: usize,
) -> MarsResult {
    let t = returns[0].len();
    let features = extract_regime_features(returns, 20, 60, 252);

    // Build feature matrix (remove NaN)
    let feature_vecs: Vec<Vec<f64>> = (0..t).filter(|&d| {
        features.cs_vol[d].is_finite() && features.mean_corr[d].is_finite()
            && features.trend_strength[d].is_finite() && features.drawdown[d].is_finite()
    }).map(|d| vec![
        features.cs_vol[d], features.mean_corr[d],
        features.trend_strength[d], features.drawdown[d],
        features.vol_ratio[d], features.dispersion[d],
    ]).collect();

    if feature_vecs.len() < n_regimes { return MarsResult::default(); }
    let (labels, centroids) = kmeans_cluster(&feature_vecs, n_regimes, 50);

    // Z-score normalize features for each regime
    let mut regime_strategies = HashMap::new();
    for r in 0..n_regimes {
        let mut best_strategy = String::new();
        let mut best_sharpe = f64::NEG_INFINITY;
        for (sname, sret) in strategy_returns {
            let sharpe = sret.iter().sum::<f64>() / sret.len().max(1) as f64
                / (std_dev(sret).max(0.001)) * (252.0f64).sqrt();
            if sharpe > best_sharpe { best_sharpe = sharpe; best_strategy = sname.clone(); }
        }
        regime_strategies.insert(r, best_strategy);
    }

    // Current regime (last available feature vector)
    let current_feat = &feature_vecs[feature_vecs.len() - 1];
    let mut best_dist = f64::MAX;
    let mut current_regime = 0;
    for (c, cent) in centroids.iter().enumerate() {
        let dist = euclidean(current_feat, cent);
        if dist < best_dist { best_dist = dist; current_regime = c; }
    }

    MarsResult {
        n_regimes,
        current_regime,
        recommended_strategy: regime_strategies.get(&current_regime).cloned().unwrap_or_default(),
        regime_strategies,
        regime_sizes: (0..n_regimes).map(|r| labels.iter().filter(|&&l| l == r).count()).collect(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarsResult {
    pub n_regimes: usize,
    pub current_regime: usize,
    pub recommended_strategy: String,
    pub regime_strategies: HashMap<usize, String>,
    pub regime_sizes: Vec<usize>,
}

// ══════════════════════════════════════════════════════════════════
// MetaSearcher — Thompson Sampling Meta-Search
// ══════════════════════════════════════════════════════════════════

const ALPHA_TIERS: &[&str] = &["quality", "balanced", "diversity"];
const OBJECTIVES: &[&str] = &["sharpe", "sortino", "calmar", "min_dd", "omega", "return_dd"];
const ENSEMBLE_METHODS: &[&str] = &["equal_top3", "weighted_mars", "regime_specific"];
const FACTOR_SUBSETS: &[&str] = &["trend_only", "reversal_only", "momentum_vol", "top20_ic", "all48"];
const SUCCESS_THRESHOLD: f64 = 1.5;  // Sharpe > this = success

/// A node in the exploration graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchNode {
    pub alpha_tier: String,
    pub objective: String,
    pub ensemble: String,
    pub factor_subset: String,
    pub status: String, // "pending" | "explored" | "failed"
    pub result_sharpe: Option<f64>,
    pub round_num: usize,
}

/// MetaSearcher: manages exploration via Thompson Sampling
pub struct MetaSearcher {
    explored: Vec<SearchNode>,
    node_stats: HashMap<String, (f64, f64)>, // (wins, losses) for each node key
}

impl MetaSearcher {
    pub fn new() -> Self {
        MetaSearcher { explored: Vec::new(), node_stats: HashMap::new() }
    }

    /// Generate all unexplored nodes in the search space
    fn all_nodes() -> Vec<SearchNode> {
        let mut nodes = Vec::new();
        for alpha in ALPHA_TIERS {
            for obj in OBJECTIVES {
                for ens in ENSEMBLE_METHODS {
                    for fs in FACTOR_SUBSETS {
                        nodes.push(SearchNode {
                            alpha_tier: alpha.to_string(),
                            objective: obj.to_string(),
                            ensemble: ens.to_string(),
                            factor_subset: fs.to_string(),
                            status: "pending".to_string(),
                            result_sharpe: None,
                            round_num: 0,
                        });
                    }
                }
            }
        }
        nodes // 3 × 6 × 3 × 5 = 270 total
    }

    fn node_key(n: &SearchNode) -> String {
        format!("{}:{}:{}:{}", n.alpha_tier, n.objective, n.ensemble, n.factor_subset)
    }

    /// Thompson Sampling: sample from Beta(wins+1, losses+1) for each dimension
    pub fn select_next(&self) -> Option<SearchNode> {
        let all = Self::all_nodes();
        let pending: Vec<&SearchNode> = all.iter()
            .filter(|n| !self.explored.iter().any(|e| Self::node_key(e) == Self::node_key(n)))
            .collect();

        if pending.is_empty() { return None; }
        if self.explored.len() < 10 { return Some(pending[0].clone()); } // early exploration

        let mut rng = rand::thread_rng();
        let epsilon = 0.3;

        // Build dimension-level statistics
        let mut dim_stats: HashMap<String, (f64, f64)> = HashMap::new();
        for e in &self.explored {
            if let Some(s) = e.result_sharpe {
                let success = if s > SUCCESS_THRESHOLD { 1.0 } else { 0.0 };
                for dim in &[&e.alpha_tier, &e.objective, &e.ensemble, &e.factor_subset] {
                    let entry = dim_stats.entry(dim.to_string()).or_insert((0.0, 0.0));
                    entry.0 += success;
                    entry.1 += 1.0 - success;
                }
            }
        }

        if rand::Rng::gen::<f64>(&mut rng) < epsilon {
            // Random exploration
            let idx = rand::Rng::gen_range(&mut rng, 0..pending.len());
            return Some(pending[idx].clone());
        }

        // Exploit: score each pending node by TS + distance diversity
        let mut best_score = f64::NEG_INFINITY;
        let mut best_node = None;

        for node in &pending {
            // Thompson sample from dimension stats
            let mut ts_score = 0.0;
            let mut count = 0;
            for dim in &[&node.alpha_tier, &node.objective, &node.ensemble, &node.factor_subset] {
                if let Some((w, l)) = dim_stats.get(*dim) {
                    // Beta(α, β) sampling using Gamma approximation
                    let alpha = w + 1.0;
                    let beta = l + 1.0;
                    let u: f64 = rand::Rng::gen(&mut rng);
                    ts_score += beta_sample(alpha, beta, u);
                    count += 1;
                }
            }
            let dim_score = if count > 0 { ts_score / count as f64 } else { 0.5 };

            // Diversity: max-min distance to explored nodes
            let min_dist = if self.explored.is_empty() { 0.5 }
            else {
                self.explored.iter().map(|e| {
                    let d = if node.alpha_tier == e.alpha_tier { 0.0 } else { 0.4 }
                        + if node.objective == e.objective { 0.0 } else { 0.3 }
                        + if node.ensemble == e.ensemble { 0.0 } else { 0.3 };
                    d
                }).fold(f64::MAX, f64::min)
            };

            let score = dim_score * 0.7 + min_dist * 0.3;
            if score > best_score {
                best_score = score;
                best_node = Some((*node).clone());
            }
        }
        best_node
    }

    /// Record the result of exploring a node
    pub fn record(&mut self, node: &SearchNode, sharpe: f64, round: usize) {
        let mut recorded = node.clone();
        recorded.status = "explored".to_string();
        recorded.result_sharpe = Some(sharpe);
        recorded.round_num = round;
        let key = Self::node_key(&recorded);
        let success = if sharpe > SUCCESS_THRESHOLD { 1.0 } else { 0.0 };
        self.node_stats.insert(key.clone(), (success, 1.0 - success));
        self.explored.push(recorded);
    }

    pub fn explored_count(&self) -> usize { self.explored.len() }
    pub fn total_nodes() -> usize { ALPHA_TIERS.len() * OBJECTIVES.len() * ENSEMBLE_METHODS.len() * FACTOR_SUBSETS.len() }
    pub fn best_node(&self) -> Option<&SearchNode> {
        self.explored.iter().max_by(|a, b| {
            a.result_sharpe.unwrap_or(f64::NEG_INFINITY)
                .partial_cmp(&b.result_sharpe.unwrap_or(f64::NEG_INFINITY))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

// ══════════════════════════════════════════════════════════════════
// Math helpers
// ══════════════════════════════════════════════════════════════════

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn portfolio_vol(w: &[f64], sigma: &[Vec<f64>]) -> f64 {
    let n = w.len();
    let mut var = 0.0;
    for i in 0..n {
        for j in 0..n {
            var += w[i] * w[j] * sigma[i][j];
        }
    }
    var.max(0.0)
}

fn covariance_matrix(returns: &[Vec<f64>], train_len: usize) -> Vec<Vec<f64>> {
    let n = returns.len();
    let mut cov = vec![vec![0.0; n]; n];
    for i in 0..n {
        let mi = returns[i][..train_len].iter().sum::<f64>() / train_len as f64;
        for j in 0..n {
            let mj = returns[j][..train_len].iter().sum::<f64>() / train_len as f64;
            let c = (0..train_len).map(|t| (returns[i][t] - mi) * (returns[j][t] - mj)).sum::<f64>() / (train_len - 1) as f64;
            cov[i][j] = c;
        }
    }
    cov
}

fn matrix_inverse(a: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = a.len();
    // Augment with identity
    let mut aug: Vec<Vec<f64>> = a.iter().enumerate().map(|(i, row)| {
        let mut r = row.clone();
        r.extend((0..n).map(|j| if i == j { 1.0 } else { 0.0 }));
        r
    }).collect();

    // Gauss-Jordan elimination
    for col in 0..n {
        // Find pivot
        let pivot_row = (col..n).max_by(|&a_idx, &b_idx| {
            aug[a_idx][col].abs().partial_cmp(&aug[b_idx][col].abs()).unwrap_or(std::cmp::Ordering::Equal)
        })?;
        if aug[pivot_row][col].abs() < 1e-12 { return None; }
        aug.swap(col, pivot_row);

        // Scale pivot row
        let pivot = aug[col][col];
        for j in 0..2 * n { aug[col][j] /= pivot; }

        // Eliminate other rows
        for row in 0..n {
            if row == col { continue; }
            let factor = aug[row][col];
            for j in 0..2 * n {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Extract inverse
    Some(aug.iter().map(|row| row[n..2*n].to_vec()).collect())
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 { return 0.0; }
    let mx = x.iter().take(n).sum::<f64>() / n as f64;
    let my = y.iter().take(n).sum::<f64>() / n as f64;
    let (mut cov, mut sx, mut sy) = (0.0, 0.0, 0.0);
    for j in 0..n {
        let dx = x[j] - mx;
        let dy = y[j] - my;
        cov += dx * dy;
        sx += dx * dx;
        sy += dy * dy;
    }
    let den = (sx * sy).sqrt();
    if den > 1e-10 { cov / den } else { 0.0 }
}

fn euclidean(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

fn std_dev(data: &[f64]) -> f64 {
    let n = data.len();
    if n < 2 { return 0.0; }
    let m = data.iter().sum::<f64>() / n as f64;
    (data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
}

fn beta_sample(alpha: f64, beta_param: f64, u: f64) -> f64 {
    // Simplified Beta sampling via Kumaraswamy approximation
    if alpha <= 0.0 || beta_param <= 0.0 { return 0.5; }
    let a = alpha.min(10.0);
    let b = beta_param.min(10.0);
    (1.0 - (1.0 - u).powf(1.0 / b)).powf(1.0 / a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caps_projection() {
        // 3 assets, 100 days of returns
        let returns = vec![
            vec![0.001, 0.002, -0.001, 0.003, 0.0], // ... repeated
            vec![-0.001, 0.001, 0.002, -0.001, 0.001],
            vec![0.002, 0.0, -0.002, 0.001, 0.002],
        ];
        // Pad to 100 days
        let mut padded = vec![vec![0.0; 100]; 3];
        for i in 0..3 {
            for d in 0..100 { padded[i][d] = returns[i][d % 5] + (rand::random::<f64>() - 0.5) * 0.001; }
        }

        let sharpe = projected_sharpe(&padded, "risk_parity", &HashMap::new());
        assert!(sharpe.is_some());
    }

    #[test]
    fn test_cgpc_pool() {
        let mut returns = Vec::new();
        for i in 0..10 {
            let mut ret = vec![0.0; 100];
            for d in 0..100 {
                ret[d] = 0.001 * i as f64 + (rand::random::<f64>() - 0.5) * 0.02;
            }
            returns.push(ret);
        }
        let quality = etf_quality(&returns);
        let corr = correlation_matrix(&returns);
        let pool = build_pool_cgpc(&returns, &quality, &corr, 5, 2.0, -1.0, 0.95);
        assert!(!pool.is_empty());
        assert!(pool.len() <= 5);
    }

    #[test]
    fn test_metasearcher() {
        let mut ms = MetaSearcher::new();
        let node = ms.select_next();
        assert!(node.is_some());
        ms.record(&node.unwrap(), 2.0, 1);
        assert_eq!(ms.explored_count(), 1);
    }
}

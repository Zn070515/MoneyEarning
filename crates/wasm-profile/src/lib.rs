use wasm_core::DataFrame;

/// Portfolio return series from weighted assets
#[derive(Debug, Clone)]
pub struct Portfolio {
    pub weights: Vec<f64>,
    pub returns: Vec<Vec<f64>>, // each inner vec is one asset's return series
    pub portfolio_returns: Vec<f64>,
}

/// Risk metrics for a portfolio or single asset
#[derive(Debug, Clone)]
pub struct RiskMetrics {
    pub total_return: f64,
    pub annual_return: f64,
    pub annual_volatility: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub var_95: f64,
    pub cvar_95: f64,
    pub var_99: f64,
    pub cvar_99: f64,
    pub calmar_ratio: f64,
    pub positive_days: usize,
    pub negative_days: usize,
    pub best_day: f64,
    pub worst_day: f64,
}

impl Portfolio {
    /// Build portfolio from multiple DataFrames and weights
    pub fn new(dfs: &[DataFrame], weights: &[f64]) -> Self {
        let n = dfs.len();
        let mut returns: Vec<Vec<f64>> = Vec::with_capacity(n);

        for df in dfs {
            let close = df.column("close")
                .map(|c| c.to_f64_vec())
                .unwrap_or_default();
            let rets = daily_returns(&close);
            returns.push(rets);
        }

        // Align all series to same length
        let min_len = returns.iter().map(|r| r.len()).min().unwrap_or(0);
        for r in returns.iter_mut() {
            r.truncate(min_len);
        }

        let portfolio_returns = if min_len > 0 {
            (0..min_len)
                .map(|i| {
                    returns.iter().zip(weights.iter())
                        .map(|(r, w)| r.get(i).copied().unwrap_or(0.0) * w)
                        .sum()
                })
                .collect()
        } else {
            vec![]
        };

        Portfolio {
            weights: weights.to_vec(),
            returns,
            portfolio_returns,
        }
    }

    /// Build single-asset portfolio
    pub fn single(df: &DataFrame) -> Self {
        let close = df.column("close")
            .map(|c| c.to_f64_vec())
            .unwrap_or_default();
        let rets = daily_returns(&close);
        Portfolio {
            weights: vec![1.0],
            returns: vec![rets.clone()],
            portfolio_returns: rets,
        }
    }

    /// Calculate comprehensive risk metrics
    pub fn metrics(&self) -> RiskMetrics {
        risk_metrics(&self.portfolio_returns)
    }
}

/// Compute daily log returns from price series
pub fn daily_returns(prices: &[f64]) -> Vec<f64> {
    if prices.len() < 2 { return vec![]; }
    (1..prices.len())
        .map(|i| {
            if prices[i - 1] > 0.0 {
                (prices[i] / prices[i - 1]).ln()
            } else {
                0.0
            }
        })
        .collect()
}

/// Simple percent returns
pub fn pct_returns(prices: &[f64]) -> Vec<f64> {
    if prices.len() < 2 { return vec![]; }
    (1..prices.len())
        .map(|i| {
            if prices[i - 1] > 0.0 {
                (prices[i] - prices[i - 1]) / prices[i - 1]
            } else {
                0.0
            }
        })
        .collect()
}

/// Comprehensive risk metrics from a return series
pub fn risk_metrics(daily_rets: &[f64]) -> RiskMetrics {
    let n = daily_rets.len();
    if n == 0 {
        return RiskMetrics {
            total_return: 0.0, annual_return: 0.0, annual_volatility: 0.0,
            sharpe_ratio: 0.0, sortino_ratio: 0.0, max_drawdown: 0.0,
            var_95: 0.0, cvar_95: 0.0, var_99: 0.0, cvar_99: 0.0,
            calmar_ratio: 0.0, positive_days: 0, negative_days: 0,
            best_day: 0.0, worst_day: 0.0,
        };
    }

    let mean = daily_rets.iter().sum::<f64>() / n as f64;
    let variance = daily_rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n as f64;
    let vol = variance.sqrt();
    let annual_vol = vol * (252.0f64).sqrt();
    let annual_return = mean * 252.0;

    // Total return (compounded)
    let total_return = daily_rets.iter().fold(1.0, |acc, r| acc * (1.0 + r)) - 1.0;

    // Sharpe ratio (assuming 2% risk-free rate for China)
    let rf_daily = 0.02 / 252.0;
    let sharpe = if annual_vol > 0.0 {
        (annual_return - 0.02) / annual_vol
    } else { 0.0 };

    // Sortino ratio (downside deviation)
    let downside: Vec<f64> = daily_rets.iter().map(|r| (r - rf_daily).min(0.0)).collect();
    let downside_var = downside.iter().map(|d| d.powi(2)).sum::<f64>() / n as f64;
    let downside_dev = downside_var.sqrt();
    let sortino = if downside_dev > 0.0 {
        (annual_return - 0.02) / (downside_dev * (252.0f64).sqrt())
    } else { 0.0 };

    // Max drawdown
    let mdd = max_drawdown_from_rets(daily_rets);

    // VaR and CVaR (historical method)
    let mut sorted_rets = daily_rets.to_vec();
    sorted_rets.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let var95_idx = ((n as f64) * 0.05) as usize;
    let var99_idx = ((n as f64) * 0.01) as usize;
    let var_95 = sorted_rets[var95_idx.min(n - 1)];
    let var_99 = sorted_rets[var99_idx.min(n - 1)];

    let cvar_95 = if var95_idx > 0 {
        sorted_rets[..var95_idx].iter().sum::<f64>() / var95_idx as f64
    } else { var_95 };
    let cvar_99 = if var99_idx > 0 {
        sorted_rets[..var99_idx].iter().sum::<f64>() / var99_idx as f64
    } else { var_99 };

    // Calmar ratio
    let calmar = if mdd > 0.0 { annual_return / mdd } else { 0.0 };

    // Day counts
    let positive_days = daily_rets.iter().filter(|&&r| r > 0.0).count();
    let negative_days = daily_rets.iter().filter(|&&r| r < 0.0).count();
    let best_day = daily_rets.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let worst_day = daily_rets.iter().cloned().fold(f64::INFINITY, f64::min);

    RiskMetrics {
        total_return, annual_return, annual_volatility: annual_vol,
        sharpe_ratio: sharpe, sortino_ratio: sortino,
        max_drawdown: mdd, var_95, cvar_95, var_99, cvar_99,
        calmar_ratio: calmar, positive_days, negative_days,
        best_day, worst_day,
    }
}

/// Maximum drawdown from daily returns
pub fn max_drawdown_from_rets(rets: &[f64]) -> f64 {
    let mut peak = 0.0;
    let mut cum = 0.0;
    let mut mdd = 0.0;
    for &r in rets {
        cum += r;
        if cum > peak { peak = cum; }
        let dd = peak - cum;
        if dd > mdd { mdd = dd; }
    }
    mdd
}

/// Maximum drawdown from price series
pub fn max_drawdown(prices: &[f64]) -> f64 {
    if prices.len() < 2 { return 0.0; }
    let mut peak = prices[0];
    let mut mdd = 0.0;
    for &p in prices.iter().skip(1) {
        if p > peak { peak = p; }
        let dd = (peak - p) / peak;
        if dd > mdd { mdd = dd; }
    }
    mdd
}

/// Beta of an asset relative to a benchmark
pub fn beta(asset_rets: &[f64], bench_rets: &[f64]) -> f64 {
    let n = asset_rets.len().min(bench_rets.len());
    if n < 2 { return 1.0; }

    let asset_mean = asset_rets[..n].iter().sum::<f64>() / n as f64;
    let bench_mean = bench_rets[..n].iter().sum::<f64>() / n as f64;

    let cov = (0..n).map(|i| (asset_rets[i] - asset_mean) * (bench_rets[i] - bench_mean)).sum::<f64>() / n as f64;
    let bench_var = (0..n).map(|i| (bench_rets[i] - bench_mean).powi(2)).sum::<f64>() / n as f64;

    if bench_var > 0.0 { cov / bench_var } else { 1.0 }
}

/// Alpha (Jensen's alpha)
pub fn alpha(asset_rets: &[f64], bench_rets: &[f64], beta_val: f64) -> f64 {
    let n = asset_rets.len().min(bench_rets.len());
    if n < 2 { return 0.0; }
    let rf_daily = 0.02 / 252.0;
    let asset_mean = asset_rets[..n].iter().sum::<f64>() / n as f64;
    let bench_mean = bench_rets[..n].iter().sum::<f64>() / n as f64;
    (asset_mean - rf_daily) - beta_val * (bench_mean - rf_daily)
}

/// Correlation matrix between multiple assets
pub fn correlation_matrix(returns: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = returns.len();
    if n == 0 { return vec![]; }

    let len = returns.iter().map(|r| r.len()).min().unwrap_or(0);
    if len < 2 { return vec![vec![1.0; n]; n]; }

    let means: Vec<f64> = returns.iter()
        .map(|r| r[..len].iter().sum::<f64>() / len as f64)
        .collect();
    let stds: Vec<f64> = returns.iter()
        .map(|r| {
            let m = r[..len].iter().sum::<f64>() / len as f64;
            (r[..len].iter().map(|x| (x - m).powi(2)).sum::<f64>() / len as f64).sqrt()
        })
        .collect();

    (0..n).map(|i| {
        (0..n).map(|j| {
            if stds[i] < 1e-12 || stds[j] < 1e-12 { return if i == j { 1.0 } else { 0.0 }; }
            let cov = (0..len)
                .map(|k| (returns[i][k] - means[i]) * (returns[j][k] - means[j]))
                .sum::<f64>() / len as f64;
            cov / (stds[i] * stds[j])
        }).collect()
    }).collect()
}

/// Compute equal-weight portfolio returns from multiple return series
pub fn equal_weight_portfolio(returns: &[Vec<f64>]) -> Vec<f64> {
    let n = returns.len();
    if n == 0 { return vec![]; }
    let w = 1.0 / n as f64;
    let len = returns.iter().map(|r| r.len()).min().unwrap_or(0);

    (0..len).map(|i| {
        returns.iter().map(|r| r.get(i).copied().unwrap_or(0.0) * w).sum()
    }).collect()
}

/// Minimum variance portfolio weights (simplified 2-asset)
pub fn min_variance_weights(rets1: &[f64], rets2: &[f64]) -> (f64, f64) {
    let n = rets1.len().min(rets2.len());
    if n < 2 { return (0.5, 0.5); }

    let var1 = variance(&rets1[..n]);
    let var2 = variance(&rets2[..n]);
    let cov = covariance(&rets1[..n], &rets2[..n]);

    let w1 = (var2 - cov) / (var1 + var2 - 2.0 * cov);
    let w1 = w1.clamp(0.0, 1.0);
    (w1, 1.0 - w1)
}

/// Efficient frontier points (for 2 assets)
pub fn efficient_frontier_2asset(rets1: &[f64], rets2: &[f64], points: usize) -> Vec<(f64, f64, f64)> {
    let n = rets1.len().min(rets2.len());
    if n < 2 { return vec![]; }

    let m1 = mean(&rets1[..n]);
    let m2 = mean(&rets2[..n]);
    let v1 = variance(&rets1[..n]);
    let v2 = variance(&rets2[..n]);
    let cov = covariance(&rets1[..n], &rets2[..n]);

    (0..=points).map(|i| {
        let w1 = i as f64 / points as f64;
        let w2 = 1.0 - w1;
        let ret = w1 * m1 + w2 * m2;
        let risk = (w1.powi(2) * v1 + w2.powi(2) * v2 + 2.0 * w1 * w2 * cov).sqrt();
        (risk, ret, w1)
    }).collect()
}

/// Rolling metrics: compute risk metrics over a rolling window
pub fn rolling_metrics(df: &DataFrame, window: usize) -> Vec<RiskMetrics> {
    let close = df.column("close")
        .map(|c| c.to_f64_vec())
        .unwrap_or_default();
    let rets = daily_returns(&close);
    let n = rets.len();

    (window..=n).map(|end| {
        let start = end - window;
        risk_metrics(&rets[start..end])
    }).collect()
}

/// Compute rolling beta vs benchmark
pub fn rolling_beta(asset_df: &DataFrame, bench_df: &DataFrame, window: usize) -> Vec<f64> {
    let a_close = asset_df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let b_close = bench_df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let a_rets = daily_returns(&a_close);
    let b_rets = daily_returns(&b_close);
    let n = a_rets.len().min(b_rets.len());

    (window..=n).map(|end| {
        let start = end - window;
        beta(&a_rets[start..end], &b_rets[start..end])
    }).collect()
}

// ── Helpers ──

fn mean(data: &[f64]) -> f64 {
    if data.is_empty() { return 0.0; }
    data.iter().sum::<f64>() / data.len() as f64
}

fn variance(data: &[f64]) -> f64 {
    if data.len() < 2 { return 0.0; }
    let m = mean(data);
    data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64
}

fn covariance(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 { return 0.0; }
    let mx = mean(&x[..n]);
    let my = mean(&y[..n]);
    (0..n).map(|i| (x[i] - mx) * (y[i] - my)).sum::<f64>() / (n - 1) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_metrics_basic() {
        let rets = vec![0.01, -0.005, 0.02, -0.01, 0.005, 0.015, -0.002, 0.008, -0.003, 0.012];
        let m = risk_metrics(&rets);
        assert!(m.annual_volatility > 0.0);
        assert!(m.sharpe_ratio.is_finite());
        assert!(m.max_drawdown >= 0.0);
    }

    #[test]
    fn test_beta() {
        let a = vec![0.01, 0.02, -0.01, 0.005];
        let b = vec![0.015, 0.025, -0.005, 0.01];
        let b_val = beta(&a, &b);
        assert!(b_val > 0.0);
    }
}

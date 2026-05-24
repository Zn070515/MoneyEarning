use rand::Rng;
use serde::{Serialize, Deserialize};
use wasm_core::DataFrame;

use crate::strategies::Signal;
use crate::BacktestConfig;

/// Monte Carlo simulation configuration
#[derive(Debug, Clone)]
pub struct MonteCarloConfig {
    pub num_simulations: usize,
    pub method: McMethod,
    pub confidence_level: f64,  // e.g., 0.95 for 95% confidence
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum McMethod {
    /// Shuffle and resample individual trade returns
    TradeShuffle,
    /// Bootstrap daily returns with replacement
    ReturnBootstrap,
    /// Parametric: assume normal returns, sample from fitted distribution
    Parametric,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        MonteCarloConfig {
            num_simulations: 1000,
            method: McMethod::TradeShuffle,
            confidence_level: 0.95,
        }
    }
}

/// Results from Monte Carlo simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    /// All simulated terminal equity values
    pub terminal_equities: Vec<f64>,
    /// All simulated max drawdowns
    pub max_drawdowns: Vec<f64>,
    /// All simulated Sharpe ratios
    pub sharpe_ratios: Vec<f64>,
    /// All simulated annual returns
    pub annual_returns: Vec<f64>,
    /// All simulated win rates
    pub win_rates: Vec<f64>,

    // Confidence intervals
    pub ci_lower_return: f64,
    pub ci_upper_return: f64,
    pub median_return: f64,
    pub mean_return: f64,

    pub ci_lower_dd: f64,
    pub ci_upper_dd: f64,
    pub median_dd: f64,

    pub ci_lower_sharpe: f64,
    pub ci_upper_sharpe: f64,
    pub median_sharpe: f64,

    // Risk metrics
    pub var_95: f64,         // 95% Value at Risk (annual return)
    pub cvar_95: f64,        // 95% Conditional VaR (expected shortfall)
    pub prob_profit: f64,    // Probability of positive return
    pub prob_ruin: f64,      // Probability of ruin (drawdown > 50%)

    /// Number of simulations
    pub num_simulations: usize,
    /// Simulation method used
    pub method: String,
}

/// Run Monte Carlo simulation on a backtest result
pub fn monte_carlo_simulate(
    df: &DataFrame,
    signals: &[Signal],
    config: &BacktestConfig,
    mc_config: &MonteCarloConfig,
) -> MonteCarloResult {
    // First, run the base simulation to extract trades
    let _base_result = crate::simulate(df, signals, config);

    match mc_config.method {
        McMethod::TradeShuffle => mc_trade_shuffle(df, signals, config, mc_config),
        McMethod::ReturnBootstrap => mc_return_bootstrap(df, signals, config, mc_config),
        McMethod::Parametric => mc_parametric(df, signals, config, mc_config),
    }
}

/// Shuffle and resample trade sequences
fn mc_trade_shuffle(
    df: &DataFrame,
    signals: &[Signal],
    config: &BacktestConfig,
    mc_config: &MonteCarloConfig,
) -> MonteCarloResult {
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let _n = close.len();

    // Extract trade returns from base simulation
    let (trade_returns, _durations) = extract_trades(df, signals, config);

    if trade_returns.is_empty() {
        return empty_mc_result(mc_config);
    }

    let mut rng = rand::thread_rng();
    let mut terminal_equities = Vec::with_capacity(mc_config.num_simulations);
    let mut max_drawdowns = Vec::with_capacity(mc_config.num_simulations);
    let mut sharpe_ratios = Vec::with_capacity(mc_config.num_simulations);
    let mut annual_returns = Vec::with_capacity(mc_config.num_simulations);
    let mut win_rates = Vec::with_capacity(mc_config.num_simulations);

    for _ in 0..mc_config.num_simulations {
        // Shuffle trade returns
        let mut shuffled_returns: Vec<f64> = trade_returns.iter()
            .map(|&r| r * (0.8 + rng.gen::<f64>() * 0.4)) // Add noise ±20%
            .collect();
        // Fisher-Yates shuffle
        for i in (1..shuffled_returns.len()).rev() {
            let j = rng.gen_range(0..=i);
            shuffled_returns.swap(i, j);
        }

        // Rebuild equity curve
        let mut equity = config.initial_capital;
        let mut peak = equity;
        let mut max_dd = 0.0f64;
        let mut winning = 0usize;
        let total = shuffled_returns.len();

        for &tr in &shuffled_returns {
            equity += tr;
            if equity > peak { peak = equity; }
            let dd = (equity - peak) / peak;
            if dd < max_dd { max_dd = dd; }
            if tr > 0.0 { winning += 1; }
        }

        let total_return = (equity - config.initial_capital) / config.initial_capital;
        let years = (total as f64 * 10.0 / 252.0).max(0.01); // rough estimate
        let ann_ret = (1.0 + total_return).powf(1.0 / years) - 1.0;
        let win_rate = if total > 0 { winning as f64 / total as f64 } else { 0.0 };

        // Sharpe: use shuffled returns distribution
        let mean_r = shuffled_returns.iter().sum::<f64>() / total.max(1) as f64;
        let var_r = shuffled_returns.iter().map(|r| (r - mean_r).powi(2)).sum::<f64>() / total.max(1) as f64;
        let std_r = var_r.sqrt();
        let sharpe = if std_r > 0.0 { mean_r / std_r * (252.0_f64).sqrt() } else { 0.0 };

        terminal_equities.push(equity);
        max_drawdowns.push(max_dd);
        sharpe_ratios.push(sharpe);
        annual_returns.push(ann_ret);
        win_rates.push(win_rate);
    }

    build_mc_result(terminal_equities, max_drawdowns, sharpe_ratios, annual_returns, win_rates, mc_config, "Trade Shuffle")
}

/// Bootstrap daily returns
fn mc_return_bootstrap(
    df: &DataFrame,
    signals: &[Signal],
    config: &BacktestConfig,
    mc_config: &MonteCarloConfig,
) -> MonteCarloResult {
    let base_result = crate::simulate(df, signals, config);
    if base_result.equity_curve.len() < 2 {
        return empty_mc_result(mc_config);
    }

    // Compute daily returns from equity curve
    let equity: Vec<f64> = base_result.equity_curve.iter().map(|(_, v)| *v).collect();
    let daily_returns: Vec<f64> = equity.windows(2).map(|w| {
        if w[0] > 0.0 { (w[1] - w[0]) / w[0] } else { 0.0 }
    }).collect();

    let mut rng = rand::thread_rng();
    let n_days = daily_returns.len();
    let years = n_days as f64 / 252.0;

    let mut terminal_equities = Vec::with_capacity(mc_config.num_simulations);
    let mut max_drawdowns = Vec::with_capacity(mc_config.num_simulations);
    let mut sharpe_ratios = Vec::with_capacity(mc_config.num_simulations);
    let mut annual_returns = Vec::with_capacity(mc_config.num_simulations);
    let mut win_rates = Vec::with_capacity(mc_config.num_simulations);

    for _ in 0..mc_config.num_simulations {
        // Bootstrap daily returns
        let mut equity = config.initial_capital;
        let mut peak = equity;
        let mut max_dd = 0.0f64;
        let mut sim_returns = Vec::with_capacity(n_days);

        for _ in 0..n_days {
            let idx = rng.gen_range(0..n_days);
            let r = daily_returns[idx];
            sim_returns.push(r);
            equity *= 1.0 + r;
            if equity > peak { peak = equity; }
            let dd = (equity - peak) / peak;
            if dd < max_dd { max_dd = dd; }
        }

        let total_return = (equity - config.initial_capital) / config.initial_capital;
        let ann_ret = (1.0 + total_return).powf(1.0 / years.max(0.01)) - 1.0;
        let win_rate = sim_returns.iter().filter(|&&r| r > 0.0).count() as f64 / n_days.max(1) as f64;

        let mean_r = sim_returns.iter().sum::<f64>() / n_days as f64;
        let var_r = sim_returns.iter().map(|&r| (r - mean_r).powi(2)).sum::<f64>() / n_days as f64;
        let std_r = var_r.sqrt();
        let sharpe = if std_r > 0.0 { mean_r / std_r * (252.0_f64).sqrt() } else { 0.0 };

        terminal_equities.push(equity);
        max_drawdowns.push(max_dd);
        sharpe_ratios.push(sharpe);
        annual_returns.push(ann_ret);
        win_rates.push(win_rate);
    }

    build_mc_result(terminal_equities, max_drawdowns, sharpe_ratios, annual_returns, win_rates, mc_config, "Return Bootstrap")
}

/// Parametric simulation (assume log-normal returns)
fn mc_parametric(
    df: &DataFrame,
    signals: &[Signal],
    config: &BacktestConfig,
    mc_config: &MonteCarloConfig,
) -> MonteCarloResult {
    let base_result = crate::simulate(df, signals, config);
    if base_result.equity_curve.len() < 2 {
        return empty_mc_result(mc_config);
    }

    let equity: Vec<f64> = base_result.equity_curve.iter().map(|(_, v)| *v).collect();
    let daily_returns: Vec<f64> = equity.windows(2).map(|w| {
        if w[0] > 0.0 { (w[1] - w[0]) / w[0] } else { 0.0 }
    }).collect();

    // Fit normal distribution to daily returns
    let n_days = daily_returns.len();
    let mean_r = daily_returns.iter().sum::<f64>() / n_days as f64;
    let var_r = daily_returns.iter().map(|&r| (r - mean_r).powi(2)).sum::<f64>() / n_days as f64;
    let std_r = var_r.sqrt().max(0.0001);
    let years = n_days as f64 / 252.0;

    let mut rng = rand::thread_rng();
    let mut terminal_equities = Vec::with_capacity(mc_config.num_simulations);
    let mut max_drawdowns = Vec::with_capacity(mc_config.num_simulations);
    let mut sharpe_ratios = Vec::with_capacity(mc_config.num_simulations);
    let mut annual_returns = Vec::with_capacity(mc_config.num_simulations);
    let mut win_rates = Vec::with_capacity(mc_config.num_simulations);

    for _ in 0..mc_config.num_simulations {
        let mut equity = config.initial_capital;
        let mut peak = equity;
        let mut max_dd = 0.0f64;
        let mut sim_returns = Vec::with_capacity(n_days);

        for _ in 0..n_days {
            // Box-Muller transform for normal random
            let u1: f64 = rng.gen::<f64>().max(1e-10);
            let u2: f64 = rng.gen();
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let r = mean_r + std_r * z;
            sim_returns.push(r);
            equity *= 1.0 + r;
            if equity > peak { peak = equity; }
            let dd = (equity - peak) / peak;
            if dd < max_dd { max_dd = dd; }
        }

        let total_return = (equity - config.initial_capital) / config.initial_capital;
        let ann_ret = (1.0 + total_return).powf(1.0 / years.max(0.01)) - 1.0;
        let win_rate = sim_returns.iter().filter(|&&r| r > 0.0).count() as f64 / n_days.max(1) as f64;
        let sharpe = if std_r > 0.0 { mean_r / std_r * (252.0_f64).sqrt() } else { 0.0 };

        terminal_equities.push(equity);
        max_drawdowns.push(max_dd);
        sharpe_ratios.push(sharpe);
        annual_returns.push(ann_ret);
        win_rates.push(win_rate);
    }

    build_mc_result(terminal_equities, max_drawdowns, sharpe_ratios, annual_returns, win_rates, mc_config, "Parametric")
}

fn extract_trades(df: &DataFrame, signals: &[Signal], config: &BacktestConfig) -> (Vec<f64>, Vec<usize>) {
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let n = close.len();
    let mut trade_returns = Vec::new();
    let mut durations = Vec::new();
    let mut buy_price = 0.0;
    let mut in_pos = false;
    let mut holding_days = 0;

    for i in 0..n {
        if in_pos { holding_days += 1; }
        match signals[i] {
            Signal::Buy if !in_pos => {
                buy_price = close[i] * (1.0 + config.slippage);
                in_pos = true;
                holding_days = 0;
            }
            Signal::Sell if in_pos => {
                let sell_price = close[i] * (1.0 - config.slippage);
                let commission = buy_price * config.commission_rate + sell_price * (config.commission_rate + config.stamp_tax_rate);
                let tr = (sell_price - buy_price) - commission;
                trade_returns.push(tr);
                durations.push(holding_days);
                in_pos = false;
            }
            _ => {}
        }
    }
    (trade_returns, durations)
}

fn build_mc_result(
    terminal_equities: Vec<f64>,
    max_drawdowns: Vec<f64>,
    sharpe_ratios: Vec<f64>,
    annual_returns: Vec<f64>,
    win_rates: Vec<f64>,
    mc_config: &MonteCarloConfig,
    method: &str,
) -> MonteCarloResult {
    let n = terminal_equities.len();
    if n == 0 { return empty_mc_result(mc_config); }

    let alpha = (1.0 - mc_config.confidence_level) / 2.0;

    let ci_returns = percentile_ci(&annual_returns, alpha);
    let ci_dd = percentile_ci_negative(&max_drawdowns, alpha);
    let ci_sharpe = percentile_ci(&sharpe_ratios, alpha);

    let median_return = percentile(&annual_returns, 0.5);
    let mean_return = annual_returns.iter().sum::<f64>() / n as f64;
    let median_dd = percentile(&max_drawdowns, 0.5);
    let median_sharpe = percentile(&sharpe_ratios, 0.5);

    let var_95 = percentile(&annual_returns, 0.05);
    let cvar_95 = {
        let tail: Vec<f64> = annual_returns.iter().filter(|&&r| r <= var_95).copied().collect();
        if tail.is_empty() { var_95 } else { tail.iter().sum::<f64>() / tail.len() as f64 }
    };

    let prob_profit = annual_returns.iter().filter(|&&r| r > 0.0).count() as f64 / n as f64;
    let prob_ruin = max_drawdowns.iter().filter(|&&d| d < -0.50).count() as f64 / n as f64;

    MonteCarloResult {
        terminal_equities,
        max_drawdowns,
        sharpe_ratios,
        annual_returns,
        win_rates,
        ci_lower_return: ci_returns.0, ci_upper_return: ci_returns.1,
        median_return, mean_return,
        ci_lower_dd: ci_dd.0, ci_upper_dd: ci_dd.1, median_dd,
        ci_lower_sharpe: ci_sharpe.0, ci_upper_sharpe: ci_sharpe.1, median_sharpe,
        var_95, cvar_95, prob_profit, prob_ruin,
        num_simulations: n,
        method: method.to_string(),
    }
}

fn percentile(data: &[f64], p: f64) -> f64 {
    let mut sorted: Vec<f64> = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = (p * (sorted.len() - 1) as f64) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn percentile_ci(data: &[f64], alpha: f64) -> (f64, f64) {
    let lo = percentile(data, alpha);
    let hi = percentile(data, 1.0 - alpha);
    (lo.min(hi), hi.max(lo))
}

fn percentile_ci_negative(data: &[f64], alpha: f64) -> (f64, f64) {
    // Max drawdowns are negative, CI is reported with lower being worse
    let lo = percentile(data, alpha);
    let hi = percentile(data, 1.0 - alpha);
    (lo.min(hi), hi.max(lo))
}

fn empty_mc_result(mc_config: &MonteCarloConfig) -> MonteCarloResult {
    MonteCarloResult {
        terminal_equities: vec![],
        max_drawdowns: vec![],
        sharpe_ratios: vec![],
        annual_returns: vec![],
        win_rates: vec![],
        ci_lower_return: 0.0, ci_upper_return: 0.0,
        median_return: 0.0, mean_return: 0.0,
        ci_lower_dd: 0.0, ci_upper_dd: 0.0, median_dd: 0.0,
        ci_lower_sharpe: 0.0, ci_upper_sharpe: 0.0, median_sharpe: 0.0,
        var_95: 0.0, cvar_95: 0.0, prob_profit: 0.0, prob_ruin: 0.0,
        num_simulations: mc_config.num_simulations,
        method: "empty".to_string(),
    }
}

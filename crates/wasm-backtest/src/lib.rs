pub mod strategies;
pub mod optimizer;
pub mod walk_forward;
pub mod monte_carlo;
pub mod benches;

#[cfg(test)]
mod strategies_verification;

use std::collections::HashMap;
use wasm_core::{DataFrame, BtResult};

pub use strategies::{generate_signals, list_strategies, StrategyMeta, Signal};
pub use optimizer::{optimize, OptimizerConfig, OptimizerMethod, OptimizerResult, TargetMetric};
pub use walk_forward::{walk_forward_analysis, walk_forward_efficiency, WalkForwardConfig, WalkForwardResult, AnchorMode};
pub use monte_carlo::{monte_carlo_simulate, MonteCarloConfig, MonteCarloResult, McMethod};

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub commission_rate: f64,
    pub stamp_tax_rate: f64,
    pub slippage: f64,
    pub position_pct: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        BacktestConfig {
            initial_capital: 100_000.0,
            commission_rate: 0.0003,   // A-share commission 0.03%
            stamp_tax_rate: 0.001,     // A-share stamp tax 0.1% (sell only)
            slippage: 0.001,           // slippage 0.1%
            position_pct: 1.0,
        }
    }
}

/// Run backtest with a named strategy template
pub fn run_with_template(
    df: &DataFrame,
    template: &str,
    params: &HashMap<String, f64>,
    config: &BacktestConfig,
) -> BtResult {
    let signals = strategies::generate_signals(df, template, params);
    simulate(df, &signals, config)
}

/// Run backtest with pre-computed signals
pub fn run_with_signals(
    df: &DataFrame,
    signals: &[Signal],
    config: &BacktestConfig,
) -> BtResult {
    simulate(df, signals, config)
}

// ── Core Trade Simulation Engine ──

pub fn simulate(df: &DataFrame, signals: &[Signal], config: &BacktestConfig) -> BtResult {
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let n = close.len();
    if n == 0 {
        return empty_bt_result();
    }

    let mut cash = config.initial_capital;
    let mut shares = 0.0f64;
    let mut equity = vec![0.0; n];
    let mut trade_list: Vec<(f64, f64, f64, String)> = Vec::new();
    let mut buy_price = 0.0;
    let mut buy_date = String::new();
    let mut in_position = false;

    for i in 0..n {
        let price = close[i];

        match signals[i] {
            Signal::Buy if !in_position => {
                let slippage_price = price * (1.0 + config.slippage);
                let max_shares_value = cash * config.position_pct;
                let trade_value = max_shares_value / (1.0 + config.commission_rate);
                shares = trade_value / slippage_price;
                let cost = trade_value * config.commission_rate;
                cash -= trade_value + cost;
                buy_price = slippage_price;
                buy_date = format!("idx_{}", i);
                in_position = true;
            }
            Signal::Sell if in_position => {
                let slippage_price = price * (1.0 - config.slippage);
                let trade_value = shares * slippage_price;
                let commission = trade_value * config.commission_rate;
                let stamp_tax = trade_value * config.stamp_tax_rate;
                cash += trade_value - commission - stamp_tax;
                let pnl = (slippage_price - buy_price) * shares - commission - stamp_tax;
                trade_list.push((buy_price, slippage_price, pnl, buy_date.clone()));
                shares = 0.0;
                in_position = false;
            }
            _ => {}
        }
        equity[i] = if in_position {
            cash + shares * close[i] * (1.0 - config.slippage)
        } else {
            cash
        };
    }

    // Close any open position at last price
    if in_position {
        let last_price = close[n - 1];
        let trade_value = shares * last_price * (1.0 - config.slippage);
        let commission = trade_value * config.commission_rate;
        let stamp_tax = trade_value * config.stamp_tax_rate;
        cash += trade_value - commission - stamp_tax;
        let pnl = (last_price * (1.0 - config.slippage) - buy_price) * shares - commission - stamp_tax;
        trade_list.push((buy_price, last_price * (1.0 - config.slippage), pnl, buy_date));
        equity[n - 1] = cash;
    }

    compute_metrics(equity, cash, trade_list, config)
}

fn compute_metrics(equity: Vec<f64>, final_cash: f64, trades: Vec<(f64, f64, f64, String)>, config: &BacktestConfig) -> BtResult {
    let n = equity.len();
    let total_return = (final_cash - config.initial_capital) / config.initial_capital;
    let trading_days = n as f64;
    let years = trading_days / 252.0;
    let annual_return = if years > 0.0 { (1.0 + total_return).powf(1.0 / years) - 1.0 } else { 0.0 };

    // Max drawdown
    let mut peak = equity[0];
    let mut max_dd = 0.0f64;
    for &v in &equity {
        if v > peak { peak = v; }
        let dd = (v - peak) / peak;
        if dd < max_dd { max_dd = dd; }
    }

    // Daily returns & Sharpe
    let daily_returns: Vec<f64> = equity.windows(2).map(|w| {
        if w[0] > 0.0 { (w[1] - w[0]) / w[0] } else { 0.0 }
    }).collect();

    let avg_ret = daily_returns.iter().sum::<f64>() / daily_returns.len().max(1) as f64;
    let variance = daily_returns.iter().map(|r| (r - avg_ret).powi(2)).sum::<f64>() / daily_returns.len().max(1) as f64;
    let std_dev = variance.sqrt();
    let sharpe_ratio = if std_dev > 0.0 { avg_ret / std_dev * (252.0_f64).sqrt() } else { 0.0 };

    // Sortino
    let down_returns: Vec<f64> = daily_returns.iter().filter(|&&r| r < 0.0).copied().collect();
    let down_var = down_returns.iter().map(|r| r.powi(2)).sum::<f64>() / down_returns.len().max(1) as f64;
    let down_dev = down_var.sqrt();
    let sortino_ratio = if down_dev > 0.0 { avg_ret / down_dev * (252.0_f64).sqrt() } else { 0.0 };

    // Calmar
    let calmar_ratio = if max_dd < 0.0 { annual_return / max_dd.abs() } else { 0.0 };

    // Win rate & P/L ratio
    let total_trades = trades.len() as u32;
    let winning: Vec<f64> = trades.iter().filter(|t| t.2 > 0.0).map(|t| t.2).collect();
    let losing: Vec<f64> = trades.iter().filter(|t| t.2 <= 0.0).map(|t| t.2).collect();
    let win_rate = if total_trades > 0 { winning.len() as f64 / total_trades as f64 } else { 0.0 };
    let avg_win = if !winning.is_empty() { winning.iter().sum::<f64>() / winning.len() as f64 } else { 0.0 };
    let avg_loss = if !losing.is_empty() { losing.iter().sum::<f64>() / losing.len() as f64 } else { 0.0 };
    let profit_loss_ratio = if avg_loss.abs() > 0.0 { avg_win / avg_loss.abs() } else { 0.0 };

    // Equity curve: sample every N bars
    let step = (n / 500).max(1);
    let equity_curve: Vec<(String, f64)> = equity.iter().enumerate()
        .step_by(step)
        .map(|(i, &v)| (i.to_string(), v))
        .collect();

    // Monthly returns
    let monthly_returns: Vec<(String, f64)> = equity.iter().enumerate()
        .filter(|(i, _)| i % 21 == 0 && *i >= 21)
        .map(|(i, &v)| {
            let prev = equity[i - 21.min(i)];
            let mret = if prev > 0.0 { (v - prev) / prev } else { 0.0 };
            (format!("m{}", i / 21), mret)
        })
        .collect();

    BtResult {
        total_return,
        annual_return,
        max_drawdown: max_dd,
        sharpe_ratio,
        sortino_ratio,
        calmar_ratio,
        win_rate,
        profit_loss_ratio,
        total_trades,
        equity_curve,
        monthly_returns,
    }
}

fn empty_bt_result() -> BtResult {
    BtResult {
        total_return: 0.0, annual_return: 0.0, max_drawdown: 0.0,
        sharpe_ratio: 0.0, sortino_ratio: 0.0, calmar_ratio: 0.0,
        win_rate: 0.0, profit_loss_ratio: 0.0, total_trades: 0,
        equity_curve: vec![], monthly_returns: vec![],
    }
}

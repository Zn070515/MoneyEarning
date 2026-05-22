use std::collections::HashMap;
use wasm_core::{DataFrame, BtResult};

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub commission_rate: f64,
    pub stamp_tax_rate: f64,
    pub slippage: f64,
    pub position_pct: f64, // % of capital per trade (0.0-1.0)
}

impl Default for BacktestConfig {
    fn default() -> Self {
        BacktestConfig {
            initial_capital: 100_000.0,
            commission_rate: 0.0003,   // A股佣金万三
            stamp_tax_rate: 0.001,     // A股印花税千一（仅卖出）
            slippage: 0.001,           // 滑点千一
            position_pct: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
enum Signal { Buy, Sell, Hold }

pub fn run_with_template(
    df: &DataFrame,
    template: &str,
    params: &HashMap<String, f64>,
    config: &BacktestConfig,
) -> BtResult {
    let signals = generate_signals(df, template, params);
    simulate(df, &signals, config)
}

fn generate_signals(df: &DataFrame, template: &str, params: &HashMap<String, f64>) -> Vec<Signal> {
    match template {
        "ma_cross" => ma_cross_signals(df, params),
        "breakout" => breakout_signals(df, params),
        "rsi_mean" => rsi_mean_signals(df, params),
        _ => vec![Signal::Hold; df.len()],
    }
}

// ── MA Crossover Strategy ──

fn ma_cross_signals(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let fast_period = params.get("fast").copied().unwrap_or(5.0) as usize;
    let slow_period = params.get("slow").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let n = close.len();

    let fast_ema = ema_vec(&close, fast_period);
    let slow_ema = ema_vec(&close, slow_period);

    let mut signals = vec![Signal::Hold; n];
    for i in slow_period.max(fast_period) + 1..n {
        if fast_ema[i - 1] <= slow_ema[i - 1] && fast_ema[i] > slow_ema[i] {
            signals[i] = Signal::Buy;
        } else if fast_ema[i - 1] >= slow_ema[i - 1] && fast_ema[i] < slow_ema[i] {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Breakout Strategy ──

fn breakout_signals(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let high = df.column("high").map(|c| c.to_f64_vec()).unwrap_or_default();
    let low = df.column("low").map(|c| c.to_f64_vec()).unwrap_or_default();
    let n = close.len();

    let mut signals = vec![Signal::Hold; n];
    for i in period + 1..n {
        let highest_high = high[i - period..i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let lowest_low = low[i - period..i].iter().cloned().fold(f64::INFINITY, f64::min);

        if close[i] > highest_high {
            signals[i] = Signal::Buy;
        } else if close[i] < lowest_low {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── RSI Mean Reversion Strategy ──

fn rsi_mean_signals(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let oversold = params.get("oversold").copied().unwrap_or(30.0);
    let overbought = params.get("overbought").copied().unwrap_or(70.0);
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let n = close.len();

    let rsi_vals = compute_rsi(&close, period);

    let mut signals = vec![Signal::Hold; n];
    for i in period + 1..n {
        if rsi_vals[i - 1] > oversold && rsi_vals[i] <= oversold {
            signals[i] = Signal::Buy;
        } else if rsi_vals[i - 1] < overbought && rsi_vals[i] >= overbought {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Trade Simulation ──

fn simulate(df: &DataFrame, signals: &[Signal], config: &BacktestConfig) -> BtResult {
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let n = close.len();
    if n == 0 {
        return BtResult {
            total_return: 0.0, annual_return: 0.0, max_drawdown: 0.0,
            sharpe_ratio: 0.0, sortino_ratio: 0.0, calmar_ratio: 0.0,
            win_rate: 0.0, profit_loss_ratio: 0.0, total_trades: 0,
            equity_curve: vec![], monthly_returns: vec![],
        };
    }

    let mut cash = config.initial_capital;
    let mut shares = 0.0f64;
    let mut equity = vec![0.0; n];
    let mut trades: Vec<(f64, f64, f64)> = Vec::new(); // (buy_price, sell_price, pnl)
    let mut buy_price = 0.0;
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
                in_position = true;
            }
            Signal::Sell if in_position => {
                let slippage_price = price * (1.0 - config.slippage);
                let trade_value = shares * slippage_price;
                let commission = trade_value * config.commission_rate;
                let stamp_tax = trade_value * config.stamp_tax_rate;
                cash += trade_value - commission - stamp_tax;
                let pnl = (slippage_price - buy_price) * shares - commission - stamp_tax;
                trades.push((buy_price, slippage_price, pnl));
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
        trades.push((buy_price, last_price * (1.0 - config.slippage), pnl));
        equity[n - 1] = cash;
    }

    // ── Metrics ──
    let final_equity = cash;
    let total_return = (final_equity - config.initial_capital) / config.initial_capital;

    let trading_days = n as f64;
    let years = trading_days / 252.0;
    let annual_return = if years > 0.0 {
        (1.0 + total_return).powf(1.0 / years) - 1.0
    } else { 0.0 };

    // Max drawdown
    let mut peak = equity[0];
    let mut max_dd = 0.0f64;
    for &v in &equity {
        if v > peak { peak = v; }
        let dd = (v - peak) / peak;
        if dd < max_dd { max_dd = dd; }
    }

    // Daily returns
    let daily_returns: Vec<f64> = equity.windows(2).map(|w| {
        if w[0] > 0.0 { (w[1] - w[0]) / w[0] } else { 0.0 }
    }).collect();

    let avg_ret = daily_returns.iter().sum::<f64>() / daily_returns.len().max(1) as f64;
    let variance = daily_returns.iter()
        .map(|r| (r - avg_ret).powi(2)).sum::<f64>() / daily_returns.len().max(1) as f64;
    let std_dev = variance.sqrt();
    let sharpe_ratio = if std_dev > 0.0 {
        avg_ret / std_dev * (252.0_f64).sqrt()
    } else { 0.0 };

    // Sortino (downside deviation)
    let down_returns: Vec<f64> = daily_returns.iter().filter(|&&r| r < 0.0).copied().collect();
    let down_variance = down_returns.iter()
        .map(|r| r.powi(2)).sum::<f64>() / down_returns.len().max(1) as f64;
    let down_dev = down_variance.sqrt();
    let sortino_ratio = if down_dev > 0.0 {
        avg_ret / down_dev * (252.0_f64).sqrt()
    } else { 0.0 };

    // Calmar
    let calmar_ratio = if max_dd < 0.0 { annual_return / max_dd.abs() } else { 0.0 };

    // Win rate & profit/loss ratio
    let total_trades = trades.len() as u32;
    let winning: Vec<f64> = trades.iter().filter(|t| t.2 > 0.0).map(|t| t.2).collect();
    let losing: Vec<f64> = trades.iter().filter(|t| t.2 <= 0.0).map(|t| t.2).collect();
    let win_rate = if total_trades > 0 {
        winning.len() as f64 / total_trades as f64
    } else { 0.0 };
    let avg_win = if !winning.is_empty() { winning.iter().sum::<f64>() / winning.len() as f64 } else { 0.0 };
    let avg_loss = if !losing.is_empty() { losing.iter().sum::<f64>() / losing.len() as f64 } else { 0.0 };
    let profit_loss_ratio = if avg_loss.abs() > 0.0 { avg_win / avg_loss.abs() } else { 0.0 };

    // Equity curve: sample every N bars to keep reasonable size
    let step = (n / 500).max(1);
    let equity_curve: Vec<(String, f64)> = equity.iter().enumerate()
        .step_by(step)
        .map(|(i, &v)| (i.to_string(), v))
        .collect();

    // Monthly returns (approximate: group by ~21 trading days)
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

// ── Helpers ──

fn ema_vec(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    if n < period { return result; }
    let k = 2.0 / (period as f64 + 1.0);
    let mut ema = data[..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = ema;
    for i in period..n {
        ema = (data[i] - ema) * k + ema;
        result[i] = ema;
    }
    result
}

fn compute_rsi(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    if n < period + 1 { return result; }
    let mut gains = 0.0f64;
    let mut losses = 0.0f64;
    for i in 1..=period {
        let diff = data[i] - data[i - 1];
        if diff > 0.0 { gains += diff; } else { losses += diff.abs(); }
    }
    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;
    result[period] = if avg_loss == 0.0 {
        100.0
    } else {
        100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
    };
    for i in period + 1..n {
        let diff = data[i] - data[i - 1];
        let (gain, loss) = if diff > 0.0 { (diff, 0.0) } else { (0.0, diff.abs()) };
        avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;
        result[i] = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
        };
    }
    result
}

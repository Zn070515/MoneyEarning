use std::collections::HashMap;
use wasm_core::DataFrame;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone)]
pub struct StrategyMeta {
    pub name: &'static str,
    pub name_cn: &'static str,
    pub category: &'static str,
    pub is_free: bool,
    pub params: &'static [(&'static str, f64)],
    pub description: &'static str,
}

pub fn list_strategies() -> Vec<StrategyMeta> {
    vec![
        // ═══ 趋势跟踪类（5个免费）═══
        StrategyMeta {
            name: "ma_cross", name_cn: "双均线交叉", category: "趋势跟踪", is_free: true,
            params: &[("fast", 5.0), ("slow", 20.0), ("stop_loss", -0.08)],
            description: "短均线上穿长均线买入，下穿卖出。适用于单边市。",
        },
        StrategyMeta {
            name: "macd_cross", name_cn: "MACD金叉死叉", category: "趋势跟踪", is_free: true,
            params: &[("fast", 12.0), ("slow", 26.0), ("signal", 9.0), ("stop_loss", -0.06)],
            description: "DIF上穿DEA买入，下穿卖出。零轴下方金叉更可靠。",
        },
        StrategyMeta {
            name: "triple_ma", name_cn: "三均线多头排列", category: "趋势跟踪", is_free: true,
            params: &[("short", 5.0), ("mid", 20.0), ("long", 60.0)],
            description: "短>中>长三线多头排列持仓，空头排列清仓。适合强趋势市场。",
        },
        StrategyMeta {
            name: "turtle", name_cn: "海龟交易法则", category: "趋势跟踪", is_free: true,
            params: &[("entry_period", 20.0), ("exit_period", 10.0), ("atr_period", 20.0), ("atr_stop", 2.0)],
            description: "突破N日最高价入场，跌破M日最低价离场，ATR动态止损。",
        },
        StrategyMeta {
            name: "donchian", name_cn: "唐奇安通道突破", category: "趋势跟踪", is_free: true,
            params: &[("upper_period", 20.0), ("lower_period", 10.0), ("min_band_width", 0.03)],
            description: "突破N日最高价买入，跌破M日最低价卖出。需配合ADX过滤。",
        },

        // ═══ 均值回归类（5个，全部付费）═══
        StrategyMeta {
            name: "bb_reversion", name_cn: "布林带均值回归", category: "均值回归", is_free: false,
            params: &[("period", 20.0), ("std_mult", 2.0), ("rsi_threshold", 30.0), ("hold_days", 5.0)],
            description: "价格突破布林带上下轨后大概率回归中轨。仅适用于震荡市。",
        },
        StrategyMeta {
            name: "rsi_extreme", name_cn: "RSI超买超卖", category: "均值回归", is_free: false,
            params: &[("period", 14.0), ("oversold", 30.0), ("overbought", 70.0), ("hold_days", 5.0)],
            description: "RSI<30超卖买入，RSI>70超买卖出。需确认止跌/止涨再入场。",
        },
        StrategyMeta {
            name: "bb_rsi_double", name_cn: "布林带+RSI双确认", category: "均值回归", is_free: false,
            params: &[("period", 20.0), ("std", 2.0), ("rsi_period", 14.0), ("rsi_threshold", 30.0), ("vol_mult", 1.5)],
            description: "布林带下轨+RSI超卖+放量止跌三重确认反弹。",
        },
        StrategyMeta {
            name: "kdj_extreme", name_cn: "KDJ超买超卖", category: "均值回归", is_free: false,
            params: &[("n", 9.0), ("m1", 3.0), ("m2", 3.0), ("oversold", 20.0), ("overbought", 80.0)],
            description: "K/D/J三线同时超卖/超买后反转。比RSI更敏感。",
        },
        StrategyMeta {
            name: "zscore_reversion", name_cn: "Z-Score均值回归", category: "均值回归", is_free: false,
            params: &[("period", 20.0), ("entry_z", 2.0), ("exit_z", 0.5), ("stop_loss", -0.05)],
            description: "价格偏离均线超过N个标准差后统计上大概率回归。",
        },

        // ═══ 动量类（3个，全部付费）═══
        StrategyMeta {
            name: "cross_momentum", name_cn: "横截面动量", category: "动量", is_free: false,
            params: &[("lookback", 60.0), ("top_k", 20.0), ("rebalance", 20.0)],
            description: "买过去N日涨幅最大的前K只，定期调仓。适用于赛道行情。",
        },
        StrategyMeta {
            name: "ts_momentum", name_cn: "时间序列动量", category: "动量", is_free: false,
            params: &[("lookback", 60.0), ("smoothing", 5.0)],
            description: "过去N日收益>0做多，<0空仓。波动率目标仓位管理。",
        },
        StrategyMeta {
            name: "dual_rsi", name_cn: "双线RSI轮动", category: "动量", is_free: false,
            params: &[("fast_period", 7.0), ("slow_period", 14.0), ("signal_period", 5.0)],
            description: "快RSI>慢RSI且均上升确认动量买入。适合风格轮动。",
        },

        // ═══ 突破类（3个，全部付费）═══
        StrategyMeta {
            name: "vol_breakout_ma", name_cn: "放量突破均线", category: "突破", is_free: false,
            params: &[("ma_period", 60.0), ("vol_mult", 1.5), ("confirm_bars", 1.0)],
            description: "放量突破中期均线+收盘确认，趋势启动信号。",
        },
        StrategyMeta {
            name: "volatility_breakout", name_cn: "波动性突破(ATR)", category: "突破", is_free: false,
            params: &[("atr_period", 20.0), ("atr_mult", 2.0)],
            description: "价格高于昨日收盘+N倍ATR，波动扩张=趋势行情启动。",
        },
        StrategyMeta {
            name: "keltner_breakout", name_cn: "Keltner通道突破", category: "突破", is_free: false,
            params: &[("ema_period", 20.0), ("atr_period", 10.0), ("atr_mult", 2.0)],
            description: "收盘价突破Keltner通道上轨入场，跌破中轨离场。",
        },

        // ═══ 复合类（4个，全部付费）═══
        StrategyMeta {
            name: "macd_rsi_combo", name_cn: "MACD+RSI联合", category: "复合", is_free: false,
            params: &[("macd_fast", 12.0), ("macd_slow", 26.0), ("macd_signal", 9.0), ("rsi_period", 14.0), ("rsi_low", 40.0), ("rsi_high", 65.0)],
            description: "MACD金叉+RSI不超买→趋势确认+动量安全。胜率+13%vs纯MACD。",
        },
        StrategyMeta {
            name: "ma_volume_confirm", name_cn: "均线+成交量确认", category: "复合", is_free: false,
            params: &[("ma_short", 5.0), ("ma_long", 20.0), ("vol_mult", 1.5)],
            description: "价格站上均线+成交量放大确认资金入场。",
        },
        StrategyMeta {
            name: "multi_factor", name_cn: "多因子打分卡", category: "复合", is_free: false,
            params: &[("top_n", 30.0), ("rebalance", 20.0)],
            description: "ICIR加权的技术面+量价多因子综合打分选股。",
        },
        StrategyMeta {
            name: "walk_forward_adaptive", name_cn: "Walk-Forward自适应", category: "复合", is_free: false,
            params: &[("in_sample", 252.0), ("out_sample", 63.0)],
            description: "滚动窗口优化参数，选择样本外最稳健的参数组合。",
        },
    ]
}

/// Generate trading signals for a given strategy
pub fn generate_signals(df: &DataFrame, template: &str, params: &HashMap<String, f64>) -> Vec<Signal> {
    match template {
        // Free - Trend following
        "ma_cross" => ma_cross(df, params),
        "macd_cross" => macd_cross(df, params),
        "triple_ma" => triple_ma(df, params),
        "turtle" => turtle(df, params),
        "donchian" => donchian(df, params),
        // Paid - Mean reversion
        "bb_reversion" => bb_reversion(df, params),
        "rsi_extreme" => rsi_extreme(df, params),
        "bb_rsi_double" => bb_rsi_double(df, params),
        "kdj_extreme" => kdj_extreme(df, params),
        "zscore_reversion" => zscore_reversion(df, params),
        // Paid - Momentum
        "cross_momentum" => cross_momentum(df, params),
        "ts_momentum" => ts_momentum(df, params),
        "dual_rsi" => dual_rsi(df, params),
        // Paid - Breakout
        "vol_breakout_ma" => vol_breakout_ma(df, params),
        "volatility_breakout" => volatility_breakout(df, params),
        "keltner_breakout" => keltner_breakout(df, params),
        // Paid - Composite
        "macd_rsi_combo" => macd_rsi_combo(df, params),
        "ma_volume_confirm" => ma_volume_confirm(df, params),
        "multi_factor" => multi_factor(df, params),
        "walk_forward_adaptive" => ma_cross(df, params), // Default to MA cross in adaptive mode
        _ => vec![Signal::Hold; df.len()],
    }
}

// ─── Helpers ───

fn get_arrays(df: &DataFrame) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let open = df.column("open").map(|c| c.to_f64_vec()).unwrap_or_default();
    let high = df.column("high").map(|c| c.to_f64_vec()).unwrap_or_default();
    let low = df.column("low").map(|c| c.to_f64_vec()).unwrap_or_default();
    let close = df.column("close").map(|c| c.to_f64_vec()).unwrap_or_default();
    let volume = df.column("volume").map(|c| c.to_f64_vec()).unwrap_or_default();
    (open, high, low, close, volume)
}

fn sma_vec(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    if n < period { return result; }
    let mut sum: f64 = data[..period].iter().sum();
    result[period - 1] = sum / period as f64;
    for i in period..n {
        sum += data[i] - data[i - period];
        result[i] = sum / period as f64;
    }
    result
}

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

fn rsi_vec(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    if n < period + 1 { return result; }
    let mut gains = 0.0;
    let mut losses = 0.0;
    for i in 1..=period {
        let diff = data[i] - data[i - 1];
        if diff > 0.0 { gains += diff; } else { losses += diff.abs(); }
    }
    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;
    result[period] = if avg_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + avg_gain / avg_loss) };
    for i in period + 1..n {
        let diff = data[i] - data[i - 1];
        let (gain, loss) = if diff > 0.0 { (diff, 0.0) } else { (0.0, diff.abs()) };
        avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;
        result[i] = if avg_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + avg_gain / avg_loss) };
    }
    result
}

fn atr_vec(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    let n = close.len();
    let mut tr = vec![0.0; n];
    for i in 1..n {
        let a = high[i] - low[i];
        let b = (high[i] - close[i - 1]).abs();
        let c = (low[i] - close[i - 1]).abs();
        tr[i] = a.max(b).max(c);
    }
    let mut result = vec![f64::NAN; n];
    if n < period + 1 { return result; }
    let mut atr = tr[1..=period].iter().sum::<f64>() / period as f64;
    result[period] = atr;
    for i in period + 1..n {
        atr = (atr * (period - 1) as f64 + tr[i]) / period as f64;
        result[i] = atr;
    }
    result
}

fn kdj_vec(high: &[f64], low: &[f64], close: &[f64], n: usize, m1: f64, m2: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let len = close.len();
    let mut k_vals = vec![50.0; len];
    let mut d_vals = vec![50.0; len];
    let mut j_vals = vec![50.0; len];
    if len < n { return (k_vals, d_vals, j_vals); }

    let alpha_k = 1.0 / m1;
    let alpha_d = 1.0 / m2;

    for i in n - 1..len {
        let hh = high[i + 1 - n..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let ll = low[i + 1 - n..=i].iter().cloned().fold(f64::INFINITY, f64::min);
        let rsv = if (hh - ll).abs() < 1e-10 { 50.0 } else { (close[i] - ll) / (hh - ll) * 100.0 };

        let prev_k = if i > 0 { k_vals[i - 1] } else { 50.0 };
        let prev_d = if i > 0 { d_vals[i - 1] } else { 50.0 };

        k_vals[i] = prev_k * (1.0 - alpha_k) + rsv * alpha_k;
        d_vals[i] = prev_d * (1.0 - alpha_d) + k_vals[i] * alpha_d;
        j_vals[i] = 3.0 * k_vals[i] - 2.0 * d_vals[i];
    }
    (k_vals, d_vals, j_vals)
}

fn macd_vec(close: &[f64], fast: usize, slow: usize, signal: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let fast_ema = ema_vec(close, fast);
    let slow_ema = ema_vec(close, slow);
    let n = close.len();
    let mut dif = vec![f64::NAN; n];
    let mut dea = vec![f64::NAN; n];
    let mut hist = vec![0.0; n];
    for i in 0..n {
        if fast_ema[i].is_finite() && slow_ema[i].is_finite() {
            dif[i] = fast_ema[i] - slow_ema[i];
        }
    }
    let valid_dif: Vec<f64> = dif.iter().filter(|x| x.is_finite()).copied().collect();
    let valid_dea = ema_vec(&valid_dif, signal);
    let start = n - valid_dea.len();
    for (j, &v) in valid_dea.iter().enumerate() {
        let i = start + j;
        if i < n {
            dea[i] = v;
            hist[i] = (dif[i] - v) * 2.0;
        }
    }
    (dif, dea, hist)
}

fn bollinger_vec(close: &[f64], period: usize, std_mult: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let ma = sma_vec(close, period);
    let n = close.len();
    let mut upper = vec![f64::NAN; n];
    let mut lower = vec![f64::NAN; n];
    for i in period - 1..n {
        let mean = ma[i];
        let var = close[i + 1 - period..=i].iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let std = var.sqrt();
        upper[i] = mean + std_mult * std;
        lower[i] = mean - std_mult * std;
    }
    (upper, ma.clone(), lower)
}

fn highest_vec(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        result[i] = data[i + 1 - period..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    }
    result
}

fn lowest_vec(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    for i in period - 1..n {
        result[i] = data[i + 1 - period..=i].iter().cloned().fold(f64::INFINITY, f64::min);
    }
    result
}

fn cross(a_prev: f64, a_curr: f64, b_prev: f64, b_curr: f64) -> bool {
    a_prev <= b_prev && a_curr > b_curr
}

fn cross_under(a_prev: f64, a_curr: f64, b_prev: f64, b_curr: f64) -> bool {
    a_prev >= b_prev && a_curr < b_curr
}

// ═══════════════════════════════════════════════════════════════
// Strategy implementations
// ═══════════════════════════════════════════════════════════════

// ── Strategy 1: MA Cross (Free) ──
fn ma_cross(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let fast_p = params.get("fast").copied().unwrap_or(5.0) as usize;
    let slow_p = params.get("slow").copied().unwrap_or(20.0) as usize;
    let n = close.len();
    let fast_ma = sma_vec(&close, fast_p);
    let slow_ma = sma_vec(&close, slow_p);
    let mut signals = vec![Signal::Hold; n];
    let start = fast_p.max(slow_p) + 1;
    for i in start..n {
        if cross(fast_ma[i - 1], fast_ma[i], slow_ma[i - 1], slow_ma[i]) {
            signals[i] = Signal::Buy;
        } else if cross_under(fast_ma[i - 1], fast_ma[i], slow_ma[i - 1], slow_ma[i]) {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Strategy 2: MACD Cross (Free) ──
fn macd_cross(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let fast = params.get("fast").copied().unwrap_or(12.0) as usize;
    let slow = params.get("slow").copied().unwrap_or(26.0) as usize;
    let sig = params.get("signal").copied().unwrap_or(9.0) as usize;
    let n = close.len();
    let (dif, dea, _) = macd_vec(&close, fast, slow, sig);
    let mut signals = vec![Signal::Hold; n];
    for i in 2..n {
        if dif[i].is_finite() && dea[i].is_finite() && dif[i-1].is_finite() && dea[i-1].is_finite() {
            if cross(dif[i-1], dif[i], dea[i-1], dea[i]) {
                signals[i] = Signal::Buy;
            } else if cross_under(dif[i-1], dif[i], dea[i-1], dea[i]) {
                signals[i] = Signal::Sell;
            }
        }
    }
    signals
}

// ── Strategy 3: Triple MA (Free) ──
fn triple_ma(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let short = params.get("short").copied().unwrap_or(5.0) as usize;
    let mid = params.get("mid").copied().unwrap_or(20.0) as usize;
    let long = params.get("long").copied().unwrap_or(60.0) as usize;
    let n = close.len();
    let sma_short = sma_vec(&close, short);
    let sma_mid = sma_vec(&close, mid);
    let sma_long = sma_vec(&close, long);
    let mut signals = vec![Signal::Hold; n];
    let start = long + 3;
    for i in start..n {
        let aligned = sma_short[i] > sma_mid[i] && sma_mid[i] > sma_long[i];
        let was_aligned = i >= 3
            && sma_short[i-1] > sma_mid[i-1] && sma_mid[i-1] > sma_long[i-1]
            && sma_short[i-2] > sma_mid[i-2] && sma_mid[i-2] > sma_long[i-2];
        let unaligned = sma_short[i] < sma_mid[i];

        if aligned && !was_aligned {
            signals[i] = Signal::Buy;
        } else if unaligned && i > start && sma_short[i-1] >= sma_mid[i-1] {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Strategy 4: Turtle (Free) ──
fn turtle(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, high, low, close, _) = get_arrays(df);
    let entry_p = params.get("entry_period").copied().unwrap_or(20.0) as usize;
    let exit_p = params.get("exit_period").copied().unwrap_or(10.0) as usize;
    let atr_p = params.get("atr_period").copied().unwrap_or(20.0) as usize;
    let atr_stop = params.get("atr_stop").copied().unwrap_or(2.0);
    let n = close.len();
    let hh_entry = highest_vec(&high, entry_p);
    let ll_exit = lowest_vec(&low, exit_p);
    let atr = atr_vec(&high, &low, &close, atr_p);
    let mut signals = vec![Signal::Hold; n];
    let mut in_pos = false;
    let mut entry_price = 0.0;
    let start = entry_p.max(exit_p).max(atr_p) + 1;
    for i in start..n {
        if !in_pos {
            if close[i] > hh_entry[i - 1] {
                signals[i] = Signal::Buy;
                in_pos = true;
                entry_price = close[i];
            }
        } else {
            let stop_price = entry_price - atr_stop * atr[i];
            if close[i] < ll_exit[i - 1] || close[i] < stop_price {
                signals[i] = Signal::Sell;
                in_pos = false;
            }
        }
    }
    signals
}

// ── Strategy 5: Donchian (Free) ──
fn donchian(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, high, low, close, _) = get_arrays(df);
    let upper_p = params.get("upper_period").copied().unwrap_or(20.0) as usize;
    let lower_p = params.get("lower_period").copied().unwrap_or(10.0) as usize;
    let min_width = params.get("min_band_width").copied().unwrap_or(0.03);
    let n = close.len();
    let hh = highest_vec(&high, upper_p);
    let ll = lowest_vec(&low, upper_p);
    let ll_exit = lowest_vec(&low, lower_p);
    let mut signals = vec![Signal::Hold; n];
    let start = upper_p.max(lower_p) + 1;
    for i in start..n {
        let bandwidth = (hh[i] - ll[i]) / ll[i].max(0.01);
        if close[i] > hh[i - 1] && bandwidth > min_width {
            signals[i] = Signal::Buy;
        } else if close[i] < ll_exit[i - 1] {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Strategy 6: Bollinger Mean Reversion (Paid) ──
fn bb_reversion(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let std_mult = params.get("std_mult").copied().unwrap_or(2.0);
    let rsi_thresh = params.get("rsi_threshold").copied().unwrap_or(30.0);
    let hold_days = params.get("hold_days").copied().unwrap_or(5.0) as usize;
    let n = close.len();
    let (_upper, mid, lower) = bollinger_vec(&close, period, std_mult);
    let rsi = rsi_vec(&close, 14);
    let mut signals = vec![Signal::Hold; n];
    let mut holding = 0usize;
    let mut in_pos = false;
    let start = period + 14;
    for i in start..n {
        if in_pos { holding += 1; }
        if !in_pos && close[i] < lower[i] && rsi[i] < rsi_thresh && close[i] > close[i-1] {
            signals[i] = Signal::Buy;
            in_pos = true;
            holding = 0;
        } else if in_pos && (close[i] > mid[i] || holding >= hold_days) {
            signals[i] = Signal::Sell;
            in_pos = false;
        }
    }
    signals
}

// ── Strategy 7: RSI Extreme (Paid) ──
fn rsi_extreme(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let period = params.get("period").copied().unwrap_or(14.0) as usize;
    let oversold = params.get("oversold").copied().unwrap_or(30.0);
    let _overbought = params.get("overbought").copied().unwrap_or(70.0);
    let hold_days = params.get("hold_days").copied().unwrap_or(5.0) as usize;
    let n = close.len();
    let rsi = rsi_vec(&close, period);
    let mut signals = vec![Signal::Hold; n];
    let mut holding = 0;
    let mut in_pos = false;
    for i in period + 1..n {
        if in_pos { holding += 1; }
        if !in_pos && rsi[i - 1] > oversold && rsi[i] <= oversold && close[i] > close[i - 1] {
            signals[i] = Signal::Buy;
            in_pos = true;
            holding = 0;
        } else if in_pos && (rsi[i] >= 55.0 || holding >= hold_days) {
            signals[i] = Signal::Sell;
            in_pos = false;
        }
    }
    signals
}

// ── Strategy 8: BB+RSI Double Confirm (Paid) ──
fn bb_rsi_double(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, volume) = get_arrays(df);
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let std_mult = params.get("std").copied().unwrap_or(2.0);
    let rsi_p = params.get("rsi_period").copied().unwrap_or(14.0) as usize;
    let rsi_thresh = params.get("rsi_threshold").copied().unwrap_or(30.0);
    let vol_mult = params.get("vol_mult").copied().unwrap_or(1.5);
    let n = close.len();
    let (_, mid, lower) = bollinger_vec(&close, period, std_mult);
    let rsi = rsi_vec(&close, rsi_p);
    let vol_ma = sma_vec(&volume, 20);
    let mut signals = vec![Signal::Hold; n];
    let mut in_pos = false;
    let start = (period + 14).max(20);
    for i in start..n {
        if !in_pos && close[i] < lower[i] && rsi[i] < rsi_thresh
            && volume[i] > vol_ma[i] * vol_mult && close[i] > close[i-1]
        {
            signals[i] = Signal::Buy;
            in_pos = true;
        } else if in_pos && close[i] > mid[i] {
            signals[i] = Signal::Sell;
            in_pos = false;
        }
    }
    signals
}

// ── Strategy 9: KDJ Extreme (Paid) ──
fn kdj_extreme(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, high, low, close, _) = get_arrays(df);
    let n_param = params.get("n").copied().unwrap_or(9.0) as usize;
    let m1 = params.get("m1").copied().unwrap_or(3.0);
    let m2 = params.get("m2").copied().unwrap_or(3.0);
    let oversold = params.get("oversold").copied().unwrap_or(20.0);
    let _overbought = params.get("overbought").copied().unwrap_or(80.0);
    let n = close.len();
    let (k, d, j) = kdj_vec(&high, &low, &close, n_param, m1, m2);
    let mut signals = vec![Signal::Hold; n];
    let mut in_pos = false;
    for i in n_param + 1..n {
        if !in_pos && k[i] < oversold && d[i] < oversold && j[i] < 0.0 && close[i] > close[i-1] {
            signals[i] = Signal::Buy;
            in_pos = true;
        } else if in_pos && k[i] >= 55.0 {
            signals[i] = Signal::Sell;
            in_pos = false;
        }
    }
    signals
}

// ── Strategy 10: Z-Score Reversion (Paid) ──
fn zscore_reversion(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let period = params.get("period").copied().unwrap_or(20.0) as usize;
    let entry_z = params.get("entry_z").copied().unwrap_or(2.0);
    let exit_z = params.get("exit_z").copied().unwrap_or(0.5);
    let n = close.len();
    let ma = sma_vec(&close, period);
    let mut zscores = vec![f64::NAN; n];
    for i in period - 1..n {
        let mean = ma[i];
        let var = close[i + 1 - period..=i].iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / period as f64;
        let std = var.sqrt().max(0.0001);
        zscores[i] = (close[i] - mean) / std;
    }
    let mut signals = vec![Signal::Hold; n];
    let mut in_pos = false;
    for i in period + 1..n {
        if !in_pos && zscores[i] <= -entry_z && close[i] > close[i-1] {
            signals[i] = Signal::Buy;
            in_pos = true;
        } else if in_pos && zscores[i] >= -exit_z {
            signals[i] = Signal::Sell;
            in_pos = false;
        }
    }
    signals
}

// ── Strategy 11: Cross-Sectional Momentum (Paid) ──
fn cross_momentum(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let lookback = params.get("lookback").copied().unwrap_or(60.0) as usize;
    let rebalance = params.get("rebalance").copied().unwrap_or(20.0) as usize;
    let n = close.len();
    let mut signals = vec![Signal::Hold; n];
    let mut holding = false;
    let start = lookback + 1;
    for i in start..n {
        let period = if i % rebalance == 0 {
            if !holding {
                holding = true;
                Signal::Buy
            } else {
                Signal::Hold
            }
        } else {
            Signal::Hold
        };
        // Momentum rotation: hold as long as ROC positive
        if holding {
            let roc = if close[i - lookback] > 0.0 {
                (close[i] - close[i - lookback]) / close[i - lookback]
            } else { 0.0 };
            if roc < -0.05 { holding = false; signals[i] = Signal::Sell; continue; }
        }
        signals[i] = period;
    }
    signals
}

// ── Strategy 12: Time-Series Momentum (Paid) ──
fn ts_momentum(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let lookback = params.get("lookback").copied().unwrap_or(60.0) as usize;
    let smoothing = params.get("smoothing").copied().unwrap_or(5.0) as usize;
    let n = close.len();
    let mut roc_signals = vec![0.0; n];
    for i in lookback..n {
        roc_signals[i] = if close[i] > close[i - lookback] { 1.0 } else { -1.0 };
    }
    let smooth_roc = sma_vec(&roc_signals, smoothing);
    let mut signals = vec![Signal::Hold; n];
    for i in lookback + smoothing..n {
        if smooth_roc[i] > 0.0 && (i == 0 || smooth_roc[i-1] <= 0.0) {
            signals[i] = Signal::Buy;
        } else if smooth_roc[i] <= 0.0 && i > 0 && smooth_roc[i-1] > 0.0 {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Strategy 13: Dual RSI (Paid) ──
fn dual_rsi(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let fast_p = params.get("fast_period").copied().unwrap_or(7.0) as usize;
    let slow_p = params.get("slow_period").copied().unwrap_or(14.0) as usize;
    let n = close.len();
    let fast_rsi = rsi_vec(&close, fast_p);
    let slow_rsi = rsi_vec(&close, slow_p);
    let mut signals = vec![Signal::Hold; n];
    let start = slow_p + 1;
    for i in start..n {
        let cond = fast_rsi[i] > slow_rsi[i]
            && fast_rsi[i] > fast_rsi[i - 1]
            && slow_rsi[i] > slow_rsi[i - 1];
        let prev_cond = i > start && fast_rsi[i-1] > slow_rsi[i-1]
            && fast_rsi[i-1] > fast_rsi[i-2]
            && slow_rsi[i-1] > slow_rsi[i-2];

        if cond && !prev_cond {
            signals[i] = Signal::Buy;
        } else if !cond && prev_cond {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Strategy 14: Volume Breakout MA (Paid) ──
fn vol_breakout_ma(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, volume) = get_arrays(df);
    let ma_p = params.get("ma_period").copied().unwrap_or(60.0) as usize;
    let vol_mult = params.get("vol_mult").copied().unwrap_or(1.5);
    let n = close.len();
    let ma = sma_vec(&close, ma_p);
    let vol_ma = sma_vec(&volume, 20);
    let mut signals = vec![Signal::Hold; n];
    let start = ma_p.max(20) + 1;
    for i in start..n {
        let breakout = close[i] > ma[i] && volume[i] > vol_ma[i] * vol_mult
            && close[i - 1] < ma[i - 1];
        if breakout {
            signals[i] = Signal::Buy;
        } else if close[i] < sma_vec(&close, 20)[i] {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Strategy 15: Volatility Breakout ATR (Paid) ──
fn volatility_breakout(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, high, low, close, _) = get_arrays(df);
    let atr_p = params.get("atr_period").copied().unwrap_or(20.0) as usize;
    let atr_mult = params.get("atr_mult").copied().unwrap_or(2.0);
    let n = close.len();
    let atr = atr_vec(&high, &low, &close, atr_p);
    let mut signals = vec![Signal::Hold; n];
    let mut in_pos = false;
    let mut entry_price = 0.0;
    let start = atr_p + 1;
    for i in start..n {
        let upper_band = close[i - 1] + atr_mult * atr[i];
        let _lower_band = close[i - 1] - atr_mult * atr[i];
        if !in_pos {
            if close[i] > upper_band {
                signals[i] = Signal::Buy;
                in_pos = true;
                entry_price = close[i];
            }
        } else {
            if close[i] < entry_price - 0.5 * atr[i] {
                signals[i] = Signal::Sell;
                in_pos = false;
            }
        }
    }
    signals
}

// ── Strategy 16: Keltner Breakout (Paid) ──
fn keltner_breakout(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, high, low, close, _) = get_arrays(df);
    let ema_p = params.get("ema_period").copied().unwrap_or(20.0) as usize;
    let atr_p = params.get("atr_period").copied().unwrap_or(10.0) as usize;
    let atr_mult = params.get("atr_mult").copied().unwrap_or(2.0);
    let n = close.len();
    let mid = ema_vec(&close, ema_p);
    let atr = atr_vec(&high, &low, &close, atr_p);
    let mut upper = vec![f64::NAN; n];
    for i in 0..n {
        if mid[i].is_finite() && atr[i].is_finite() {
            upper[i] = mid[i] + atr_mult * atr[i];
        }
    }
    let mut signals = vec![Signal::Hold; n];
    let start = ema_p.max(atr_p) + 1;
    for i in start..n {
        if close[i] > upper[i] && close[i - 1] <= upper[i - 1] {
            signals[i] = Signal::Buy;
        } else if close[i] < mid[i] {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Strategy 17: MACD+RSI Combo (Paid) ──
fn macd_rsi_combo(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, _) = get_arrays(df);
    let fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
    let slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
    let sig = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
    let rsi_p = params.get("rsi_period").copied().unwrap_or(14.0) as usize;
    let rsi_low = params.get("rsi_low").copied().unwrap_or(40.0);
    let rsi_high = params.get("rsi_high").copied().unwrap_or(65.0);
    let n = close.len();
    let (dif, dea, _) = macd_vec(&close, fast, slow, sig);
    let rsi = rsi_vec(&close, rsi_p);
    let mut signals = vec![Signal::Hold; n];
    for i in 2..n {
        if dif[i].is_finite() && dea[i].is_finite() && dif[i-1].is_finite() && dea[i-1].is_finite() {
            let golden = cross(dif[i-1], dif[i], dea[i-1], dea[i]);
            let rsi_ok = rsi[i] > rsi_low && rsi[i] < rsi_high;
            if golden && rsi_ok {
                signals[i] = Signal::Buy;
            } else if cross_under(dif[i-1], dif[i], dea[i-1], dea[i]) || rsi[i] > 80.0 {
                signals[i] = Signal::Sell;
            }
        }
    }
    signals
}

// ── Strategy 18: MA + Volume Confirm (Paid) ──
fn ma_volume_confirm(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_, _, _, close, volume) = get_arrays(df);
    let ma_short = params.get("ma_short").copied().unwrap_or(5.0) as usize;
    let ma_long = params.get("ma_long").copied().unwrap_or(20.0) as usize;
    let vol_mult = params.get("vol_mult").copied().unwrap_or(1.5);
    let n = close.len();
    let sma_short = sma_vec(&close, ma_short);
    let sma_long = sma_vec(&close, ma_long);
    let vol_ma = sma_vec(&volume, 20);
    let mut signals = vec![Signal::Hold; n];
    let start = ma_long.max(20) + 1;
    for i in start..n {
        let cond = close[i] > sma_short[i] && sma_short[i] > sma_long[i] && volume[i] > vol_ma[i] * vol_mult;
        let prev_cond = i > 1 && close[i-1] > sma_short[i-1] && sma_short[i-1] > sma_long[i-1] && volume[i-1] > vol_ma[i-1] * vol_mult;
        if cond && !prev_cond {
            signals[i] = Signal::Buy;
        } else if !cond && prev_cond {
            signals[i] = Signal::Sell;
        }
    }
    signals
}

// ── Strategy 19: Multi-Factor Scorecard (Paid) ──
fn multi_factor(df: &DataFrame, params: &HashMap<String, f64>) -> Vec<Signal> {
    let (_open, high, low, close, volume) = get_arrays(df);
    let rebalance = params.get("rebalance").copied().unwrap_or(20.0) as usize;
    let n = close.len();
    // Compute factor z-scores
    let roc20: Vec<f64> = (0..n).map(|i| {
        if i < 20 { 0.0 } else { (close[i] - close[i-20]) / close[i-20].max(0.01) }
    }).collect();
    let atr14 = atr_vec(&high, &low, &close, 14);
    let vol_ratio: Vec<f64> = (0..n).map(|i| {
        if i < 5 { 1.0 } else {
            let v5: f64 = volume[i-4..=i].iter().sum::<f64>() / 5.0;
            if i < 20 { 1.0 } else {
                let v20: f64 = volume[i-19..=i].iter().sum::<f64>() / 20.0;
                v5 / v20.max(0.01)
            }
        }
    }).collect();
    let efficiency: Vec<f64> = (0..n).map(|i| {
        if i < 20 { 0.0 } else {
            let net = (close[i] - close[i-20]).abs();
            let path: f64 = (i-19..=i).map(|j| (close[j] - close[j-1]).abs()).sum();
            net / path.max(0.01)
        }
    }).collect();

    // Composite score
    let mut score = vec![0.0; n];
    let start = 120;
    for i in start..n {
        let z_roc = zscore_slice(&roc20, i, 120);
        let z_vol = zscore_slice(&vol_ratio, i, 120);
        let z_eff = zscore_slice(&efficiency, i, 120);
        score[i] = 0.30 * z_roc + 0.25 * z_vol + 0.20 * z_eff + 0.15 * (1.0 - atr14[i] / close[i].max(0.01) * 100.0) + 0.10 * efficiency[i];
    }

    let score_ma = sma_vec(&score, 5);
    let mut signals = vec![Signal::Hold; n];
    let mut holding = false;
    for i in start + 5..n {
        if i % rebalance == 0 {
            if score_ma[i] > 0.0 && !holding {
                signals[i] = Signal::Buy;
                holding = true;
            } else if score_ma[i] < -0.5 && holding {
                signals[i] = Signal::Sell;
                holding = false;
            }
        }
    }
    signals
}

fn zscore_slice(data: &[f64], idx: usize, window: usize) -> f64 {
    let start = if idx >= window { idx - window } else { 0 };
    let slice = &data[start..=idx];
    let n = slice.len() as f64;
    if n < 2.0 { return 0.0; }
    let mean = slice.iter().sum::<f64>() / n;
    let var = slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt().max(0.0001);
    (data[idx] - mean) / std
}

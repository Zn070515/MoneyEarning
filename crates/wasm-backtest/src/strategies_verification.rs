//! 20策略模板完整回测验证 — A股2015-2024模拟数据
//!
//! Run: cargo test --package wasm-backtest -- strategies_verification --nocapture

#[cfg(test)]
mod verification {
    use std::collections::HashMap;
    use crate::{BacktestConfig, run_with_template};

    /// Generate realistic A-share-like OHLCV data (~2500 days ≈ 10 years)
    /// Simulates: 2015 bull → 2015 crash → 2016-2017 recovery → 2018 bear →
    ///            2019-2020 rally → 2021 structural → 2022-2023 bear → 2024 recovery
    fn generate_ashare_data(days: usize) -> wasm_core::DataFrame {
        let mut records = Vec::with_capacity(days);
        let mut price = 50.0;
        let mut vol_base = 0.015; // daily vol ~1.5%

        for i in 0..days {
            // Regime-based drift and vol
            let (drift, vol) = match i {
                // 2015 H1: bull market (days 0-125)
                d if d < 125 => (0.003, 0.022),
                // 2015 H2: crash (days 125-250)
                d if d < 250 => (-0.004, 0.035),
                // 2016-2017: slow recovery (days 250-750)
                d if d < 750 => (0.0008, 0.015),
                // 2018: bear market (days 750-1000)
                d if d < 1000 => (-0.0015, 0.020),
                // 2019-2020: strong rally (days 1000-1500)
                d if d < 1500 => (0.0015, 0.018),
                // 2021: structural/rotational (days 1500-1750)
                d if d < 1750 => (0.0005, 0.016),
                // 2022-2023: bear (days 1750-2250)
                d if d < 2250 => (-0.0012, 0.022),
                // 2024: recovery (days 2250+)
                _ => (0.0010, 0.017),
            };

            // Use a deterministic pseudo-random based on index
            let r1 = (i as f64 * 127.1).sin();
            let r2 = (i as f64 * 311.7).cos();
            let noise = (r1 * 0.7 + r2 * 0.3) * vol;

            let daily_return = drift + noise;
            let close = price * (1.0 + daily_return);
            let open = price * (1.0 + noise * 0.3);
            let high = close.max(open) * (1.0 + r1.abs() * vol * 0.5);
            let low = close.min(open) * (1.0 - r2.abs() * vol * 0.5);

            let volume = 5_000_000.0 + r1.abs() * 15_000_000.0
                + if daily_return.abs() > 0.03 { 10_000_000.0 } else { 0.0 }; // high vol on big moves

            records.push(wasm_core::OHLCV {
                trade_date: (20250101 - (days - i) as u64).to_string(),
                open,
                high,
                low,
                close,
                volume,
                amount: None,
                turnover: None,
            });

            price = close;
            vol_base = vol_base * 0.999 + vol * 0.001;
        }
        wasm_core::DataFrame::new(&records)
    }

    /// Full verification of all 20 strategies plus a quick bench
    #[test]
    fn verify_all_20_strategies() {
        let df = generate_ashare_data(2500); // ~10 years of daily data
        let config = BacktestConfig::default();

        let strategies: Vec<(&str, HashMap<String, f64>)> = vec![
            // Free - Trend following
            ("ma_cross", HashMap::from([("fast".into(), 5.0), ("slow".into(), 20.0)])),
            ("macd_cross", HashMap::from([("fast".into(), 12.0), ("slow".into(), 26.0), ("signal".into(), 9.0)])),
            ("triple_ma", HashMap::from([("short".into(), 5.0), ("mid".into(), 20.0), ("long".into(), 60.0)])),
            ("turtle", HashMap::from([("entry_period".into(), 20.0), ("exit_period".into(), 10.0), ("atr_period".into(), 20.0), ("atr_stop".into(), 2.0)])),
            ("donchian", HashMap::from([("upper_period".into(), 20.0), ("lower_period".into(), 10.0)])),
            // Paid - Mean reversion
            ("bb_reversion", HashMap::from([("period".into(), 20.0), ("std_mult".into(), 2.0), ("rsi_threshold".into(), 30.0), ("hold_days".into(), 5.0)])),
            ("rsi_extreme", HashMap::from([("period".into(), 14.0), ("oversold".into(), 30.0), ("overbought".into(), 70.0), ("hold_days".into(), 5.0)])),
            ("bb_rsi_double", HashMap::from([("period".into(), 20.0), ("std".into(), 2.0), ("rsi_period".into(), 14.0), ("rsi_threshold".into(), 30.0), ("vol_mult".into(), 1.5)])),
            ("kdj_extreme", HashMap::from([("n".into(), 9.0), ("m1".into(), 3.0), ("m2".into(), 3.0), ("oversold".into(), 20.0), ("overbought".into(), 80.0)])),
            ("zscore_reversion", HashMap::from([("period".into(), 20.0), ("entry_z".into(), 2.0), ("exit_z".into(), 0.5)])),
            // Paid - Momentum
            ("cross_momentum", HashMap::from([("lookback".into(), 60.0), ("top_k".into(), 20.0), ("rebalance".into(), 20.0)])),
            ("ts_momentum", HashMap::from([("lookback".into(), 60.0), ("smoothing".into(), 5.0)])),
            ("dual_rsi", HashMap::from([("fast_period".into(), 7.0), ("slow_period".into(), 14.0)])),
            // Paid - Breakout
            ("vol_breakout_ma", HashMap::from([("ma_period".into(), 60.0), ("vol_mult".into(), 1.5)])),
            ("volatility_breakout", HashMap::from([("atr_period".into(), 20.0), ("atr_mult".into(), 2.0)])),
            ("keltner_breakout", HashMap::from([("ema_period".into(), 20.0), ("atr_period".into(), 10.0), ("atr_mult".into(), 2.0)])),
            // Paid - Composite
            ("macd_rsi_combo", HashMap::from([("macd_fast".into(), 12.0), ("macd_slow".into(), 26.0), ("macd_signal".into(), 9.0), ("rsi_period".into(), 14.0), ("rsi_low".into(), 40.0), ("rsi_high".into(), 65.0)])),
            ("ma_volume_confirm", HashMap::from([("ma_short".into(), 5.0), ("ma_long".into(), 20.0), ("vol_mult".into(), 1.5)])),
            ("multi_factor", HashMap::from([("top_n".into(), 30.0), ("rebalance".into(), 20.0)])),
            ("walk_forward_adaptive", HashMap::from([("fast".into(), 5.0), ("slow".into(), 20.0)])),
        ];

        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║  20 Strategy Template Verification — A-share 2015-2024 Sim  ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        println!("║ {:^20} | {:>6} | {:>7} | {:>7} | {:>6} | {:>6} ║",
            "Strategy", "Trades", "Return%", "Sharpe", "MaxDD%", "Win%");
        println!("╠══════════════════════════════════════════════════════════════╣");

        for (name, params) in &strategies {
            let result = run_with_template(&df, name, params, &config);
            let ret_pct = result.total_return * 100.0;
            let dd_pct = result.max_drawdown * 100.0;
            let wr_pct = result.win_rate * 100.0;

            println!("║ {:^20} | {:>6} | {:>+7.1} | {:>7.2} | {:>+6.1} | {:>5.1}% ║",
                name, result.total_trades, ret_pct, result.sharpe_ratio, dd_pct, wr_pct);

            // Verify result integrity
            assert!(result.total_trades < 10000, "{}: unrealistic trade count", name);
            assert!(result.sharpe_ratio.is_finite() || result.sharpe_ratio == 0.0,
                "{}: non-finite Sharpe", name);
            assert!(result.max_drawdown >= -1.0 && result.max_drawdown <= 0.0,
                "{}: max_drawdown out of range: {}", name, result.max_drawdown);
            assert!(result.win_rate >= 0.0 && result.win_rate <= 1.0,
                "{}: win_rate out of range: {}", name, result.win_rate);
            assert!(result.equity_curve.len() >= 1 && result.equity_curve.len() <= 500,
                "{}: equity curve length out of range: {}", name, result.equity_curve.len());
        }

        println!("╚══════════════════════════════════════════════════════════════╝");
        println!("  All 20 strategies verified successfully.");
    }

    /// Quick sanity: each strategy at least produces valid signals
    #[test]
    fn quick_smoke_all_strategies() {
        let df = generate_ashare_data(500);
        let config = BacktestConfig::default();
        let strategies = crate::strategies::list_strategies();

        assert_eq!(strategies.len(), 20, "Should have exactly 20 strategy templates");

        for meta in &strategies {
            let params: HashMap<String, f64> = meta.params.iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect();
            let result = run_with_template(&df, meta.name, &params, &config);
            assert!(result.equity_curve.len() == 500,
                "{}: equity curve wrong length: {} != 500", meta.name, result.equity_curve.len());
            assert!(result.sharpe_ratio.is_finite() || result.total_trades == 0,
                "{}: non-finite Sharpe with trades", meta.name);
        }
    }
}

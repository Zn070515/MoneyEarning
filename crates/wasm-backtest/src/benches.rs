//! Performance benchmarks for backtest engine
//!
//! Run with: cargo test --package wasm-backtest -- benches --nocapture

#[cfg(test)]
mod benches {
    use std::time::Instant;
    use std::collections::HashMap;
    use wasm_core::{DataFrame, OHLCV};

    fn fake_df(n: usize) -> DataFrame {
        let mut records = Vec::with_capacity(n);
        for i in 0..n {
            let phase = i as f64 * 0.1;
            let price = 100.0 + (phase).sin() * 20.0 + (phase * 1.7).cos() * 8.0;
            records.push(OHLCV {
                trade_date: (1000000 + i as u64 * 86400).to_string(),
                open: price - 0.3,
                high: price + 0.5 + (phase * 3.0).sin().abs() * 1.5,
                low: price - 0.5 - (phase * 2.7).cos().abs() * 1.5,
                close: price,
                volume: 1_000_000.0 + (phase * 5.0).sin().abs() * 5_000_000.0,
                amount: None,
                turnover: None,
            });
        }
        DataFrame::new(&records)
    }

    #[test]
    fn bench_sma_cross_10k() {
        let df = fake_df(10_000);
        let params = HashMap::from([
            ("fast".into(), 5.0),
            ("slow".into(), 20.0),
        ]);
        let config = crate::BacktestConfig::default();
        let start = Instant::now();
        let result = crate::run_with_template(&df, "sma_cross", &params, &config);
        let elapsed = start.elapsed();
        println!(
            "  SMA Cross 10K: {:?} | Sharpe={:.2} Trades={}",
            elapsed, result.sharpe_ratio, result.total_trades
        );
        assert!(elapsed.as_secs_f64() < 2.0, "Backtest too slow: {:?}", elapsed);
    }

    #[test]
    fn bench_macd_cross_10k() {
        let df = fake_df(10_000);
        let params = HashMap::from([
            ("fast".into(), 12.0),
            ("slow".into(), 26.0),
            ("signal".into(), 9.0),
        ]);
        let config = crate::BacktestConfig::default();
        let start = Instant::now();
        let result = crate::run_with_template(&df, "macd_cross", &params, &config);
        let elapsed = start.elapsed();
        println!(
            "  MACD Cross 10K: {:?} | Sharpe={:.2} Trades={}",
            elapsed, result.sharpe_ratio, result.total_trades
        );
        assert!(elapsed.as_secs_f64() < 2.0, "Backtest too slow: {:?}", elapsed);
    }

    #[test]
    fn bench_5_strategies_10k() {
        let df = fake_df(10_000);
        let strategies = [
            ("sma_cross", vec![("fast", 5.0), ("slow", 20.0)]),
            ("macd_cross", vec![("fast", 12.0), ("slow", 26.0), ("signal", 9.0)]),
            ("turtle", vec![("entry_period", 20.0), ("exit_period", 10.0), ("atr_period", 20.0)]),
            ("bollinger", vec![("period", 20.0), ("std", 2.0)]),
            ("donchian", vec![("upper_period", 20.0), ("lower_period", 10.0)]),
        ];
        let config = crate::BacktestConfig::default();
        let start = Instant::now();
        for (name, params) in &strategies {
            let p: HashMap<String, f64> = params.iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect();
            let r = crate::run_with_template(&df, name, &p, &config);
            assert!(r.total_trades < 1_000_000); // sanity: can't have that many trades
        }
        let elapsed = start.elapsed();
        println!("  5 strategies on 10K: {:?}", elapsed);
        assert!(elapsed.as_secs_f64() < 5.0, "5 backtests too slow: {:?}", elapsed);
    }
}

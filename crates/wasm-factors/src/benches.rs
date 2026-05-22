//! Performance benchmarks for Alpha158 factor computation
//!
//! Run with: cargo test --package wasm-factors -- benches --nocapture

#[cfg(test)]
mod benches {
    use std::time::Instant;
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
    fn bench_all_factors_1k() {
        let df = fake_df(1_000);
        let start = Instant::now();
        let results = crate::compute_all(&df);
        let elapsed = start.elapsed();
        println!(
            "  All {} factors on 1K candles: {:?} ({:.0} factor-rows/ms)",
            results.len(),
            elapsed,
            1000.0 / elapsed.as_secs_f64().max(0.001)
        );
        assert!(results.len() >= 140, "Expected >=140 factors, got {}", results.len());
        assert!(elapsed.as_secs_f64() < 10.0, "Factor computation too slow: {:?}", elapsed);
    }

    #[test]
    fn bench_all_factors_5k() {
        let df = fake_df(5_000);
        let start = Instant::now();
        let results = crate::compute_all(&df);
        let elapsed = start.elapsed();
        println!(
            "  All {} factors on 5K candles: {:?} ({:.1} K factor-rows/s)",
            results.len(),
            elapsed,
            5.0 / elapsed.as_secs_f64().max(0.001)
        );
        assert!(results.len() >= 140);
    }

    #[test]
    fn bench_single_factor_10k() {
        let df = fake_df(10_000);
        let start = Instant::now();
        for _ in 0..10 {
            crate::compute_by_name(&df, "rsi12").unwrap();
            crate::compute_by_name(&df, "macd_dif").unwrap();
            crate::compute_by_name(&df, "atr10").unwrap();
            crate::compute_by_name(&df, "ret20").unwrap();
            crate::compute_by_name(&df, "vol_ratio").unwrap();
        }
        let elapsed = start.elapsed();
        println!("  5 factors x10 on 10K: {:?}", elapsed);
        assert!(elapsed.as_secs_f64() < 5.0);
    }
}

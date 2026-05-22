//! Performance benchmarks for WASM indicator computation
//!
//! Run with: cargo test --package wasm-indicators -- benches --nocapture

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
    fn bench_macd_50k() {
        let df = fake_df(50_000);
        let params = std::collections::HashMap::from([
            ("fast".into(), 12.0),
            ("slow".into(), 26.0),
            ("signal".into(), 9.0),
        ]);
        let start = Instant::now();
        let result = crate::compute("macd", &df, &params).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(result.len(), 3);
        // DIF + DEA + HIST lines
        assert!(result[0].values.len() > 0);
        let kps = 50.0 / elapsed.as_secs_f64().max(0.001);
        println!("  MACD 50K candles: {:?} ({:.0} K candles/s)", elapsed, kps);
        assert!(elapsed.as_secs_f64() < 5.0, "MACD too slow: {:?}", elapsed);
    }

    #[test]
    fn bench_rsi_50k() {
        let df = fake_df(50_000);
        let params = std::collections::HashMap::from([("period".into(), 14.0)]);
        let start = Instant::now();
        let result = crate::compute("rsi", &df, &params).unwrap();
        let elapsed = start.elapsed();
        assert!(!result.is_empty());
        let kps = 50.0 / elapsed.as_secs_f64().max(0.001);
        println!("  RSI  50K candles: {:?} ({:.0} K candles/s)", elapsed, kps);
        assert!(elapsed.as_secs_f64() < 5.0, "RSI too slow: {:?}", elapsed);
    }

    #[test]
    fn bench_8_indicators_5k() {
        let df = fake_df(5_000);
        let indicators = [
            ("rsi", vec![("period", 14.0)]),
            ("atr", vec![("period", 14.0)]),
            ("bb", vec![("period", 20.0), ("std", 2.0)]),
            ("sma", vec![("period", 20.0)]),
            ("ema", vec![("period", 12.0)]),
            ("kdj", vec![("n", 9.0), ("m1", 3.0), ("m2", 3.0)]),
            ("obv", vec![]),
            ("cci", vec![("period", 14.0)]),
        ];
        let start = Instant::now();
        for (name, params) in &indicators {
            let p: std::collections::HashMap<String, f64> = params.iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect();
            crate::compute(name, &df, &p).unwrap();
        }
        let elapsed = start.elapsed();
        println!("  8 indicators on 5K: {:?}", elapsed);
    }

    #[test]
    fn bench_sma_500_x100() {
        let df = fake_df(500);
        let params = std::collections::HashMap::from([("period".into(), 20.0)]);
        let start = Instant::now();
        for _ in 0..100 {
            crate::compute("sma", &df, &params).unwrap();
        }
        let elapsed = start.elapsed();
        println!("  SMA 500 candles x100: {:?}", elapsed);
    }
}

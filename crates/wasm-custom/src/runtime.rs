//! ME Script Runtime — executes the computation IR against a DataFrame
//! Evaluates each IrNode to produce a Vec<f64> series, then chains them.

use std::collections::HashMap;
use wasm_core::DataFrame;
use crate::ast::{IrProgram, IrNode, IrBinOp, IrUnaryOp, SeriesFn};
use crate::functions;

pub struct Runtime {
    cache: Vec<Option<Vec<f64>>>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime { cache: Vec::new() }
    }

    /// Execute an IR program against a DataFrame, returning named output series
    pub fn execute(&mut self, ir: &IrProgram, df: &DataFrame, params: &HashMap<String, f64>) -> HashMap<String, Vec<f64>> {
        let n = df.len();
        self.cache = vec![None; ir.nodes.len()];

        // Pre-fill params
        let mut final_params = HashMap::new();
        for p in &ir.params {
            let val = params.get(&p.name).copied().unwrap_or(p.default);
            final_params.insert(p.name.clone(), val);
        }

        // Evaluate all nodes on demand
        for (_name, node_idx) in &ir.outputs {
            self.eval_node(*node_idx, ir, df, n);
        }

        let mut results = HashMap::new();
        for (name, node_idx) in &ir.outputs {
            if let Some(series) = self.cache[*node_idx].clone() {
                results.insert(name.clone(), series);
            }
        }
        results
    }

    /// Execute and return a single named output
    pub fn execute_named(&mut self, ir: &IrProgram, df: &DataFrame, _params: &HashMap<String, f64>, name: &str) -> Option<Vec<f64>> {
        let n = df.len();
        self.cache = vec![None; ir.nodes.len()];
        for (out_name, node_idx) in &ir.outputs {
            if out_name == name {
                self.eval_node(*node_idx, ir, df, n);
                return self.cache[*node_idx].clone();
            }
        }
        None
    }

    fn eval_node(&mut self, idx: usize, ir: &IrProgram, df: &DataFrame, n: usize) {
        if self.cache[idx].is_some() {
            return;
        }

        let result = match &ir.nodes[idx] {
            IrNode::Input { name, .. } => {
                df.column(name).map(|c| c.to_f64_vec()).unwrap_or_else(|| vec![0.0; n])
            }
            IrNode::Constant(val) => {
                vec![*val; n]
            }
            IrNode::SeriesOp { op, inputs, period } => {
                self.eval_series_op(op, inputs, period, ir, df, n)
            }
            IrNode::BinaryOp { op, left, right } => {
                self.eval_binary(*op, *left, *right, ir, df, n)
            }
            IrNode::UnaryOp { op, input } => {
                self.eval_unary(op, *input, ir, df, n)
            }
            IrNode::Shift { input, offset } => {
                self.eval_node(*input, ir, df, n);
                let data = self.cache[*input].as_ref().unwrap();
                functions::ref_n(data, offset.abs() as usize)
            }
            IrNode::Conditional { cond, then_val, else_val } => {
                self.eval_node(*cond, ir, df, n);
                self.eval_node(*then_val, ir, df, n);
                self.eval_node(*else_val, ir, df, n);
                let c = self.cache[*cond].as_ref().unwrap();
                let t = self.cache[*then_val].as_ref().unwrap();
                let e = self.cache[*else_val].as_ref().unwrap();
                let mut result = vec![0.0; n];
                for i in 0..n {
                    result[i] = if c[i] > 0.5 { t[i] } else { e[i] };
                }
                result
            }
        };

        self.cache[idx] = Some(result);
    }

    fn resolve_args(&mut self, inputs: &[usize], ir: &IrProgram, df: &DataFrame, n: usize) -> Vec<Vec<f64>> {
        inputs.iter().map(|&idx| {
            self.eval_node(idx, ir, df, n);
            self.cache[idx].clone().unwrap_or_else(|| vec![0.0; n])
        }).collect()
    }

    #[allow(dead_code)]
    fn get_period(&self, inputs: &[usize], arg_idx: usize, ir: &IrProgram) -> usize {
        inputs.get(arg_idx).and_then(|&idx| {
            if let IrNode::Constant(v) = &ir.nodes[idx] {
                Some(*v as usize)
            } else {
                None
            }
        }).unwrap_or(20)
    }

    fn eval_series_op(&mut self, op: &SeriesFn, inputs: &[usize], _period: &Option<usize>, ir: &IrProgram, df: &DataFrame, n: usize) -> Vec<f64> {
        let args = self.resolve_args(inputs, ir, df, n);
        let p = |i: usize, d: usize| -> usize {
            inputs.get(i).and_then(|&idx| {
                if let IrNode::Constant(v) = &ir.nodes[idx] { Some(*v as usize) } else { None }
            }).unwrap_or(d)
        };

        let one_arg = |idx: usize, default: usize| -> (&[f64], usize) {
            let period = p(idx, default);
            (&args[0], period)
        };

        match op {
            SeriesFn::Sma => { let (d, pr) = one_arg(1, 20); functions::sma(d, pr) }
            SeriesFn::Ema => { let (d, pr) = one_arg(1, 20); functions::ema(d, pr) }
            SeriesFn::Wma => { let (d, pr) = one_arg(1, 20); functions::wma(d, pr) }
            SeriesFn::Rma => { let (d, pr) = one_arg(1, 20); functions::rma(d, pr) }
            SeriesFn::Hma => { let (d, pr) = one_arg(1, 20); functions::hma(d, pr) }
            SeriesFn::Stdev => { let (d, pr) = one_arg(1, 20); functions::stdev(d, pr) }
            SeriesFn::ZScore => { let (d, pr) = one_arg(1, 20); functions::zscore(d, pr) }
            SeriesFn::Rsi => { let (d, pr) = one_arg(1, 14); functions::rsi(d, pr) }
            SeriesFn::Cci => {
                let pr = p(3, 14);
                functions::cci(&args[0], &args[1], &args[2], pr)
            }
            SeriesFn::Atr => {
                let pr = p(3, 14);
                functions::atr(&args[0], &args[1], &args[2], pr)
            }
            SeriesFn::Hhv => { let (d, pr) = one_arg(1, 20); functions::hhv(d, pr) }
            SeriesFn::Llv => { let (d, pr) = one_arg(1, 20); functions::llv(d, pr) }
            SeriesFn::Sum => { let (d, pr) = one_arg(1, 20); functions::sum(d, pr) }
            SeriesFn::Cross => {
                let above = if inputs.len() > 2 {
                    self.eval_node(inputs[2], ir, df, n);
                    self.cache[inputs[2]].as_ref().map(|v| v.last().copied().unwrap_or(1.0) > 0.0).unwrap_or(true)
                } else {
                    true
                };
                functions::cross(&args[0], &args[1], above)
            }
            SeriesFn::Ref => {
                let offset = p(1, 1);
                functions::ref_n(&args[0], offset)
            }
            SeriesFn::BarsLast => {
                let cond: Vec<bool> = args[0].iter().map(|&v| v > 0.5).collect();
                functions::barslast(&cond)
            }
            SeriesFn::BarsSince => {
                let cond: Vec<bool> = args[0].iter().map(|&v| v > 0.5).collect();
                functions::barssince(&cond)
            }
            SeriesFn::Count => {
                let pr = p(1, 20);
                let cond: Vec<bool> = args[0].iter().map(|&v| v > 0.5).collect();
                functions::count(&cond, pr)
            }
            SeriesFn::Every => {
                let pr = p(1, 20);
                let cond: Vec<bool> = args[0].iter().map(|&v| v > 0.5).collect();
                let result = functions::every(&cond, pr);
                result.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect()
            }
            SeriesFn::Exist => {
                let pr = p(1, 20);
                let cond: Vec<bool> = args[0].iter().map(|&v| v > 0.5).collect();
                let result = functions::exist(&cond, pr);
                result.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect()
            }
            SeriesFn::Filter => {
                let pr = p(1, 5);
                let cond: Vec<bool> = args[0].iter().map(|&v| v > 0.5).collect();
                let result = functions::filter(&cond, pr);
                result.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect()
            }
            SeriesFn::Abs => functions::vec_abs(&args[0]),
            SeriesFn::Min => functions::vec_min(&args[0], &args.get(1).unwrap_or(&args[0])),
            SeriesFn::Max => functions::vec_max(&args[0], &args.get(1).unwrap_or(&args[0])),
            SeriesFn::Sqrt => functions::vec_sqrt(&args[0]),
            SeriesFn::Log => functions::vec_log(&args[0]),
            SeriesFn::Ln => functions::vec_ln(&args[0]),
            SeriesFn::Exp => functions::vec_exp(&args[0]),
            SeriesFn::Round => functions::vec_round(&args[0]),
            SeriesFn::Ceil => functions::vec_ceil(&args[0]),
            SeriesFn::Floor => functions::vec_floor(&args[0]),
            SeriesFn::Sign => functions::vec_sign(&args[0]),
            SeriesFn::Sin => functions::vec_sin(&args[0]),
            SeriesFn::Cos => functions::vec_cos(&args[0]),
            SeriesFn::Tan => functions::vec_tan(&args[0]),
            SeriesFn::Between => {
                let a = args.get(1).unwrap_or(&args[0]);
                let b = args.get(2).unwrap_or(&args[0]);
                let r = functions::between(&args[0], a, b);
                r.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect()
            }
            SeriesFn::Range => {
                let a = args.get(1).unwrap_or(&args[0]);
                let b = args.get(2).unwrap_or(&args[0]);
                let r = functions::range(&args[0], a, b);
                r.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect()
            }
            SeriesFn::Obv => functions::obv(&args[0], &args.get(1).unwrap_or(&args[0])),
            SeriesFn::Roc => { let (d, pr) = one_arg(1, 20); functions::roc(d, pr) }
            SeriesFn::MacdDif => {
                let fast = p(1, 12);
                let slow = p(2, 26);
                let ema_f = functions::ema(&args[0], fast);
                let ema_s = functions::ema(&args[0], slow);
                let n = args[0].len();
                (0..n).map(|i| if ema_f[i].is_finite() && ema_s[i].is_finite() { ema_f[i] - ema_s[i] } else { f64::NAN }).collect()
            }
            SeriesFn::MacdDea => {
                let fast = p(1, 12);
                let slow = p(2, 26);
                let signal_p = p(3, 9);
                let ema_f = functions::ema(&args[0], fast);
                let ema_s = functions::ema(&args[0], slow);
                let n = args[0].len();
                let dif: Vec<f64> = (0..n).map(|i| if ema_f[i].is_finite() && ema_s[i].is_finite() { ema_f[i] - ema_s[i] } else { f64::NAN }).collect();
                functions::ema(&dif, signal_p)
            }
            SeriesFn::MacdHist => {
                let fast = p(1, 12);
                let slow = p(2, 26);
                let signal_p = p(3, 9);
                let ema_f = functions::ema(&args[0], fast);
                let ema_s = functions::ema(&args[0], slow);
                let n = args[0].len();
                let dif: Vec<f64> = (0..n).map(|i| if ema_f[i].is_finite() && ema_s[i].is_finite() { ema_f[i] - ema_s[i] } else { f64::NAN }).collect();
                let dea = functions::ema(&dif, signal_p);
                (0..n).map(|i| if dif[i].is_finite() && dea[i].is_finite() { (dif[i] - dea[i]) * 2.0 } else { f64::NAN }).collect()
            }
            SeriesFn::KdjK | SeriesFn::KdjD | SeriesFn::KdjJ => {
                let period = p(3, 9);
                let m1 = p(4, 3).max(1);
                let m2 = p(5, 3).max(1);
                let n = args[0].len();
                let mut k_vals = vec![50.0; n];
                let mut d_vals = vec![50.0; n];
                let mut j_vals = vec![50.0; n];
                for i in 0..n {
                    let start = if i >= period { i - period + 1 } else { 0 };
                    let hh = args[0][start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let ll = args[1][start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
                    let rsv = if (hh - ll).abs() > 1e-12 { (args[2][i] - ll) / (hh - ll) * 100.0 } else { 50.0 };
                    if i == 0 {
                        k_vals[i] = 50.0;
                        d_vals[i] = 50.0;
                    } else {
                        k_vals[i] = (k_vals[i - 1] * (m1 - 1) as f64 + rsv) / m1 as f64;
                        d_vals[i] = (d_vals[i - 1] * (m2 - 1) as f64 + k_vals[i]) / m2 as f64;
                    }
                    j_vals[i] = 3.0 * k_vals[i] - 2.0 * d_vals[i];
                }
                match op {
                    SeriesFn::KdjK => k_vals,
                    SeriesFn::KdjD => d_vals,
                    _ => j_vals,
                }
            }
            // Candle patterns — simplified boolean (0/1) output
            SeriesFn::CdlDoji => {
                let n = args[0].len();
                (0..n).map(|i| {
                    let body = (args[3][i] - args[0][i]).abs();
                    let range = args[1][i] - args[2][i];
                    if range > 1e-12 && body / range < 0.1 { 1.0 } else { 0.0 }
                }).collect()
            }
            SeriesFn::CdlHammer => {
                let n = args[0].len();
                (0..n).map(|i| {
                    let body = (args[3][i] - args[0][i]).abs();
                    let range = args[1][i] - args[2][i];
                    let lower_shadow = args[0][i].min(args[3][i]) - args[2][i];
                    if range > 1e-12 && body < range * 0.3 && lower_shadow > body * 2.0 { 1.0 } else { 0.0 }
                }).collect()
            }
            SeriesFn::CdlShootingStar => {
                let n = args[0].len();
                (0..n).map(|i| {
                    let body = (args[3][i] - args[0][i]).abs();
                    let range = args[1][i] - args[2][i];
                    let upper_shadow = args[1][i] - args[0][i].max(args[3][i]);
                    if range > 1e-12 && body < range * 0.3 && upper_shadow > body * 2.0 { 1.0 } else { 0.0 }
                }).collect()
            }
            SeriesFn::CdlEngulfing => {
                let n = args[0].len();
                let mut result = vec![0.0; n];
                for i in 1..n {
                    let prev_body = args[3][i - 1] - args[0][i - 1];
                    let curr_body = args[3][i] - args[0][i];
                    if prev_body < 0.0 && curr_body > 0.0 && curr_body.abs() > prev_body.abs()
                        && args[0][i] < args[2][i - 1] && args[3][i] > args[1][i - 1]
                    {
                        result[i] = 1.0; // bullish engulfing
                    } else if prev_body > 0.0 && curr_body < 0.0 && curr_body.abs() > prev_body.abs()
                        && args[0][i] > args[1][i - 1] && args[3][i] < args[2][i - 1]
                    {
                        result[i] = -1.0; // bearish engulfing
                    }
                }
                result
            }
            // Default: pass through first argument
            _ => args.first().cloned().unwrap_or_else(|| vec![0.0; n]),
        }
    }

    fn eval_binary(&mut self, op: IrBinOp, left: usize, right: usize, ir: &IrProgram, df: &DataFrame, n: usize) -> Vec<f64> {
        self.eval_node(left, ir, df, n);
        self.eval_node(right, ir, df, n);
        let l = self.cache[left].as_ref().unwrap();
        let r = self.cache[right].as_ref().unwrap();
        let len = l.len().min(r.len());
        let mut result = vec![0.0; len];
        for i in 0..len {
            result[i] = match op {
                IrBinOp::Add => l[i] + r[i],
                IrBinOp::Sub => l[i] - r[i],
                IrBinOp::Mul => l[i] * r[i],
                IrBinOp::Div => if r[i].abs() > 1e-12 { l[i] / r[i] } else { 0.0 },
                IrBinOp::Mod => l[i] % r[i],
                IrBinOp::Pow => l[i].powf(r[i]),
                IrBinOp::Eq => if (l[i] - r[i]).abs() < 1e-10 { 1.0 } else { 0.0 },
                IrBinOp::Neq => if (l[i] - r[i]).abs() >= 1e-10 { 1.0 } else { 0.0 },
                IrBinOp::Gt => if l[i] > r[i] { 1.0 } else { 0.0 },
                IrBinOp::Lt => if l[i] < r[i] { 1.0 } else { 0.0 },
                IrBinOp::Gte => if l[i] >= r[i] { 1.0 } else { 0.0 },
                IrBinOp::Lte => if l[i] <= r[i] { 1.0 } else { 0.0 },
                IrBinOp::And => if l[i] > 0.5 && r[i] > 0.5 { 1.0 } else { 0.0 },
                IrBinOp::Or => if l[i] > 0.5 || r[i] > 0.5 { 1.0 } else { 0.0 },
                IrBinOp::Xor => if (l[i] > 0.5) != (r[i] > 0.5) { 1.0 } else { 0.0 },
            };
        }
        result
    }

    fn eval_unary(&mut self, op: &IrUnaryOp, input: usize, ir: &IrProgram, df: &DataFrame, n: usize) -> Vec<f64> {
        self.eval_node(input, ir, df, n);
        let v = self.cache[input].as_ref().unwrap();
        match op {
            IrUnaryOp::Neg => v.iter().map(|&x| -x).collect(),
            IrUnaryOp::Not => v.iter().map(|&x| if x > 0.5 { 0.0 } else { 1.0 }).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::compiler::compile;
    use wasm_core::{DataFrame, OHLCV};

    fn test_df() -> DataFrame {
        let mut data = Vec::new();
        let mut price = 10.0;
        for i in 0..100 {
            data.push(OHLCV {
                trade_date: i.to_string(),
                open: price,
                high: price + 0.5,
                low: price - 0.5,
                close: price + 0.1,
                volume: 1000.0,
                amount: None,
                turnover: None,
            });
            price += 0.1;
        }
        DataFrame::new(&data)
    }

    fn run(source: &str, df: &DataFrame) -> HashMap<String, Vec<f64>> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();
        let mut errors = Vec::new();
        let ir = compile(&program, &mut errors).unwrap();
        let mut runtime = Runtime::new();
        runtime.execute(&ir, df, &HashMap::new())
    }

    #[test]
    fn test_run_sma() {
        let df = test_df();
        let results = run("plot ma5 = sma(close, 5);", &df);
        assert!(results.contains_key("ma5"));
        let ma5 = &results["ma5"];
        assert!(ma5.len() == 100);
        // SMA values should be around the price (starts at 10, ends at ~20)
        assert!(ma5[99] > 15.0);
    }

    #[test]
    fn test_run_cross() {
        let df = test_df();
        let results = run("plot sig = cross(sma(close, 5), sma(close, 20));", &df);
        assert!(results.contains_key("sig"));
    }

    #[test]
    fn test_run_macd() {
        let df = test_df();
        let results = run("plot d = macd(close, 12, 26, 9).dif;", &df);
        assert!(results.contains_key("d"));
    }

    #[test]
    fn test_run_arithmetic() {
        let df = test_df();
        let results = run("plot tp = (high + low + close) / 3;", &df);
        let tp = &results["tp"];
        assert!((tp[50] - (df.column("high").unwrap().to_f64_vec()[50]
            + df.column("low").unwrap().to_f64_vec()[50]
            + df.column("close").unwrap().to_f64_vec()[50]) / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_run_complex_signal() {
        let df = test_df();
        let source = r#"
            let fast_ma = sma(close, 5);
            let slow_ma = sma(close, 20);
            plot buy_signal = cross(fast_ma, slow_ma);
        "#;
        let results = run(source, &df);
        assert!(results.contains_key("buy_signal"));
    }
}

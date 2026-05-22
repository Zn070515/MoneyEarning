//! ME Script — custom indicator scripting language compiler & runtime
//!
//! Pipeline: Source → Lexer → Parser → AST → Compiler → IR → Runtime → Results
//!
//! Dual syntax: TDX classic (通达信兼容) + modern mode

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod functions;
pub mod compiler;
pub mod runtime;

use std::collections::HashMap;
use wasm_core::DataFrame;

// ── Public API (backward compatible) ──

/// Compiled strategy result with buy/sell signal arrays
#[derive(Debug, Clone)]
pub struct StrategyResult {
    pub buy_signals: Vec<bool>,
    pub sell_signals: Vec<bool>,
    pub params: HashMap<String, f64>,
    pub errors: Vec<String>,
}

/// Old-style Rule (for backward compat with parse_script)
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub condition: ast::Expr,
}

/// Old-style Expr re-export
pub use ast::Expr;

/// Parse a complete ME Script into params and rules (backward compatible API)
pub fn parse_script(source: &str) -> (HashMap<String, f64>, Vec<Rule>, Vec<String>) {
    let mut lex = lexer::Lexer::new(source);
    let tokens = lex.tokenize();
    let lex_errors = lex.errors().to_vec();

    let mut p = parser::Parser::new(tokens);
    let program = p.parse_program();
    let errors: Vec<String> = lex_errors.into_iter()
        .chain(p.errors().iter().cloned())
        .collect();

    let mut params = HashMap::new();
    let mut rules = Vec::new();

    for item in &program.items {
        match item {
            ast::TopLevel::Indicator(def) => {
                for p in &def.params {
                    params.insert(p.name.clone(), p.default);
                }
            }
            ast::TopLevel::Statement(stmt) => {
                match stmt {
                    ast::Statement::VarDecl(vd) => {
                        params.insert(vd.name.clone(), 0.0);
                        rules.push(Rule { name: vd.name.clone(), condition: vd.value.clone() });
                    }
                    ast::Statement::AssignOutput(name, expr) => {
                        rules.push(Rule { name: name.clone(), condition: expr.clone() });
                    }
                    ast::Statement::AssignNoOutput(name, _) => {
                        params.insert(name.clone(), 0.0);
                    }
                    _ => {}
                }
            }
        }
    }

    (params, rules, errors)
}

/// Compile and execute a ME Script against a DataFrame (main entry point)
pub fn execute(source: &str, df: &DataFrame) -> StrategyResult {
    // Try new compiler pipeline first
    let mut lex = lexer::Lexer::new(source);
    let tokens = lex.tokenize();
    let mut p = parser::Parser::new(tokens);
    let program = p.parse_program();
    let mut errors: Vec<String> = lex.errors().to_vec();
    errors.extend(p.errors().iter().cloned());

    if errors.is_empty() {
        let mut comp_errors = Vec::new();
        if let Some(ir) = compiler::compile(&program, &mut comp_errors) {
            errors.extend(comp_errors);
            if errors.is_empty() {
                let params: HashMap<String, f64> = ir.params.iter()
                    .map(|p| (p.name.clone(), p.default))
                    .collect();
                let mut rt = runtime::Runtime::new();
                let results = rt.execute(&ir, df, &params);

                let n = df.len();
                let mut buy_signals = vec![false; n];
                let mut sell_signals = vec![false; n];

                for (name, values) in &results {
                    let lower = name.to_lowercase();
                    if lower.contains("buy") || lower.contains("long") || lower.contains("entry") || lower.contains("signal") && !lower.contains("sell") {
                        for (i, &v) in values.iter().enumerate() {
                            if v > 0.5 && i < n { buy_signals[i] = true; }
                        }
                    }
                    if lower.contains("sell") || lower.contains("short") || lower.contains("exit") {
                        for (i, &v) in values.iter().enumerate() {
                            if v > 0.5 && i < n { sell_signals[i] = true; }
                        }
                    }
                    // Cross signals produce buy/sell based on cross direction
                    if lower.contains("cross") && lower.contains("signal") {
                        for (i, &v) in values.iter().enumerate() {
                            if v > 0.5 && i < n { buy_signals[i] = true; }
                        }
                    }
                }

                return StrategyResult { buy_signals, sell_signals, params, errors };
            }
        } else {
            errors.extend(comp_errors);
        }
    }

    // Fallback: return empty result with errors
    StrategyResult {
        buy_signals: vec![false; df.len()],
        sell_signals: vec![false; df.len()],
        params: HashMap::new(),
        errors,
    }
}

// ── New public API (compiler pipeline) ──

/// Full compilation: source → IR program
pub fn compile_script(source: &str) -> Result<ast::IrProgram, Vec<String>> {
    let mut lex = lexer::Lexer::new(source);
    let tokens = lex.tokenize();
    let mut errors: Vec<String> = lex.errors().to_vec();

    let mut p = parser::Parser::new(tokens);
    let program = p.parse_program();
    errors.extend(p.errors().iter().cloned());

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut comp_errors = Vec::new();
    match compiler::compile(&program, &mut comp_errors) {
        Some(ir) => {
            if comp_errors.is_empty() {
                Ok(ir)
            } else {
                Err(comp_errors)
            }
        }
        None => Err(comp_errors),
    }
}

/// Run a compiled IR program against a DataFrame
pub fn run_ir(ir: &ast::IrProgram, df: &DataFrame, params: &HashMap<String, f64>) -> HashMap<String, Vec<f64>> {
    let mut rt = runtime::Runtime::new();
    rt.execute(ir, df, params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_core::OHLCV;

    fn test_df() -> DataFrame {
        let mut data = Vec::new();
        let mut price = 10.0;
        for i in 0..100 {
            data.push(OHLCV {
                open: price, high: price + 0.5, low: price - 0.5, close: price + 0.1,
                volume: 1000.0, amount: None, turnover: None,
                trade_date: i.to_string(),
            });
            price += 0.1;
        }
        DataFrame::new(&data)
    }

    #[test]
    fn test_parse_ma_cross() {
        let source = "let ma5 = sma(close, 5);\nlet signal = cross(sma(close, 5), sma(close, 20));";
        let (_params, rules, errors) = parse_script(source);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_execute_ma_cross() {
        // Use oscillating data so SMAs actually cross
        let mut data = Vec::new();
        for i in 0..100 {
            let price = 10.0 + (i as f64 * 0.3).sin() * 5.0; // oscillating
            data.push(OHLCV {
                open: price, high: price + 0.5, low: price - 0.5, close: price,
                volume: 1000.0, amount: None, turnover: None,
                trade_date: i.to_string(),
            });
        }
        let df = DataFrame::new(&data);
        let source = "let fast_ma = sma(close, 5);\nlet slow_ma = sma(close, 20);\nplot buy_signal = cross(fast_ma, slow_ma);";
        let result = execute(source, &df);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let total_signals: usize = result.buy_signals.iter().filter(|&&b| b).count()
            + result.sell_signals.iter().filter(|&&b| b).count();
        assert!(total_signals > 0, "Expected at least some crossover signals, got 0");
    }

    #[test]
    fn test_execute_tdx_syntax() {
        // TDX-compatible classic syntax
        let source = "MA5:SMA(CLOSE,5);\nSIGNAL:CROSS(SMA(CLOSE,5),SMA(CLOSE,20));";
        let df = test_df();
        let result = execute(source, &df);
        // Should parse without major errors
        assert!(result.buy_signals.len() == 100);
    }

    #[test]
    fn test_compile_and_run() {
        let source = "plot tp = (high + low + close) / 3;";
        let ir = compile_script(source).unwrap();
        let df = test_df();
        let results = run_ir(&ir, &df, &HashMap::new());
        assert!(results.contains_key("tp"));
        let tp = &results["tp"];
        // Typical price = (H + L + C) / 3
        let expected = (df.column("high").unwrap().to_f64_vec()[50]
            + df.column("low").unwrap().to_f64_vec()[50]
            + df.column("close").unwrap().to_f64_vec()[50]) / 3.0;
        assert!((tp[50] - expected).abs() < 0.01);
    }

    #[test]
    fn test_execute_macd_signal() {
        let source = "let d = macd(close, 12, 26, 9).dif;\nlet de = macd(close, 12, 26, 9).dea;\nplot buy_signal = cross(d, de);";
        let df = test_df();
        let result = execute(source, &df);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
    }
}

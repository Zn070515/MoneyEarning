//! ME Script Compiler — AST → Computation IR
//! The IR is a flat list of operations that can be executed efficiently
//! against a DataFrame (no recursion, no branching at runtime).

use std::collections::HashMap;
use crate::ast::*;

struct Compiler<'a> {
    nodes: Vec<IrNode>,
    var_map: HashMap<String, usize>,
    outputs: Vec<(String, usize)>,
    params: Vec<IrParam>,
    errors: &'a mut Vec<String>,
}

pub fn compile(program: &Program, errors: &mut Vec<String>) -> Option<IrProgram> {
    let mut c = Compiler {
        nodes: Vec::new(),
        var_map: HashMap::new(),
        outputs: Vec::new(),
        params: Vec::new(),
        errors,
    };

    // Register input series
    for (idx, name) in ["open", "high", "low", "close", "volume", "amount", "turnover"]
        .iter().enumerate()
    {
        let node_idx = c.nodes.len();
        c.nodes.push(IrNode::Input { name: name.to_string(), index: idx });
        c.var_map.insert(name.to_string(), node_idx);
    }

    let mut name = None;
    for item in &program.items {
        match item {
            TopLevel::Indicator(def) => {
                name = Some(def.name.clone());
                for p in &def.params {
                    let node_idx = c.nodes.len();
                    c.nodes.push(IrNode::Constant(p.default));
                    c.var_map.insert(p.name.clone(), node_idx);
                    c.params.push(IrParam {
                        name: p.name.clone(),
                        default: p.default,
                        min: p.min,
                        max: p.max,
                    });
                }
                for stmt in &def.body {
                    c.compile_stmt(stmt);
                }
            }
            TopLevel::Statement(stmt) => {
                c.compile_stmt(stmt);
            }
        }
    }

    Some(IrProgram {
        name,
        params: c.params,
        nodes: c.nodes,
        outputs: c.outputs,
        input_len: 7,
    })
}

impl Compiler<'_> {
    fn add_node(&mut self, node: IrNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    fn compile_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl(vd) => {
                let node_idx = self.compile_expr(&vd.value);
                self.var_map.insert(vd.name.clone(), node_idx);
            }
            Statement::PlotDecl(pd) => {
                let node_idx = self.compile_expr(&pd.value);
                self.var_map.insert(pd.name.clone(), node_idx);
                self.outputs.push((pd.name.clone(), node_idx));
            }
            Statement::AssignOutput(name, expr) => {
                let node_idx = self.compile_expr(expr);
                self.var_map.insert(name.clone(), node_idx);
                self.outputs.push((name.clone(), node_idx));
            }
            Statement::AssignNoOutput(name, expr) => {
                let node_idx = self.compile_expr(expr);
                self.var_map.insert(name.clone(), node_idx);
            }
            Statement::Expr(expr) => {
                self.compile_expr(expr);
            }
            Statement::IfStmt(_) => {
                // Skip if statements at top level (handled in expressions)
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> usize {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::Number(n) => self.add_node(IrNode::Constant(*n)),
                    Literal::Bool(b) => self.add_node(IrNode::Constant(if *b { 1.0 } else { 0.0 })),
                    Literal::String(_) => self.add_node(IrNode::Constant(0.0)),
                    Literal::Null => self.add_node(IrNode::Constant(0.0)),
                }
            }
            Expr::Ident(name, _) => {
                if let Some(&idx) = self.var_map.get(name) {
                    return idx;
                }
                // Check if it's a built-in data series
                if let Ok(val) = name.parse::<f64>() {
                    return self.add_node(IrNode::Constant(val));
                }
                let input_names = ["open", "high", "low", "close", "volume", "amount", "turnover"];
                if let Some(pos) = input_names.iter().position(|&n| n == name) {
                    return pos; // These were registered at positions 0-6
                }
                self.errors.push(format!("未定义的变量: '{}'", name));
                self.add_node(IrNode::Constant(0.0))
            }
            Expr::Call { func, args, .. } => {
                self.compile_call(func, args)
            }
            Expr::Binary { op, left, right, .. } => {
                let l = self.compile_expr(left);
                let r = self.compile_expr(right);
                let ir_op = match op {
                    BinOp::Add => IrBinOp::Add,
                    BinOp::Sub => IrBinOp::Sub,
                    BinOp::Mul => IrBinOp::Mul,
                    BinOp::Div => IrBinOp::Div,
                    BinOp::Mod => IrBinOp::Mod,
                    BinOp::Pow => IrBinOp::Pow,
                    BinOp::Eq => IrBinOp::Eq,
                    BinOp::Neq => IrBinOp::Neq,
                    BinOp::Gt => IrBinOp::Gt,
                    BinOp::Lt => IrBinOp::Lt,
                    BinOp::Gte => IrBinOp::Gte,
                    BinOp::Lte => IrBinOp::Lte,
                    BinOp::And => IrBinOp::And,
                    BinOp::Or => IrBinOp::Or,
                    BinOp::Xor => IrBinOp::Xor,
                };
                self.add_node(IrNode::BinaryOp { op: ir_op, left: l, right: r })
            }
            Expr::Unary { op, expr, .. } => {
                let input = self.compile_expr(expr);
                let ir_op = match op {
                    UnaryOp::Neg => IrUnaryOp::Neg,
                    UnaryOp::Not => IrUnaryOp::Not,
                };
                self.add_node(IrNode::UnaryOp { op: ir_op, input })
            }
            Expr::FieldAccess { object, field, .. } => {
                // macd(close, 12, 26, 9).dif → treat as a special call
                match object.as_ref() {
                    Expr::Call { func, args, .. } => {
                        match func.as_ref() {
                            Expr::Ident(fname, _) => {
                                let fname_lower = fname.to_lowercase();
                                match (fname_lower.as_str(), field.as_str()) {
                                    ("macd", "dif") => self.compile_named_call("macd_dif", args),
                                    ("macd", "dea") => self.compile_named_call("macd_dea", args),
                                    ("macd", "hist") | ("macd", "macd") => self.compile_named_call("macd_hist", args),
                                    ("kdj" | "stoch", "k") => self.compile_named_call("kdj_k", args),
                                    ("kdj" | "stoch", "d") => self.compile_named_call("kdj_d", args),
                                    ("kdj", "j") => self.compile_named_call("kdj_j", args),
                                    _ => {
                                        self.errors.push(format!("未知的字段访问: {}.{}", fname, field));
                                        self.add_node(IrNode::Constant(0.0))
                                    }
                                }
                            }
                            _ => {
                                self.errors.push("不支持动态字段访问".into());
                                self.add_node(IrNode::Constant(0.0))
                            }
                        }
                    }
                    _ => {
                        self.errors.push("字段访问只能用于函数调用结果".into());
                        self.add_node(IrNode::Constant(0.0))
                    }
                }
            }
            Expr::HistoryRef { expr, offset, .. } => {
                let input = self.compile_expr(expr);
                self.add_node(IrNode::Shift { input, offset: *offset })
            }
            Expr::IfExpr { condition, then_branch, else_branch, .. } => {
                let cond = self.compile_expr(condition);
                let then_val = self.compile_expr(then_branch);
                let else_val = else_branch.as_ref()
                    .map(|e| self.compile_expr(e))
                    .unwrap_or_else(|| self.add_node(IrNode::Constant(0.0)));
                self.add_node(IrNode::Conditional { cond, then_val, else_val })
            }
            Expr::CrossPeriod { .. } => {
                self.errors.push("跨周期引用暂未实现".into());
                self.add_node(IrNode::Constant(0.0))
            }
            Expr::CrossSymbol { .. } => {
                self.errors.push("跨品种引用暂未实现".into());
                self.add_node(IrNode::Constant(0.0))
            }
            Expr::PlotRef(name, _) => {
                if let Some(&idx) = self.var_map.get(name) {
                    return idx;
                }
                self.errors.push(format!("未定义的plot引用: '{}'", name));
                self.add_node(IrNode::Constant(0.0))
            }
        }
    }

    fn compile_call(&mut self, func: &Expr, args: &[Expr]) -> usize {
        let func_name = match func {
            Expr::Ident(name, _) => name.to_lowercase(),
            _ => {
                self.errors.push("函数名必须是标识符".into());
                return self.add_node(IrNode::Constant(0.0));
            }
        };

        self.compile_named_call(&func_name, args)
    }

    fn compile_named_call(&mut self, func_name: &str, args: &[Expr]) -> usize {
        let compiled_args: Vec<usize> = args.iter().map(|a| self.compile_expr(a)).collect();
        let period = compiled_args.get(1).copied(); // Most functions have period as 2nd arg

        let series_fn = match func_name {
            "sma" | "ma" | "simple_ma" => SeriesFn::Sma,
            "ema" => SeriesFn::Ema,
            "wma" => SeriesFn::Wma,
            "rma" => SeriesFn::Rma,
            "hma" => SeriesFn::Hma,
            "stdev" | "std" => SeriesFn::Stdev,
            "variance" | "var" => SeriesFn::Variance,
            "zscore" => SeriesFn::ZScore,
            "rsi" => SeriesFn::Rsi,
            "macd_dif" => SeriesFn::MacdDif,
            "macd_dea" => SeriesFn::MacdDea,
            "macd_hist" => SeriesFn::MacdHist,
            "kdj_k" => SeriesFn::KdjK,
            "kdj_d" => SeriesFn::KdjD,
            "kdj_j" => SeriesFn::KdjJ,
            "cci" => SeriesFn::Cci,
            "mfi" => SeriesFn::Mfi,
            "atr" => SeriesFn::Atr,
            "adx" => SeriesFn::Adx,
            "bb_upper" | "bbands_upper" => SeriesFn::BbUpper,
            "bb_middle" | "bbands_middle" => SeriesFn::BbMiddle,
            "bb_lower" | "bbands_lower" => SeriesFn::BbLower,
            "obv" => SeriesFn::Obv,
            "cross" | "cross_over" | "crossunder" => SeriesFn::Cross,
            "ref" | "shift" => {
                let input = compiled_args.first().copied().unwrap_or(0);
                let offset = compiled_args.get(1).copied().unwrap_or(0);
                return self.add_node(IrNode::Shift { input, offset: offset as i32 });
            }
            "hhv" | "highest" => SeriesFn::Hhv,
            "llv" | "lowest" => SeriesFn::Llv,
            "sum" => SeriesFn::Sum,
            "barslast" => SeriesFn::BarsLast,
            "barssince" => SeriesFn::BarsSince,
            "count" => SeriesFn::Count,
            "every" => SeriesFn::Every,
            "exist" => SeriesFn::Exist,
            "filter" => SeriesFn::Filter,
            "abs" => SeriesFn::Abs,
            "max" => SeriesFn::Max,
            "min" => SeriesFn::Min,
            "pow" => SeriesFn::Pow,
            "sqrt" => SeriesFn::Sqrt,
            "log" => SeriesFn::Log,
            "ln" => SeriesFn::Ln,
            "exp" => SeriesFn::Exp,
            "round" => SeriesFn::Round,
            "ceil" => SeriesFn::Ceil,
            "floor" => SeriesFn::Floor,
            "sign" => SeriesFn::Sign,
            "sin" => SeriesFn::Sin,
            "cos" => SeriesFn::Cos,
            "tan" => SeriesFn::Tan,
            "if" | "iff" => SeriesFn::If,
            "between" => SeriesFn::Between,
            "range" => SeriesFn::Range,
            "roc" => SeriesFn::Roc,
            "percent_return" => SeriesFn::PercentReturn,
            "drawdown" => SeriesFn::Drawdown,
            "cdl_doji" => SeriesFn::CdlDoji,
            "cdl_hammer" => SeriesFn::CdlHammer,
            "cdl_shooting_star" => SeriesFn::CdlShootingStar,
            "cdl_marubozu" => SeriesFn::CdlMarubozu,
            "cdl_engulfing" => SeriesFn::CdlEngulfing,
            "cdl_harami" => SeriesFn::CdlHarami,
            "cdl_morning_star" => SeriesFn::CdlMorningStar,
            "cdl_evening_star" => SeriesFn::CdlEveningStar,
            "cdl_three_white_soldiers" => SeriesFn::CdlThreeWhiteSoldiers,
            _ => {
                self.errors.push(format!("未知函数: '{}'", func_name));
                return self.add_node(IrNode::Constant(0.0));
            }
        };

        self.add_node(IrNode::SeriesOp {
            op: series_fn,
            inputs: compiled_args,
            period,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn compile_source(source: &str) -> Option<IrProgram> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program();
        let mut errors = Vec::new();
        compile(&program, &mut errors)
    }

    #[test]
    fn test_compile_sma_cross() {
        let ir = compile_source("plot signal = cross(sma(close, 5), sma(close, 20));").unwrap();
        // Should have: 7 inputs + 2 sma nodes + 1 cross node = 10+ nodes
        assert!(ir.nodes.len() >= 10, "Expected {} >= 10 nodes", ir.nodes.len());
        assert_eq!(ir.outputs.len(), 1);
    }

    #[test]
    fn test_compile_macd() {
        let ir = compile_source("let dif = macd(close, 12, 26, 9).dif;").unwrap();
        assert!(ir.nodes.len() > 7, "Expected more than input nodes");
    }

    #[test]
    fn test_compile_arithmetic() {
        let ir = compile_source("let mid = (high + low + close) / 3;").unwrap();
        assert!(ir.nodes.len() >= 10);
    }
}

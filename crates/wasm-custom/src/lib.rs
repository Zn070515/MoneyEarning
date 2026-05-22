use std::collections::HashMap;
use wasm_core::DataFrame;

/// Compiled strategy with buy/sell signal arrays
#[derive(Debug, Clone)]
pub struct StrategyResult {
    pub buy_signals: Vec<bool>,
    pub sell_signals: Vec<bool>,
    pub params: HashMap<String, f64>,
    pub errors: Vec<String>,
}

/// Token from lexer
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Number(f64),
    LParen,
    RParen,
    Comma,
    Colon,
    LBrace,
    RBrace,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    Rule,
    Params,
    Eof,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Lexer { chars: source.chars().collect(), pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn skip_line_comment(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
            self.pos += 1;
        }
    }

    fn read_number(&mut self) -> f64 {
        let start = self.pos;
        if self.pos < self.chars.len() && self.chars[self.pos] == '-' {
            self.pos += 1;
        }
        while self.pos < self.chars.len() && (self.chars[self.pos].is_ascii_digit() || self.chars[self.pos] == '.') {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse().unwrap_or(0.0)
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.chars.len() && (self.chars[self.pos].is_alphanumeric() || self.chars[self.pos] == '_') {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn next_token(&mut self) -> Token {
        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                return Token::Eof;
            }

            // Line comment
            if self.chars[self.pos] == '/' && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '/' {
                self.skip_line_comment();
                continue;
            }

            let ch = self.chars[self.pos];

            if ch.is_ascii_digit() || (ch == '-' && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1].is_ascii_digit()) {
                return Token::Number(self.read_number());
            }

            if ch.is_alphabetic() || ch == '_' {
                let ident = self.read_ident();
                return match ident.as_str() {
                    "rule" => { self.read_ident(); Token::Rule } // consume "buy" or "sell"
                    "params" => Token::Params,
                    _ => Token::Ident(ident),
                };
            }

            self.pos += 1;
            match ch {
                '(' => return Token::LParen,
                ')' => return Token::RParen,
                ',' => return Token::Comma,
                ':' => return Token::Colon,
                '{' => return Token::LBrace,
                '}' => return Token::RBrace,
                '>' => {
                    if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
                        self.pos += 1;
                        return Token::Gte;
                    }
                    return Token::Gt;
                }
                '<' => {
                    if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
                        self.pos += 1;
                        return Token::Lte;
                    }
                    return Token::Lt;
                }
                '=' => {
                    if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
                        self.pos += 1;
                        return Token::Eq;
                    }
                    // standalone '=' is skipped (for params)
                    continue;
                }
                '!' => {
                    if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
                        self.pos += 1;
                        return Token::Neq;
                    }
                    continue;
                }
                _ => continue, // Skip unknown characters
            }
        }
    }
}

// ── AST ──

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(f64),
    Variable(String),
    Call {
        func: String,
        args: Vec<Expr>,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String, // "buy" or "sell"
    pub condition: Expr,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        if self.pos < self.tokens.len() { &self.tokens[self.pos] } else { &Token::Eof }
    }

    fn advance(&mut self) -> Token {
        let t = self.peek().clone();
        self.pos += 1;
        t
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        let left = self.parse_primary()?;

        // Binary operators
        match self.peek() {
            Token::Gt | Token::Lt | Token::Gte | Token::Lte | Token::Eq | Token::Neq => {
                let op = match self.advance() {
                    Token::Gt => ">".to_string(),
                    Token::Lt => "<".to_string(),
                    Token::Gte => ">=".to_string(),
                    Token::Lte => "<=".to_string(),
                    Token::Eq => "==".to_string(),
                    Token::Neq => "!=".to_string(),
                    _ => return Some(left),
                };
                let right = self.parse_primary()?;
                return Some(Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
            _ => {}
        }
        Some(left)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.peek() {
            Token::Number(n) => {
                let val = *n;
                self.advance();
                Some(Expr::Literal(val))
            }
            Token::Ident(_) => {
                let name = match self.advance() { Token::Ident(s) => s, _ => return None };

                // Check if it's a function call
                if matches!(self.peek(), Token::LParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RParen) {
                        loop {
                            if let Some(arg) = self.parse_expr() {
                                args.push(arg);
                            }
                            if matches!(self.peek(), Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    if matches!(self.peek(), Token::RParen) {
                        self.advance();
                    }
                    Some(Expr::Call { func: name, args })
                } else {
                    Some(Expr::Variable(name))
                }
            }
            Token::Eof => None,
            _ => { self.advance(); self.parse_primary() }
        }
    }
}

/// Parse a complete ME Script into rules and params
pub fn parse_script(source: &str) -> (HashMap<String, f64>, Vec<Rule>, Vec<String>) {
    let mut errors = Vec::new();
    let mut params = HashMap::new();
    let mut rules = Vec::new();

    // Extract params block manually (simpler than full parser)
    if let Some(params_start) = source.find("params:") {
        let after_params = &source[params_start + 7..];
        if let Some(open) = after_params.find('{') {
            if let Some(close) = after_params.find('}') {
                let params_str = &after_params[open + 1..close];
                for part in params_str.split(',') {
                    let part = part.trim();
                    if let Some(colon) = part.find(':') {
                        let key = part[..colon].trim().trim_matches('"');
                        let val_str = part[colon + 1..].trim();
                        if let Ok(val) = val_str.parse::<f64>() {
                            params.insert(key.to_string(), val);
                        }
                    }
                }
            }
        }
    }

    // Parse rules
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let t = lexer.next_token();
        if t == Token::Eof { break; }
        tokens.push(t);
    }

    // Extract rules: "Rule" token followed by expression until next Rule or Eof
    let mut i = 0;
    while i < tokens.len() {
        if matches!(tokens[i], Token::Rule) {
            // Next should be an ident (buy/sell)
            if i + 1 < tokens.len() {
                if let Token::Ident(rule_name) = &tokens[i + 1] {
                    i += 2;
                    // Collect tokens until next Rule
                    let start = i;
                    while i < tokens.len() && !matches!(tokens[i], Token::Rule) {
                        i += 1;
                    }
                    // Parse expression from these tokens
                    let rule_tokens: Vec<Token> = tokens[start..i].iter()
                        .filter(|t| !matches!(t, Token::Colon))
                        .cloned()
                        .collect();

                    let mut parser = Parser::new(rule_tokens);
                    if let Some(expr) = parser.parse_expr() {
                        rules.push(Rule { name: rule_name.clone(), condition: expr });
                    } else {
                        errors.push(format!("无法解析规则 '{}'", rule_name));
                    }
                }
            }
        } else {
            i += 1;
        }
    }

    (params, rules, errors)
}

/// Compile and execute a ME Script against a DataFrame
pub fn execute(source: &str, df: &DataFrame) -> StrategyResult {
    let (params, rules, errors) = parse_script(source);

    let n = df.len();
    let mut buy_signals = vec![false; n];
    let mut sell_signals = vec![false; n];

    for rule in &rules {
        let result = eval_expr(&rule.condition, df, &params, n - 1);
        match rule.name.as_str() {
            "buy" => {
                for (i, &v) in result.iter().enumerate() {
                    if v && !buy_signals[i] { buy_signals[i] = true; }
                }
            }
            "sell" => {
                for (i, &v) in result.iter().enumerate() {
                    if v && !sell_signals[i] { sell_signals[i] = true; }
                }
            }
            _ => {}
        }
    }

    StrategyResult { buy_signals, sell_signals, params, errors }
}

/// Evaluate an expression tree into a boolean array (one value per bar)
fn eval_expr(expr: &Expr, df: &DataFrame, params: &HashMap<String, f64>, default_idx: usize) -> Vec<bool> {
    match expr {
        Expr::Literal(v) => {
            let n = df.len();
            vec![*v != 0.0; n]
        }
        Expr::Variable(name) => {
            eval_variable(name, df)
        }
        Expr::Call { func, args } => {
            eval_call(func, args, df, params, default_idx)
        }
        Expr::Binary { op, left, right } => {
            let l_vals = eval_to_f64(left, df, params, default_idx);
            let r_vals = eval_to_f64(right, df, params, default_idx);
            let n = l_vals.len().min(r_vals.len());
            let mut result = vec![false; n];
            match op.as_str() {
                ">" => { for i in 0..n { result[i] = l_vals[i] > r_vals[i]; } }
                "<" => { for i in 0..n { result[i] = l_vals[i] < r_vals[i]; } }
                ">=" => { for i in 0..n { result[i] = l_vals[i] >= r_vals[i]; } }
                "<=" => { for i in 0..n { result[i] = l_vals[i] <= r_vals[i]; } }
                "==" => { for i in 0..n { result[i] = (l_vals[i] - r_vals[i]).abs() < 1e-10; } }
                "!=" => { for i in 0..n { result[i] = (l_vals[i] - r_vals[i]).abs() >= 1e-10; } }
                _ => {}
            }
            result
        }
    }
}

/// Evaluate expression to f64 values (one per bar)
fn eval_to_f64(expr: &Expr, df: &DataFrame, params: &HashMap<String, f64>, default_idx: usize) -> Vec<f64> {
    match expr {
        Expr::Literal(v) => vec![*v; df.len()],
        Expr::Variable(name) => {
            if let Ok(val) = name.parse::<f64>() {
                return vec![val; df.len()];
            }
            if let Some(p) = params.get(name.as_str()) {
                return vec![*p; df.len()];
            }
            get_price_series(name, df)
        }
        Expr::Call { func, args } => {
            eval_call_f64(func, args, df, params, default_idx)
        }
        Expr::Binary { op, left, right } => {
            let l = eval_to_f64(left, df, params, default_idx);
            let r = eval_to_f64(right, df, params, default_idx);
            let n = l.len().min(r.len());
            match op.as_str() {
                "+" => (0..n).map(|i| l[i] + r[i]).collect(),
                "-" => (0..n).map(|i| l[i] - r[i]).collect(),
                "*" => (0..n).map(|i| l[i] * r[i]).collect(),
                "/" => (0..n).map(|i| if r[i].abs() > 1e-12 { l[i] / r[i] } else { 0.0 }).collect(),
                _ => vec![0.0; n],
            }
        }
    }
}

fn get_price_series(name: &str, df: &DataFrame) -> Vec<f64> {
    df.column(name)
        .map(|c| c.to_f64_vec())
        .unwrap_or_else(|| vec![0.0; df.len()])
}

fn eval_variable(name: &str, df: &DataFrame) -> Vec<bool> {
    // A standalone variable evaluates to truthy based on its value at each bar
    let vals = get_price_series(name, df);
    vals.iter().map(|&v| v > 0.0).collect()
}

fn eval_call(func: &str, args: &[Expr], df: &DataFrame, params: &HashMap<String, f64>, default_idx: usize) -> Vec<bool> {
    let f64_vals = eval_call_f64(func, args, df, params, default_idx);
    f64_vals.iter().map(|&v| v > 0.0).collect()
}

fn eval_call_f64(func: &str, args: &[Expr], df: &DataFrame, params: &HashMap<String, f64>, default_idx: usize) -> Vec<f64> {
    let n = df.len();

    // Evaluate arguments
    let arg_vals: Vec<Vec<f64>> = args.iter()
        .map(|a| eval_to_f64(a, df, params, default_idx))
        .collect();

    let get_arg = |i: usize, default: f64| -> f64 {
        arg_vals.get(i).and_then(|v| v.get(v.len() - 1).copied()).unwrap_or(default)
    };

    match func {
        "sma" | "SMA" | "ma" | "MA" => {
            let period = get_arg(1, 20.0) as usize;
            simple_ma(&arg_vals[0], period)
        }
        "ema" | "EMA" => {
            let period = get_arg(1, 20.0) as usize;
            ema_vec(&arg_vals[0], period)
        }
        "rsi" | "RSI" => {
            let period = get_arg(1, 14.0) as usize;
            compute_rsi(&arg_vals[0], period)
        }
        "cross" | "CROSS" | "CROSSOVER" => {
            let dir = get_arg(2, 1.0);
            cross_over(&arg_vals[0], &arg_vals[1], dir > 0.0)
        }
        "ref" | "REF" => {
            let offset = get_arg(1, 1.0) as usize;
            shift_vec(&arg_vals[0], offset)
        }
        "highest" | "HHV" => {
            let period = get_arg(1, 20.0) as usize;
            rolling_max(&arg_vals[0], period)
        }
        "lowest" | "LLV" => {
            let period = get_arg(1, 20.0) as usize;
            rolling_min(&arg_vals[0], period)
        }
        "abs" | "ABS" => {
            arg_vals[0].iter().map(|&v| v.abs()).collect()
        }
        _ => arg_vals.first().cloned().unwrap_or_else(|| vec![0.0; n]),
    }
}

// ── Indicator helpers (duplicated from wasm-indicators for self-contained evaluation) ──

fn simple_ma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    let p = period.min(n);
    for i in p - 1..n {
        result[i] = data[i - p + 1..=i].iter().sum::<f64>() / p as f64;
    }
    for i in 0..p - 1 {
        result[i] = data[..=i].iter().sum::<f64>() / (i + 1) as f64;
    }
    result
}

fn ema_vec(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    if n == 0 { return result; }
    let alpha = 2.0 / (period as f64 + 1.0);
    result[0] = data[0];
    for i in 1..n {
        result[i] = data[i] * alpha + result[i - 1] * (1.0 - alpha);
    }
    result
}

fn compute_rsi(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![50.0; n];
    if n < period + 1 { return result; }

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..=period {
        let diff = data[i] - data[i - 1];
        if diff > 0.0 { gains += diff; } else { losses += -diff; }
    }

    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;

    for i in period + 1..n {
        let diff = data[i] - data[i - 1];
        let gain = if diff > 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };
        avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;
        result[i] = if avg_loss > 0.0 {
            100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
        } else {
            100.0
        };
    }
    result
}

fn cross_over(fast: &[f64], slow: &[f64], above: bool) -> Vec<f64> {
    let n = fast.len().min(slow.len());
    let mut result = vec![0.0; fast.len().max(slow.len())];
    for i in 1..n {
        if above {
            result[i] = if fast[i - 1] <= slow[i - 1] && fast[i] > slow[i] { 1.0 } else { 0.0 };
        } else {
            result[i] = if fast[i - 1] >= slow[i - 1] && fast[i] < slow[i] { 1.0 } else { 0.0 };
        }
    }
    result
}

fn shift_vec(data: &[f64], offset: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in offset..n {
        result[i] = data[i - offset];
    }
    result
}

fn rolling_max(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = data[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    }
    result
}

fn rolling_min(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0; n];
    for i in 0..n {
        let start = if i >= period { i - period + 1 } else { 0 };
        result[i] = data[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
    }
    result
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
        let source = r#"
params: { fast: 5, slow: 20 }
rule buy:
  cross(ema(close, fast), ema(close, slow), 1)
rule sell:
  cross(ema(close, fast), ema(close, slow), -1)
"#;
        let (params, rules, errors) = parse_script(source);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
        assert_eq!(params.get("fast"), Some(&5.0));
        assert_eq!(params.get("slow"), Some(&20.0));
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_execute_ma_cross() {
        let source = "params: { fast: 5, slow: 20 }\nrule buy: cross(ema(close, fast), ema(close, slow), 1)\nrule sell: cross(ema(close, fast), ema(close, slow), -1)";
        let df = test_df();
        let result = execute(source, &df);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        // Should produce some signals
        let total_signals: usize = result.buy_signals.iter().filter(|&&b| b).count()
            + result.sell_signals.iter().filter(|&&b| b).count();
        assert!(total_signals > 0, "Expected at least some crossover signals");
    }
}

//! ME Script Parser — recursive-descent parser producing AST
//! Follows the EBNF grammar from GOAL_SPEC §14.12

use crate::ast::*;
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0, errors: Vec::new() }
    }

    pub fn errors(&self) -> &[String] { &self.errors }

    pub fn parse_program(&mut self) -> Program {
        let mut items = Vec::new();
        while !self.is_eof() {
            if self.match_kw("indicator") {
                if let Some(def) = self.parse_indicator_def() {
                    items.push(TopLevel::Indicator(def));
                }
            } else if let Some(stmt) = self.parse_statement() {
                items.push(TopLevel::Statement(stmt));
            } else {
                // Skip to next recoverable point
                self.advance();
            }
        }
        Program { items }
    }

    // ── Indicator definition ──

    fn parse_indicator_def(&mut self) -> Option<IndicatorDef> {
        let span = self.span();
        self.expect_kw("indicator")?;
        let name = if let Token::StrLit(s) = self.peek_clone() {
            self.advance();
            s
        } else if let Token::Ident(s) = self.peek_clone() {
            self.advance();
            s
        } else {
            self.error("期望指标名称");
            return None;
        };

        let mut params = Vec::new();
        if self.peek_is(&Token::LBrace) {
            self.advance(); // {
            while !self.peek_is(&Token::RBrace) && !self.peek_is(&Token::Eof) {
                if self.match_kw("param") {
                    if let Some(p) = self.parse_param_decl() {
                        params.push(p);
                    }
                } else if let Some(_stmt) = self.parse_statement() {
                    // body statements collected below
                } else {
                    self.advance();
                }
            }
            self.expect(&Token::RBrace);
        }

        // Re-parse body (simplified: second pass over content — in a real impl we'd collect during traversal)
        Some(IndicatorDef { name, params, body: vec![], span })
    }

    fn parse_param_decl(&mut self) -> Option<ParamDecl> {
        let span = self.span();
        let name = if let Token::Ident(s) = self.peek_clone() {
            self.advance();
            s
        } else {
            self.error("期望参数名");
            return None;
        };
        let default = if self.peek_is(&Token::AssignEq) {
            self.advance();
            self.parse_number_literal().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut min = None;
        let mut max = None;
        let mut step = None;
        if self.peek_is(&Token::LBrace) {
            self.advance();
            while !self.peek_is(&Token::RBrace) && !self.peek_is(&Token::Eof) {
                if let Token::Ident(key) = self.peek_clone() {
                    self.advance();
                    if self.peek_is(&Token::ColonAssign) || self.peek_is(&Token::Colon) {
                        self.advance();
                        let val = self.parse_number_literal().unwrap_or(0.0);
                        match key.as_str() {
                            "min" => min = Some(val),
                            "max" => max = Some(val),
                            "step" => step = Some(val),
                            _ => {}
                        }
                    }
                } else {
                    self.advance();
                }
                if self.peek_is(&Token::Comma) { self.advance(); }
            }
            self.expect(&Token::RBrace);
        }
        self.expect(&Token::Semicolon);
        Some(ParamDecl { name, default, min, max, step, span })
    }

    // ── Statements ──

    fn parse_statement(&mut self) -> Option<Statement> {
        let saved = self.pos;

        // let name = expr;
        if self.peek_is_kw("let") {
            return self.parse_var_decl();
        }
        // plot name = expr { ... };
        if self.peek_is_kw("plot") {
            return self.parse_plot_decl();
        }
        // if (...) { ... }
        if self.peek_is_kw("if") {
            return self.parse_if_stmt();
        }
        // TDX classic: NAME:EXPR; or NAME:=EXPR;
        if let Token::Ident(name) = self.peek_clone() {
            let name_str = name.clone();
            self.advance();
            if self.peek_is(&Token::Colon) {
                // NAME:EXPR; → TDX output assignment
                self.advance();
                let expr = self.parse_expr().unwrap_or(Expr::Literal(Literal::Number(0.0)));
                self.expect(&Token::Semicolon);
                return Some(Statement::AssignOutput(name_str, expr));
            } else if self.peek_is(&Token::Assign) {
                // NAME:=EXPR; → TDX no-output assignment
                self.advance();
                let expr = self.parse_expr().unwrap_or(Expr::Literal(Literal::Number(0.0)));
                self.expect(&Token::Semicolon);
                return Some(Statement::AssignNoOutput(name_str, expr));
            } else {
                // Not an assignment — rewind and try expression
                self.pos = saved;
            }
        }

        // Expression statement
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon);
        Some(Statement::Expr(expr))
    }

    fn parse_var_decl(&mut self) -> Option<Statement> {
        let span = self.span();
        self.expect_kw("let")?;
        let name = if let Token::Ident(s) = self.peek_clone() {
            self.advance();
            s
        } else {
            self.error("期望变量名");
            return None;
        };
        let type_ann = self.parse_type_annotation();
        self.expect(&Token::AssignEq);
        let value = self.parse_expr().unwrap_or(Expr::Literal(Literal::Number(0.0)));
        self.expect(&Token::Semicolon);
        Some(Statement::VarDecl(VarDecl { name, type_ann, value, mutable: false, span }))
    }

    fn parse_plot_decl(&mut self) -> Option<Statement> {
        let span = self.span();
        self.expect_kw("plot")?;
        let name = if let Token::Ident(s) = self.peek_clone() {
            self.advance();
            s
        } else {
            self.error("期望plot名称");
            return None;
        };
        self.expect(&Token::AssignEq);
        let value = self.parse_expr().unwrap_or(Expr::Literal(Literal::Number(0.0)));
        let attrs = if self.peek_is(&Token::LBrace) {
            self.parse_plot_attrs()
        } else {
            vec![]
        };
        self.expect(&Token::Semicolon);
        Some(Statement::PlotDecl(PlotDecl { name, value, attrs, span }))
    }

    fn parse_plot_attrs(&mut self) -> Vec<PlotAttr> {
        let mut attrs = Vec::new();
        self.expect(&Token::LBrace);
        while !self.peek_is(&Token::RBrace) && !self.peek_is(&Token::Eof) {
            if let Token::Ident(key) = self.peek_clone() {
                self.advance();
                self.expect(&Token::Colon);
                let value = self.parse_expr().unwrap_or(Expr::Literal(Literal::Number(0.0)));
                attrs.push(PlotAttr { key, value });
            } else {
                self.advance();
            }
            if self.peek_is(&Token::Comma) { self.advance(); }
        }
        self.expect(&Token::RBrace);
        attrs
    }

    fn parse_if_stmt(&mut self) -> Option<Statement> {
        let span = self.span();
        self.expect_kw("if")?;
        self.expect(&Token::LParen);
        let condition = self.parse_expr().unwrap_or(Expr::Literal(Literal::Bool(false)));
        self.expect(&Token::RParen);
        if self.peek_is_kw("then") { self.advance(); }

        let then_branch = if self.peek_is(&Token::LBrace) {
            self.parse_block_expr()
        } else {
            let expr = self.parse_expr().unwrap_or(Expr::Literal(Literal::Number(0.0)));
            BlockExpr { statements: vec![], tail_expr: Some(Box::new(expr)) }
        };

        let mut else_branch = None;
        if self.peek_is_kw("else") {
            self.advance();
            else_branch = if self.peek_is(&Token::LBrace) {
                Some(self.parse_block_expr())
            } else if self.peek_is_kw("if") {
                // else if — wrap as block with if-stmt inside
                if let Some(else_if_stmt) = self.parse_if_stmt() {
                    Some(BlockExpr { statements: vec![else_if_stmt], tail_expr: None })
                } else {
                    None
                }
            } else {
                let expr = self.parse_expr().unwrap_or(Expr::Literal(Literal::Number(0.0)));
                Some(BlockExpr { statements: vec![], tail_expr: Some(Box::new(expr)) })
            };
        }

        Some(Statement::IfStmt(IfStmt {
            condition,
            then_branch,
            else_ifs: vec![],
            else_branch,
            span,
        }))
    }

    fn parse_block_expr(&mut self) -> BlockExpr {
        self.expect(&Token::LBrace);
        let mut statements = Vec::new();
        let mut tail_expr = None;
        while !self.peek_is(&Token::RBrace) && !self.peek_is(&Token::Eof) {
            if self.peek_is_kw("let") || self.peek_is_kw("plot") || self.peek_is(&Token::Semicolon) {
                if let Some(stmt) = self.parse_statement() {
                    statements.push(stmt);
                }
            } else {
                // Try expression (might be tail)
                let expr = self.parse_expr();
                if self.peek_is(&Token::Semicolon) {
                    self.advance();
                    if let Some(e) = expr {
                        statements.push(Statement::Expr(e));
                    }
                } else {
                    tail_expr = expr.map(Box::new);
                    break;
                }
            }
        }
        self.expect(&Token::RBrace);
        BlockExpr { statements, tail_expr }
    }

    // ── Expressions (Pratt parser) ──

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_logic_expr()
    }

    fn parse_logic_expr(&mut self) -> Option<Expr> {
        let mut left = self.parse_comp_expr()?;
        while self.peek_is(&Token::And) || self.peek_is(&Token::Or) || self.peek_is(&Token::Xor) {
            let op = match self.advance() {
                Token::And => BinOp::And,
                Token::Or => BinOp::Or,
                Token::Xor => BinOp::Xor,
                _ => break,
            };
            let right = self.parse_comp_expr()?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right), span: self.span() };
        }
        Some(left)
    }

    fn parse_comp_expr(&mut self) -> Option<Expr> {
        let mut left = self.parse_add_expr()?;
        match self.peek() {
            Token::Gt | Token::Lt | Token::Gte | Token::Lte | Token::Eq | Token::Neq => {
                let op = match self.advance() {
                    Token::Gt => BinOp::Gt,
                    Token::Lt => BinOp::Lt,
                    Token::Gte => BinOp::Gte,
                    Token::Lte => BinOp::Lte,
                    Token::Eq => BinOp::Eq,
                    Token::Neq => BinOp::Neq,
                    _ => unreachable!(),
                };
                let right = self.parse_add_expr()?;
                left = Expr::Binary { op, left: Box::new(left), right: Box::new(right), span: self.span() };
            }
            _ => {}
        }
        Some(left)
    }

    fn parse_add_expr(&mut self) -> Option<Expr> {
        let mut left = self.parse_mul_expr()?;
        loop {
            if self.peek_is(&Token::Plus) {
                self.advance();
                let right = self.parse_mul_expr()?;
                left = Expr::Binary { op: BinOp::Add, left: Box::new(left), right: Box::new(right), span: self.span() };
            } else if self.peek_is(&Token::Minus) {
                self.advance();
                let right = self.parse_mul_expr()?;
                left = Expr::Binary { op: BinOp::Sub, left: Box::new(left), right: Box::new(right), span: self.span() };
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_mul_expr(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            if self.peek_is(&Token::Star) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::Binary { op: BinOp::Mul, left: Box::new(left), right: Box::new(right), span: self.span() };
            } else if self.peek_is(&Token::Slash) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::Binary { op: BinOp::Div, left: Box::new(left), right: Box::new(right), span: self.span() };
            } else if self.peek_is(&Token::Percent) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::Binary { op: BinOp::Mod, left: Box::new(left), right: Box::new(right), span: self.span() };
            } else if self.peek_is(&Token::Caret) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::Binary { op: BinOp::Pow, left: Box::new(left), right: Box::new(right), span: self.span() };
            } else {
                break;
            }
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        if self.peek_is(&Token::Minus) {
            self.advance();
            if let Token::NumLit(_) = self.peek() {
                // Let parse_primary handle negative numbers
                self.pos -= 1;
            } else {
                let expr = self.parse_unary()?;
                return Some(Expr::Unary { op: UnaryOp::Neg, expr: Box::new(expr), span: self.span() });
            }
        }
        if self.peek_is(&Token::Not) {
            let span = self.span();
            self.advance();
            let expr = self.parse_unary()?;
            return Some(Expr::Unary { op: UnaryOp::Not, expr: Box::new(expr), span });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            // Function call: expr(args)
            if self.peek_is(&Token::LParen) {
                self.advance();
                let mut args = Vec::new();
                if !self.peek_is(&Token::RParen) {
                    loop {
                        if let Some(arg) = self.parse_expr() {
                            args.push(arg);
                        }
                        if self.peek_is(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::RParen);
                let span = self.span();
                expr = Expr::Call { func: Box::new(expr), args, span };
                continue;
            }

            // Field access: expr.field
            if self.peek_is(&Token::Dot) {
                self.advance();
                if let Token::Ident(field) = self.peek_clone() {
                    self.advance();
                    let span = self.span();
                    expr = Expr::FieldAccess { object: Box::new(expr), field, span };
                    continue;
                }
                break;
            }

            // History reference: expr[N]
            if self.peek_is(&Token::LBracket) {
                self.advance();
                let offset = self.parse_number_literal().unwrap_or(0.0) as i32;
                self.expect(&Token::RBracket);
                let span = self.span();
                expr = Expr::HistoryRef { expr: Box::new(expr), offset, span };
                continue;
            }

            // Cross-period: expr|period|
            if self.peek_is(&Token::Pipe) {
                self.advance();
                if let Token::Ident(period_str) = self.peek_clone() {
                    self.advance();
                    if let Some(period) = Period::from_str(&period_str) {
                        self.expect(&Token::Pipe);
                        let span = self.span();
                        expr = Expr::CrossPeriod { expr: Box::new(expr), period, span };
                        continue;
                    }
                }
                self.expect(&Token::Pipe);
                continue;
            }

            break;
        }
        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let span = self.span();
        match self.peek_clone() {
            Token::NumLit(n) => {
                self.advance();
                Some(Expr::Literal(Literal::Number(n)))
            }
            Token::StrLit(s) => {
                self.advance();
                Some(Expr::Literal(Literal::String(s)))
            }
            Token::True => {
                self.advance();
                Some(Expr::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Some(Expr::Literal(Literal::Bool(false)))
            }
            Token::Null => {
                self.advance();
                Some(Expr::Literal(Literal::Null))
            }
            Token::Ident(name) => {
                self.advance();
                Some(Expr::Ident(name, span))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr();
                self.expect(&Token::RParen);
                expr
            }
            Token::If => {
                // Inline if-expression: if (cond) then_expr else else_expr
                self.advance();
                self.expect(&Token::LParen);
                let condition = self.parse_expr()?;
                self.expect(&Token::RParen);
                if self.peek_is_kw("then") { self.advance(); }
                let then_branch = self.parse_expr()?;
                let mut else_branch = None;
                if self.peek_is_kw("else") {
                    self.advance();
                    else_branch = self.parse_expr();
                }
                Some(Expr::IfExpr {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_if: vec![],
                    else_branch: else_branch.map(Box::new),
                    span,
                })
            }
            // Cross-symbol: "symbol"$expr
            Token::Dollar => {
                self.error("跨品种引用必须以品种代码开头");
                None
            }
            _ => {
                None
            }
        }
    }

    // ── Helpers ──

    fn peek(&self) -> &Token {
        if self.pos < self.tokens.len() { &self.tokens[self.pos] } else { &Token::Eof }
    }

    fn peek_clone(&self) -> Token {
        if self.pos < self.tokens.len() { self.tokens[self.pos].clone() } else { Token::Eof }
    }

    fn peek_is(&self, tok: &Token) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(tok)
    }

    fn peek_is_kw(&self, kw: &str) -> bool {
        match self.peek() {
            Token::Ident(s) => s == kw,
            Token::Let => kw == "let",
            Token::Plot => kw == "plot",
            Token::Param => kw == "param",
            Token::Indicator => kw == "indicator",
            Token::If => kw == "if",
            Token::Then => kw == "then",
            Token::Else => kw == "else",
            _ => false,
        }
    }

    fn match_kw(&mut self, kw: &str) -> bool {
        if self.peek_is_kw(kw) {
            self.advance();
            return true;
        }
        false
    }

    fn expect_kw(&mut self, kw: &str) -> Option<()> {
        if self.match_kw(kw) {
            Some(())
        } else {
            self.error(&format!("期望 '{}', 但得到 {:?}", kw, self.peek()));
            None
        }
    }

    fn advance(&mut self) -> Token {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            t
        } else {
            Token::Eof
        }
    }

    fn expect(&mut self, expected: &Token) -> Option<()> {
        if self.peek_is(expected) {
            self.advance();
            Some(())
        } else {
            self.error(&format!("期望 {:?}, 但得到 {:?}", expected, self.peek()));
            None
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn parse_number_literal(&mut self) -> Option<f64> {
        match self.peek_clone() {
            Token::NumLit(n) => { self.advance(); Some(n) }
            Token::Minus => {
                self.advance();
                if let Token::NumLit(n) = self.peek_clone() {
                    self.advance();
                    Some(-n)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_type_annotation(&mut self) -> Option<Type> {
        if self.peek_is(&Token::Colon) {
            self.advance();
            match self.peek_clone() {
                Token::Bool => { self.advance(); Some(Type::Bool) }
                Token::String => { self.advance(); Some(Type::String) }
                Token::Color => { self.advance(); Some(Type::Color) }
                Token::Number => { self.advance(); Some(Type::Number) }
                _ => None,
            }
        } else {
            None
        }
    }

    fn span(&self) -> Span {
        Span::new(0, 0) // TODO: track actual line/col from tokens
    }

    fn error(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Program {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    #[test]
    fn test_parse_simple_var() {
        let prog = parse("let ma5 = sma(close, 20);");
        assert_eq!(prog.items.len(), 1);
        match &prog.items[0] {
            TopLevel::Statement(Statement::VarDecl(vd)) => {
                assert_eq!(vd.name, "ma5");
            }
            _ => panic!("Expected VarDecl"),
        }
    }

    #[test]
    fn test_parse_comparison() {
        let prog = parse("let signal = close > sma(close, 20);");
        assert_eq!(prog.items.len(), 1);
    }

    #[test]
    fn test_parse_if_else() {
        let prog = parse("let s = if (rsi < 30) then 1 else 0;");
        assert_eq!(prog.items.len(), 1);
    }

    #[test]
    fn test_parse_field_access() {
        let prog = parse("let dif = macd(close, 12, 26, 9).dif;");
        assert_eq!(prog.items.len(), 1);
    }

    #[test]
    fn test_parse_history_ref() {
        let prog = parse("let prev = close[1];");
        assert_eq!(prog.items.len(), 1);
    }

    #[test]
    fn test_parse_cross() {
        let prog = parse("let signal = cross(ema(close, 12), ema(close, 26));");
        assert_eq!(prog.items.len(), 1);
    }
}

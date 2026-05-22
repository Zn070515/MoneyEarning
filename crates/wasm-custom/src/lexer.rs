//! ME Script Lexer — tokenizes source into token stream
//! Supports dual syntax: TDX classic mode + modern mode

use crate::ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Let, Plot, Param, Indicator, If, Then, Else,
    True, False, Null,
    And, Or, Xor, Not,
    Bool, String, Color, Number,
    // TDX classic keywords
    DrawNull, NoDraw, ColorStick, Stick, LineThick,
    DotLine, PointDot, DrawIcon, DrawText,
    // Identifiers & literals
    Ident(String),
    NumLit(f64),
    StrLit(String),
    // Operators
    Plus, Minus, Star, Slash, Percent, Caret,
    Eq, Neq, Gt, Lt, Gte, Lte,
    Assign,          // := TDX no-output
    ColonAssign,     // : TDX output
    AssignEq,        // = standalone assignment
    Colon, Semicolon,
    // Delimiters
    LParen, RParen, LBracket, RBracket, LBrace, RBrace,
    Comma, Dot, Pipe, Dollar, Hash,
    Arrow,           // ->
    // Special
    Eof,
}

impl Token {
    pub fn is_bin_op(&self) -> bool {
        matches!(self, Token::Plus | Token::Minus | Token::Star | Token::Slash
            | Token::Percent | Token::Caret | Token::Eq | Token::Neq
            | Token::Gt | Token::Lt | Token::Gte | Token::Lte
            | Token::And | Token::Or | Token::Xor)
    }
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    errors: Vec<String>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            errors: Vec::new(),
        }
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    #[allow(dead_code)]
    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' { break; }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        let mut depth = 1;
        while depth > 0 {
            match self.advance() {
                None => {
                    self.errors.push("未闭合的块注释".to_string());
                    return;
                }
                Some('/') if self.peek() == Some('*') => {
                    self.advance();
                    depth += 1;
                }
                Some('*') if self.peek() == Some('/') => {
                    self.advance();
                    depth -= 1;
                }
                _ => {}
            }
        }
    }

    fn read_number(&mut self) -> f64 {
        let start = self.pos;
        // Negative sign
        if self.peek() == Some('-') && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
        }
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some('.') && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
            self.advance(); // consume '.'
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse().unwrap_or(0.0)
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while self.peek().map_or(false, |c| c.is_alphanumeric() || c == '_') {
            self.advance();
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn read_string(&mut self) -> String {
        self.advance(); // consume opening quote
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch == '"' { break; }
            if ch == '\\' { self.advance(); }
            self.advance();
        }
        let end = self.pos;
        if self.peek() == Some('"') { self.advance(); }
        self.chars[start..end].iter().collect()
    }

    fn keyword_or_ident(&self, s: &str) -> Token {
        match s {
            "let" => Token::Let,
            "plot" => Token::Plot,
            "param" => Token::Param,
            "indicator" => Token::Indicator,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            "and" => Token::And,
            "or" => Token::Or,
            "xor" => Token::Xor,
            "not" => Token::Not,
            "bool" => Token::Bool,
            "string" => Token::String,
            "color" => Token::Color,
            "number" => Token::Number,
            "DRAWTEXT" | "DRAWTEXT_ZQ" | "DRAWTEXT_FIX" => Token::DrawText,
            "DRAWICON" | "DRAWICON_ZQ" => Token::DrawIcon,
            "STICK" | "STICKLINE" => Token::Stick,
            "COLORSTICK" => Token::ColorStick,
            "DRAWNULL" => Token::DrawNull,
            "NODRAW" => Token::NoDraw,
            "DOTLINE" => Token::DotLine,
            "POINTDOT" => Token::PointDot,
            "LINETHICK" => Token::LineThick,
            "DRAWBAND" | "DRAWNUMBER" | "DRAWKLINE" | "DRAWSL" | "DRAWTEXTEX" | "DRAWBMP" => {
                Token::Ident(s.to_string())
            }
            _ => Token::Ident(s.to_string()),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            let ch = match self.peek() {
                None => break,
                Some(c) => c,
            };

            // Line comment //
            if ch == '/' && self.peek_ahead(1) == Some('/') {
                self.advance();
                self.advance();
                self.skip_line_comment();
                continue;
            }

            // Block comment /* */
            if ch == '/' && self.peek_ahead(1) == Some('*') {
                self.advance();
                self.advance();
                self.skip_block_comment();
                continue;
            }

            // String
            if ch == '"' {
                tokens.push(Token::StrLit(self.read_string()));
                continue;
            }

            // Number (or negative number)
            if ch.is_ascii_digit() || (ch == '-' && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit())) {
                tokens.push(Token::NumLit(self.read_number()));
                continue;
            }

            // Identifier or keyword
            if ch.is_alphabetic() || ch == '_' {
                let ident = self.read_ident();
                tokens.push(self.keyword_or_ident(&ident));
                continue;
            }

            // Operators and delimiters
            self.advance();
            match ch {
                '(' => tokens.push(Token::LParen),
                ')' => tokens.push(Token::RParen),
                '[' => tokens.push(Token::LBracket),
                ']' => tokens.push(Token::RBracket),
                '{' => tokens.push(Token::LBrace),
                '}' => tokens.push(Token::RBrace),
                ',' => tokens.push(Token::Comma),
                '.' => tokens.push(Token::Dot),
                ';' => tokens.push(Token::Semicolon),
                '|' => tokens.push(Token::Pipe),
                '$' => tokens.push(Token::Dollar),
                '#' => tokens.push(Token::Hash),
                '^' => tokens.push(Token::Caret),
                '%' => tokens.push(Token::Percent),
                '+' => tokens.push(Token::Plus),
                '-' => {
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::Arrow);
                    } else {
                        tokens.push(Token::Minus)
                    }
                }
                '*' => {
                    if self.peek() == Some('*') {
                        self.advance();
                        tokens.push(Token::Caret); // ** = pow, reuse Caret
                    } else {
                        tokens.push(Token::Star)
                    }
                }
                '/' => tokens.push(Token::Slash),
                ':' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::Assign); // := TDX no-output
                    } else {
                        tokens.push(Token::Colon) // : TDX output or type annotation
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::Gte);
                    } else {
                        tokens.push(Token::Gt)
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::Lte);
                    } else if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::Neq);
                    } else {
                        tokens.push(Token::Lt)
                    }
                }
                '=' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::Eq); // == comparison
                    } else {
                        tokens.push(Token::AssignEq) // = assignment
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::Neq);
                    } else {
                        tokens.push(Token::Not)
                    }
                }
                _ => {
                    // Skip unknown characters (TDX compatibility: ignore junk)
                }
            }
        }
        tokens.push(Token::Eof);
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_simple_expr() {
        let mut lexer = Lexer::new("sma(close, 20) > close[1]");
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[0], Token::Ident(_))); // sma
        assert!(matches!(tokens[1], Token::LParen));
        assert!(matches!(tokens[2], Token::Ident(_))); // close
        assert!(matches!(tokens[3], Token::Comma));
        assert!(matches!(tokens[4], Token::NumLit(20.0)));
        assert!(matches!(tokens[5], Token::RParen));
        assert!(matches!(tokens[6], Token::Gt));
        // close[1] → Ident("close"), LBracket, NumLit(1.0), RBracket
        assert!(matches!(tokens[7], Token::Ident(_)));
        assert!(matches!(tokens[8], Token::LBracket));
        assert!(matches!(tokens[9], Token::NumLit(1.0)));
        assert!(matches!(tokens[10], Token::RBracket));
        assert!(matches!(tokens[11], Token::Eof));
    }

    #[test]
    fn test_lex_indicator() {
        let mut lexer = Lexer::new(r#"indicator "MACD" { let dif = ema(close, 12) - ema(close, 26); }"#);
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[0], Token::Indicator));
        assert!(matches!(tokens[1], Token::StrLit(_)));
        assert!(matches!(tokens[2], Token::LBrace));
        assert!(matches!(tokens[3], Token::Let));
    }

    #[test]
    fn test_tdx_assign() {
        let mut lexer = Lexer::new("MA5:MA(C,5);\nRSI_VAL:=RSI(C,14);");
        let tokens = lexer.tokenize();
        // MA5 : MA ( C , 5 ) ;
        assert!(matches!(tokens[0], Token::Ident(_))); // MA5
        assert!(matches!(tokens[1], Token::Colon));
        assert!(matches!(tokens[2], Token::Ident(_))); // MA
    }

    #[test]
    fn test_negative_number() {
        let mut lexer = Lexer::new("close - 5 > -3");
        let tokens = lexer.tokenize();
        // close, -, 5, >, -3
        assert!(matches!(tokens[0], Token::Ident(_)));
        assert!(matches!(tokens[1], Token::Minus));
        assert!(matches!(tokens[2], Token::NumLit(5.0)));
        assert!(matches!(tokens[3], Token::Gt));
        assert!(matches!(tokens[4], Token::NumLit(-3.0)));
    }
}

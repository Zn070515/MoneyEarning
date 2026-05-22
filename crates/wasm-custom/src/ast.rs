//! ME Script AST — full node types matching the EBNF grammar (§14.12)

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col }
    }
}

// ── Top-level ──

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<TopLevel>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Indicator(IndicatorDef),
    Statement(Statement),
}

#[derive(Debug, Clone)]
pub struct IndicatorDef {
    pub name: String,
    pub params: Vec<ParamDecl>,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub name: String,
    pub default: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub span: Span,
}

// ── Statements ──

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl(VarDecl),
    PlotDecl(PlotDecl),
    IfStmt(IfStmt),
    Expr(Expr),
    AssignNoOutput(String, Expr), // TDX :=
    AssignOutput(String, Expr),   // TDX :
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub type_ann: Option<Type>,
    pub value: Expr,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PlotDecl {
    pub name: String,
    pub value: Expr,
    pub attrs: Vec<PlotAttr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PlotAttr {
    pub key: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: BlockExpr,
    pub else_ifs: Vec<(Expr, BlockExpr)>,
    pub else_branch: Option<BlockExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BlockExpr {
    pub statements: Vec<Statement>,
    pub tail_expr: Option<Box<Expr>>,
}

// ── Types ──

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    Bool,
    String,
    Color,
}

// ── Expressions ──

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(String, Span),
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
        span: Span,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    CrossPeriod {
        expr: Box<Expr>,
        period: Period,
        span: Span,
    },
    CrossSymbol {
        symbol: String,
        expr: Box<Expr>,
        span: Span,
    },
    HistoryRef {
        expr: Box<Expr>,
        offset: i32,
        span: Span,
    },
    IfExpr {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_if: Vec<(Box<Expr>, Box<Expr>)>,
        else_branch: Option<Box<Expr>>,
        span: Span,
    },
    PlotRef(String, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    Bool(bool),
    String(String),
    Null,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Pow,
    // Comparison
    Eq, Neq, Gt, Lt, Gte, Lte,
    // Logical
    And, Or, Xor,
}

impl BinOp {
    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::And | BinOp::Or | BinOp::Xor => 1,
            BinOp::Eq | BinOp::Neq | BinOp::Gt | BinOp::Lt | BinOp::Gte | BinOp::Lte => 2,
            BinOp::Add | BinOp::Sub => 3,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 4,
            BinOp::Pow => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Period {
    Min1, Min5, Min15, Min30, Min60,
    Day, Week, Month, Quarter, Year,
}

impl Period {
    pub fn from_str(s: &str) -> Option<Period> {
        match s.to_lowercase().as_str() {
            "1min" => Some(Period::Min1),
            "5min" => Some(Period::Min5),
            "15min" => Some(Period::Min15),
            "30min" => Some(Period::Min30),
            "60min" => Some(Period::Min60),
            "d" | "day" => Some(Period::Day),
            "w" | "week" => Some(Period::Week),
            "m" | "month" => Some(Period::Month),
            "q" | "quarter" => Some(Period::Quarter),
            "y" | "year" => Some(Period::Year),
            _ => None,
        }
    }
}

// ── Compiler IR ──

/// Flattened computation IR node (output of compilation, input to runtime)
#[derive(Debug, Clone)]
pub struct IrProgram {
    pub name: Option<String>,
    pub params: Vec<IrParam>,
    pub nodes: Vec<IrNode>,
    pub outputs: Vec<(String, usize)>, // (name, node_index)
    pub input_len: usize,
}

#[derive(Debug, Clone)]
pub struct IrParam {
    pub name: String,
    pub default: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum IrNode {
    Input { name: String, index: usize },
    Constant(f64),
    SeriesOp {
        op: SeriesFn,
        inputs: Vec<usize>,
        period: Option<usize>,
    },
    BinaryOp {
        op: IrBinOp,
        left: usize,
        right: usize,
    },
    UnaryOp {
        op: IrUnaryOp,
        input: usize,
    },
    Shift {
        input: usize,
        offset: i32,
    },
    Conditional {
        cond: usize,
        then_val: usize,
        else_val: usize,
    },
}

#[derive(Debug, Clone)]
pub enum SeriesFn {
    Sma, Ema, Wma, Rma, Kama, Hma,
    Stdev, Variance, ZScore,
    Rsi, MacdDif, MacdDea, MacdHist,
    KdjK, KdjD, KdjJ,
    StochK, StochD,
    Cci, Mfi, Trix, Fisher,
    Adx, AroonUp, AroonDown, Psar,
    BbUpper, BbMiddle, BbLower, BbBw, BbPctB,
    Atr, KcUpper, KcMiddle, KcLower,
    DonchianUpper, DonchianMiddle, DonchianLower,
    Obv, AdOsc, Cmf, Eom, Pvt,
    Cross, LongCross,
    Ref, Hhv, Llv, Sum, BarsLast, BarsSince,
    Count, Every, Exist, Filter, BackSet,
    Abs, Max, Min, Pow, Sqrt, Log, Ln, Exp,
    Round, Ceil, Floor, Mod, Sign,
    Sin, Cos, Tan,
    If, Between, Range,
    CdlDoji, CdlHammer, CdlShootingStar, CdlMarubozu,
    CdlEngulfing, CdlHarami, CdlMorningStar, CdlEveningStar,
    CdlThreeWhiteSoldiers,
    Roc, PercentReturn, Drawdown,
    CandleOpen, CandleHigh, CandleLow, CandleClose, CandleVolume,
}

#[derive(Debug, Clone, Copy)]
pub enum IrBinOp {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, Neq, Gt, Lt, Gte, Lte,
    And, Or, Xor,
}

#[derive(Debug, Clone, Copy)]
pub enum IrUnaryOp {
    Neg, Not,
}

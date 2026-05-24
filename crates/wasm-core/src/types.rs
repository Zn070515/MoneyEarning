use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCV {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: Option<f64>,
    pub turnover: Option<f64>,
    pub trade_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Column {
    F64(Vec<f64>),
    I32(Vec<i32>),
    Bool(Vec<bool>),
    String(Vec<String>),
}

impl Column {
    pub fn len(&self) -> usize {
        match self {
            Column::F64(v) => v.len(),
            Column::I32(v) => v.len(),
            Column::Bool(v) => v.len(),
            Column::String(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn f64_values(&self) -> Option<&[f64]> {
        match self {
            Column::F64(v) => Some(v),
            _ => None,
        }
    }

    pub fn f64_slice(&self, start: usize, end: usize) -> Column {
        match self {
            Column::F64(v) => Column::F64(v[start..end].to_vec()),
            _ => Column::F64(vec![]),
        }
    }

    pub fn to_f64_vec(&self) -> Vec<f64> {
        match self {
            Column::F64(v) => v.clone(),
            Column::I32(v) => v.iter().map(|&x| x as f64).collect(),
            Column::Bool(v) => v.iter().map(|&x| if x { 1.0 } else { 0.0 }).collect(),
            Column::String(_) => vec![],
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorOutput {
    pub name: String,
    pub values: Column,
    pub style: OutputStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputStyle {
    Line,
    Histogram,
    Dots,
    Band { upper: Column, lower: Column },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndError {
    InvalidName,
    InvalidParams(String),
    DataInsufficient(usize),
    ComputationFailed(String),
}

impl std::fmt::Display for IndError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndError::InvalidName => write!(f, "无效的指标名称"),
            IndError::InvalidParams(s) => write!(f, "无效参数: {}", s),
            IndError::DataInsufficient(n) => write!(f, "数据不足，需要至少 {} 条", n),
            IndError::ComputationFailed(s) => write!(f, "计算失败: {}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorMeta {
    pub name: String,
    pub name_cn: String,
    pub category: String,
    pub params: Vec<ParamDef>,
    pub is_free: bool,
    pub tdx_equivalent: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    pub name: String,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanExpr {
    pub op: String,
    pub children: Vec<ScanExpr>,
    pub indicator: Option<String>,
    pub params: Option<std::collections::HashMap<String, f64>>,
    pub compare_op: Option<String>,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtResult {
    pub total_return: f64,
    pub annual_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub win_rate: f64,
    pub profit_loss_ratio: f64,
    pub total_trades: u32,
    pub equity_curve: Vec<(String, f64)>,
    pub monthly_returns: Vec<(String, f64)>,
    pub trades: Vec<TradeRecord>,
    pub max_drawdown_duration: i64,
    pub annual_volatility: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub buy_date: String,
    pub sell_date: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub holding_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    pub version: String,
    pub fingerprint: String,
    pub date: String,
    pub tier: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionResult {
    pub price_levels: Vec<f64>,
    pub chip_volume: Vec<f64>,
    pub avg_cost: f64,
    pub weighted_avg_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeProfileResult {
    pub levels: Vec<ProfileLevel>,
    pub poc: f64,
    pub vah: f64,
    pub val: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileLevel {
    pub price: f64,
    pub volume: f64,
    pub is_poc: bool,
}

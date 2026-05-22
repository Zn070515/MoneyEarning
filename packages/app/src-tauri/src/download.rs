/// A-share market data downloader — fetches from free public APIs (Eastmoney/Sina)
use serde::{Deserialize, Serialize};
use std::sync::MutexGuard;
use rusqlite::Connection;

type DbGuard<'a> = MutexGuard<'a, Option<Connection>>;

/// Download stock list from Eastmoney API
pub fn download_stock_list() -> Result<Vec<StockListItem>, String> {
    let url = "https://push2.eastmoney.com/api/qt/clist/get?fid=f3&po=1&pz=6000&pn=1&np=1&fltt=2&invt=2&fields=f2,f3,f12,f14";
    let resp = reqwest::blocking::get(url).map_err(|e| format!("HTTP请求失败: {}", e))?;
    let body = resp.text().map_err(|e| format!("读取响应失败: {}", e))?;
    let parsed: EastmoneyStockList = serde_json::from_str(&body)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    Ok(parsed.data.diff.iter().map(|item| StockListItem {
        code: item.code.clone(),
        name: item.name.clone(),
        price: item.price,
        change_pct: item.change_pct,
    }).collect())
}

/// Download daily K-line data for a stock (free, no auth needed)
pub fn download_daily_kline(code: &str, market: &str) -> Result<Vec<KlineRow>, String> {
    // Determine secid: 1=SH, 0=SZ
    let secid = match market.to_uppercase().as_str() {
        "SH" => format!("1.{}", code),
        "SZ" => format!("0.{}", code),
        _ => {
            // Auto-detect: 60xxxx = SH, others = SZ
            if code.starts_with("60") || code.starts_with("68") {
                format!("1.{}", code)
            } else {
                format!("0.{}", code)
            }
        }
    };

    let url = format!(
        "https://push2his.eastmoney.com/api/qt/stock/kline/get?secid={}&fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61&klt=101&fqt=0&beg=19900101&end=20991231",
        secid
    );

    let resp = reqwest::blocking::get(&url).map_err(|e| format!("HTTP请求失败: {}", e))?;
    let body = resp.text().map_err(|e| format!("读取响应失败: {}", e))?;
    let parsed: EastmoneyKline = serde_json::from_str(&body)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    let rows: Vec<KlineRow> = parsed.data.klines.iter()
        .filter_map(|line| parse_kline_line(line))
        .collect();

    Ok(rows)
}

fn parse_kline_line(line: &str) -> Option<KlineRow> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 11 { return None; }
    // Format: 日期,开盘,收盘,最高,最低,成交量,成交额,振幅,涨跌幅,涨跌额,换手率
    Some(KlineRow {
        trade_date: parts[0].to_string(),
        open: parts[1].parse().ok()?,
        close: parts[2].parse().ok()?,
        high: parts[3].parse().ok()?,
        low: parts[4].parse().ok()?,
        volume: parts[5].parse().ok()?,
        amount: parts[6].parse().ok()?,
        amplitude: parts[7].parse().ok()?,
        change_pct: parts[8].parse().ok()?,
        change_amount: parts[9].parse().ok()?,
        turnover: parts[10].parse().unwrap_or(0.0),
    })
}

/// Save downloaded kline data to the database
pub fn save_kline_to_db(guard: &DbGuard<'_>, code: &str, name: &str, exchange: &str, rows: &[KlineRow]) -> Result<(usize, usize), String> {
    use crate::db;

    // Ensure stock exists
    let ipo_date = rows.first().map(|r| r.trade_date.clone());
    let stock_id = db::upsert_stock(guard, code, name, exchange, ipo_date.as_deref())
        .map_err(|e| e.to_string())?;

    let inserted = db::bulk_insert_daily(guard, stock_id, rows).map_err(|e| e.to_string())?;

    Ok((1, inserted))
}

/// Download and import a single stock's full history
pub fn download_and_import(guard: &DbGuard<'_>, code: &str, name: Option<&str>) -> Result<ImportSummary, String> {
    let market = if code.starts_with("60") || code.starts_with("68") { "SH" } else { "SZ" };
    let rows = download_daily_kline(code, market)?;
    if rows.is_empty() {
        return Err("未获取到数据，请检查股票代码".into());
    }
    let name = name.unwrap_or(code);
    let (_, inserted) = save_kline_to_db(guard, code, name, market, &rows)?;

    let first = rows.first().map(|r| r.trade_date.clone());
    let last = rows.last().map(|r| r.trade_date.clone());

    Ok(ImportSummary {
        code: code.to_string(),
        name: name.to_string(),
        rows_inserted: inserted,
        date_range: first.zip(last),
    })
}

/// Download minute-level K-line data (klt: 1=1min, 5=5min, 15=15min, 30=30min, 60=60min)
pub fn download_minute_kline(code: &str, market: &str, klt: u32) -> Result<Vec<MinuteKlineRow>, String> {
    let secid = match market.to_uppercase().as_str() {
        "SH" => format!("1.{}", code),
        "SZ" => format!("0.{}", code),
        _ => {
            if code.starts_with("60") || code.starts_with("68") {
                format!("1.{}", code)
            } else {
                format!("0.{}", code)
            }
        }
    };

    let url = format!(
        "https://push2his.eastmoney.com/api/qt/stock/kline/get?secid={}&fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61&klt={}&fqt=0&beg=20200101&end=20991231",
        secid, klt
    );

    let resp = reqwest::blocking::get(&url).map_err(|e| format!("HTTP请求失败: {}", e))?;
    let body = resp.text().map_err(|e| format!("读取响应失败: {}", e))?;
    let parsed: EastmoneyKline = serde_json::from_str(&body)
        .map_err(|e| format!("JSON解析失败: {}", e))?;

    let rows: Vec<MinuteKlineRow> = parsed.data.klines.iter()
        .filter_map(|line| parse_minute_line(line))
        .collect();

    Ok(rows)
}

fn parse_minute_line(line: &str) -> Option<MinuteKlineRow> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 8 { return None; }
    // Minute format: 时间,开盘,收盘,最高,最低,成交量,成交额,振幅,涨跌幅,涨跌额,换手率
    Some(MinuteKlineRow {
        trade_time: parts[0].to_string(),
        open: parts[1].parse().ok()?,
        close: parts[2].parse().ok()?,
        high: parts[3].parse().ok()?,
        low: parts[4].parse().ok()?,
        volume: parts[5].parse().ok()?,
        amount: parts[6].parse().ok()?,
    })
}

/// Download and import minute data for a single stock
pub fn download_minute_and_import(
    guard: &DbGuard<'_>, code: &str, klt: u32,
) -> Result<MinuteImportSummary, String> {
    let market = if code.starts_with("60") || code.starts_with("68") { "SH" } else { "SZ" };
    let rows = download_minute_kline(code, market, klt)?;
    if rows.is_empty() {
        return Err("未获取到分钟数据，请检查股票代码或交易日".into());
    }

    use crate::db;
    // Ensure stock exists
    let stock_id = db::upsert_stock(guard, code, code, market, None)
        .map_err(|e| e.to_string())?;

    // Convert to db MinuteRow
    let db_rows: Vec<db::MinuteRow> = rows.iter().map(|r| db::MinuteRow {
        trade_time: r.trade_time.clone(),
        open: r.open, high: r.high, low: r.low, close: r.close,
        volume: r.volume, amount: r.amount,
    }).collect();

    let inserted = db::bulk_insert_minute(guard, stock_id, &db_rows).map_err(|e| e.to_string())?;

    let first = rows.first().map(|r| r.trade_time.clone());
    let last = rows.last().map(|r| r.trade_time.clone());

    Ok(MinuteImportSummary {
        code: code.to_string(),
        klt,
        klt_label: klt_label(klt),
        rows_inserted: inserted,
        time_range: first.zip(last),
    })
}

fn klt_label(klt: u32) -> String {
    match klt {
        1 => "1分钟".to_string(),
        5 => "5分钟".to_string(),
        15 => "15分钟".to_string(),
        30 => "30分钟".to_string(),
        60 => "60分钟".to_string(),
        _ => format!("{}分钟", klt),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteImportSummary {
    pub code: String,
    pub klt: u32,
    pub klt_label: String,
    pub rows_inserted: usize,
    pub time_range: Option<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteKlineRow {
    pub trade_time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
}

// ── Data types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockListItem {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineRow {
    pub trade_date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub amplitude: f64,
    pub change_pct: f64,
    pub change_amount: f64,
    pub turnover: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub code: String,
    pub name: String,
    pub rows_inserted: usize,
    pub date_range: Option<(String, String)>,
}

// ── JSON response types ──

#[derive(Deserialize)]
struct EastmoneyStockList {
    data: EastmoneyStockData,
}

#[derive(Deserialize)]
struct EastmoneyStockData {
    diff: Vec<EastmoneyStockItem>,
}

#[derive(Deserialize)]
struct EastmoneyStockItem {
    #[serde(rename = "f12")]
    code: String,
    #[serde(rename = "f14")]
    name: String,
    #[serde(rename = "f2")]
    price: f64,
    #[serde(rename = "f3")]
    change_pct: f64,
}

#[derive(Deserialize)]
struct EastmoneyKline {
    data: EastmoneyKlineData,
}

#[derive(Deserialize)]
struct EastmoneyKlineData {
    klines: Vec<String>,
}

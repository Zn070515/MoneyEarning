use std::fs;
use std::io::Read;
use std::path::Path;
use rusqlite::params;

/// 通达信 .day 日线数据文件格式
/// 每条记录 32 字节:
///   [0..4)   date:   i32  YYYYMMDD
///   [4..8)   open:   i32  开盘价×100（分）
///   [8..12)  high:   i32  最高价×100
///   [12..16) low:    i32  最低价×100
///   [16..20) close:  i32  收盘价×100
///   [20..24) amount: f32  成交额（元）
///   [24..28) volume: i32  成交量（股）
///   [28..32) reserved: i32 保留

#[derive(Debug, Clone)]
pub struct DayRecord {
    pub date: String,    // YYYY-MM-DD
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub amount: f64,
    pub volume: f64,
}

pub fn parse_day_file(path: &Path) -> Result<Vec<DayRecord>, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("无法打开文件: {}", e))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| format!("读取文件失败: {}", e))?;

    if buf.len() % 32 != 0 {
        return Err(format!(
            "文件大小 {} 不是 32 的倍数，可能不是有效的通达信.day文件",
            buf.len()
        ));
    }

    let count = buf.len() / 32;
    let mut records = Vec::with_capacity(count);

    for i in 0..count {
        let offset = i * 32;
        let date_i32 = i32_from_le(&buf, offset);
        let open_raw = i32_from_le(&buf, offset + 4);
        let high_raw = i32_from_le(&buf, offset + 8);
        let low_raw = i32_from_le(&buf, offset + 12);
        let close_raw = i32_from_le(&buf, offset + 16);
        let amount = f32_from_le(&buf, offset + 20) as f64;
        let volume = i32_from_le(&buf, offset + 24) as f64;

        let date_str = format!("{:08}", date_i32);
        if date_str.len() != 8 || date_i32 < 19900101 {
            continue; // skip invalid dates
        }
        let date = format!(
            "{}-{}-{}",
            &date_str[0..4],
            &date_str[4..6],
            &date_str[6..8]
        );

        records.push(DayRecord {
            date,
            open: open_raw as f64 / 100.0,
            high: high_raw as f64 / 100.0,
            low: low_raw as f64 / 100.0,
            close: close_raw as f64 / 100.0,
            amount,
            volume,
        });
    }

    Ok(records)
}

/// 从文件名提取代码和交易所
/// sh600519.day → (SH, 600519)
/// sz000001.day → (SZ, 000001)
pub fn parse_filename(path: &Path) -> Option<(String, String)> {
    let stem = path.file_stem()?.to_str()?;
    if stem.len() < 3 {
        return None;
    }
    let (exch, code) = match &stem[..2] {
        "sh" | "SH" => ("SH", &stem[2..]),
        "sz" | "SZ" => ("SZ", &stem[2..]),
        "bj" | "BJ" => ("BJ", &stem[2..]),
        _ => {
            // Try auto-detect by code prefix
            if stem.starts_with("60") {
                ("SH", stem)
            } else if stem.starts_with("00") || stem.starts_with("30") {
                ("SZ", stem)
            } else if stem.starts_with("83") || stem.starts_with("87") || stem.starts_with("43") {
                ("BJ", stem)
            } else {
                return None;
            }
        }
    };
    if code.len() < 6 { return None; }
    Some((exch.to_string(), code[..6].to_string()))
}

/// 扫描目录下所有 .day 文件
pub fn scan_day_files(dir: &Path) -> Result<Vec<(std::path::PathBuf, String, String)>, String> {
    let mut results = Vec::new();
    // 支持两种目录结构:
    //   1. vipdoc/sh/lday/*.day  (递归扫描)
    //   2. flat directory with *.day files
    scan_dir_recursive(dir, &mut results, 0)?;
    Ok(results)
}

fn scan_dir_recursive(
    dir: &Path,
    results: &mut Vec<(std::path::PathBuf, String, String)>,
    depth: usize,
) -> Result<(), String> {
    if depth > 4 || !dir.is_dir() {
        return Ok(());
    }
    let entries = fs::read_dir(dir).map_err(|e| format!("读取目录失败 {}: {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("目录遍历失败: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(&path, results, depth + 1)?;
        } else if let Some(ext) = path.extension() {
            if ext.to_str() == Some("day") {
                if let Some((exch, code)) = parse_filename(&path) {
                    results.push((path, exch, code));
                }
            }
        }
    }
    Ok(())
}

/// 导入单个 .day 文件到数据库
pub fn import_day_file(
    guard: &std::sync::MutexGuard<'static, Option<rusqlite::Connection>>,
    file_path: &str, stock_code: Option<&str>, exchange: Option<&str>,
) -> Result<super::ImportResult, Box<dyn std::error::Error>> {
    let conn = guard.as_ref().ok_or("数据库未初始化")?;
    let path = Path::new(file_path);

    let (exch, code) = if let (Some(c), Some(e)) = (stock_code, exchange) {
        (e.to_string(), c.to_string())
    } else if let Some((e, c)) = parse_filename(path) {
        (e, c)
    } else {
        return Err("无法从文件名识别股票代码和交易所，请手动指定".into());
    };

    let records = parse_day_file(path).map_err(|e| format!("解析通达信文件失败: {}", e))?;
    if records.is_empty() {
        return Err("文件中无有效数据".into());
    }

    // Ensure stock exists
    let display_name = stock_code.map(|s| s.to_string()).unwrap_or_else(|| code.clone());
    conn.execute(
        "INSERT OR IGNORE INTO stocks (code, name, exchange) VALUES (?1, ?2, ?3)",
        params![code, display_name, exch],
    )?;
    let stock_id: i64 = conn.query_row(
        "SELECT id FROM stocks WHERE code = ?1", params![code], |r| r.get(0),
    )?;

    let tx = conn.unchecked_transaction()?;
    let mut row_count = 0usize;
    let mut skipped = 0usize;
    let mut first_date = None;
    let mut last_date = None;

    for rec in &records {
        let result = tx.execute(
            "INSERT OR IGNORE INTO daily_prices (stock_id, trade_date, open, high, low, close, volume, amount)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![stock_id, rec.date, rec.open, rec.high, rec.low, rec.close, rec.volume, rec.amount],
        );
        match result {
            Ok(1) => {
                row_count += 1;
                if first_date.is_none() {
                    first_date = Some(rec.date.clone());
                }
                last_date = Some(rec.date.clone());
            }
            _ => skipped += 1,
        }
    }
    tx.commit()?;

    Ok(super::ImportResult {
        stock_count: 1,
        row_count,
        skipped,
        date_range: if let (Some(f), Some(l)) = (first_date, last_date) {
            Some((f, l))
        } else {
            None
        },
    })
}

/// 批量导入目录下所有 .day 文件
pub fn import_day_directory(
    guard: &std::sync::MutexGuard<'static, Option<rusqlite::Connection>>,
    dir_path: &str,
) -> Result<Vec<super::ImportResult>, Box<dyn std::error::Error>> {
    let dir = Path::new(dir_path);
    let files = scan_day_files(dir).map_err(|e| format!("扫描目录失败: {}", e))?;
    if files.is_empty() {
        return Err("目录中未找到 .day 文件".into());
    }

    let mut results = Vec::new();
    for (path, exch, code) in &files {
        match import_day_file(guard, path.to_str().unwrap_or(""), Some(code), Some(exch)) {
            Ok(r) => results.push(r),
            Err(e) => {
                eprintln!("导入 {} 失败: {}", path.display(), e);
            }
        }
    }

    if results.is_empty() {
        return Err("所有文件导入均失败".into());
    }
    Ok(results)
}

// ── Binary helpers ──

fn i32_from_le(buf: &[u8], offset: usize) -> i32 {
    let b = &buf[offset..offset + 4];
    i32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

fn f32_from_le(buf: &[u8], offset: usize) -> f32 {
    let b = &buf[offset..offset + 4];
    f32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

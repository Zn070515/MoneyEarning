use rusqlite::{Connection, params};
use std::io::{BufRead, BufReader};
use std::fs::File;

pub fn import_csv_file(
    guard: &std::sync::MutexGuard<'static, Option<Connection>>,
    file_path: &str, stock_code: &str, exchange: &str,
) -> Result<super::ImportResult, Box<dyn std::error::Error>> {
    let conn = guard.as_ref().ok_or("数据库未初始化")?;
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Detect header
    let header = lines.next().ok_or("CSV文件为空")??;

    let (date_col, open_col, high_col, low_col, close_col, vol_col, amt_col) =
        detect_columns(&header)?;

    // Ensure stock exists
    conn.execute(
        "INSERT OR IGNORE INTO stocks (code, name, exchange) VALUES (?1, ?2, ?3)",
        params![stock_code, stock_code, exchange],
    )?;
    let stock_id: i64 = conn.query_row(
        "SELECT id FROM stocks WHERE code = ?1", params![stock_code], |r| r.get(0),
    )?;

    let tx = conn.unchecked_transaction()?;
    let mut row_count = 0usize;
    let mut skipped = 0usize;
    let mut first_date = None;
    let mut last_date = None;

    for line in lines {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 6 { skipped += 1; continue; }

        let date = fields.get(date_col).map(|s| s.trim().trim_matches('"')).unwrap_or("");
        let open: f64 = get_field(&fields, open_col);
        let high: f64 = get_field(&fields, high_col);
        let low: f64 = get_field(&fields, low_col);
        let close: f64 = get_field(&fields, close_col);
        let volume: f64 = get_field(&fields, vol_col);
        let amount: f64 = get_field(&fields, amt_col);

        if date.len() < 7 { skipped += 1; continue; }
        // Normalize date: handle both "2020-01-01", "2020-1-1", and "20200101"
        let date_norm = normalize_date(date);

        let result = tx.execute(
            "INSERT OR IGNORE INTO daily_prices (stock_id, trade_date, open, high, low, close, volume, amount)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![stock_id, date_norm, open, high, low, close, volume, amount],
        );
        match result {
            Ok(1) => {
                row_count += 1;
                if first_date.is_none() { first_date = Some(date_norm.clone()); }
                last_date = Some(date_norm);
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
        } else { None },
    })
}

fn detect_columns(header: &str) -> Result<(usize, usize, usize, usize, usize, usize, usize), Box<dyn std::error::Error>> {
    let header_lower = header.to_lowercase();
    let cols: Vec<&str> = header_lower.split(',').map(|s| s.trim().trim_matches('"')).collect();
    let find = |names: &[&str]| -> Option<usize> {
        cols.iter().position(|c| names.iter().any(|n| c.contains(n)))
    };
    let require = |names: &[&str], label: &str| -> Result<usize, Box<dyn std::error::Error>> {
        find(names).ok_or_else(|| format!("找不到{}列，表头: {}", label, header).into())
    };
    Ok((
        require(&["date", "trade_date", "日期"], "日期")?,
        require(&["open", "开盘价", "开盘"], "开盘价")?,
        require(&["high", "最高价", "最高"], "最高价")?,
        require(&["low", "最低价", "最低"], "最低价")?,
        require(&["close", "收盘价", "收盘"], "收盘价")?,
        require(&["vol", "volume", "成交量"], "成交量")?,
        find(&["amount", "amt", "成交额", "成交金额"]).unwrap_or(6),
    ))
}

fn normalize_date(date: &str) -> String {
    let clean = date.trim().trim_matches('"');
    if clean.contains('-') {
        // Handle "2020-1-1" → pad to "2020-01-01"
        let parts: Vec<&str> = clean.split('-').collect();
        if parts.len() == 3 {
            if let (Ok(y), Ok(m), Ok(d)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                return format!("{:04}-{:02}-{:02}", y, m, d);
            }
        }
        clean.to_string()
    } else if clean.len() == 8 && clean.chars().all(|c| c.is_ascii_digit()) {
        // "20200101" → "2020-01-01"
        format!("{}-{}-{}", &clean[0..4], &clean[4..6], &clean[6..8])
    } else {
        clean.to_string()
    }
}

fn get_field(fields: &[&str], idx: usize) -> f64 {
    fields.get(idx)
        .map(|s| s.trim().trim_matches('"').trim_matches('\''))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}

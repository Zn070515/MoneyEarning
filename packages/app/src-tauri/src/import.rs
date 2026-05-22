use rusqlite::{Connection, params};
use std::io::{BufRead, BufReader};
use std::fs::File;

pub fn import_csv_file(
    guard: &std::sync::MutexGuard<'static, Option<Connection>>,
    file_path: &str, stock_code: &str, exchange: &str,
) -> Result<super::ImportResult, Box<dyn std::error::Error>> {
    let conn = guard.as_ref().expect("DB not initialized");
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

        if date.len() < 8 { skipped += 1; continue; }
        // Normalize date: 2020-01-01 or 20200101 → 2020-01-01
        let date_norm = if date.contains('-') { date.to_string() }
            else { format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8]) };

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
    Ok((
        find(&["date", "trade_date", "日期"]).ok_or("找不到日期列")?,
        find(&["open", "开盘价", "开盘"]).unwrap_or(1),
        find(&["high", "最高价", "最高"]).unwrap_or(2),
        find(&["low", "最低价", "最低"]).unwrap_or(3),
        find(&["close", "收盘价", "收盘"]).unwrap_or(4),
        find(&["vol", "volume", "成交量"]).unwrap_or(5),
        find(&["amount", "amt", "成交额", "成交金额"]).unwrap_or(6),
    ))
}

fn get_field(fields: &[&str], idx: usize) -> f64 {
    fields.get(idx)
        .map(|s| s.trim().trim_matches('"').trim_matches('\''))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}

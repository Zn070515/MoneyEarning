use rusqlite::{Connection, Result, params};
use std::sync::{Mutex, MutexGuard};

static DB: Mutex<Option<Connection>> = Mutex::new(None);

pub fn set_db(conn: Connection) {
    *DB.lock().unwrap_or_else(|e| e.into_inner()) = Some(conn);
}

pub fn get_db(_app: &tauri::AppHandle) -> Result<std::sync::MutexGuard<'static, Option<Connection>>> {
    DB.lock().map_err(|_| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_MISUSE),
            Some("数据库锁中毒".to_string()),
        )
    })
}

#[inline]
fn conn<'a>(guard: &'a MutexGuard<'a, Option<Connection>>) -> Result<&'a Connection> {
    guard
        .as_ref()
        .ok_or_else(|| rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_MISUSE),
            Some("数据库未初始化".to_string()),
        ))
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA busy_timeout=5000;
        PRAGMA synchronous=NORMAL;
        PRAGMA cache_size=-8000;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS stocks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE,
            name TEXT,
            exchange TEXT DEFAULT 'SZ',
            ipo_date TEXT,
            created_at TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS daily_prices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            stock_id INTEGER NOT NULL REFERENCES stocks(id) ON DELETE CASCADE,
            trade_date TEXT NOT NULL,
            open REAL NOT NULL,
            high REAL NOT NULL,
            low REAL NOT NULL,
            close REAL NOT NULL,
            volume REAL NOT NULL,
            amount REAL NOT NULL DEFAULT 0,
            turnover REAL,
            UNIQUE(stock_id, trade_date)
        );
        CREATE INDEX IF NOT EXISTS idx_daily_stock ON daily_prices(stock_id);
        CREATE INDEX IF NOT EXISTS idx_daily_date ON daily_prices(trade_date);

        CREATE TABLE IF NOT EXISTS watchlists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE TABLE IF NOT EXISTS watchlist_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            watchlist_id INTEGER NOT NULL REFERENCES watchlists(id) ON DELETE CASCADE,
            stock_id INTEGER NOT NULL REFERENCES stocks(id) ON DELETE CASCADE,
            UNIQUE(watchlist_id, stock_id)
        );

        CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            stock_id INTEGER REFERENCES stocks(id),
            trade_date TEXT NOT NULL,
            direction TEXT NOT NULL CHECK(direction IN ('buy','sell')),
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            commission REAL DEFAULT 0,
            stamp_tax REAL DEFAULT 0,
            strategy_name TEXT,
            emotion_tag TEXT,
            notes TEXT,
            created_at TEXT DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS strategies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            script TEXT,
            params TEXT,
            template_type TEXT,
            created_at TEXT DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS minute_prices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            stock_id INTEGER NOT NULL REFERENCES stocks(id) ON DELETE CASCADE,
            trade_time TEXT NOT NULL,
            open REAL NOT NULL,
            high REAL NOT NULL,
            low REAL NOT NULL,
            close REAL NOT NULL,
            volume REAL NOT NULL DEFAULT 0,
            amount REAL NOT NULL DEFAULT 0,
            UNIQUE(stock_id, trade_time)
        );
        CREATE INDEX IF NOT EXISTS idx_minute_stock ON minute_prices(stock_id);
        CREATE INDEX IF NOT EXISTS idx_minute_time ON minute_prices(trade_time);

        CREATE TABLE IF NOT EXISTS app_config (
            key TEXT PRIMARY KEY,
            value TEXT,
            updated_at TEXT DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS license_info (
            id INTEGER PRIMARY KEY,
            license_key TEXT NOT NULL,
            tier TEXT NOT NULL,
            expiry TEXT,
            activated_at TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            signature TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS alert_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            stock_id INTEGER NOT NULL REFERENCES stocks(id) ON DELETE CASCADE,
            condition_type TEXT NOT NULL CHECK(condition_type IN ('price_breakout','ma_cross','volume_spike')),
            params TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            last_triggered TEXT,
            created_at TEXT DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_alert_stock ON alert_rules(stock_id);
    ")?;
    add_column_if_missing(conn, "trades", "emotion_tag", "emotion_tag TEXT")?;
    add_column_if_missing(conn, "stocks", "industry", "industry TEXT")?;
    // 首次运行记录安装日期（仅写入一次）
    conn.execute(
        "INSERT OR IGNORE INTO app_config (key, value, updated_at) VALUES ('install_date', date('now'), datetime('now','localtime'))",
        [],
    )?;
    // 首次运行预置三支演示股票
    seed_demo_data(conn)?;
    Ok(())
}

fn add_column_if_missing(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
    if !column_exists(conn, table, column)? {
        conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {definition}"), [])?;
    }
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for name in columns {
        if name? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// 首次启动预置三支演示股票（仅DB为空时写入）
/// 从东方财富 API 拉取真实日线数据，失败则跳过该股票
fn seed_demo_data(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM stocks", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    let demos: [(&str, &str, &str); 3] = [
        ("600519", "贵州茅台", "1"),
        ("300750", "宁德时代", "0"),
        ("600036", "招商银行", "1"),
    ];

    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO daily_prices (stock_id, trade_date, open, high, low, close, volume, amount)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )?;

    for (code, name, market) in &demos {
        conn.execute(
            "INSERT OR IGNORE INTO stocks (code, name, exchange) VALUES (?1, ?2, ?3)",
            params![code, name, if *market == "1" { "SH" } else { "SZ" }],
        )?;
        let stock_id: i64 = conn.query_row(
            "SELECT id FROM stocks WHERE code = ?1", params![code], |r| r.get(0),
        )?;

        let url = format!(
            "https://push2his.eastmoney.com/api/qt/stock/kline/get?secid={}.{}&\
             fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f54,f55,f56,f57&\
             klt=101&fqt=1&end=20500101&lmt=120",
            market, code
        );

        let mut inserted: usize = 0;
        match reqwest::blocking::get(&url) {
            Ok(resp) => {
                let body = resp.text().unwrap_or_default();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(klines) = json["data"]["klines"].as_array() {
                        for line in klines {
                            if let Some(s) = line.as_str() {
                                let parts: Vec<&str> = s.split(',').collect();
                                if parts.len() < 7 { continue; }
                                let date = parts[0];
                                let open: f64 = parts[1].parse().unwrap_or(0.0);
                                let close: f64 = parts[2].parse().unwrap_or(0.0);
                                let high: f64 = parts[3].parse().unwrap_or(0.0);
                                let low: f64 = parts[4].parse().unwrap_or(0.0);
                                let volume: f64 = parts[5].parse().unwrap_or(0.0);
                                let amount: f64 = parts[6].parse().unwrap_or(0.0);
                                if open <= 0.0 || close <= 0.0 { continue; }
                                let _ = stmt.execute(params![
                                    stock_id, date,
                                    open, high, low, close, volume, amount
                                ]);
                                inserted += 1;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("[QuantVault] 预置数据下载失败 {} ({}): {}", name, code, e);
            }
        }
        println!("[QuantVault] 预置演示数据: {} ({}), {} 条日线", name, code, inserted);
    }
    Ok(())
}

// ── Config helpers ──

pub fn get_config(guard: &MutexGuard<'_, Option<Connection>>, key: &str) -> Result<Option<String>> {
    let conn = conn(guard)?;
    let mut stmt = conn.prepare("SELECT value FROM app_config WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

// ── License persistence ──

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LicenseRecord {
    pub license_key: String,
    pub tier: String,
    pub expiry: Option<String>,
    pub activated_at: String,
    pub fingerprint: String,
    pub signature: String,
}

pub fn save_license(guard: &MutexGuard<'_, Option<Connection>>,
                    license_key: &str, tier: &str, expiry: Option<&str>,
                    fingerprint: &str, signature: &str) -> Result<()> {
    let conn = conn(guard)?;
    conn.execute("DELETE FROM license_info", [])?;
    conn.execute(
        "INSERT INTO license_info (license_key, tier, expiry, activated_at, fingerprint, signature)
         VALUES (?1, ?2, ?3, datetime('now','localtime'), ?4, ?5)",
        params![license_key, tier, expiry, fingerprint, signature],
    )?;
    Ok(())
}

pub fn load_license(guard: &MutexGuard<'_, Option<Connection>>) -> Result<Option<LicenseRecord>> {
    let conn = conn(guard)?;
    let mut stmt = conn.prepare(
        "SELECT license_key, tier, expiry, activated_at, fingerprint, signature FROM license_info LIMIT 1"
    )?;
    let mut rows = stmt.query_map([], |row| {
        Ok(LicenseRecord {
            license_key: row.get(0)?,
            tier: row.get(1)?,
            expiry: row.get(2)?,
            activated_at: row.get(3)?,
            fingerprint: row.get(4)?,
            signature: row.get(5)?,
        })
    })?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

pub fn clear_license(guard: &MutexGuard<'_, Option<Connection>>) -> Result<()> {
    let conn = conn(guard)?;
    conn.execute("DELETE FROM license_info", [])?;
    Ok(())
}

pub fn query_daily(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                   stock_id: i64, start: &str, end: &str) -> Result<Vec<super::DailyPrice>> {
    let conn = conn(guard)?;
    let mut stmt = conn.prepare(
        "SELECT id, stock_id, trade_date, open, high, low, close, volume, amount, turnover
         FROM daily_prices WHERE stock_id = ?1 AND trade_date >= ?2 AND trade_date <= ?3
         ORDER BY trade_date"
    )?;
    let rows = stmt.query_map(params![stock_id, start, end], |row| {
        Ok(super::DailyPrice {
            id: row.get(0)?, stock_id: row.get(1)?, trade_date: row.get(2)?,
            open: row.get(3)?, high: row.get(4)?, low: row.get(5)?, close: row.get(6)?,
            volume: row.get(7)?, amount: row.get(8)?, turnover: row.get(9)?,
        })
    })?;
    rows.collect()
}

pub fn list_stocks(guard: &std::sync::MutexGuard<'_, Option<Connection>>) -> Result<Vec<super::StockInfo>> {
    let conn = conn(guard)?;
    let mut stmt = conn.prepare(
        "SELECT s.id, s.code, s.name, s.exchange, s.ipo_date,
                COUNT(dp.id), MIN(dp.trade_date), MAX(dp.trade_date)
         FROM stocks s LEFT JOIN daily_prices dp ON s.id = dp.stock_id
         GROUP BY s.id ORDER BY s.code"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(super::StockInfo {
            id: row.get(0)?, code: row.get(1)?, name: row.get(2)?,
            exchange: row.get(3)?, ipo_date: row.get(4)?,
            total_rows: row.get(5)?, first_date: row.get(6)?, last_date: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn find_stock(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                  code: &str) -> Result<Option<super::StockInfo>> {
    let conn = conn(guard)?;
    let mut stmt = conn.prepare(
        "SELECT s.id, s.code, s.name, s.exchange, s.ipo_date,
                COUNT(dp.id), MIN(dp.trade_date), MAX(dp.trade_date)
         FROM stocks s LEFT JOIN daily_prices dp ON s.id = dp.stock_id
         WHERE s.code = ?1 GROUP BY s.id"
    )?;
    let mut rows = stmt.query_map(params![code], |row| {
        Ok(super::StockInfo {
            id: row.get(0)?, code: row.get(1)?, name: row.get(2)?,
            exchange: row.get(3)?, ipo_date: row.get(4)?,
            total_rows: row.get(5)?, first_date: row.get(6)?, last_date: row.get(7)?,
        })
    })?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

pub fn summary(guard: &std::sync::MutexGuard<'_, Option<Connection>>) -> Result<super::DataSummary> {
    let conn = conn(guard)?;
    let total_stocks: i64 = conn.query_row("SELECT COUNT(*) FROM stocks", [], |r| r.get(0))?;
    let total_rows: i64 = conn.query_row("SELECT COUNT(*) FROM daily_prices", [], |r| r.get(0))?;
    let db_size: i64 = conn.query_row("PRAGMA page_count", [], |r| r.get::<_,i64>(0))?;
    let page_size: i64 = conn.query_row("PRAGMA page_size", [], |r| r.get::<_,i64>(0))?;
    Ok(super::DataSummary {
        total_stocks,
        total_rows,
        db_size_mb: (db_size as f64 * page_size as f64) / 1_048_576.0,
    })
}

// ── Watchlist CRUD ──

pub fn watchlist_list(guard: &std::sync::MutexGuard<'_, Option<Connection>>) -> Result<Vec<super::Watchlist>> {
    let conn = conn(guard)?;
    let mut stmt = conn.prepare(
        "SELECT w.id, w.name, w.description, w.created_at,
                COUNT(wi.id) as item_count
         FROM watchlists w LEFT JOIN watchlist_items wi ON w.id = wi.watchlist_id
         GROUP BY w.id ORDER BY w.name"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(super::Watchlist {
            id: row.get(0)?, name: row.get(1)?, description: row.get(2)?,
            created_at: row.get(3)?, item_count: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn watchlist_create(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                         name: &str, description: &str) -> Result<i64> {
    let conn = conn(guard)?;
    conn.execute(
        "INSERT INTO watchlists (name, description) VALUES (?1, ?2)",
        params![name, description],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn watchlist_delete(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                         id: i64) -> Result<()> {
    let conn = conn(guard)?;
    conn.execute("DELETE FROM watchlists WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn watchlist_items(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                        watchlist_id: i64) -> Result<Vec<super::StockInfo>> {
    let conn = conn(guard)?;
    let mut stmt = conn.prepare(
        "SELECT s.id, s.code, s.name, s.exchange, s.ipo_date,
                COUNT(dp.id), MIN(dp.trade_date), MAX(dp.trade_date)
         FROM watchlist_items wi
         JOIN stocks s ON wi.stock_id = s.id
         LEFT JOIN daily_prices dp ON s.id = dp.stock_id
         WHERE wi.watchlist_id = ?1
         GROUP BY s.id ORDER BY s.code"
    )?;
    let rows = stmt.query_map(params![watchlist_id], |row| {
        Ok(super::StockInfo {
            id: row.get(0)?, code: row.get(1)?, name: row.get(2)?,
            exchange: row.get(3)?, ipo_date: row.get(4)?,
            total_rows: row.get(5)?, first_date: row.get(6)?, last_date: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn watchlist_add_item(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                           watchlist_id: i64, stock_id: i64) -> Result<()> {
    let conn = conn(guard)?;
    conn.execute(
        "INSERT OR IGNORE INTO watchlist_items (watchlist_id, stock_id) VALUES (?1, ?2)",
        params![watchlist_id, stock_id],
    )?;
    Ok(())
}

pub fn watchlist_remove_item(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                              watchlist_id: i64, stock_id: i64) -> Result<()> {
    let conn = conn(guard)?;
    conn.execute(
        "DELETE FROM watchlist_items WHERE watchlist_id = ?1 AND stock_id = ?2",
        params![watchlist_id, stock_id],
    )?;
    Ok(())
}

// ── Data import helpers ──

pub fn upsert_stock(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                    code: &str, name: &str, exchange: &str, ipo_date: Option<&str>)
                    -> Result<i64> {
    let conn = conn(guard)?;
    conn.execute(
        "INSERT INTO stocks (code, name, exchange, ipo_date)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(code) DO UPDATE SET name=?2, exchange=?3, ipo_date=COALESCE(?4, ipo_date)",
        params![code, name, exchange, ipo_date],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn bulk_insert_daily(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                         stock_id: i64, rows: &[super::download::KlineRow])
                         -> Result<usize> {
    let conn = conn(guard)?;
    conn.execute("BEGIN", [])?;
    let mut count = 0;
    for row in rows {
        match conn.execute(
            "INSERT OR IGNORE INTO daily_prices (stock_id, trade_date, open, high, low, close, volume, amount, turnover)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![stock_id, row.trade_date, row.open, row.high, row.low, row.close, row.volume, row.amount, row.turnover],
        ) {
            Ok(c) => count += c,
            Err(e) => { let _ = conn.execute("ROLLBACK", []); return Err(e); }
        }
    }
    conn.execute("COMMIT", [])?;
    Ok(count)
}

pub fn delete_stock(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                    id: i64) -> Result<()> {
    let conn = conn(guard)?;
    conn.execute("DELETE FROM stocks WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Strategies ──

pub fn strategy_list(guard: &std::sync::MutexGuard<'_, Option<Connection>>) -> Result<Vec<super::Strategy>> {
    let conn = conn(guard)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, script, params, template_type, created_at
         FROM strategies ORDER BY name"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(super::Strategy {
            id: row.get(0)?, name: row.get(1)?, script: row.get(2)?,
            params: row.get(3)?, template_type: row.get(4)?, created_at: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn strategy_create(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                        name: &str, script: Option<&str>, params: Option<&str>,
                        template_type: Option<&str>) -> Result<i64> {
    let conn = conn(guard)?;
    conn.execute(
        "INSERT INTO strategies (name, script, params, template_type) VALUES (?1, ?2, ?3, ?4)",
        params![name, script, params, template_type],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn strategy_update(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                        id: i64, name: &str, script: Option<&str>,
                        params: Option<&str>, template_type: Option<&str>) -> Result<()> {
    let conn = conn(guard)?;
    conn.execute(
        "UPDATE strategies SET name=?2, script=?3, params=?4, template_type=?5 WHERE id=?1",
        params![id, name, script, params, template_type],
    )?;
    Ok(())
}

pub fn strategy_delete(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                        id: i64) -> Result<()> {
    let conn = conn(guard)?;
    conn.execute("DELETE FROM strategies WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Trade journal ──

pub fn trade_create(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                    stock_id: i64, trade_date: &str, direction: &str,
                    price: f64, quantity: f64, commission: f64, stamp_tax: f64,
                    strategy_name: Option<&str>, emotion_tag: Option<&str>,
                    notes: Option<&str>) -> Result<i64> {
    let conn = conn(guard)?;
    conn.execute(
        "INSERT INTO trades (stock_id, trade_date, direction, price, quantity, commission, stamp_tax, strategy_name, emotion_tag, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![stock_id, trade_date, direction, price, quantity, commission, stamp_tax, strategy_name, emotion_tag, notes],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn trade_list(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                   stock_id: Option<i64>) -> Result<Vec<super::Trade>> {
    let conn = conn(guard)?;
    let sql = if stock_id.map_or(false, |id| id > 0) {
        "SELECT t.id, t.stock_id, s.code, s.name, t.trade_date, t.direction,
                t.price, t.quantity, t.commission, t.stamp_tax,
                t.strategy_name, t.emotion_tag, t.notes, t.created_at
         FROM trades t LEFT JOIN stocks s ON t.stock_id = s.id
         WHERE t.stock_id = ?1
         ORDER BY t.trade_date DESC, t.id DESC"
    } else {
        "SELECT t.id, t.stock_id, s.code, s.name, t.trade_date, t.direction,
                t.price, t.quantity, t.commission, t.stamp_tax,
                t.strategy_name, t.emotion_tag, t.notes, t.created_at
         FROM trades t LEFT JOIN stocks s ON t.stock_id = s.id
         ORDER BY t.trade_date DESC, t.id DESC"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = if stock_id.map_or(false, |id| id > 0) {
        stmt.query_map(params![stock_id], map_trade)?
    } else {
        stmt.query_map([], map_trade)?
    };
    rows.collect()
}

fn map_trade(row: &rusqlite::Row) -> rusqlite::Result<super::Trade> {
    Ok(super::Trade {
        id: row.get(0)?, stock_id: row.get(1)?,
        stock_code: row.get(2)?, stock_name: row.get(3)?,
        trade_date: row.get(4)?, direction: row.get(5)?,
        price: row.get(6)?, quantity: row.get(7)?,
        commission: row.get(8)?, stamp_tax: row.get(9)?,
        strategy_name: row.get(10)?, emotion_tag: row.get(11)?,
        notes: row.get(12)?, created_at: row.get(13)?,
    })
}

pub fn trade_pnl(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                  stock_id: Option<i64>) -> Result<super::PnLSummary> {
    let trades = trade_list(guard, stock_id)?;
    let total = trades.len() as f64;
    if total == 0.0 {
        return Ok(super::PnLSummary::default());
    }

    // Simple PnL: match buy-sell pairs by FIFO and compute realized PnL
    let mut buy_queue: Vec<(f64, f64)> = Vec::new(); // (price, qty remaining)
    let mut wins = 0.0f64;
    let mut losses = 0.0f64;
    let mut total_pnl = 0.0f64;
    let mut gross_profit = 0.0f64;
    let mut gross_loss = 0.0f64;
    let mut max_win = 0.0f64;
    let mut max_loss = 0.0f64;
    let mut closed_pnl_count = 0.0f64;

    // trades are newest-first, reverse for FIFO
    let mut chronological: Vec<&super::Trade> = trades.iter().collect();
    chronological.reverse();

    for t in &chronological {
        if t.direction == "buy" {
            buy_queue.push((t.price, t.quantity));
        } else {
            let mut remaining_sell = t.quantity;
            let sell_price = t.price;
            let cost = t.commission + t.stamp_tax;
            let mut sell_pnl = 0.0f64;

            while remaining_sell > 0.0 && !buy_queue.is_empty() {
                let (buy_price, buy_qty) = buy_queue[0];
                let matched = buy_qty.min(remaining_sell);
                let trade_pnl = (sell_price - buy_price) * matched - cost * (matched / t.quantity);
                sell_pnl += trade_pnl;
                remaining_sell -= matched;

                if matched >= buy_qty {
                    buy_queue.remove(0);
                } else {
                    buy_queue[0].1 -= matched;
                }
            }
            // remaining_sell > 0 means short selling (unmatched), skip for simplicity
            total_pnl += sell_pnl;
            if sell_pnl > 0.0 {
                wins += 1.0;
                gross_profit += sell_pnl;
            } else if sell_pnl < 0.0 {
                losses += 1.0;
                gross_loss += sell_pnl;
            }
            max_win = max_win.max(sell_pnl);
            max_loss = max_loss.min(sell_pnl);
            closed_pnl_count += 1.0;
        }
    }

    let win_rate = if closed_pnl_count > 0.0 { wins / closed_pnl_count * 100.0 } else { 0.0 };
    let avg_win = if wins > 0.0 { gross_profit / wins } else { 0.0 };
    let avg_loss = if losses > 0.0 { gross_loss / losses } else { 0.0 };
    let profit_factor = if gross_loss.abs() > 1e-9 {
        (gross_profit / gross_loss.abs()).abs()
    } else { 0.0 };

    Ok(super::PnLSummary {
        total_trades: trades.len() as i64,
        winning_trades: wins as i64,
        losing_trades: losses as i64,
        win_rate,
        total_pnl,
        avg_win,
        avg_loss,
        max_win,
        max_loss,
        profit_factor,
    })
}

// ── Minute prices ──

pub fn query_minute_prices(
    guard: &std::sync::MutexGuard<'_, Option<Connection>>,
    stock_id: i64, start: &str, end: &str,
) -> Result<Vec<MinutePrice>> {
    let conn = conn(guard)?;
    let mut stmt = conn.prepare(
        "SELECT id, stock_id, trade_time, open, high, low, close, volume, amount
         FROM minute_prices WHERE stock_id = ?1 AND trade_time >= ?2 AND trade_time <= ?3
         ORDER BY trade_time LIMIT 50000"
    )?;
    let rows = stmt.query_map(params![stock_id, start, end], |row| {
        Ok(MinutePrice {
            id: row.get(0)?,
            stock_id: row.get(1)?,
            trade_time: row.get(2)?,
            open: row.get(3)?,
            high: row.get(4)?,
            low: row.get(5)?,
            close: row.get(6)?,
            volume: row.get(7)?,
            amount: row.get(8)?,
        })
    })?;
    rows.collect()
}

pub fn bulk_insert_minute(
    guard: &std::sync::MutexGuard<'_, Option<Connection>>,
    stock_id: i64, rows: &[MinuteRow],
) -> Result<usize> {
    let conn = conn(guard)?;
    conn.execute("BEGIN", [])?;
    let mut count = 0;
    for row in rows {
        match conn.execute(
            "INSERT OR IGNORE INTO minute_prices (stock_id, trade_time, open, high, low, close, volume, amount)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![stock_id, row.trade_time, row.open, row.high, row.low, row.close, row.volume, row.amount],
        ) {
            Ok(c) => count += c,
            Err(e) => { let _ = conn.execute("ROLLBACK", []); return Err(e); }
        }
    }
    conn.execute("COMMIT", [])?;
    Ok(count)
}

#[derive(Debug, Clone)]
pub struct MinuteRow {
    pub trade_time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
}

// ── Portfolio Analysis ──

pub fn fetch_close_prices(
    guard: &std::sync::MutexGuard<'_, Option<Connection>>,
    stock_ids: &[i64], start_date: &str, end_date: &str,
) -> Result<std::collections::HashMap<i64, Vec<f64>>> {
    let conn = conn(guard)?;
    let mut result: std::collections::HashMap<i64, Vec<f64>> = std::collections::HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT close FROM daily_prices WHERE stock_id=?1 AND trade_date>=?2 AND trade_date<=?3 ORDER BY trade_date"
    )?;
    for &sid in stock_ids {
        let closes: Vec<f64> = stmt.query_map(params![sid, start_date, end_date], |r| r.get(0))?
            .filter_map(|v| v.ok())
            .collect();
        if closes.len() >= 20 {
            result.insert(sid, closes);
        }
    }
    Ok(result)
}

pub fn fetch_stock_industries(
    guard: &std::sync::MutexGuard<'_, Option<Connection>>,
    stock_ids: &[i64],
) -> Result<std::collections::HashMap<i64, String>> {
    let conn = conn(guard)?;
    let mut result = std::collections::HashMap::new();
    let mut stmt = conn.prepare("SELECT code, name, industry FROM stocks WHERE id=?1")?;
    for &sid in stock_ids {
        let row: Result<(String, String, Option<String>)> = stmt.query_row(params![sid], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))
        });
        if let Ok((code, name, industry)) = row {
            let label = industry.unwrap_or_else(|| "未分类".to_string());
            result.insert(sid, label);
            let _ = (code, name);
        }
    }
    Ok(result)
}

pub fn set_stock_industry(
    guard: &std::sync::MutexGuard<'_, Option<Connection>>,
    stock_id: i64, industry: &str,
) -> Result<()> {
    let conn = conn(guard)?;
    conn.execute("UPDATE stocks SET industry=?1 WHERE id=?2", params![industry, stock_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        assert!(column_exists(&conn, "trades", "emotion_tag").unwrap());
    }

    #[test]
    fn migrations_add_emotion_tag_to_existing_trades_table() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                stock_id INTEGER,
                trade_date TEXT NOT NULL,
                direction TEXT NOT NULL CHECK(direction IN ('buy','sell')),
                price REAL NOT NULL,
                quantity REAL NOT NULL,
                commission REAL DEFAULT 0,
                stamp_tax REAL DEFAULT 0,
                strategy_name TEXT,
                notes TEXT,
                created_at TEXT DEFAULT (datetime('now','localtime'))
            );
            ",
        )
        .unwrap();

        run_migrations(&conn).unwrap();

        assert!(column_exists(&conn, "trades", "emotion_tag").unwrap());
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MinutePrice {
    pub id: i64,
    pub stock_id: i64,
    pub trade_time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
}

use rusqlite::{Connection, Result, params};
use std::sync::Mutex;

static DB: Mutex<Option<Connection>> = Mutex::new(None);

pub fn set_db(conn: Connection) {
    *DB.lock().unwrap() = Some(conn);
}

pub fn get_db(_app: &tauri::AppHandle) -> Result<std::sync::MutexGuard<'static, Option<Connection>>> {
    Ok(DB.lock().unwrap())
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch("
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

        -- Migration: add emotion_tag to existing trades table
        ALTER TABLE trades ADD COLUMN emotion_tag TEXT;

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
    ")?;
    Ok(())
}

pub fn query_daily(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                   stock_id: i64, start: &str, end: &str) -> Result<Vec<super::DailyPrice>> {
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
    conn.execute(
        "INSERT INTO watchlists (name, description) VALUES (?1, ?2)",
        params![name, description],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn watchlist_delete(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                         id: i64) -> Result<()> {
    let conn = guard.as_ref().expect("DB not initialized");
    conn.execute("DELETE FROM watchlists WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn watchlist_items(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                        watchlist_id: i64) -> Result<Vec<super::StockInfo>> {
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
    conn.execute(
        "INSERT OR IGNORE INTO watchlist_items (watchlist_id, stock_id) VALUES (?1, ?2)",
        params![watchlist_id, stock_id],
    )?;
    Ok(())
}

pub fn watchlist_remove_item(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                              watchlist_id: i64, stock_id: i64) -> Result<()> {
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
    let mut count = 0;
    for row in rows {
        let result = conn.execute(
            "INSERT OR IGNORE INTO daily_prices (stock_id, trade_date, open, high, low, close, volume, amount, turnover)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![stock_id, row.trade_date, row.open, row.high, row.low, row.close, row.volume, row.amount, row.turnover],
        );
        if let Ok(c) = result { count += c; }
    }
    Ok(count)
}

pub fn delete_stock(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                    id: i64) -> Result<()> {
    let conn = guard.as_ref().expect("DB not initialized");
    conn.execute("DELETE FROM stocks WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Strategies ──

pub fn strategy_list(guard: &std::sync::MutexGuard<'_, Option<Connection>>) -> Result<Vec<super::Strategy>> {
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
    conn.execute(
        "INSERT INTO strategies (name, script, params, template_type) VALUES (?1, ?2, ?3, ?4)",
        params![name, script, params, template_type],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn strategy_update(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                        id: i64, name: &str, script: Option<&str>,
                        params: Option<&str>, template_type: Option<&str>) -> Result<()> {
    let conn = guard.as_ref().expect("DB not initialized");
    conn.execute(
        "UPDATE strategies SET name=?2, script=?3, params=?4, template_type=?5 WHERE id=?1",
        params![id, name, script, params, template_type],
    )?;
    Ok(())
}

pub fn strategy_delete(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                        id: i64) -> Result<()> {
    let conn = guard.as_ref().expect("DB not initialized");
    conn.execute("DELETE FROM strategies WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Trade journal ──

pub fn trade_create(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                    stock_id: i64, trade_date: &str, direction: &str,
                    price: f64, quantity: f64, commission: f64, stamp_tax: f64,
                    strategy_name: Option<&str>, emotion_tag: Option<&str>,
                    notes: Option<&str>) -> Result<i64> {
    let conn = guard.as_ref().expect("DB not initialized");
    conn.execute(
        "INSERT INTO trades (stock_id, trade_date, direction, price, quantity, commission, stamp_tax, strategy_name, emotion_tag, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![stock_id, trade_date, direction, price, quantity, commission, stamp_tax, strategy_name, emotion_tag, notes],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn trade_list(guard: &std::sync::MutexGuard<'_, Option<Connection>>,
                   stock_id: Option<i64>) -> Result<Vec<super::Trade>> {
    let conn = guard.as_ref().expect("DB not initialized");
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
            if sell_pnl > 0.0 { wins += 1.0; }
            else if sell_pnl < 0.0 { losses += 1.0; }
            max_win = max_win.max(sell_pnl);
            max_loss = max_loss.min(sell_pnl);
            closed_pnl_count += 1.0;
        }
    }

    let win_rate = if closed_pnl_count > 0.0 { wins / closed_pnl_count * 100.0 } else { 0.0 };
    let avg_win = if wins > 0.0 {
        // approximate: total_win / wins where total_win ≈ portion of total_pnl from wins
        total_pnl / wins.max(1.0)
    } else { 0.0 };
    let avg_loss = if losses > 0.0 {
        total_pnl / losses.max(1.0)
    } else { 0.0 };
    let gross_loss = if losses > 0.0 { avg_loss * losses } else { 0.0 };
    let profit_factor = if gross_loss != 0.0 && gross_loss.abs() > 1e-9 {
        (avg_win * wins / gross_loss.abs()).abs()
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
    let conn = guard.as_ref().expect("DB not initialized");
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
    let conn = guard.as_ref().expect("DB not initialized");
    let mut count = 0;
    for row in rows {
        let result = conn.execute(
            "INSERT OR IGNORE INTO minute_prices (stock_id, trade_time, open, high, low, close, volume, amount)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![stock_id, row.trade_time, row.open, row.high, row.low, row.close, row.volume, row.amount],
        );
        if let Ok(c) = result { count += c; }
    }
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

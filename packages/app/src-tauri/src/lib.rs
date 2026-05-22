use tauri::Manager;
use serde::{Serialize, Deserialize};

mod db;
mod license;
mod import;
mod download;

// ── License ──

#[tauri::command]
fn get_machine_fingerprint() -> Result<String, String> {
    license::generate_fingerprint().map_err(|e| e.to_string())
}

#[tauri::command]
fn activate_license(license_key: String, fingerprint: String) -> Result<LicenseInfo, String> {
    license::activate(&license_key, &fingerprint).map_err(|e| e.to_string())
}

#[tauri::command]
fn check_license() -> Result<LicenseStatus, String> {
    license::check().map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicenseInfo {
    tier: String,
    expiry: Option<String>,
    features: Vec<String>,
    valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicenseStatus {
    valid: bool,
    tier: String,
    expiry: Option<String>,
    trial_days_left: Option<i32>,
}

// ── Database ──

#[tauri::command]
fn query_daily_prices(stock_id: i64, start_date: String, end_date: String,
                      app: tauri::AppHandle) -> Result<Vec<DailyPrice>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::query_daily(&db, stock_id, &start_date, &end_date).map_err(|e| e.to_string())
}

#[tauri::command]
fn query_stock_list(app: tauri::AppHandle) -> Result<Vec<StockInfo>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::list_stocks(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn query_stock_by_code(code: String, app: tauri::AppHandle) -> Result<Option<StockInfo>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::find_stock(&db, &code).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_data_summary(app: tauri::AppHandle) -> Result<DataSummary, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::summary(&db).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DailyPrice {
    id: i64,
    stock_id: i64,
    trade_date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    amount: f64,
    turnover: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StockInfo {
    id: i64,
    code: String,
    name: String,
    exchange: String,
    ipo_date: Option<String>,
    total_rows: i64,
    first_date: Option<String>,
    last_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataSummary {
    total_stocks: i64,
    total_rows: i64,
    db_size_mb: f64,
}

// ── Import ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImportResult {
    stock_count: usize,
    row_count: usize,
    skipped: usize,
    date_range: Option<(String, String)>,
}

#[tauri::command]
fn import_csv(file_path: String, stock_code: String, exchange: String,
              app: tauri::AppHandle) -> Result<ImportResult, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    import::import_csv_file(&db, &file_path, &stock_code, &exchange).map_err(|e| e.to_string())
}

// ── Watchlist ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Watchlist {
    id: i64,
    name: String,
    description: Option<String>,
    created_at: String,
    item_count: i64,
}

#[tauri::command]
fn watchlist_list(app: tauri::AppHandle) -> Result<Vec<Watchlist>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::watchlist_list(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn watchlist_create(name: String, description: String, app: tauri::AppHandle) -> Result<i64, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::watchlist_create(&db, &name, &description).map_err(|e| e.to_string())
}

#[tauri::command]
fn watchlist_delete(id: i64, app: tauri::AppHandle) -> Result<(), String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::watchlist_delete(&db, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn watchlist_items(watchlist_id: i64, app: tauri::AppHandle) -> Result<Vec<StockInfo>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::watchlist_items(&db, watchlist_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn watchlist_add_item(watchlist_id: i64, stock_id: i64, app: tauri::AppHandle) -> Result<(), String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::watchlist_add_item(&db, watchlist_id, stock_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn watchlist_remove_item(watchlist_id: i64, stock_id: i64, app: tauri::AppHandle) -> Result<(), String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::watchlist_remove_item(&db, watchlist_id, stock_id).map_err(|e| e.to_string())
}

// ── Trade Journal ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trade {
    id: i64,
    stock_id: i64,
    stock_code: Option<String>,
    stock_name: Option<String>,
    trade_date: String,
    direction: String,
    price: f64,
    quantity: f64,
    commission: f64,
    stamp_tax: f64,
    strategy_name: Option<String>,
    notes: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PnLSummary {
    total_trades: i64,
    winning_trades: i64,
    losing_trades: i64,
    win_rate: f64,
    total_pnl: f64,
    avg_win: f64,
    avg_loss: f64,
    max_win: f64,
    max_loss: f64,
    profit_factor: f64,
}

#[tauri::command]
fn trade_create(stock_id: i64, trade_date: String, direction: String,
                price: f64, quantity: f64, commission: f64, stamp_tax: f64,
                strategy_name: Option<String>, notes: Option<String>,
                app: tauri::AppHandle) -> Result<i64, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::trade_create(&db, stock_id, &trade_date, &direction,
        price, quantity, commission, stamp_tax,
        strategy_name.as_deref(), notes.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn trade_list(stock_id: Option<i64>, app: tauri::AppHandle) -> Result<Vec<Trade>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::trade_list(&db, stock_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn trade_pnl(stock_id: Option<i64>, app: tauri::AppHandle) -> Result<PnLSummary, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::trade_pnl(&db, stock_id).map_err(|e| e.to_string())
}

// ── Data Management ──

#[tauri::command]
fn delete_stock(id: i64, app: tauri::AppHandle) -> Result<(), String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::delete_stock(&db, id).map_err(|e| e.to_string())
}

// ── Strategies ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Strategy {
    id: i64,
    name: String,
    script: Option<String>,
    params: Option<String>,
    template_type: Option<String>,
    created_at: String,
}

#[tauri::command]
fn strategy_list(app: tauri::AppHandle) -> Result<Vec<Strategy>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::strategy_list(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn strategy_create(name: String, script: Option<String>, params: Option<String>,
                   template_type: Option<String>,
                   app: tauri::AppHandle) -> Result<i64, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::strategy_create(&db, &name,
        script.as_deref(), params.as_deref(), template_type.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn strategy_update(id: i64, name: String, script: Option<String>,
                   params: Option<String>, template_type: Option<String>,
                   app: tauri::AppHandle) -> Result<(), String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::strategy_update(&db, id, &name,
        script.as_deref(), params.as_deref(), template_type.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn strategy_delete(id: i64, app: tauri::AppHandle) -> Result<(), String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::strategy_delete(&db, id).map_err(|e| e.to_string())
}

// ── Backtest ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BtConfig {
    #[serde(default = "default_initial_capital")]
    initial_capital: f64,
    #[serde(default = "default_commission")]
    commission_rate: f64,
    #[serde(default = "default_stamp_tax")]
    stamp_tax_rate: f64,
    #[serde(default = "default_slippage")]
    slippage: f64,
    #[serde(default = "default_position_pct")]
    position_pct: f64,
}

fn default_initial_capital() -> f64 { 100_000.0 }
fn default_commission() -> f64 { 0.0003 }
fn default_stamp_tax() -> f64 { 0.001 }
fn default_slippage() -> f64 { 0.001 }
fn default_position_pct() -> f64 { 1.0 }

impl From<BtConfig> for wasm_backtest::BacktestConfig {
    fn from(c: BtConfig) -> Self {
        wasm_backtest::BacktestConfig {
            initial_capital: c.initial_capital,
            commission_rate: c.commission_rate,
            stamp_tax_rate: c.stamp_tax_rate,
            slippage: c.slippage,
            position_pct: c.position_pct,
        }
    }
}

#[tauri::command]
fn run_backtest(data: Vec<IndicatorInput>, template: String,
                params: std::collections::HashMap<String, f64>,
                config: Option<BtConfig>)
                -> Result<wasm_core::BtResult, String> {
    let ohlcv: Vec<wasm_core::OHLCV> = data.iter().map(|d| wasm_core::OHLCV {
        open: d.open, high: d.high, low: d.low, close: d.close,
        volume: d.volume, amount: d.amount, turnover: d.turnover,
        trade_date: d.time.to_string(),
    }).collect();
    let df = wasm_core::DataFrame::new(&ohlcv);
    let bt_config: wasm_backtest::BacktestConfig = config.map(Into::into).unwrap_or_default();
    Ok(wasm_backtest::run_with_template(&df, &template, &params, &bt_config))
}

// ── Scanner ──

#[tauri::command]
fn run_scanner(stock_ids: Vec<i64>, expr: wasm_core::ScanExpr,
               app: tauri::AppHandle) -> Result<Vec<ScanResultItem>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;

    // For each stock, load daily data, build DataFrame, evaluate
    let mut results = Vec::new();
    for (idx, stock_id) in stock_ids.iter().enumerate() {
        let prices = db::query_daily(&db, *stock_id, "2020-01-01", "2099-12-31")
            .map_err(|e| e.to_string())?;

        if prices.is_empty() { continue; }

        let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
            open: dp.open, high: dp.high, low: dp.low, close: dp.close,
            volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
            trade_date: dp.trade_date.clone(),
        }).collect();

        let df = wasm_core::DataFrame::new(&ohlcv);
        let matches = wasm_scanner::scan(&[(idx, df)], &expr);

        for m in &matches {
            results.push(ScanResultItem {
                stock_id: *stock_id,
                score: m.score,
                signals: m.signals.clone(),
            });
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScanResultItem {
    stock_id: i64,
    score: f64,
    signals: Vec<String>,
}

// ── Search Algorithms: CAPS / CGPC / MARS / MetaSearcher ──

/// Load daily returns for a list of stock IDs from the DB
fn load_returns_batch(
    guard: &std::sync::MutexGuard<'_, Option<rusqlite::Connection>>,
    stock_ids: &[i64],
) -> Result<Vec<Vec<f64>>, String> {
    let mut all_returns: Vec<Vec<f64>> = Vec::new();
    for stock_id in stock_ids {
        let prices = db::query_daily(guard, *stock_id, "2020-01-01", "2099-12-31")
            .map_err(|e| e.to_string())?;
        if prices.len() < 20 { continue; }
        let returns: Vec<f64> = prices.windows(2)
            .map(|w| (w[1].close - w[0].close) / w[0].close.max(0.0001))
            .collect();
        all_returns.push(returns);
    }
    Ok(all_returns)
}

#[tauri::command]
fn run_caps_search(
    stock_ids: Vec<i64>,
    app: tauri::AppHandle,
) -> Result<Vec<wasm_scanner::CapsResult>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let returns = load_returns_batch(&db, &stock_ids)?;
    if returns.is_empty() {
        return Err("No stocks with sufficient data".into());
    }

    let pool_name = format!("Pool_{}_stocks", returns.len());
    let pools = vec![(pool_name, returns)];
    let strategies = vec![
        "risk_parity".to_string(),
        "min_variance".to_string(),
        "hierarchical_rp".to_string(),
    ];
    let params = std::collections::HashMap::new();

    Ok(wasm_scanner::run_caps(&pools, &strategies, &params))
}

#[tauri::command]
fn run_cgpc_search(
    stock_ids: Vec<i64>,
    n_pools: usize,
    pool_size: usize,
    app: tauri::AppHandle,
) -> Result<Vec<wasm_scanner::CgpcPool>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let returns = load_returns_batch(&db, &stock_ids)?;
    if returns.len() < 3 {
        return Err("Need at least 3 stocks with sufficient data".into());
    }

    Ok(wasm_scanner::build_diverse_pools(&returns, n_pools, pool_size))
}

#[tauri::command]
fn run_mars_search(
    stock_ids: Vec<i64>,
    n_regimes: usize,
    app: tauri::AppHandle,
) -> Result<wasm_scanner::MarsResult, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let returns = load_returns_batch(&db, &stock_ids)?;
    if returns.len() < 5 {
        return Err("Need at least 5 stocks with sufficient data".into());
    }

    // Build strategy returns as HashMap: each strategy → daily returns
    let n = returns[0].len();
    let mut strategy_returns: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

    // Momentum strategy: equal-weighted average of all stock returns
    let momentum: Vec<f64> = (0..n).map(|d| {
        returns.iter().map(|r| r[d]).sum::<f64>() / returns.len() as f64
    }).collect();
    strategy_returns.insert("momentum".to_string(), momentum);

    Ok(wasm_scanner::run_mars(&returns, &strategy_returns, n_regimes))
}

fn get_meta() -> &'static std::sync::Mutex<wasm_scanner::MetaSearcher> {
    use std::sync::{Mutex, OnceLock};
    static META: OnceLock<Mutex<wasm_scanner::MetaSearcher>> = OnceLock::new();
    META.get_or_init(|| Mutex::new(wasm_scanner::MetaSearcher::new()))
}

#[tauri::command]
fn run_metasearcher_select() -> Result<Option<wasm_scanner::SearchNode>, String> {
    Ok(get_meta().lock().map_err(|e| e.to_string())?.select_next())
}

#[tauri::command]
fn run_metasearcher_record(
    node: wasm_scanner::SearchNode,
    sharpe: f64,
    round: usize,
) -> Result<(), String> {
    get_meta().lock().map_err(|e| e.to_string())?.record(&node, sharpe, round);
    Ok(())
}

#[tauri::command]
fn get_metasearcher_best() -> Result<Option<wasm_scanner::SearchNode>, String> {
    Ok(get_meta().lock().map_err(|e| e.to_string())?.best_node().cloned())
}

#[tauri::command]
fn get_metasearcher_count() -> Result<usize, String> {
    Ok(get_meta().lock().map_err(|e| e.to_string())?.explored_count())
}

// ── Distribution ──

#[tauri::command]
fn compute_volume_profile(stock_id: i64, num_buckets: Option<usize>,
                          app: tauri::AppHandle) -> Result<wasm_core::VolumeProfileResult, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, "2020-01-01", "2099-12-31")
        .map_err(|e| e.to_string())?;

    let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
        open: dp.open, high: dp.high, low: dp.low, close: dp.close,
        volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
        trade_date: dp.trade_date.clone(),
    }).collect();

    let df = wasm_core::DataFrame::new(&ohlcv);
    Ok(wasm_distribution::volume_profile(&df, num_buckets.unwrap_or(100)))
}

#[tauri::command]
fn compute_chip_distribution(stock_id: i64,
                             app: tauri::AppHandle) -> Result<wasm_core::DistributionResult, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, "2020-01-01", "2099-12-31")
        .map_err(|e| e.to_string())?;

    let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
        open: dp.open, high: dp.high, low: dp.low, close: dp.close,
        volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
        trade_date: dp.trade_date.clone(),
    }).collect();

    let df = wasm_core::DataFrame::new(&ohlcv);
    Ok(wasm_distribution::cost_distribution(&df))
}

#[tauri::command]
fn compute_sr_levels(stock_id: i64, num_levels: Option<usize>,
                     app: tauri::AppHandle) -> Result<Vec<(f64, f64, String)>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, "2020-01-01", "2099-12-31")
        .map_err(|e| e.to_string())?;

    let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
        open: dp.open, high: dp.high, low: dp.low, close: dp.close,
        volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
        trade_date: dp.trade_date.clone(),
    }).collect();

    let df = wasm_core::DataFrame::new(&ohlcv);
    Ok(wasm_distribution::volume_sr_levels(&df, num_levels.unwrap_or(50)))
}

// ── Distribution (extended) ──

#[tauri::command]
fn compute_concentration(stock_id: i64,
                         app: tauri::AppHandle) -> Result<ConcentrationOutput, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, "2020-01-01", "2099-12-31")
        .map_err(|e| e.to_string())?;
    let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
        open: dp.open, high: dp.high, low: dp.low, close: dp.close,
        volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
        trade_date: dp.trade_date.clone(),
    }).collect();
    let df = wasm_core::DataFrame::new(&ohlcv);
    let c = wasm_distribution::concentration_analysis(&df);
    Ok(ConcentrationOutput {
        cr5: c.cr5, cr10: c.cr10, cr20: c.cr20,
        trend: c.trend, description: c.description,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConcentrationOutput {
    cr5: f64,
    cr10: f64,
    cr20: f64,
    trend: f64,
    description: String,
}

#[tauri::command]
fn compute_profit_loss_ratio(stock_id: i64,
                             app: tauri::AppHandle) -> Result<ProfitLossOutput, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, "2020-01-01", "2099-12-31")
        .map_err(|e| e.to_string())?;
    let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
        open: dp.open, high: dp.high, low: dp.low, close: dp.close,
        volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
        trade_date: dp.trade_date.clone(),
    }).collect();
    let df = wasm_core::DataFrame::new(&ohlcv);
    let pl = wasm_distribution::profit_loss_ratio(&df);
    Ok(ProfitLossOutput {
        profit_pct: pl.profit_pct,
        loss_pct: pl.loss_pct,
        avg_cost: pl.avg_cost,
        weighted_avg_cost: pl.weighted_avg_cost,
        last_price: pl.last_price,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfitLossOutput {
    profit_pct: f64,
    loss_pct: f64,
    avg_cost: f64,
    weighted_avg_cost: f64,
    last_price: f64,
}

#[tauri::command]
fn compute_historical_frames(stock_id: i64, frame_count: Option<usize>,
                             app: tauri::AppHandle) -> Result<Vec<FrameOutput>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, "2020-01-01", "2099-12-31")
        .map_err(|e| e.to_string())?;
    let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
        open: dp.open, high: dp.high, low: dp.low, close: dp.close,
        volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
        trade_date: dp.trade_date.clone(),
    }).collect();
    let df = wasm_core::DataFrame::new(&ohlcv);
    let frames = wasm_distribution::historical_frames(&df, frame_count.unwrap_or(30));
    Ok(frames.iter().map(|f| FrameOutput {
        date: f.date.clone(),
        price_levels: f.price_levels.clone(),
        chip_volume: f.chip_volume.clone(),
        avg_cost: f.avg_cost,
        profit_pct: f.profit_pct,
        loss_pct: f.loss_pct,
    }).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrameOutput {
    date: String,
    price_levels: Vec<f64>,
    chip_volume: Vec<f64>,
    avg_cost: f64,
    profit_pct: f64,
    loss_pct: f64,
}

// ── Indicators ──

/// Input OHLCV from frontend (matches chart data format)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndicatorInput {
    time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    amount: Option<f64>,
    turnover: Option<f64>,
}

#[tauri::command]
fn compute_indicator(name: String, data: Vec<IndicatorInput>,
                     params: std::collections::HashMap<String, f64>)
                     -> Result<Vec<wasm_core::IndicatorOutput>, String> {
    // Check license for PRO indicators
    let meta = wasm_indicators::metadata(&name);
    if meta.as_ref().map_or(false, |m| !m.is_free) {
        let ls = license::check().map_err(|e| e.to_string())?;
        if ls.tier != "pro" {
            return Err(format!("「{}」为专业版指标，需要PRO授权。当前授权: {}",
                meta.unwrap().name_cn, if ls.tier == "trial" { "试用版" } else { "免费版" }));
        }
    }

    let ohlcv: Vec<wasm_core::OHLCV> = data.iter().map(|d| wasm_core::OHLCV {
        open: d.open, high: d.high, low: d.low, close: d.close,
        volume: d.volume, amount: d.amount, turnover: d.turnover,
        trade_date: d.time.to_string(),
    }).collect();
    let df = wasm_core::DataFrame::new(&ohlcv);
    wasm_indicators::compute(&name, &df, &params).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_indicators() -> Vec<wasm_core::IndicatorMeta> {
    wasm_indicators::list_all()
}

// ── Pattern Recognition ──

#[tauri::command]
fn list_patterns() -> Vec<PatternDef> {
    wasm_pattern::list_patterns()
        .iter()
        .map(|p| PatternDef {
            name: p.name.clone(),
            name_cn: p.name_cn.clone(),
            category: p.category.clone(),
        })
        .collect()
}

#[tauri::command]
fn scan_all_patterns(stock_id: i64,
                     app: tauri::AppHandle) -> Result<Vec<PatternResult>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, "2020-01-01", "2099-12-31")
        .map_err(|e| e.to_string())?;
    let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
        open: dp.open, high: dp.high, low: dp.low, close: dp.close,
        volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
        trade_date: dp.trade_date.clone(),
    }).collect();
    let df = wasm_core::DataFrame::new(&ohlcv);
    let matches = wasm_pattern::scan_all(&df);
    Ok(matches.iter().map(|m| PatternResult {
        name: m.name.clone(),
        name_cn: m.name_cn.clone(),
        start_idx: m.start_idx,
        end_idx: m.end_idx,
        confidence: m.confidence,
        direction: m.direction.clone(),
        description: m.description.clone(),
    }).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PatternDef {
    name: String,
    name_cn: String,
    category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PatternResult {
    name: String,
    name_cn: String,
    start_idx: usize,
    end_idx: usize,
    confidence: f64,
    direction: String,
    description: String,
}

// ── Portfolio / Risk ──

#[tauri::command]
fn compute_risk(stock_id: i64,
                app: tauri::AppHandle) -> Result<RiskOutput, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, "2020-01-01", "2099-12-31")
        .map_err(|e| e.to_string())?;
    let close: Vec<f64> = prices.iter().map(|dp| dp.close).collect();
    let rets = wasm_profile::daily_returns(&close);
    let m = wasm_profile::risk_metrics(&rets);
    Ok(RiskOutput {
        total_return: m.total_return,
        annual_return: m.annual_return,
        annual_volatility: m.annual_volatility,
        sharpe_ratio: m.sharpe_ratio,
        sortino_ratio: m.sortino_ratio,
        max_drawdown: m.max_drawdown,
        var_95: m.var_95,
        cvar_95: m.cvar_95,
        calmar_ratio: m.calmar_ratio,
        positive_days: m.positive_days,
        negative_days: m.negative_days,
        best_day: m.best_day,
        worst_day: m.worst_day,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RiskOutput {
    total_return: f64,
    annual_return: f64,
    annual_volatility: f64,
    sharpe_ratio: f64,
    sortino_ratio: f64,
    max_drawdown: f64,
    var_95: f64,
    cvar_95: f64,
    calmar_ratio: f64,
    positive_days: usize,
    negative_days: usize,
    best_day: f64,
    worst_day: f64,
}

// ── Custom Script ──

#[tauri::command]
fn execute_custom_script(script: String, stock_id: i64,
                         app: tauri::AppHandle) -> Result<ScriptResult, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, "2020-01-01", "2099-12-31")
        .map_err(|e| e.to_string())?;
    let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
        open: dp.open, high: dp.high, low: dp.low, close: dp.close,
        volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
        trade_date: dp.trade_date.clone(),
    }).collect();
    let df = wasm_core::DataFrame::new(&ohlcv);
    let result = wasm_custom::execute(&script, &df);
    Ok(ScriptResult {
        buy_count: result.buy_signals.iter().filter(|&&b| b).count(),
        sell_count: result.sell_signals.iter().filter(|&&b| b).count(),
        params: result.params,
        errors: result.errors,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScriptResult {
    buy_count: usize,
    sell_count: usize,
    params: std::collections::HashMap<String, f64>,
    errors: Vec<String>,
}

// ── Data Download ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadSummary {
    code: String,
    name: String,
    rows_inserted: usize,
    date_range: Option<(String, String)>,
}

#[tauri::command]
fn download_stock_list() -> Result<Vec<download::StockListItem>, String> {
    download::download_stock_list().map_err(|e| e.to_string())
}

#[tauri::command]
fn download_stock_data(code: String, name: Option<String>,
                       app: tauri::AppHandle) -> Result<DownloadSummary, String> {
    let guard = db::get_db(&app).map_err(|e| e.to_string())?;
    let summary = download::download_and_import(
        &guard,
        &code,
        name.as_deref(),
    ).map_err(|e| e.to_string())?;
    Ok(DownloadSummary {
        code: summary.code,
        name: summary.name,
        rows_inserted: summary.rows_inserted,
        date_range: summary.date_range,
    })
}

#[tauri::command]
fn check_for_app_update(_app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    // This will be implemented when the update server is deployed
    // For now, returns None indicating no update available
    Ok(None)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateInfo {
    version: String,
    notes: String,
    download_url: String,
}

// ── File System ──

#[tauri::command]
fn get_app_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    app.path().app_data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let db_path = app.path().app_data_dir()
                .map(|p| p.join("moneyearning.db"))?;
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let db = rusqlite::Connection::open(&db_path)
                .expect("无法打开数据库");
            db::run_migrations(&db).expect("数据库迁移失败");
            db::set_db(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_machine_fingerprint, activate_license, check_license,
            query_daily_prices, query_stock_list, query_stock_by_code, get_data_summary,
            import_csv,
            watchlist_list, watchlist_create, watchlist_delete,
            watchlist_items, watchlist_add_item, watchlist_remove_item,
            trade_create, trade_list, trade_pnl,
            delete_stock,
            strategy_list, strategy_create, strategy_update, strategy_delete,
            compute_indicator, list_indicators,
            run_backtest,
            run_scanner,
            run_caps_search, run_cgpc_search, run_mars_search,
            run_metasearcher_select, run_metasearcher_record,
            get_metasearcher_best, get_metasearcher_count,
            compute_volume_profile, compute_chip_distribution, compute_sr_levels,
            compute_concentration, compute_profit_loss_ratio, compute_historical_frames,
            list_patterns, scan_all_patterns,
            compute_risk,
            execute_custom_script,
            download_stock_list, download_stock_data,
            check_for_app_update,
            get_app_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}

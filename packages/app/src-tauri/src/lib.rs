use tauri::{Emitter, Manager};
use serde::{Serialize, Deserialize};

mod db;
mod license;
mod import;
mod download;
mod tdx;
mod alerts;

// ── License ──

#[tauri::command]
fn get_machine_fingerprint() -> Result<String, String> {
    license::generate_fingerprint().map_err(|e| e.to_string())
}

#[tauri::command]
fn activate_license(app: tauri::AppHandle, license_key: String, fingerprint: String) -> Result<LicenseInfo, String> {
    license::activate(&app, &license_key, &fingerprint).map_err(|e| e.to_string())
}

#[tauri::command]
fn check_license(app: tauri::AppHandle) -> Result<LicenseStatus, String> {
    license::check_with_db(&app)
}

fn require_pro_tier(app: &tauri::AppHandle) -> Result<(), String> {
    let ls = license::check_with_db(app)?;
    if !ls.valid || (ls.tier != "pro" && ls.tier != "trial") {
        return Err(format!(
            "此功能需要PRO授权。当前授权: {}（{}天试用剩余）",
            if ls.tier == "trial" { "试用版" } else { "免费版" },
            ls.trial_days_left.unwrap_or(0)
        ));
    }
    Ok(())
}

fn recent_start_date(days_back: i64) -> String {
    let now = chrono::Local::now().date_naive();
    let past = now - chrono::Duration::days(days_back);
    past.format("%Y-%m-%d").to_string()
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

#[tauri::command]
fn import_tdx_day(file_path: String, stock_code: Option<String>, exchange: Option<String>,
                   app: tauri::AppHandle) -> Result<ImportResult, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    tdx::import_day_file(&db, &file_path,
        stock_code.as_deref(), exchange.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn import_tdx_directory(dir_path: String, app: tauri::AppHandle) -> Result<Vec<ImportResult>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    tdx::import_day_directory(&db, &dir_path).map_err(|e| e.to_string())
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
    emotion_tag: Option<String>,
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
                strategy_name: Option<String>, emotion_tag: Option<String>,
                notes: Option<String>,
                app: tauri::AppHandle) -> Result<i64, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::trade_create(&db, stock_id, &trade_date, &direction,
        price, quantity, commission, stamp_tax,
        strategy_name.as_deref(), emotion_tag.as_deref(),
        notes.as_deref())
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

// ── Alerts ──

#[tauri::command]
fn create_alert(name: String, stock_id: i64, condition_type: String, params: String,
                app: tauri::AppHandle) -> Result<i64, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    alerts::create_alert(&db, &name, stock_id, &condition_type, &params)
}

#[tauri::command]
fn list_alerts(app: tauri::AppHandle) -> Result<Vec<alerts::AlertRule>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    alerts::list_alerts(&db)
}

#[tauri::command]
fn update_alert(id: i64, name: String, condition_type: String, params: String,
                enabled: bool, app: tauri::AppHandle) -> Result<(), String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    alerts::update_alert(&db, id, &name, &condition_type, &params, enabled)
}

#[tauri::command]
fn delete_alert(id: i64, app: tauri::AppHandle) -> Result<(), String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    alerts::delete_alert(&db, id)
}

#[tauri::command]
fn check_alerts(app: tauri::AppHandle) -> Result<Vec<alerts::AlertTrigger>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let triggers = alerts::check_alerts(&db, &today)?;
    for t in &triggers {
        let _ = app.emit("alert:triggered", serde_json::json!({
            "id": t.rule.id,
            "name": t.rule.name,
            "stock_code": t.rule.stock_code,
            "stock_name": t.rule.stock_name,
            "message": t.message,
            "current_value": t.current_value,
            "threshold_value": t.threshold_value,
        }));
    }
    Ok(triggers)
}

// ── Portfolio Analysis ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CorrelationResult {
    codes: Vec<String>,
    matrix: Vec<Vec<f64>>,
    date_range: String,
}

#[tauri::command]
fn portfolio_correlation(app: tauri::AppHandle, stock_ids: Vec<i64>,
                          days: Option<i64>) -> Result<CorrelationResult, String> {
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let end = recent_start_date(0);
    let start = recent_start_date(days.unwrap_or(250));
    let price_map = db::fetch_close_prices(&db, &stock_ids, &start, &end)
        .map_err(|e| e.to_string())?;
    let mut codes = Vec::new();
    let mut series: Vec<(String, Vec<f64>)> = Vec::new();
    for &sid in &stock_ids {
        if let Some(closes) = price_map.get(&sid) {
            let code = format!("#{}", sid);
            codes.push(code.clone());
            series.push((code, closes.clone()));
        }
    }
    let n = series.len();
    let mut matrix = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        matrix[i][i] = 1.0;
        for j in 0..i {
            let r = pearson(&series[i].1, &series[j].1);
            matrix[i][j] = r;
            matrix[j][i] = r;
        }
    }
    Ok(CorrelationResult {
        codes: series.iter().map(|s| s.0.clone()).collect(),
        matrix,
        date_range: format!("{} → {}", start, end),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaRResult {
    var_95: f64,
    var_99: f64,
    cvar_95: f64,
    cvar_99: f64,
    portfolio_value: f64,
    daily_volatility: f64,
    period_days: usize,
}

#[tauri::command]
fn portfolio_var(app: tauri::AppHandle, stock_ids: Vec<i64>,
                  weights: Vec<f64>, days: Option<i64>) -> Result<VaRResult, String> {
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let end = recent_start_date(0);
    let start = recent_start_date(days.unwrap_or(250));
    let price_map = db::fetch_close_prices(&db, &stock_ids, &start, &end)
        .map_err(|e| e.to_string())?;
    if price_map.is_empty() {
        return Err("无可用价格数据".into());
    }
    // Build aligned return matrix
    let mut all_returns: Vec<Vec<f64>> = Vec::new();
    for (i, &sid) in stock_ids.iter().enumerate() {
        if let Some(closes) = price_map.get(&sid) {
            if closes.len() < 2 { continue; }
            let returns: Vec<f64> = closes.windows(2).map(|w| w[1]/w[0] - 1.0).collect();
            if all_returns.is_empty() {
                all_returns = vec![vec![0.0; returns.len()]; stock_ids.len()];
            }
            if returns.len() == all_returns[0].len() {
                all_returns[i] = returns;
            }
        }
    }
    if all_returns.is_empty() || all_returns[0].is_empty() {
        return Err("数据不足，无法计算VaR".into());
    }
    let n_days = all_returns[0].len();
    let w: Vec<f64> = if weights.len() == stock_ids.len() && weights.iter().sum::<f64>() > 1e-9 {
        let total: f64 = weights.iter().sum();
        weights.iter().map(|x| x / total).collect()
    } else {
        let n = stock_ids.len() as f64;
        vec![1.0 / n; stock_ids.len()]
    };
    // Portfolio daily returns
    let mut portfolio_returns: Vec<f64> = (0..n_days).map(|t| {
        (0..stock_ids.len()).map(|i| w[i] * all_returns[i][t]).sum()
    }).collect();
    portfolio_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let var_95_idx = ((1.0 - 0.95) * n_days as f64) as usize;
    let var_99_idx = ((1.0 - 0.99) * n_days as f64) as usize;
    let var_95 = -portfolio_returns[var_95_idx.max(0)];
    let var_99 = -portfolio_returns[var_99_idx.max(0)];
    let cvar_95 = if var_95_idx > 0 {
        -portfolio_returns[..var_95_idx].iter().sum::<f64>() / var_95_idx as f64
    } else { var_95 };
    let cvar_99 = if var_99_idx > 0 {
        -portfolio_returns[..var_99_idx].iter().sum::<f64>() / var_99_idx as f64
    } else { var_99 };
    let mean_ret = portfolio_returns.iter().sum::<f64>() / n_days as f64;
    let variance = portfolio_returns.iter().map(|r| (r - mean_ret).powi(2)).sum::<f64>() / n_days as f64;
    Ok(VaRResult {
        var_95, var_99, cvar_95, cvar_99,
        portfolio_value: 0.0,
        daily_volatility: variance.sqrt(),
        period_days: n_days,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConcentrationItem {
    industry: String,
    stock_count: i64,
    weight: f64,
}

#[tauri::command]
fn portfolio_concentration(app: tauri::AppHandle, stock_ids: Vec<i64>,
                            weights: Vec<f64>) -> Result<Vec<ConcentrationItem>, String> {
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let industries = db::fetch_stock_industries(&db, &stock_ids)
        .map_err(|e| e.to_string())?;
    let w: Vec<f64> = if weights.len() == stock_ids.len() && weights.iter().sum::<f64>() > 1e-9 {
        let total: f64 = weights.iter().sum();
        weights.iter().map(|x| x / total).collect()
    } else {
        let n = stock_ids.len() as f64;
        vec![1.0 / n; stock_ids.len()]
    };
    let mut groups: std::collections::HashMap<String, (i64, f64)> = std::collections::HashMap::new();
    for (i, &sid) in stock_ids.iter().enumerate() {
        let industry = industries.get(&sid).cloned().unwrap_or_else(|| "未分类".to_string());
        let entry = groups.entry(industry).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += w[i];
    }
    let mut items: Vec<ConcentrationItem> = groups.into_iter().map(|(industry, (count, weight))| {
        ConcentrationItem { industry, stock_count: count, weight: (weight * 100.0 * 100.0).round() / 100.0 }
    }).collect();
    items.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
    Ok(items)
}

#[tauri::command]
fn stock_set_industry(app: tauri::AppHandle, stock_id: i64,
                       industry: String) -> Result<(), String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::set_stock_industry(&db, stock_id, &industry).map_err(|e| e.to_string())
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len()) as f64;
    if n < 3.0 { return 0.0; }
    let mean_a = a.iter().sum::<f64>() / n;
    let mean_b = b.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..(n as usize) {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }
    let denom = (var_a * var_b).sqrt();
    if denom < 1e-12 { 0.0 } else { cov / denom }
}

// ── Strategies ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StrategyTemplate {
    name: String,
    name_cn: String,
    category: String,
    is_free: bool,
    params: std::collections::HashMap<String, f64>,
    description: String,
}

#[tauri::command]
fn list_strategy_templates(app: tauri::AppHandle) -> Vec<StrategyTemplate> {
    let ls = license::check_with_db(&app).unwrap_or(license::check_cached());
    let is_paid = ls.valid && (ls.tier == "pro" || ls.tier == "trial");
    wasm_backtest::list_strategies()
        .iter()
        .filter(|m| is_paid || m.is_free)
        .map(|m| StrategyTemplate {
            name: m.name.to_string(),
            name_cn: m.name_cn.to_string(),
            category: m.category.to_string(),
            is_free: m.is_free,
            params: m.params.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            description: m.description.to_string(),
        })
        .collect()
}

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
async fn run_backtest(app: tauri::AppHandle, data: Vec<IndicatorInput>, template: String,
                params: std::collections::HashMap<String, f64>,
                config: Option<BtConfig>)
                -> Result<wasm_core::BtResult, String> {
    require_pro_tier(&app)?;
    let bt_config: wasm_backtest::BacktestConfig = config.map(Into::into).unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || {
        let ohlcv: Vec<wasm_core::OHLCV> = data.iter().map(|d| wasm_core::OHLCV {
            open: d.open, high: d.high, low: d.low, close: d.close,
            volume: d.volume, amount: d.amount, turnover: d.turnover,
            trade_date: d.time.to_string(),
        }).collect();
        let df = wasm_core::DataFrame::new(&ohlcv);
        Ok(wasm_backtest::run_with_template(&df, &template, &params, &bt_config))
    }).await.map_err(|e| format!("{}", e))?
}

// ── Walk-Forward ──

#[tauri::command]
async fn run_walk_forward(
    app: tauri::AppHandle,
    data: Vec<IndicatorInput>, template: String,
    param_grid: std::collections::HashMap<String, Vec<f64>>,
    in_sample: usize, out_sample: usize, step_size: usize,
    anchor_mode: String,
    config: Option<BtConfig>,
) -> Result<wasm_backtest::WalkForwardResult, String> {
    require_pro_tier(&app)?;
    let bt_config: wasm_backtest::BacktestConfig = config.map(Into::into).unwrap_or_default();
    let wf_config = wasm_backtest::WalkForwardConfig {
        in_sample_size: in_sample.max(1),
        out_sample_size: out_sample.max(1),
        anchor_mode: match anchor_mode.as_str() {
            "anchored" => wasm_backtest::AnchorMode::Anchored,
            _ => wasm_backtest::AnchorMode::Rolling,
        },
        step_size: step_size.max(1),
    };
    tauri::async_runtime::spawn_blocking(move || {
        let ohlcv: Vec<wasm_core::OHLCV> = data.iter().map(|d| wasm_core::OHLCV {
            open: d.open, high: d.high, low: d.low, close: d.close,
            volume: d.volume, amount: d.amount, turnover: d.turnover,
            trade_date: d.time.to_string(),
        }).collect();
        let df = wasm_core::DataFrame::new(&ohlcv);
        Ok(wasm_backtest::walk_forward_analysis(&df, &template, &param_grid, &wf_config, &bt_config))
    }).await.map_err(|e| format!("{}", e))?
}

// ── Monte Carlo ──

#[tauri::command]
async fn run_monte_carlo(
    app: tauri::AppHandle,
    data: Vec<IndicatorInput>, template: String,
    params: std::collections::HashMap<String, f64>,
    num_simulations: usize, method: String,
    confidence_level: Option<f64>,
    config: Option<BtConfig>,
) -> Result<wasm_backtest::MonteCarloResult, String> {
    require_pro_tier(&app)?;
    let bt_config: wasm_backtest::BacktestConfig = config.map(Into::into).unwrap_or_default();
    let mc_config = wasm_backtest::MonteCarloConfig {
        num_simulations: num_simulations.max(100).min(5000),
        method: match method.as_str() {
            "return_bootstrap" => wasm_backtest::McMethod::ReturnBootstrap,
            "parametric" => wasm_backtest::McMethod::Parametric,
            _ => wasm_backtest::McMethod::TradeShuffle,
        },
        confidence_level: confidence_level.unwrap_or(0.95),
    };
    tauri::async_runtime::spawn_blocking(move || {
        let ohlcv: Vec<wasm_core::OHLCV> = data.iter().map(|d| wasm_core::OHLCV {
            open: d.open, high: d.high, low: d.low, close: d.close,
            volume: d.volume, amount: d.amount, turnover: d.turnover,
            trade_date: d.time.to_string(),
        }).collect();
        let df = wasm_core::DataFrame::new(&ohlcv);
        let signals = wasm_backtest::generate_signals(&df, &template, &params);
        Ok(wasm_backtest::monte_carlo_simulate(&df, &signals, &bt_config, &mc_config))
    }).await.map_err(|e| format!("{}", e))?
}

// ── Parameter Optimization ──

#[tauri::command]
async fn run_optimization(
    app: tauri::AppHandle,
    data: Vec<IndicatorInput>, template: String,
    param_grid: std::collections::HashMap<String, Vec<f64>>,
    method: String, target_metric: String,
    max_iterations: Option<usize>,
    config: Option<BtConfig>,
) -> Result<wasm_backtest::OptimizerResult, String> {
    require_pro_tier(&app)?;
    let bt_config: wasm_backtest::BacktestConfig = config.map(Into::into).unwrap_or_default();
    let grid: std::collections::HashMap<String, (f64, f64, f64)> = param_grid.iter().map(|(k, v)| {
        let min = v.first().copied().unwrap_or(0.0);
        let max = v.get(1).copied().unwrap_or(100.0);
        let step = v.get(2).copied().unwrap_or(1.0);
        (k.clone(), (min, max, step.max(0.001)))
    }).collect();
    let opt_config = wasm_backtest::OptimizerConfig {
        method: match method.as_str() {
            "genetic_algorithm" => wasm_backtest::OptimizerMethod::GeneticAlgorithm,
            _ => wasm_backtest::OptimizerMethod::GridSearch,
        },
        max_iterations: max_iterations.unwrap_or(5000).min(10000),
        target_metric: match target_metric.as_str() {
            "total_return" => wasm_backtest::TargetMetric::TotalReturn,
            "calmar_ratio" => wasm_backtest::TargetMetric::CalmarRatio,
            "sortino_ratio" => wasm_backtest::TargetMetric::SortinoRatio,
            _ => wasm_backtest::TargetMetric::SharpeRatio,
        },
        early_stop_rounds: 50,
    };
    tauri::async_runtime::spawn_blocking(move || {
        let ohlcv: Vec<wasm_core::OHLCV> = data.iter().map(|d| wasm_core::OHLCV {
            open: d.open, high: d.high, low: d.low, close: d.close,
            volume: d.volume, amount: d.amount, turnover: d.turnover,
            trade_date: d.time.to_string(),
        }).collect();
        let df = wasm_core::DataFrame::new(&ohlcv);
        Ok(wasm_backtest::optimize(&df, &template, &grid, &opt_config, &bt_config))
    }).await.map_err(|e| format!("{}", e))?
}

// ── Scanner ──

#[tauri::command]
async fn run_scanner(stock_ids: Vec<i64>, expr: wasm_core::ScanExpr,
               app: tauri::AppHandle) -> Result<Vec<ScanResultItem>, String> {
    require_pro_tier(&app)?;
    let pairs = {
        let db = db::get_db(&app).map_err(|e| e.to_string())?;
        let mut pairs: Vec<(i64, wasm_core::DataFrame)> = Vec::new();
        for stock_id in &stock_ids {
            let prices = db::query_daily(&db, *stock_id, &recent_start_date(365), "2099-12-31")
                .map_err(|e| e.to_string())?;
            if prices.is_empty() { continue; }
            let ohlcv: Vec<wasm_core::OHLCV> = prices.iter().map(|dp| wasm_core::OHLCV {
                open: dp.open, high: dp.high, low: dp.low, close: dp.close,
                volume: dp.volume, amount: Some(dp.amount), turnover: dp.turnover,
                trade_date: dp.trade_date.clone(),
            }).collect();
            pairs.push((*stock_id, wasm_core::DataFrame::new(&ohlcv)));
        }
        pairs
    };

    let total = pairs.len();
    use std::sync::atomic::{AtomicUsize, Ordering};
    let progress = AtomicUsize::new(0);
    let app_handle = app.clone();

    tauri::async_runtime::spawn_blocking(move || {
        use rayon::prelude::*;
        let mut results: Vec<ScanResultItem> = pairs.par_iter()
            .flat_map(|(stock_id, df)| {
                let matches = wasm_scanner::scan(&[(0, df.clone())], &expr);
                let done = progress.fetch_add(1, Ordering::Relaxed) + 1;
                if done % 10 == 0 || done == total {
                    let _ = app_handle.emit("scanner:progress", serde_json::json!({
                        "current": done,
                        "total": total,
                    }));
                }
                matches.iter().map(|m| ScanResultItem {
                    stock_id: *stock_id,
                    score: m.score,
                    signals: m.signals.clone(),
                }).collect::<Vec<_>>()
            })
            .collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }).await.map_err(|e| format!("{}", e))
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
        let prices = db::query_daily(guard, *stock_id, &recent_start_date(365), "2099-12-31")
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
async fn run_caps_search(
    stock_ids: Vec<i64>,
    app: tauri::AppHandle,
) -> Result<Vec<wasm_scanner::CapsResult>, String> {
    require_pro_tier(&app)?;
    let returns = {
        let db = db::get_db(&app).map_err(|e| e.to_string())?;
        load_returns_batch(&db, &stock_ids)?
    };
    if returns.is_empty() {
        return Err("No stocks with sufficient data".into());
    }

    tauri::async_runtime::spawn_blocking(move || {
        let pool_name = format!("Pool_{}_stocks", returns.len());
        let pools = vec![(pool_name, returns)];
        let strategies = vec![
            "risk_parity".to_string(),
            "min_variance".to_string(),
            "hierarchical_rp".to_string(),
        ];
        let params = std::collections::HashMap::new();
        Ok(wasm_scanner::run_caps(&pools, &strategies, &params))
    }).await.map_err(|e| format!("{}", e))?
}

#[tauri::command]
async fn run_cgpc_search(
    stock_ids: Vec<i64>,
    n_pools: usize,
    pool_size: usize,
    app: tauri::AppHandle,
) -> Result<Vec<wasm_scanner::CgpcPool>, String> {
    require_pro_tier(&app)?;
    let returns = {
        let db = db::get_db(&app).map_err(|e| e.to_string())?;
        load_returns_batch(&db, &stock_ids)?
    };
    if returns.len() < 3 {
        return Err("Need at least 3 stocks with sufficient data".into());
    }

    tauri::async_runtime::spawn_blocking(move || {
        Ok(wasm_scanner::build_diverse_pools(&returns, n_pools, pool_size))
    }).await.map_err(|e| format!("{}", e))?
}

#[tauri::command]
async fn run_mars_search(
    stock_ids: Vec<i64>,
    n_regimes: usize,
    app: tauri::AppHandle,
) -> Result<wasm_scanner::MarsResult, String> {
    require_pro_tier(&app)?;
    let (returns, strategy_returns) = {
        let db = db::get_db(&app).map_err(|e| e.to_string())?;
        let returns = load_returns_batch(&db, &stock_ids)?;
        if returns.len() < 5 {
            return Err("Need at least 5 stocks with sufficient data".into());
        }
        let n = returns[0].len();
        let mut strategy_returns: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
        let momentum: Vec<f64> = (0..n).map(|d| {
            returns.iter().map(|r| r[d]).sum::<f64>() / returns.len() as f64
        }).collect();
        strategy_returns.insert("momentum".to_string(), momentum);
        (returns, strategy_returns)
    };

    tauri::async_runtime::spawn_blocking(move || {
        Ok(wasm_scanner::run_mars(&returns, &strategy_returns, n_regimes))
    }).await.map_err(|e| format!("{}", e))?
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
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, &recent_start_date(1000), "2099-12-31")
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
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, &recent_start_date(1000), "2099-12-31")
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
    let prices = db::query_daily(&db, stock_id, &recent_start_date(500), "2099-12-31")
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
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, &recent_start_date(1000), "2099-12-31")
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
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, &recent_start_date(1000), "2099-12-31")
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
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, &recent_start_date(1000), "2099-12-31")
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
fn compute_indicator(app: tauri::AppHandle, name: String, data: Vec<IndicatorInput>,
                     params: std::collections::HashMap<String, f64>)
                     -> Result<Vec<wasm_core::IndicatorOutput>, String> {
    // Check license for PRO indicators
    let meta = wasm_indicators::metadata(&name);
    if meta.as_ref().map_or(false, |m| !m.is_free) {
        let ls = license::check_with_db(&app)?;
        if ls.tier != "pro" && ls.tier != "trial" {
            return Err(format!("「{}」为专业版指标，需要PRO授权。当前授权: 免费版",
                meta.unwrap().name_cn));
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
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, &recent_start_date(365), "2099-12-31")
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
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, &recent_start_date(500), "2099-12-31")
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
    require_pro_tier(&app)?;
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    let prices = db::query_daily(&db, stock_id, &recent_start_date(500), "2099-12-31")
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
    // Phase 1: Download (no DB lock — HTTP can be slow)
    let market = if code.starts_with("60") || code.starts_with("68") { "SH" } else { "SZ" };
    let rows = download::download_daily_kline(&code, market).map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Err("未获取到数据，请检查股票代码".into());
    }
    let stock_name = name.unwrap_or_else(|| code.clone());
    let date_range = rows.first().and_then(|f| rows.last().map(|l| (f.trade_date.clone(), l.trade_date.clone())));

    // Phase 2: Quick DB import (lock held briefly)
    let (_, inserted) = {
        let guard = db::get_db(&app).map_err(|e| e.to_string())?;
        download::save_kline_to_db(&guard, &code, &stock_name, market, &rows)
            .map_err(|e| e.to_string())?
    };

    Ok(DownloadSummary {
        code,
        name: stock_name,
        rows_inserted: inserted,
        date_range,
    })
}

// ── Minute Data ──

#[tauri::command]
fn query_minute_prices(stock_id: i64, start: String, end: String,
                       app: tauri::AppHandle) -> Result<Vec<db::MinutePrice>, String> {
    let db = db::get_db(&app).map_err(|e| e.to_string())?;
    db::query_minute_prices(&db, stock_id, &start, &end).map_err(|e| e.to_string())
}

#[tauri::command]
fn download_minute_data(code: String, klt: Option<u32>,
                        app: tauri::AppHandle) -> Result<download::MinuteImportSummary, String> {
    let klt = klt.unwrap_or(5);
    let market = if code.starts_with("60") || code.starts_with("68") { "SH" } else { "SZ" };

    // Phase 1: Download (no DB lock)
    let rows = download::download_minute_kline(&code, market, klt)
        .map_err(|e| e.to_string())?;
    if rows.is_empty() {
        return Err("未获取到分钟数据，请检查股票代码或交易日".into());
    }
    let time_range = rows.first().and_then(|f| rows.last().map(|l| (f.trade_time.clone(), l.trade_time.clone())));

    // Phase 2: Quick DB import
    let inserted = {
        let guard = db::get_db(&app).map_err(|e| e.to_string())?;
        let stock_id = db::upsert_stock(&guard, &code, &code, market, None)
            .map_err(|e| e.to_string())?;
        let db_rows: Vec<db::MinuteRow> = rows.iter().map(|r| db::MinuteRow {
            trade_time: r.trade_time.clone(),
            open: r.open, high: r.high, low: r.low, close: r.close,
            volume: r.volume, amount: r.amount,
        }).collect();
        db::bulk_insert_minute(&guard, stock_id, &db_rows).map_err(|e| e.to_string())?
    };

    Ok(download::MinuteImportSummary {
        code, klt,
        klt_label: match klt {
            1 => "1分钟".into(), 5 => "5分钟".into(),
            15 => "15分钟".into(), 30 => "30分钟".into(),
            60 => "60分钟".into(),
            _ => format!("{}分钟", klt),
        },
        rows_inserted: inserted,
        time_range,
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
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = app.path().app_data_dir()
                .map(|p| p.join("moneyearning.db"))?;
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let db = rusqlite::Connection::open(&db_path)
                .map_err(|e| format!("无法打开数据库 ({}): {}", db_path.display(), e))?;
            db::run_migrations(&db)
                .map_err(|e| format!("数据库迁移失败: {}", e))?;
            db::set_db(db);
            // Populate license cache at startup (best-effort)
            let handle = app.handle();
            let _ = license::check_with_db(&handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_machine_fingerprint, activate_license, check_license,
            query_daily_prices, query_stock_list, query_stock_by_code, get_data_summary,
            import_csv, import_tdx_day, import_tdx_directory,
            watchlist_list, watchlist_create, watchlist_delete,
            watchlist_items, watchlist_add_item, watchlist_remove_item,
            trade_create, trade_list, trade_pnl,
            delete_stock,
            create_alert, list_alerts, update_alert, delete_alert, check_alerts,
            strategy_list, strategy_create, strategy_update, strategy_delete,
            list_strategy_templates,
            compute_indicator, list_indicators,
            run_backtest,
            run_walk_forward, run_monte_carlo, run_optimization,
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
            query_minute_prices, download_minute_data,
            check_for_app_update,
            get_app_data_dir,
            // Portfolio analysis (PRO)
            portfolio_correlation, portfolio_var, portfolio_concentration,
            stock_set_industry,
        ])
        .run(tauri::generate_context!());
    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("QuantVault 启动失败: {}", e);
            std::process::exit(1);
        }
    }
}

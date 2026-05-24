# Architecture Design — 项目架构设计文档

> 版本 1.1 | 2026-05-24 | 基于 GOAL_SPEC.md v0.8.0

本文档定义了 MoneyEarning 项目的：目录结构、模块接口契约、前端组件架构、三期实现路线图。是 `/goal` 指令实现产品的施工图纸。

---

## 一、项目完整目录结构

```
MoneyEarning/
├── packages/                    # pnpm monorepo (前端)
│   ├── chart-engine/            # K线图自研Canvas+WebGL引擎
│   ├── ui/                      # 通用UI组件库
│   └── app/                     # Tauri主应用 (React+TypeScript)
│
├── crates/                      # Cargo workspace (Rust/WASM)
│   ├── wasm-core/               # 核心基础设施 (DataFrame, linalg, stats)
│   ├── wasm-indicators/         # 316个技术指标
│   ├── wasm-pattern/            # 61种K线形态识别
│   ├── wasm-scanner/            # 全市场选股扫描
│   ├── wasm-backtest/           # 策略回测引擎
│   ├── wasm-distribution/       # 筹码分布估算
│   ├── wasm-profile/            # Market/Volume Profile
│   ├── wasm-license/            # 授权验证 (RSA+指纹)
│   └── wasm-custom/             # ME Script脚本编译器+运行时
│
├── src-tauri/                   # Tauri 2.x 主进程
│   ├── src/                     # Rust原生层 (IPC+SQLite+文件IO+授权)
│   └── tauri.conf.json
│
├── data/                        # 用户数据目录 (SQLite DB)
├── docs/                        # 文档
├── scripts/                     # 构建/打包/开发脚本
├── GOAL_SPEC.md
├── pnpm-workspace.yaml
├── Cargo.toml                   # Cargo workspace root
└── package.json
```

### packages/ 内部结构

```
packages/
├── chart-engine/                # 自研K线图引擎 (独立包，可单独测试)
│   ├── src/
│   │   ├── ViewportManager.ts   # 视口管理 (缩放/平移/惯性滚动/十字光标)
│   │   ├── LayerManager.ts      # 图层系统
│   │   ├── renderers/
│   │   │   ├── CandleRenderer.ts      # K线实体+影线
│   │   │   ├── VolumeRenderer.ts      # 成交量柱状图
│   │   │   ├── IndicatorRenderer.ts   # 指标图层
│   │   │   ├── DrawingRenderer.ts     # 画线图层 (SVG Overlay)
│   │   │   ├── PatternRenderer.ts     # 形态标注图层
│   │   │   ├── ProfileRenderer.ts     # Volume/Market Profile
│   │   │   └── WebGLRenderer.ts       # WebGL大数据量场景
│   │   ├── InteractionSystem.ts       # 交互系统
│   │   ├── ChartConfig.ts             # 配色/布局/周期配置
│   │   └── index.ts
│   ├── __tests__/
│   └── package.json
│
├── ui/                          # 通用UI组件库
│   ├── src/
│   │   ├── Button/ Modal/ Table/ Tabs/
│   │   ├── Form/ (Input, Select, Slider, ColorPicker...)
│   │   ├── Layout/ (SplitPane, Toolbar, Sidebar...)
│   │   ├── LicenseDialog/
│   │   └── index.ts
│   └── package.json
│
└── app/                         # Tauri主应用
    ├── src/
    │   ├── pages/
    │   │   ├── Dashboard/
    │   │   ├── KLineChart/
    │   │   ├── Scanner/
    │   │   ├── Backtest/
    │   │   ├── TradeReview/
    │   │   ├── Portfolio/
    │   │   ├── MEScriptEditor/
    │   │   ├── DataManager/
    │   │   └── Settings/
    │   ├── components/          # 跨页面共享组件
    │   ├── stores/              # Zustand状态管理
    │   │   ├── chartStore.ts
    │   │   ├── scannerStore.ts
    │   │   ├── backtestStore.ts
    │   │   ├── dataStore.ts
    │   │   ├── licenseStore.ts
    │   │   └── preferencesStore.ts
    │   ├── hooks/
    │   └── main.tsx
    ├── index.html
    └── package.json
```

### crates/ 内部结构 (以 wasm-indicators 为例)

```
crates/wasm-indicators/
├── Cargo.toml
├── src/
│   ├── lib.rs                   # WASM导出入口 + register_indicator!宏
│   ├── trend/                   # 18个趋势指标
│   │   ├── mod.rs, adx.rs, aroon.rs, macd.rs, psar.rs, supertrend.rs, ...
│   ├── momentum/                # 41个动量指标
│   │   ├── mod.rs, rsi.rs, stoch.rs, kdj.rs, cci.rs, mfi.rs, fisher.rs, ...
│   ├── volatility/              # 14个波动指标
│   ├── volume/                  # 15个成交量指标
│   ├── cycle/                   # 8个周期指标
│   ├── composite/               # 自定义组合
│   └── custom/                  # ME Script解释器
│       ├── lexer.rs
│       ├── parser.rs
│       ├── codegen.rs
│       └── runtime.rs
└── tests/
```

### Cargo.toml 依赖关系

```
wasm-core (DataFrame, linalg, stats, types)
   ├── wasm-indicators
   ├── wasm-pattern
   ├── wasm-scanner
   ├── wasm-distribution
   ├── wasm-profile
   │       │
   │       ├── wasm-backtest (依赖 indicators + scanner)
   │       └── wasm-custom (依赖 indicators 的所有实现)
   └── wasm-license (独立，无外部依赖)
```

---

## 二、Tauri IPC 命令清单（React ↔ Rust后端）

所有命令为 `#[tauri::command]` 签名，前端通过 `invoke()` 调用。长耗时操作通过 Tauri event 推送进度。

### 2.1 数据导入模块

```rust
fn import_csv(file_path: String, stock_code: String, exchange: String) -> Result<ImportResult, Error>
fn import_tdx(file_path: String) -> Result<ImportResult, Error>
fn import_ths(file_path: String) -> Result<ImportResult, Error>
fn import_dzh(file_path: String) -> Result<ImportResult, Error>
fn batch_import(files: Vec<String>) -> Result<BatchImportResult, Error>
// ImportResult = { stock_count, row_count, skipped, date_range: (Date, Date) }
```

### 2.2 SQLite 数据库模块

```rust
fn query_daily_prices(stock_id: i64, start_date: String, end_date: String) -> Result<Vec<DailyPrice>, Error>
fn query_minute_prices(stock_id: i64, trade_date: String) -> Result<Vec<MinutePrice>, Error>
fn query_stock_list() -> Result<Vec<Stock>, Error>
fn query_stock_by_code(code: String) -> Result<Stock, Error>
fn search_stocks(keyword: String) -> Result<Vec<Stock>, Error>
fn delete_stock_data(stock_id: i64) -> Result<(), Error>
fn get_data_summary() -> Result<DataSummary, Error>
fn vacuum_database() -> Result<(), Error>
```

### 2.3 交易记录 & 复盘模块

```rust
fn save_trade(trade: NewTrade) -> Result<i64, Error>
fn update_trade(id: i64, trade: UpdateTrade) -> Result<(), Error>
fn delete_trade(id: i64) -> Result<(), Error>
fn query_trades(stock_id: Option<i64>, start: String, end: String) -> Result<Vec<Trade>, Error>
fn get_pnl_summary(start: String, end: String) -> Result<PnLSummary, Error>
```

### 2.4 自选股 & 策略管理模块

```rust
fn create_watchlist(name: String, description: Option<String>) -> Result<i64, Error>
fn delete_watchlist(id: i64) -> Result<(), Error>
fn add_to_watchlist(watchlist_id: i64, stock_id: i64) -> Result<(), Error>
fn remove_from_watchlist(watchlist_id: i64, stock_id: i64) -> Result<(), Error>
fn list_watchlists() -> Result<Vec<Watchlist>, Error>
fn save_strategy(name: String, script: String, params: String) -> Result<i64, Error>
fn list_strategies() -> Result<Vec<Strategy>, Error>
fn delete_strategy(id: i64) -> Result<(), Error>
```

### 2.5 WASM 计算调度模块（核心）

```rust
fn compute_indicator(stock_id: i64, indicator: String, params: String,
                     start: String, end: String) -> Result<IndicatorResult, Error>
fn compute_indicators_batch(stock_ids: Vec<i64>, indicators: Vec<IndicatorSpec>,
                            start: String, end: String) -> Result<Vec<IndicatorResult>, Error>
fn scan_market(conditions: String) -> Result<ScanResult, Error>
fn scan_market_stream(conditions: String) -> Result<(), Error>    // emit progress events
fn run_backtest(strategy_id: i64, stock_ids: Vec<i64>,
                start: String, end: String, capital: f64,
                params: Option<String>) -> Result<BacktestResult, Error>
fn run_walk_forward(strategy_id: i64, stock_ids: Vec<i64>,
                    start: String, end: String,
                    in_sample_days: u32, out_sample_days: u32) -> Result<WalkForwardResult, Error>
fn compute_distribution(stock_id: i64, trade_date: String) -> Result<DistributionResult, Error>
fn compute_profile(stock_id: i64, start: String, end: String,
                   profile_type: String) -> Result<ProfileResult, Error>
fn compile_me_script(source: String) -> Result<CompileResult, Error>
fn execute_me_script(bytecode_id: String, stock_id: i64,
                     start: String, end: String) -> Result<ScriptResult, Error>
```

### 2.6 授权验证模块

```rust
fn get_machine_fingerprint() -> Result<String, Error>
fn activate_license(license_key: String) -> Result<LicenseInfo, Error>
fn check_license() -> Result<LicenseStatus, Error>
fn get_license_info() -> Result<LicenseInfo, Error>
// LicenseStatus = { valid: bool, tier: "free"|"pro", expiry: Option<Date>, features: Vec<String> }
```

### 2.7 用户偏好 & 文件系统模块

```rust
fn get_preference(key: String) -> Result<Option<String>, Error>
fn set_preference(key: String, value: String) -> Result<(), Error>
fn list_preferences() -> Result<Vec<(String, String)>, Error>
fn open_file_dialog(filters: Vec<String>) -> Result<Option<String>, Error>
fn get_app_data_dir() -> Result<String, Error>
fn export_report(format: String, data: String, output_path: String) -> Result<(), Error>
```

### 2.8 Tauri 事件（后端→前端 推送）

```rust
emit("scan-progress", ScanProgress { current: u32, total: u32, found: u32 })
emit("backtest-progress", BtProgress { step: String, progress: f64 })
emit("import-progress", ImportProgress { file: String, rows: u32, total_rows: u32 })
emit("compile-error", CompileError { line: u32, col: u32, message: String })
```

---

## 三、WASM 模块导出接口

### 3.1 wasm-core（核心基础设施）

```rust
pub struct DataFrame { columns: HashMap<String, Column>, index: Vec<i64> }
impl DataFrame {
    pub fn new(records: &[OHLCV]) -> DataFrame;
    pub fn column(&self, name: &str) -> Option<&Column>;
    pub fn len(&self) -> usize;
    pub fn slice(&self, start: usize, end: usize) -> DataFrame;
    pub fn add_column(&mut self, name: &str, data: Column);
}
// Column 类型：F64(Vec<f64>), I32(Vec<i32>), Bool(Vec<bool>)

pub fn covariance_matrix(data: &DataFrame, cols: &[&str]) -> Vec<Vec<f64>>;
pub fn eigenvalue_decomp(matrix: &[Vec<f64>]) -> (Vec<f64>, Vec<Vec<f64>>);
pub fn correlation_matrix(data: &DataFrame, cols: &[&str]) -> Vec<Vec<f64>>;
pub fn mean(data: &[f64]) -> f64;
pub fn std_dev(data: &[f64]) -> f64;
pub fn skew(data: &[f64]) -> f64;
pub fn kurtosis(data: &[f64]) -> f64;
pub fn quantile(data: &[f64], q: f64) -> f64;
pub fn zscore(data: &[f64]) -> Vec<f64>;
pub fn winsorize(data: &[f64], lower: f64, upper: f64) -> Vec<f64>;
pub fn spearman_rank(x: &[f64], y: &[f64]) -> f64;
pub fn pearson_corr(x: &[f64], y: &[f64]) -> f64;
pub fn loewdin_orthogonalize(ic_matrix: &[Vec<f64>]) -> Vec<Vec<f64>>;

pub struct OHLCV { pub open: f64, high: f64, low: f64, close: f64, volume: f64,
                   pub trade_date: String, pub amount: Option<f64>, turnover: Option<f64> }
pub struct IndicatorOutput { pub name: String, pub values: Column, pub style: OutputStyle }
pub enum OutputStyle { Line, Histogram, Dots, Band { upper: Column, lower: Column } }
```

### 3.2 wasm-indicators（316个技术指标）

```rust
pub fn compute(name: &str, df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError>;
pub fn compute_many(specs: &[IndicatorSpec], df: &DataFrame) -> Result<Vec<Vec<IndicatorOutput>>, IndError>;
pub fn metadata(name: &str) -> Option<IndicatorMeta>;
pub fn list_all() -> Vec<IndicatorMeta>;
pub fn list_by_category(cat: &str) -> Vec<IndicatorMeta>;

#[macro_export] macro_rules! register_indicator { ... }

pub enum IndError { InvalidName, InvalidParams(String), DataInsufficient(usize), ComputationFailed }
```

### 3.3 wasm-pattern（61种K线形态）

```rust
pub fn recognize(df: &DataFrame, pattern_name: &str) -> Result<PatternResult, PatternError>;
pub fn recognize_all(df: &DataFrame) -> Vec<PatternResult>;
pub fn pattern_list() -> Vec<PatternMeta>;

pub struct PatternResult {
    pub name: String, pub name_cn: String,
    pub positions: Vec<usize>, pub directions: Vec<i32>, pub confidence: Vec<f64>,
}
```

### 3.4 wasm-scanner（全市场选股扫描）

```rust
pub fn scan(stocks: &[DataFrame], condition: &ScanExpr) -> Result<ScanResult, ScanError>;
pub fn rank(stocks: &[DataFrame], factors: &[FactorSpec]) -> Vec<RankedStock>;

pub enum ScanExpr {
    Indicator { name: String, params: HashMap<String,f64>, op: CompareOp, value: f64 },
    And(Box<ScanExpr>, Box<ScanExpr>), Or(Box<ScanExpr>, Box<ScanExpr>),
    Not(Box<ScanExpr>), Cross(Box<ScanExpr>, Box<ScanExpr>),
}
pub enum CompareOp { Gt, Lt, Gte, Lte, Eq }
```

### 3.5 wasm-backtest（策略回测引擎）

```rust
pub fn vector_backtest(df: &DataFrame, signal: &[f64], capital: f64,
                       commission: f64, slippage: f64) -> Result<BtResult, BtError>;
pub fn event_backtest(df: &DataFrame, strategy: &BtStrategy,
                      capital: f64, config: &BtConfig) -> Result<BtResult, BtError>;
pub fn grid_search(df: &DataFrame, strategy: &BtStrategy,
                   param_grid: &HashMap<String, Vec<f64>>) -> Vec<OptimizationResult>;
pub fn genetic_optimize(df: &DataFrame, strategy: &BtStrategy,
                        param_space: &[ParamRange], generations: u32) -> Vec<OptimizationResult>;
pub fn walk_forward(df: &DataFrame, strategy: &BtStrategy,
                    in_sample: usize, out_sample: usize, optimizer: &str) -> WfResult;
pub fn monte_carlo(trades: &[TradeRecord], simulations: u32) -> MonteCarloResult;

pub struct BtResult {
    pub total_return: f64, pub annual_return: f64, pub max_drawdown: f64,
    pub sharpe_ratio: f64, pub sortino_ratio: f64, pub calmar_ratio: f64,
    pub win_rate: f64, pub profit_loss_ratio: f64, pub total_trades: u32,
    pub equity_curve: Vec<(String, f64)>, pub monthly_returns: Vec<(String, f64)>,
    pub trades: Vec<TradeRecord>,
}
```

### 3.6 wasm-distribution + wasm-profile

```rust
// wasm-distribution
pub fn estimate_distribution(df: &DataFrame, trade_date: usize) -> DistributionResult;
pub fn concentration(dist: &DistributionResult) -> ConcentrationResult;
pub fn profit_ratio(dist: &DistributionResult, current_price: f64) -> f64;
pub fn historical_animation(df: &DataFrame, start: usize, end: usize) -> Vec<DistributionResult>;

// wasm-profile
pub fn market_profile(df: &DataFrame, start: usize, end: usize) -> MarketProfileResult;
pub fn volume_profile(df: &DataFrame, mode: ProfileMode, start: usize, end: usize) -> VolumeProfileResult;
pub fn poc(profile: &VolumeProfileResult) -> (f64, f64);
pub fn value_area(profile: &VolumeProfileResult, pct: f64) -> (f64, f64);
pub enum ProfileMode { Fixed, Variable, Developing }
```

### 3.7 wasm-license + wasm-custom

```rust
// wasm-license
pub fn verify_signature(license_key: &str, public_key: &[u8]) -> Result<LicensePayload, LicenseError>;
pub fn hash_fingerprint(mac: &str, hostname: &str, os_serial: &str) -> String;
pub fn checksum_module() -> [u8; 32];

// wasm-custom: ME Script 编译器+运行时
pub fn compile(source: &str) -> Result<CompiledScript, CompileError>;
pub fn execute(bytecode: &CompiledScript, df: &DataFrame, stock_pool: Option<&[DataFrame]>) -> Result<ScriptOutput, ScriptError>;
pub struct CompileError { pub line: u32, pub col: u32, pub message: String, pub hint: Option<String> }
```

---

## 四、React 前端架构

### 4.1 页面路由表

| 路由 | 页面 | 免费/付费 | 核心功能 |
|------|------|:---:|------|
| `/` | Dashboard | 全部 | 概览面板：今日涨跌/自选股快照/最近交易 |
| `/chart/:stockId` | KLineChart | 全部(有限制) | K线图主工作区+指标叠加+画线 |
| `/scanner` | Scanner | 付费 | 全市场选股扫描+排序 |
| `/backtest` | Backtest | 付费 | 策略回测+参数优化+Walk-Forward |
| `/review` | TradeReview | 免费 | 交易记录/盈亏统计 |
| `/review/:tradeId` | TradeReviewDetail | 付费 | 结构化复盘+情绪标签+归因 |
| `/portfolio` | Portfolio | 付费 | 组合分析/相关性/行业集中度/VaR |
| `/watchlist` | Watchlist | 免费 | 自选股管理 |
| `/strategies` | Strategies | 付费 | 策略管理器：20模板+自定义 |
| `/editor` | MEScriptEditor | 付费 | ME Script编辑器+编译+实时预览 |
| `/data` | DataManager | 全部 | 数据导入/导出/质量管理 |
| `/settings` | Settings | 全部 | 偏好设置/授权管理/关于 |

### 4.2 组件树

```
App (Layout: Sidebar + MainArea)
├── Sidebar (StockSearchBar, NavMenu, LicenseBadge)
├── pages/
│   ├── KLineChart/ (核心页面)
│   │   ├── ChartContainer ← packages/chart-engine
│   │   ├── Toolbar (PeriodSelector, ChartTypeSelector, LayoutToggle)
│   │   ├── IndicatorPanel (IndicatorSearch, IndicatorList, IndicatorConfig)
│   │   ├── DrawingToolbar
│   │   └── OverlayPanel (VolumePanel, IndicatorSubChart)
│   ├── Scanner/ (ConditionBuilder, ScanResultTable, ScanProgressBar)
│   ├── Backtest/ (StrategySelector, ParamPanel, BacktestChart, MetricsPanel)
│   ├── TradeReview/ (TradeTable, PnLChart, ReviewTemplate)
│   ├── Portfolio/ (HoldingTable, CorrelationHeatmap, RiskGauge)
│   ├── MEScriptEditor/ (CodeEditor→Monaco, CompileOutput, PreviewChart)
│   ├── DataManager/ (ImportWizard, DataTable, QualityReport)
│   └── Settings/ (LicensePanel, PreferencesPanel, AboutPanel)
├── components/ (StockPicker, DateRangePicker, IndicatorParamEditor, ...)
└── stores/ (chartStore, scannerStore, backtestStore, dataStore, licenseStore, preferencesStore)
```

### 4.3 Zustand Store 设计

**chartStore**（最核心）：stockId, period, chartType, activeIndicators, viewport, crosshair, drawings, ohlcvData, computedIndicators, loadData(), computeIndicators()

**licenseStore**：status("loading"|"free"|"pro"|"trial"|"expired"), trialDaysLeft, machineFingerprint, checkLicense(), activate(key), isFeatureEnabled(feature)

---

## 五、三期实现路线图

### Phase 1 — 最小骨架（跑通全链路）

| # | 任务 | 产出 | 验收标准 |
|---|------|------|---------|
| 1.1 | Cargo workspace 初始化 | 各crate骨架 + wasm-core实现 | `cargo build` 全workspace通过 |
| 1.2 | pnpm monorepo 初始化 | 3个package骨架 + chart-engine基础 | chart-engine独立渲染蜡烛图 |
| 1.3 | Tauri 主进程搭建 | SQLite建表 + 数据导入/查询IPC | `cargo tauri dev` 窗口正常启动 |
| 1.4 | 5个免费指标 WASM计算 | sma, ema, macd, rsi, kdj | 结果与通达信偏差<0.1% |
| 1.5 | K线图基础渲染 | K线图层+成交量+视口+周期切换 | 2500根K线交互60fps |

> ✅ Phase 1 完成：启动App→导入数据→K线图→叠加RSI→全链路通

### Phase 2 — 核心打磨（免费版完整可分发）

| # | 任务 | 产出 |
|---|------|------|
| 2.1 | 图表引擎完整化 | 7个Renderer + 画线系统 + 多图多屏 + 配色 |
| 2.2 | 90个免费指标实现 | 10大类全部免费指标 + 单元测试 + 交叉验证 |
| 2.3 | 自选股管理 | 创建/编辑/删除自选列表 |
| 2.4 | 交易记录+盈亏统计 | 录入/编辑/删除 + 按策略/股票分组统计 |
| 2.5 | 数据管理完整化 | 多格式兼容 + 质量报告 + 批量导入 |
| 2.6 | RSA授权系统 | wasm-license + 机器指纹 + 激活码工具 + 防篡改 |
| 2.7 | 打包分发 | Windows .msi + macOS .dmg + 代码混淆 |

> ✅ Phase 2 完成：可打包为安装包分发给用户

### Phase 3 — 付费版（专业功能补齐）

| # | 任务 | 产出 |
|---|------|------|
| 3.1 | 226个付费指标实现 | 全部付费指标 + 测试 + 验证 |
| 3.2 | 全市场选股扫描 | wasm-scanner + 条件构建器 + 结果表格 |
| 3.3 | 策略回测引擎 | 双引擎 + 优化 + Walk-Forward + Monte Carlo + 20模板 |
| 3.4 | ME Script 编译器 | Lexer→Parser→Codegen→Runtime + Monaco集成 |
| 3.5 | Profile + 筹码分布 | Market/Volume Profile + 筹码分布估算+动画 |
| 3.6 | 组合分析+复盘 | 相关性/集中度/VaR + 复盘模板+归因+PDF导出 |
| 3.7 | 适老化UI+落地页 | 大字体/高对比度 + 官网落地页 |

> ✅ Phase 3 完成：产品可正式售卖

### Phase 4 — 稳定性与性能（v0.8.0）

| # | 任务 | 产出 |
|---|------|------|
| 4.1 | 防止闪退 | 移除 `panic="abort"`，WAL模式，错误传播替代 `.expect()` |
| 4.2 | 授权系统实施 | RSA离线授权+机器指纹+14天试用+DB持久化+PRO功能守卫 |
| 4.3 | 性能优化 | WAL+事务批量INSERT，日期窗口限制，rayon并行扫描，async+spawn_blocking |

> ✅ Phase 4 完成（2026-05-24）：v0.8.0 构建通过，零警告

### 依赖关系

```
wasm-core ─────────────────────→ (所有后续依赖此)
  ├── wasm-indicators(5)  ──→ wasm-indicators(90)  ──→ wasm-indicators(226)
  ├── chart-engine(基础)   ──→ chart-engine(完整)
  ├── Tauri壳+SQLite+IPC   ──→ 数据管理完整化
  └── (app路由)            ──→ 自选股+交易记录
                                  ├── RSA授权系统 ──→
                                  └── 打包分发 ──→
                                                      ├── wasm-scanner
                                                      ├── wasm-backtest
                                                      ├── wasm-custom(ME Script)
                                                      ├── wasm-profile+distribution
                                                      ├── 组合分析+复盘
                                                      └── 适老化UI+落地页
```

---

> **此文档是 `/goal` 指令实现产品的施工图纸。所有接口签名、目录结构、组件拆解均已经过确认，Phase 1→4 全部完成（v0.8.0），后续按实际需求迭代。**

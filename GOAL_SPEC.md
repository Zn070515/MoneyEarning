# GOAL_SPEC — 产品目标规格文档

> 版本 v0.1-draft | 2026-05-21 | 待逐步完善

---

## 一、产品愿景

一款**完全本地离线**的桌面量化分析工作站，面向中国大陆 35-55 岁中年股票投资者，提供从选股→技术分析→策略回测→交易复盘→组合管理的一站式工作流。定位一句话：

> "通达信级别的分析能力 + ThinkorSwim 级别的回测深度 + 你的数据永远留在你自己电脑上"

### 核心价值主张

| 维度 | 用户现状 | 产品提供 |
|------|---------|---------|
| 隐私 | 所有券商APP/同花顺/通达信都上传交易行为 | 100%本地存储，零网络依赖 |
| 深度 | 现有免费工具只有基础功能 | 300+指标、策略回测、Monte Carlo模拟 |
| 成本 | Level-2/投顾/VIP年费数百到数千元 | 一次性买断 ¥149-199 |
| 认知减负 | 5个APP间切换，信息过载 | 一站式闭环，AI辅助降噪而非增噪 |

---

## 二、目标用户

### 用户画像

- **年龄段**：35-55 岁男性为主
- **股龄**：1-3 年以上，已形成交易习惯
- **痛点**：用过同花顺/通达信但觉得功能不够深、不想交易行为被平台记录
- **行为模式**：每天收盘后复盘 1-2 小时，周末花半天选股回测
- **计算机水平**：会用 Excel、能下载安装软件，不会编程
- **付费意愿**：已在为 Level-2、投顾服务付费，愿为"隐私+深度功能"一次性付费

### 用户核心痛点（按全链路排序）

| # | 环节 | 痛点 |
|---|------|------|
| 1 | 选股 | 5000+只股票无从下手，热点追不上，筛选靠感觉 |
| 2 | 分析 | 工具碎片化，指标计算繁琐，指标间切换效率低 |
| 3 | 盯盘 | 上班族没时间盯盘，基础提醒维度单一 |
| 4 | 交易 | 情绪干扰导致止盈止损不纪律 |
| 5 | 复盘 | 复盘全靠手动，缺乏系统化框架 |
| 6 | 回测 | 编程门槛高、专业平台年费数万元 |
| 7 | 认知 | 信息越多决策越慢（AI时代的认知过载悖论） |

---

## 三、竞品格局与市场空白

### 国内主流软件

| 软件 | 定位 | 优势 | 劣势 |
|------|------|------|------|
| 通达信 | 技术分析专业工具 | 公式语言最强、指标最多、画线最专业 | 联网+账号制、无回测 |
| 同花顺 | 全能交易中枢 | AI选股、新手友好、交易对接 | 功能不够深、数据上传 |
| 大智慧 | 量化投资平台 | 5000+因子、极速回测、量化实盘 | 收费高、对散户门槛高 |

### 国际专业平台（功能对标来源）

| 平台 | 关键能力 |
|------|---------|
| ThinkorSwim | 400+指标、thinkScript脚本、Stock Hacker扫描、paperMoney模拟交易 |
| TradeStation (TITAN X) | EasyLanguage脚本、全链路自动化、AI助理 |
| MetaStock 20 | 300+指标、K线形态识别引擎、System Test回测 |
| Sierra Chart | Footprint订单流、Market Profile、C++级别定制 |

### 市场空白

> **没有任何一款产品做到：本地离线 + 专业级技术分析 + 策略回测 + 非编程用户可操作**

这是一个明确未被覆盖的交叉地带。

---

## 四、分发与商业模式

### 分发生命周期

```
知乎/雪球/公众号 内容引流
（股票分析干货文章，隐性植入产品价值）
         ↓
官网落地页（静态站点，Cloudflare Pages 免费部署）
         ↓
下载免费版安装包（~80MB）
         ↓
安装即赠送 14 天付费版全功能试用
         ↓
免费版日常使用 → 自然触及功能边界
         ↓
软件内付费引导 → 扫码支付 → 激活码
         ↓
输入激活码 → RSA离线验签解锁
```

### 定价策略

| 方案 | 价格 | 内容 |
|------|------|------|
| 免费版 | ¥0 | 日常使用够用，无限数据容量 |
| 付费版首年优惠 | ¥149 | 全功能解锁 |
| 付费版永久买断 | ¥199 | 全功能 + 终身升级 |

中国消费者厌恶订阅制，买断制更符合预期。"首年优惠"作为心理锚点。

### 收款方式

- 个人微信/支付宝收款码（初始阶段）
- V免签方案实现自动发货（后续升级）
- 建行储蓄卡转账（大额/批发）
- Cloudflare Workers 作为轻量支付回调代理（不存储敏感数据）

---

## 五、技术架构

### 选型决策

| 层 | 选择 | 理由 |
|---|------|------|
| **桌面框架** | Tauri 2.x (Rust + Web前端) | 打包体积 5-20MB，跨平台 Windows/macOS/Linux，Rust 核心天然防盗 |
| **核心计算** | Rust → WASM | 300+指标、回测、扫描引擎全在 WASM 沙箱执行，逆向难度极高 |
| **数据存储** | SQLite（本地） | 单文件数据库、零配置、完全私密 |
| **前端UI** | React + TypeScript + Canvas API | 高性能K线图自研渲染，对标 TradingView 交互 |
| **图表引擎** | 自研 Canvas + WebGL | 市面无现成库能满足专业 K线 + Volume Profile + Market Profile |
| **代码保护** | 四层防御 | 见下方 |
| **授权验证** | RSA-4096 离线签名 + 机器指纹绑定 | 纯本地验签，不联网 |
| **平台支持** | Windows（主力）+ macOS + Linux（暂缓） | Tauri 原生跨平台 |

### 代码保护四层防御

| 层级 | 技术 | 防护目标 |
|------|------|---------|
| **第1层** | Rust 编译为原生二进制 (.exe/.app) | 主逻辑不可直接阅读 |
| **第2层** | JS 前端 obfuscator.io 混淆 + 控制流平坦化 + 字符串加密 | JS 层不可读 |
| **第3层** | WASM 核心算法加密嵌入 | 计算引擎逆向难度 = 反编译二进制 |
| **第4层** | RSA-4096 离线授权 + 机器指纹（MAC+Hostname+OS序列号哈希）+ 域名绑定 | 激活码不可伪造、不可跨机使用 |

### 数据流架构

```
用户操作 (React UI)
       ↓ IPC 命令调用
Tauri Rust Backend（原生二进制）
       ↓
┌──────────────────────────────────────┐
│  Rust 原生层                          │
│  ├─ 数据导入 (通达信/同花顺/CSV兼容)  │
│  ├─ 文件I/O                           │
│  ├─ SQLite 读写                        │
│  ├─ 授权验证 (RSA验签 + 机器指纹)     │
│  └─ IPC 桥接 → WASM                   │
├──────────────────────────────────────┤
│  WASM 核心计算引擎（加密嵌入）         │
│  ├─ 技术指标计算 (300+ 指标)          │
│  ├─ K线形态识别 (61+ 形态)            │
│  ├─ 全市场智能选股扫描                │
│  ├─ 策略回测引擎 (向量化+事件驱动)    │
│  ├─ Monte Carlo 模拟                  │
│  ├─ 筹码分布估算                      │
│  └─ Market Profile / Volume Profile   │
└──────────────────────────────────────┘
```

---

## 六、功能矩阵（初步框架，待细化）

### 模块一：行情数据中枢

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 手动录入行情 | ✅ | ✅ |
| CSV/Excel 导入 | ✅ | ✅ |
| 通达信数据格式兼容导入 | ✅ | ✅ |
| 同花顺数据格式兼容导入 | ✅ | ✅ |
| 大智慧数据格式兼容导入 | ✅ | ✅ |
| 批量下载A股全市场历史数据 | ✅ 单次最多50只 | ✅ 无限制+定时自动更新 |
| 数据质量管理（完整性/异常/去重） | ✅ | ✅ |
| 自定义数据字段扩展 | ❌ | ✅ |
| 多数据源切换（支持不同券商导出格式） | ✅ | ✅ |
| PIT（Point-in-Time）数据查询 | ❌ | ✅ |
| 分钟级/逐笔数据导入 | ❌ | ✅ |

### 模块二：K线图表系统（自研Canvas引擎）

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 蜡烛图、OHLC、收盘价线 | ✅ | ✅ |
| Heikin-Ashi、Renko、点数图、卡吉图 | ✅ | ✅ |
| 1分钟→月线任意周期 | ✅ | ✅ |
| 多图表同时打开 | 最多4个 | 无限制 |
| 多屏支持 | ✅ | ✅ |
| 趋势线、水平线、通道线 | ✅ | ✅ |
| 斐波那契回调/扩展、黄金分割 | ✅ | ✅ |
| 江恩角度线、安德鲁音叉 | ❌ | ✅ |
| 自定义画线模板保存 | ❌ | ✅ |
| 多周期画线同步 | ❌ | ✅ |
| Volume Profile（成交量分布图） | ❌ | ✅ |
| Market Profile（市场剖面图/TPO） | ❌ | ✅ |
| 相对强度对比图（任意股票K线叠加） | ❌ | ✅ |
| 自定义图表配色/布局保存 | ✅ | ✅ |

### 模块三：技术指标引擎

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 内置常用指标 | 80个 | 300+个 |
| 参数自由调节 | ✅ | ✅ |
| 多指标叠加显示 | ✅ | ✅ |
| 自定义指标公式编辑器 | ❌ | ✅（TDX语法兼容 + 自有脚本） |
| 指标模板导入导出 | ❌ | ✅ |
| 分时图叠加技术指标 | ❌ | ✅ |
| 自定义指标社区/分享 | ❌ | ✅ |

### 模块四：选股扫描引擎

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 单条件筛选（20+预设条件） | ✅ | ✅ |
| 多条件组合筛选（AND/OR逻辑） | 最多5条件 | 30+条件嵌套逻辑 |
| 全市场一键扫描 | ❌ | ✅ |
| 排名打分系统（综合技术面+量价+形态） | ❌ | ✅ |
| 盘中实时预警（弹窗+声音） | ❌ | ✅ |
| 扫描结果导出 | ✅ | ✅ |
| 预警模板（MACD金叉、放量突破等） | ❌ | ✅ |

### 模块五：策略回测引擎

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 内置经典策略模板 | 5个 | 20+个 |
| 自编策略回测 | ❌ | ✅ |
| 回测报告（收益/回撤/夏普/胜率/盈亏比） | ❌ | ✅ |
| 参数优化（网格搜索） | ❌ | ✅ |
| 参数优化（遗传算法） | ❌ | ✅ |
| Walk-Forward 交叉验证 | ❌ | ✅ |
| Monte Carlo 模拟 | ❌ | ✅ |
| 资金曲线图 + 回撤曲线叠加 | ❌ | ✅ |
| 滑点/手续费/涨跌停限制模拟 | ❌ | ✅ |

### 模块六：复盘工具

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 交易记录录入（无限条） | ✅ | ✅ |
| 持仓成本自动计算（移动加权平均） | ✅ | ✅ |
| 盈亏统计面板 | ✅ | ✅ |
| 历史分时逐笔回放（可调速） | ❌ | ✅ |
| 训练模式（隐藏后续K线逐根判断） | ❌ | ✅ |
| 结构化复盘模板 | ❌ | ✅ |
| 情绪标签系统（冲动追高/理性建仓等） | ❌ | ✅ |
| 按策略/情绪/时段的盈亏归因分析 | ❌ | ✅ |
| 月度/季度/年度收益报告 | ❌ | ✅ |
| 复盘报告导出 PDF/图片 | ❌ | ✅ |

### 模块七：K线形态识别（61+形态）

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 基础形态识别（10种） | ✅ | ✅ |
| 完整61种形态自动识别标注 | ❌ | ✅ |
| 形态出现自动高亮+提醒 | ❌ | ✅ |
| 形态历史胜率回测统计 | ❌ | ✅ |

### 模块八：筹码分布分析

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 基础筹码分布图 | ✅ | ✅ |
| 历史筹码分布动画 | ❌ | ✅ |
| 筹码集中度分析 | ❌ | ✅ |
| 获利盘/套牢盘比例 | ❌ | ✅ |
| 成本均线 | ❌ | ✅ |

### 模块九：组合与风险分析

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 多账户/多策略并行管理 | ✅ 3个 | ✅ 无限制 |
| 组合收益率归因分析 | ❌ | ✅ |
| 相关性矩阵（跨持仓） | ❌ | ✅ |
| 行业集中度分析 | ❌ | ✅ |
| VaR（风险价值）估算 | ❌ | ✅ |
| 最大回撤归因分析 | ❌ | ✅ |
| 沙盘推演（"假如我在X日以Y价买入..."） | ❌ | ✅ |

### 模块十：数据导入导出与报表

| 功能 | 免费版 | 付费版 |
|------|:---:|:---:|
| 通达信/同花顺/大智慧格式导入 | ✅ | ✅ |
| CSV/XLS/JSON 导入导出 | ✅ | ✅ |
| 自定义报表 | ❌ | ✅ |
| PDF/图片导出 | ❌ | ✅ |
| 数据云备份（加密后手动上传） | ✅ | ✅ |

---

## 七、设计原则

1. **免费版不阉割数据**：所有用户数据库结构完全一致（SQLite），付费只解锁功能模块。免费用户升级无需数据迁移
2. **首装即送14天付费全功能试用**：体验完整产品后再决定
3. **免费版功能有诚意**：覆盖散户80%的日常操作——手动录入、80个指标、基础画线、5条件选股、交易记录、基础盈亏统计
4. **加密是默认行为**：所有本地数据AES-256加密存储
5. **离线是第一原则**：可联网下载行情（可选），但核心功能永远离线可用
6. **隐私零妥协**：不上传任何用户数据、无遥测、无埋点

---

## 八、参考项目与经验

### 用户量化平台 quant_platform V6

位于 `C:\Users\16275\Desktop\quant_platform`，关键资产：
- Python 3.11+，142 源文件，~21,700 行代码
- DuckDB + Parquet 混合存储
- 20 种策略（4单资产 + 16组合/ML驱动）
- 151/158 Alpha158 因子库 + DAG计算管线
- 4种原创搜索算法（CAPS / CGPC / MARS / MetaSearcher）
- 完整风险管理 + 保护插件体系
- Streamlit Web 仪表盘（Glassmorphism 暗色主题）
- 63 个测试文件覆盖

**可复用经验**：
- 策略/因子/风险分层架构设计
- 向量化回测 `position = signal.shift(1)` 防未来信息泄露
- Walk-Forward CV 参数稳定性验证
- 实验追踪系统（参数→指标→标签→对比）
- 止损保护插件可组合模式

---

## 九、WASM 计算引擎模块拆分

所有核心计算逻辑编译为 WASM，在浏览器沙箱中运行。模块划分如下：

```
wasm/
├── wasm-core/           # 核心基础设施
│   ├── dataframe.rs     # 内部 DataFrame 实现（类 pandas 接口）
│   ├── linalg.rs        # 线性代数（协方差矩阵、特征值分解等）
│   ├── stats.rs         # 统计函数库（分位数、正态分布、t检验等）
│   └── types.rs         # 共享类型定义
│
├── wasm-indicators/     # 技术指标计算引擎
│   ├── trend/           # 趋势类指标 (60+)
│   ├── momentum/        # 动量类指标 (50+)
│   ├── volatility/      # 波动类指标 (40+)
│   ├── volume/          # 成交量类指标 (30+)
│   ├── cycle/           # 周期类指标 (20+)
│   ├── composite/       # 复合指标 (30+)
│   └── custom/          # 自定义指标脚本解释器
│
├── wasm-pattern/        # K线形态识别引擎
│   ├── single_line.rs   # 单线形态 (10种)
│   ├── double_line.rs   # 双线形态 (15种)
│   ├── triple_line.rs   # 三线形态 (20种)
│   ├── multi_line.rs    # 多线复合形态 (16种)
│   └── confidence.rs    # 形态置信度评分
│
├── wasm-scanner/        # 选股扫描引擎
│   ├── condition.rs     # 条件表达式解析器
│   ├── executor.rs      # 并行扫描执行器
│   ├── ranking.rs       # 多因子打分排序
│   └── cache.rs         # 增量扫描结果缓存
│
├── wasm-backtest/       # 策略回测引擎
│   ├── vector_engine.rs # 向量化回测（核心：return = position.shift(1) * market_return）
│   ├── event_engine.rs  # 事件驱动回测
│   ├── metrics.rs       # 绩效指标计算
│   ├── optimizer.rs     # 参数优化（网格搜索 + 遗传算法 + Optuna贝叶斯）
│   ├── cross_val.rs     # Walk-Forward 交叉验证
│   └── monte_carlo.rs   # Monte Carlo 模拟
│
├── wasm-distribution/   # 筹码分布估算引擎
│   ├── estimate.rs      # 成本分布估计算法
│   ├── concentration.rs # 集中度/获利比例计算
│   └── history.rs       # 历史筹码动画数据
│
├── wasm-profile/        # Market/Volume Profile 引擎
│   ├── market_profile.rs # TPO/Market Profile 计算
│   ├── volume_profile.rs # Volume Profile 计算
│   └── poc.rs           # POC/VAH/VAL 关键价位
│
└── wasm-license/        # 授权验证模块
    ├── rsa_verify.rs    # RSA-4096 签名验证
    ├── fingerprint.rs   # 机器指纹采集与哈希
    └── anti_tamper.rs   # 完整性校验（运行时 checksum）
```

## 十、SQLite 数据库 Schema 设计（草案）

```sql
-- 股票基础信息表
CREATE TABLE stocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,         -- '000001.SZ'
    name TEXT NOT NULL,                 -- '平安银行'
    exchange TEXT NOT NULL,             -- 'SZ' | 'SH' | 'BJ'
    industry TEXT,                      -- 行业分类
    sector TEXT,                        -- 板块
    listing_date TEXT,                  -- 上市日期
    is_st BOOLEAN DEFAULT 0,           -- 是否ST
    created_at TEXT DEFAULT (datetime('now'))
);

-- 日线行情数据表
CREATE TABLE daily_prices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id),
    trade_date TEXT NOT NULL,           -- '2026-05-21'
    open REAL NOT NULL,
    high REAL NOT NULL,
    low REAL NOT NULL,
    close REAL NOT NULL,
    volume REAL NOT NULL,               -- 成交量（手）
    amount REAL NOT NULL,               -- 成交额（元）
    turnover_rate REAL,                 -- 换手率
    pre_close REAL,                     -- 前收盘
    change_pct REAL,                    -- 涨跌幅
    UNIQUE(stock_id, trade_date)
);

-- CREATE INDEX idx_daily_date ON daily_prices(trade_date);
-- CREATE INDEX idx_daily_stock ON daily_prices(stock_id);

-- 分钟线行情（可选，付费功能）
CREATE TABLE minute_prices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id),
    trade_time TEXT NOT NULL,           -- '2026-05-21 09:35:00'
    open REAL, high REAL, low REAL, close REAL,
    volume REAL, amount REAL,
    UNIQUE(stock_id, trade_time)
);

-- 交易记录表
CREATE TABLE trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id),
    direction TEXT NOT NULL,            -- 'buy' | 'sell'
    price REAL NOT NULL,
    quantity INTEGER NOT NULL,          -- 股数
    trade_date TEXT NOT NULL,
    trade_time TEXT,
    commission REAL DEFAULT 0,          -- 手续费
    stamp_tax REAL DEFAULT 0,           -- 印花税
    strategy_name TEXT,                 -- 关联策略名称
    emotion_tag TEXT,                   -- 情绪标签
    notes TEXT,                         -- 备注
    created_at TEXT DEFAULT (datetime('now'))
);

-- 策略定义表
CREATE TABLE strategies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    type TEXT NOT NULL,                 -- 'trend' | 'mean_reversion' | 'momentum' | 'ml' | 'composite'
    script TEXT,                        -- 策略脚本（自定义指标语法）
    params TEXT,                        -- JSON格式参数字典
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- 回测实验结果表
CREATE TABLE backtest_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    strategy_id INTEGER REFERENCES strategies(id),
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    initial_capital REAL NOT NULL,
    final_capital REAL,
    total_return REAL,
    annual_return REAL,
    max_drawdown REAL,
    sharpe_ratio REAL,
    win_rate REAL,
    profit_loss_ratio REAL,
    total_trades INTEGER,
    params_json TEXT,                   -- 本次回测使用的参数（JSON）
    metrics_json TEXT,                  -- 完整指标（JSON）
    equity_curve_json TEXT,             -- 资金曲线压缩数据
    created_at TEXT DEFAULT (datetime('now'))
);

-- 自选股/监控列表
CREATE TABLE watchlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                  -- 列表名称
    description TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE watchlist_items (
    watchlist_id INTEGER REFERENCES watchlists(id) ON DELETE CASCADE,
    stock_id INTEGER REFERENCES stocks(id),
    added_at TEXT DEFAULT (datetime('now')),
    PRIMARY KEY (watchlist_id, stock_id)
);

-- 用户设置/偏好
CREATE TABLE user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,                 -- JSON格式
    updated_at TEXT DEFAULT (datetime('now'))
);

-- 授权信息表（加密存储）
CREATE TABLE license_info (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- 单行表
    license_key TEXT NOT NULL,
    activation_date TEXT NOT NULL,
    expiry_date TEXT,                    -- NULL 表示永久
    machine_fingerprint TEXT NOT NULL,
    feature_flags TEXT NOT NULL,         -- JSON: {"pro": true, "expiry": null}
    signature TEXT NOT NULL              -- RSA签名，防篡改
);
```

## 十一、自研 K线图 Canvas 引擎技术方案

### 为什么自研
- ECharts/TradingView 的 K线图库不支持 Volume Profile、Market Profile 叠加
- 专业功能（筹码分布叠加、自定义画线持久化）需底层控制
- 体积可控，不依赖外部商业授权

### 分层架构

```
React 组件层 (KLineChart, IndicatorOverlay, DrawingTools)
       ↓
Canvas 渲染引擎 (自研)
       ↓
┌──────────────────────────────────┐
│  视口管理器 (ViewportManager)     │
│  ├─ 可视K线范围计算               │
│  ├─ 缩放/平移/惯性滚动            │
│  ├─ 十字光标追踪                  │
│  └─ 多图表视口联动                │
├──────────────────────────────────┤
│  图层系统 (LayerManager)          │
│  ├─ K线图层（蜡烛实体+影线）      │
│  ├─ 成交量图层（柱状+颜色映射）   │
│  ├─ 指标图层（线/柱/散点/带状）   │
│  ├─ 画线图层（SVG Overlay）       │
│  ├─ 形态标注图层                   │
│  ├─ 订单标记图层                   │
│  ├─ Volume Profile 图层            │
│  ├─ Market Profile (TPO) 图层      │
│  └─ 网格/坐标轴图层                │
├──────────────────────────────────┤
│  交互系统                          │
│  ├─ Tooltip（跟随十字光标）        │
│  ├─ 右键菜单                       │
│  ├─ 画线工具（拖拽起点→终点）     │
│  └─ 键盘快捷键                     │
├──────────────────────────────────┤
│  WebGL 加速层（大数据量场景）      │
│  └─ 全市场扫描结果可视化           │
└──────────────────────────────────┘
```

### 性能目标

| 场景 | 数据量 | 目标帧率 |
|------|--------|---------|
| 单股票日K，10年数据 | ~2500根K线 | 60fps（缩放/平移） |
| 全市场扫描结果散点图 | 5000+ 数据点 | 30fps（WebGL） |
| Volume Profile 计算+渲染 | 每根K线的成交分布 | 500ms 内完成 |

## 十二、授权系统完整设计

### 密钥体系

```
开发者端（你的电脑上，离线操作）
  ├─ 私钥 (RSA-4096) → 签署激活码，永不离开你的电脑
  └─ 公钥 → 嵌入软件分发版本

用户端（软件内WASM模块）
  └─ 公钥 → 验证激活码签名
```

### 激活码格式

```
ME-V1-AbCdEf123456-2026-05-21-PRO-<RSA签名>
 ─── ─── ────────── ────────── ─── ──────────
  │    │      │          │       │       │
  │    │      │          │       │     4096位签名(base64)
  │    │      │          │      PRO/STD
  │    │      │       激活日期
  │    │    机器指纹哈希(前12位)
  │  版本号
 前缀(产品标识)
```

### 验证流程

```
用户输入激活码
    ↓
WASM模块提取：机器指纹哈希 对比 激活码中的哈希 → 不匹配则拒绝
    ↓
WASM模块：公钥验签 激活码签名 → 签名无效则拒绝
    ↓
SQLite 写入 license_info 表
    ↓
运行时每次启动：checksum license_info → 被篡改则功能退化到免费版
```

### 防篡改机制

1. 激活信息表 `license_info` 整行RSA签名，任何字段被手工修改→验签失败
2. WASM二进制整体SHA256哈希嵌入Rust原生层，启动时比对
3. JS前端obfuscator.io开启 `selfDefending` 选项，检测格式化/调试即崩溃
4. DevTools检测（DevLock方案）：DevTools打开时触发全屏遮罩+DOM清空

### 激活码生成工具

一个独立的小工具（你自己用），输入：
- 用户提供的机器指纹
- 购买的版本（PRO/STD）
- 有效期（永久/年付）

输出一行激活码字符串，直接复制发给用户。

## 十三、待完成事项

- [ ] 功能矩阵每个细分项的用户故事（谁、在什么场景、解决什么问题）
- [ ] 每个模块的 UI/UX 线框图
- [ ] 技术指标 80/300+ 完整分类清单
- [ ] 20个策略模板详细说明
- [ ] 自定义指标脚本语言语法设计
- [ ] 打包/分发/自动更新方案
- [ ] 适老化UI设计（大字体模式、高对比度模式）
- [ ] 官网落地页设计
- [ ] 定价策略 A/B 测试方案

---

> **此文档为活文档，随设计推进逐步完善。所有决策记录于此，避免上下文丢失。**

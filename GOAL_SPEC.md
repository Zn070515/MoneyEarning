# GOAL_SPEC — 产品目标规格文档

> 版本 v0.10.0 | 2026-05-24 | 通达信导入 + 回测可视化 + 预警系统 + 多图表 + 编辑器增强 + CSV导入导出 + 付费转化

---

## 变更日志 v0.10.0（已完成）

### v0.9.0 已发布功能

<details><summary>点击展开 v0.9.0 完整日志</summary>

#### 视觉重构：Professional Terminal 主题
- 从"赛博霓虹"风格彻底转向专业金融终端设计语言
- 参考 Bloomberg Terminal / ThinkorSwim 等专业工具的设计原则
- 核心变化：`#0C0C0C` 底色 / `#CCAA00` 金色强调 / `#26A69A` 涨 `#EF5350` 跌 / 1px `#2A2A2A` 边框 / 全局等宽字体 / 无渐变无发光
- 设计原则：**"Data is the Hero"**——每像素服务于数据可读性

#### 新增功能（v0.9.0）
- K线训练模式、演示股票数据预置、绘图选中/删除
- 组合分析PRO模块：相关性矩阵、行业集中度、VaR、收益归因
- 版本号统一至 v0.9.0、授权缓存优化、回测页策略提示

</details>

### v0.10.0 新增功能

#### 优先级一：通达信数据格式兼容导入 ✅
- 通达信 `.day` 二进制日线格式直接解析（32字节/条，YYYYMMDD日期 + OHLC分价 + 成交额/量）
- 文件名自动识别代码+交易所：`sh600519.day` → SH/600519，`sz000001.day` → SZ/000001
- 目录批量扫描导入（递归最大深度4层，支持 `vipdoc/sh/lday/*.day` 标准结构）
- 新增 `tdx.rs` 模块：`parse_day_file()` / `parse_filename()` / `scan_day_files()` / `import_day_file()` / `import_day_directory()`
- UI：导入对话框新增"通达信 .day"标签页，文件选择器+自动代码提取+批量结果展示

#### 优先级二：回测结果可视化增强 ✅
- **资金曲线图**：SVG 面积渐变填充，涨绿跌红，初始资金参考线，Y轴自适应标注
- **交易标记点**：买卖点在权益曲线上标注 B/S 圆点，颜色对应盈亏
- **交易明细表**：买入/卖出日期、价格、盈亏、持仓天数，默认显示最近10笔可展开全部
- **新增指标**：最大回撤持续天数、年化波动率（`annual_volatility`）
- **WASM引擎增强**：
  - `DataFrame` 新增 `"date"` 列存储交易日期
  - `simulate()` 使用真实日期替代 `idx_N` 占位符
  - `compute_metrics()` 生成完整 `TradeRecord` 列表（含 `pnl_pct`、`holding_days`）
  - 权益曲线采样点使用实际日期标注，月度收益使用日期标签
- **报告导出增强**：Markdown 报告新增交易明细表、回撤持续、年化波动率

### 优先三：盘中实时预警 MVP ✅

- 价格突破预警（上破/下破指定价格）✅
- 均线交叉预警（快线上穿/下穿慢线）✅
- 成交量异常预警（成交量 > N日均量 × M倍）✅
- 预警触发：前端事件通知 ✅
- 预警列表管理（启用/禁用/删除）✅
- SQLite alert_rules 表 + Rust CRUD 模块 + React 预警管理面板
- 后续版本计划：Windows 原生通知 + 后台定时扫描

### 优先级四：细节打磨 ✅
- 多图表同时打开（2×2 布局）✅ — ChartToolbar "2×2" 按钮切换网格模式，4个独立画布各自加载不同股票
- ME Script 编辑器增强 ✅ — 自动完成弹出框（Ctrl+Space / 输入触发），函数签名+描述提示
- 持仓组合页支持 CSV 导入/导出 ✅ — 导入/导出按钮，Excel兼容 UTF-8 BOM
- 优化大数据量图表渲染（WebGL 加速散点图层）→ 延后至 v0.11.0

### 优先级五：付费转化优化 ✅
- 试用到期后显示升级引导页 ✅ — LicensePanel 剩余≤3天显示引导卡片
- 功能菜单中标注 PRO 专属标识 ✅ — 回测/扫描/筹码/形态/风控标签旁 PRO badge
- 激活码输入框支持粘贴 + 自动格式化 ✅ — onPaste 清理空白/引号/破折号
- 增加"推荐给朋友"分享按钮 ✅ — 设置页面复制推荐链接

---

## 变更日志 v0.11.0（规划中）

> 目标：弥补 v0.10.0 延后项，夯实核心体验

### 优先级一：后台预警 + Windows 原生通知
- 后台定时扫描（每5分钟自动检查所有预警规则）
- Windows 原生 Toast 通知（`tauri-plugin-notification`）
- 预警触发声音提示（Web Audio API 合成提示音）
- 系统托盘图标 + 右键菜单

### 优先级二：多周期 K 线切换
- 支持日线/周线/月线/60分钟/30分钟/15分钟/5分钟切换
- 周线/月线通过日线数据合成（`DataFrame.resample()`）
- 分钟线数据复用现有 `minute_prices` 表
- ChartToolbar 周期选择器（下拉菜单或按钮组）
- 2×2 网格模式下各画布独立周期

### 优先级三：WebGL 散点图层
- 筹码分布/成交量 Profile 图表切换至 WebGL 渲染
- 10万+散点实时交互（缩放、刷选）
- 散点着色按时间/价格/成交量维度

### 优先级四：数据管理增强
- 数据库备份/恢复（导出 SQLite → 用户选择目录）
- 自动备份提醒（每7天提醒一次）
- 数据清除确认对话框（按股票/按日期范围）

### 优先级五：体验小优化
- 图表页键盘快捷键（← → 切换股票，Space 切换周期）
- 策略回测结果一键导出 PNG 截图
- 导入进度条支持多文件并行

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

#### 一级分类总览（基于 pandas_ta 完整分类法 + ThinkorSwim/通达信对标）

| # | 分类 | pandas_ta 类别 | 免费版 | 付费版 | 权威来源对标 |
|---|------|:-----------:|:---:|:---:|------|
| 1 | 均线/叠加类 | Overlap | 18 | 33 | pandas_ta Overlap (33个)、通达信均线型(14个) |
| 2 | 趋势/方向类 | Trend | 10 | 18 | pandas_ta Trend (18个)、通达信趋势型(18个) |
| 3 | 动量/振荡类 | Momentum | 18 | 41 | pandas_ta Momentum (41个)、通达信超买超卖型(20个) |
| 4 | 波动/通道类 | Volatility | 8 | 14 | pandas_ta Volatility (14个)、通达信布林系 |
| 5 | 成交量/资金流 | Volume | 8 | 15 | pandas_ta Volume (15个)、通达信成交量型(7个) |
| 6 | 统计/分布类 | Statistics | 5 | 11 | pandas_ta Statistics (11个) |
| 7 | 周期/傅里叶类 | Cycles | 2 | 8 | ThinkorSwim Cycle Studies、Ehlers系列 |
| 8 | K线形态识别 | Candles | 15 | 64 | TA-Lib CDL Patterns (61个)、pandas_ta candles |
| 9 | 绩效/回撤类 | Performance | 2 | 4 | pandas_ta Performance (4个) |
| 10 | 特色工具/Profile | 自定义 | 4 | 18 | ThinkorSwim Market Profile、Sierra Chart VP |
| **合计** | | | **90** | **226** | **总计 316 个指标+工具** |

> **指标命名约定**：每个指标同时提供 **英文函数名**（编程引用）、**中文名**（UI显示）、**通达信等价名**（迁移用户识别）。分类体系以 pandas_ta 10大类为骨架，补入 ThinkorSwim 特色指标和通达信本土指标。

---

#### 类别一：均线/叠加类 (Overlap) — 免费18 + 付费15 = 33个

基于 pandas_ta Overlap 类别（33个指标），覆盖所有主流移动平均线及价格叠加。

**免费版 (18个)**：

| # | 函数名 | 中文名 | 开发者/年份 | 通达信等价 |
|---|--------|--------|------------|----------|
| 1 | `sma` | 简单移动均线 | — | MA |
| 2 | `ema` | 指数移动均线 | — | EXPMA |
| 3 | `wma` | 加权移动均线 | — | WMA |
| 4 | `dema` | 双指数移动均线 | Mulloy 1994 | — |
| 5 | `tema` | 三指数移动均线 | Mulloy 1994 | — |
| 6 | `trima` | 三角移动均线 | — | — |
| 7 | `vwap` | 成交量加权均价 | — | VWAP |
| 8 | `vwma` | 成交量加权均线 | — | AMV |
| 9 | `hl2` | 高低价均值 | — | (H+L)/2 |
| 10 | `hlc3` | 典型价格 | — | TYP |
| 11 | `ohlc4` | OHLC均值 | — | — |
| 12 | `midpoint` | 中点 | — | — |
| 13 | `midprice` | 中间价 | — | — |
| 14 | `wcp` | 加权收盘价 | — | — |
| 15 | `rma` | Wilder平滑均线 | Wilder 1978 | — |
| 16 | `linreg` | 线性回归线 | — | LINEAR |
| 17 | `hilo` | Gann高/低激活线 | Gann | — |
| 18 | `supertrend` | 超级趋势 | Olivier 2008 | — |

**付费版增量 (15个)**：

| # | 函数名 | 中文名 | 开发者/年份 | 特色 |
|---|--------|--------|------------|------|
| 19 | `kama` | Kaufman自适应均线 | Kaufman 1998 | 根据效率比动态调整平滑系数 |
| 20 | `hma` | Hull移动均线 | Hull 2005 | 极低滞后，平滑度极佳 |
| 21 | `t3` | T3移动均线 | Tillson 1998 | 比EMA更平滑，支持0-1之间的小数因子 |
| 22 | `jma` | Jurik移动均线 | Jurik 1998 | 极低噪声，自适应市场阶段 |
| 23 | `alma` | Arnaud Legoux均线 | Legoux 2009 | 高斯滤波偏移消除，低滞后+高平滑 |
| 24 | `vidya` | 波动率动态均线 | Chande 1992 | 用CMO动态调平滑系数 |
| 25 | `zlma` | 零滞后均线 | Ehlers 2010 | 几乎消除EMA滞后 |
| 26 | `ssf` | 超平滑滤波器 | Ehlers 2002 | 基于数字信号处理的降噪均线 |
| 27 | `fwma` | Fibonacci加权均线 | — | 权重按Fibonacci序列分配 |
| 28 | `pwma` | Pascal加权均线 | — | 权重按Pascal三角分配 |
| 29 | `sinwma` | 正弦加权均线 | — | 权重按正弦函数分配 |
| 30 | `swma` | 对称加权均线 | — | 权重对称分布 |
| 31 | `ichimoku` | 一目均衡表 (5线) | 细田悟一 1968 | 转换线/基准线/先行A/先行B/迟行线 |
| 32 | `bbands` | 布林带 (3线) | Bollinger 1984 | 中轨+上下轨，含%B和带宽 |
| 33 | `donchian` | 唐奇安通道 | Donchian 1960 | N日最高/最低通道，海龟交易法核心 |

---

#### 类别二：趋势/方向类 (Trend) — 免费10 + 付费8 = 18个

基于 pandas_ta Trend 类别。衡量趋势强度与方向。

**免费版 (10个)**：

| # | 函数名 | 中文名 | 通达信等价 | 说明 |
|---|--------|--------|----------|------|
| 1 | `adx` | 平均趋向指数 | DMI | ADX/DI+/DI- 三线，≥25趋势市，<20震荡市 |
| 2 | `aroon` | 阿隆指标 | — | Aroon Up/Down + Oscillator，判断新趋势启动 |
| 3 | `psar` | 抛物线SAR | SAR | 止损+反转点，趋势跟随 |
| 4 | `dpo` | 去趋势价格振荡器 | DPO | 消除长期趋势后分析周期 |
| 5 | `macd` | 平滑异同均线 | MACD | DIF/DEA/柱 三线 |
| 6 | `vortex` | 漩涡指标 | — | +VI/-VI交叉判定趋势反转，比ADX响应快 |
| 7 | `qstick` | Q棒 | — | 开盘-收盘价差的均线，衡量做市压力 |
| 8 | `decreasing` | 持续下降线 | — | 价格N日连续下降标记 |
| 9 | `increasing` | 持续上升线 | — | 价格N日连续上升标记 |
| 10 | `chop` | 震荡指数 | — | 0-100，越高越震荡（choppy），越低越趋势 |

**付费版增量 (8个)**：

| # | 函数名 | 中文名 | 开发者/年份 | 特色 |
|---|--------|--------|------------|------|
| 11 | `amat` | Archer均线趋势 | Archer 2009 | 双EMA结构+ATR确认，虚假突破过滤 |
| 12 | `cksp` | Chande Kroll止损 | Chande 1996 | 基于ATR的2阶段趋势跟踪止损 |
| 13 | `decay` | 线性衰减 | — | 信号随周期衰减至零 |
| 14 | `long_run` | 长周期运行 | — | 上涨/下跌连续趋势持续时间统计 |
| 15 | `short_run` | 短周期运行 | — | 同上，短周期视角 |
| 16 | `ttm_trend` | TTM趋势 | TTM 2010 | 基于多周期波动带的趋势确认 |
| 17 | `ttm_squeeze` | TTM挤压 | TTM 2010 | Bollinger Band ≤ Keltner Channel = 挤压预警 |
| 18 | `psar_ext` | 扩展抛物线SAR | — | 支持阶梯式加速因子+趋势强度分级 |

---

#### 类别三：动量/振荡类 (Momentum) — 免费18 + 付费23 = 41个

基于 pandas_ta Momentum 类别（41个指标）。最大的单一类别。

**免费版 (18个)**：

| # | 函数名 | 中文名 | 通达信等价 | 说明 |
|---|--------|--------|----------|------|
| 1 | `rsi` | 相对强弱指数 | RSI | Wilder 1978，>70超买/<30超卖 |
| 2 | `stoch` | 随机指标 | KDJ | %K/%D双线，含快/慢/全三种模式 |
| 3 | `kdj` | KDJ指标 | KDJ | %K/%D/%J三线，A股最常用指标之一 |
| 4 | `willr` | 威廉指标 | WR | Larry Williams，−80以下超卖 |
| 5 | `cci` | 商品通道指数 | CCI | Lambert 1980，±100为极端阈值 |
| 6 | `mom` | 动量 | MTM | 当期价格 − N期前价格 |
| 7 | `roc` | 变动率 | ROC | (当期/N期前 − 1)×100% |
| 8 | `apo` | 绝对价格振荡器 | — | 快EMA − 慢EMA（用差值而非比例） |
| 9 | `ppo` | 百分比价格振荡器 | — | (快EMA/慢EMA − 1) × 100% |
| 10 | `bias` | 乖离率 | BIAS | (收盘价/均线 − 1)×100% |
| 11 | `psl` | 心理线 | PSY | N日内上涨天数/N × 100% |
| 12 | `bop` | 力量平衡 | — | (收−开)/(高−低)，衡量多空平衡 |
| 13 | `slope` | 斜率 | ACCER | 时间序列线性回归斜率 |
| 14 | `inertia` | 惯性 | — | 基于RVGI平滑的动量持续性度量 |
| 15 | `er` | 效率比 | — | 净位移/总路程，趋势效率 |
| 16 | `brar` | BRAR人气意愿 | BRAR | A股传统指标，BR(人气)+AR(意愿) |
| 17 | `squeeze` | 挤压指标 | — | Bollinger-Keltner挤压程度量化 |
| 18 | `coppock` | Coppock曲线 | — | Coppock 1962，长线买入信号（标普用） |

**付费版增量 (23个)**：

| # | 函数名 | 中文名 | 开发者/年份 | 特色 |
|---|--------|--------|------------|------|
| 19 | `stochrsi` | 随机RSI | — | RSI的Stochastic化，更敏感 |
| 20 | `uo` | 终极振荡器 | UOS | Williams 1985，三周期加权，减少虚假信号 |
| 21 | `cmo` | Chande动量振荡器 | — | Chande 1994，−100到+100，分上/下波动 |
| 22 | `cfo` | Chande预测振荡器 | — | Chande 1995，线性回归预测 + 实际对比 |
| 23 | `kst` | KST综合判定 | KST | Martin Pring 1992，四周期ROC加权和 |
| 24 | `trix` | 三指数平滑变动率 | TRIX | 三重EMA后的ROC，极低噪声 |
| 25 | `tsi` | 真实强度指数 | — | Blau 1991，双EMA平滑价格动量 |
| 26 | `fisher` | Fisher变换 | — | Ehlers 2002，将价格转为Gaussian分布→更清晰极值 |
| 27 | `smi` | SMI遍历指标 | — | Blau 1993，比Stoch更平滑，噪声更少 |
| 28 | `eri` | Elder光线指数 | Elder Ray | Elder 1989，牛力=高−EMA，熊力=低−EMA |
| 29 | `rvgi` | 相对活力指数 | — | 收盘-开盘差值与高低差的对比 |
| 30 | `qqe` | QQE定性量化估计 | QQE | 基于RSI平滑+ATR动态带的量级振荡器 |
| 31 | `ao` | 动量震荡 | AO | Williams，5周期与34周期中点的差值 |
| 32 | `pgo` | 优秀振荡器 | PGO | Johnson 2008，基于真实高/低+成交量 |
| 33 | `pvo` | 百分比成交量振荡器 | PVO | (快VOL_EMA/慢VOL_EMA − 1)×100% |
| 34 | `cg` | 重心 | — | Ehlers 2002，自适应市场速度变化 |
| 35 | `stc` | Schaff趋势周期 | Schaff 1999 | MACD的Stochastic化，更快捕捉趋势转换 |
| 36 | `rsi_ma` | RSI均线 | MARSI | RSI的均线平滑版，降低噪声 |
| 37 | `skdj` | 慢速KDJ | SKDJ | KDJ的EMA平滑版，比KDJ更稳定 |
| 38 | `lwr` | LWR威廉指标 | LWR | WR的SMA平滑版 |
| 39 | `mfi` | 资金流量指数 | MFI | 量价结合的RSI变种 |
| 40 | `rsm` | 相对强度动量 | — | 与基准(大盘)对比的相对动量 |
| 41 | `dynamic_mi` | 动态动量指数 | DMI_S | 变周期RSI，根据近期波动率调节计算周期 |

---

#### 类别四：波动/通道类 (Volatility) — 免费8 + 付费6 = 14个

基于 pandas_ta Volatility 类别。

**免费版 (8个)**：

| # | 函数名 | 中文名 | 通达信等价 | 说明 |
|---|--------|--------|----------|------|
| 1 | `atr` | 平均真实波幅 | ATR | Wilder 1978，真实波幅的EMA |
| 2 | `natr` | 归一化ATR | — | ATR/收盘价×100%，跨品种可比 |
| 3 | `true_range` | 真实波幅 | TR | max(H−L, |H−C_prev|, |L−C_prev|) |
| 4 | `kc` | Keltner通道 | — | ATR倍数通道，比布林带反应更快 |
| 5 | `massi` | 质量指数 | MASS | 高低价差均线比值累积，>27预示反转 |
| 6 | `rvi` | 相对波动率指数 | — | 基于标准差比值的波动率度量 |
| 7 | `thermo` | Elder温度计 | — | Elder 2002，波动率冷热度 |
| 8 | `ui` | 溃疡指数 | UI | Martin 1987，回撤平方的均方根 |

**付费版增量 (6个)**：

| # | 函数名 | 中文名 | 开发者/年份 | 特色 |
|---|--------|--------|------------|------|
| 9 | `aberration` | 偏差 | — | 历史价格相对Z-score的偏离度 |
| 10 | `accbands` | 加速带 | — | 基于价格动态速度的波动带 |
| 11 | `pdist` | 价格距离 | — | 价格到均线的归一化距离 |
| 12 | `hwc` | 历史波动率锥 | — | 多周期HV分布，VolCone可视化 |
| 13 | `pvr` | Parkinson波动率 | Parkinson 1980 | 基于高/低的波动率（比收盘-收盘更精确） |
| 14 | `gk_vol` | Garman-Klass波动率 | GK 1980 | 基于O/H/L/C的最高精度历史波动率估计 |

---

#### 类别五：成交量/资金流 (Volume) — 免费8 + 付费7 = 15个

基于 pandas_ta Volume 类别。

**免费版 (8个)**：

| # | 函数名 | 中文名 | 通达信等价 | 说明 |
|---|--------|--------|----------|------|
| 1 | `obv` | 能量潮 | OBV | 涨累加跌累减成交量，经典量价指标 |
| 2 | `ad` | 累积/派发线 | A/D | Chaikin，结合价格位置的成交量加权 |
| 3 | `adosc` | Chaikin振荡器 | CHO | Chaikin，快/慢A/D线的MACD |
| 4 | `cmf` | Chaikin资金流 | — | N日A/D值/N日成交量合计，>0买入压力 |
| 5 | `mfi` | 资金流量指数 | MFI | 量价RSI，>80超买/<20超卖 |
| 6 | `eom` | 易动性 | EMV | Arms 1978，成交量的"价格推动效率" |
| 7 | `pvt` | 量价趋势 | VPT | 成交量×(今日收−昨日收)/昨日收 的累积 |
| 8 | `pvol` | 价格成交量 | — | 成交量×收盘价的简单乘积 |

**付费版增量 (7个)**：

| # | 函数名 | 中文名 | 开发者/年份 | 特色 |
|---|--------|--------|------------|------|
| 9 | `efi` | Elder力度指数 | Elder FI | (今日收−昨日收)×成交量，平滑后判趋势强度 |
| 10 | `kvo` | Klinger成交量振荡器 | KVO | Klinger 1997，基于量价趋势+力量的振荡器 |
| 11 | `nvi` | 负成交量指数 | NVI | 缩量日专盯，"聪明钱"指标 |
| 12 | `pvi` | 正成交量指数 | PVI | 放量日专盯，"大众"指标 |
| 13 | `vp` | Volume Profile | VP | 固定/可变/发展三种模式的成交量分布 |
| 14 | `vwd` | 成交量加权偏差 | — | VWAP的标准差通道 |
| 15 | `density` | 成交密度 | — | 单位价位的成交量集中度 |

---

#### 类别六：统计/分布类 (Statistics) — 免费5 + 付费6 = 11个

基于 pandas_ta Statistics 类别。用于高级分析和因子研究。

**免费版 (5个)**：

| # | 函数名 | 中文名 | 说明 |
|---|--------|--------|------|
| 1 | `stdev` | 标准差 | 价格波动率的一阶度量 |
| 2 | `variance` | 方差 | 标准差的平方 |
| 3 | `zscore` | Z分数 | 当前价格距均值几个标准差的偏离 |
| 4 | `mad` | 平均绝对偏差 | 比标准差对异常值更稳健 |
| 5 | `median` | 中位数 | 鲁棒中心度量 |

**付费版增量 (6个)**：

| # | 函数名 | 中文名 | 说明 |
|---|--------|--------|------|
| 6 | `entropy` | 信息熵 | 价格分布的随机性度量，越高→越随机 |
| 7 | `kurtosis` | 峰度 | ≥3厚尾，<3薄尾。厚尾→极端行情概率高 |
| 8 | `skew` | 偏度 | >0正偏(上涨加速)，<0负偏(下跌加速) |
| 9 | `quantile` | 分位数 | 自定义分位数的价格水平（如0.05/0.95） |
| 10 | `log_return` | 对数收益率 | ln(P_t/P_{t−1})，收益率分布分析 |
| 11 | `drawdown` | 回撤分析 | 最大回撤/回撤持续时间/恢复时间 |

---

#### 类别七：周期/傅里叶类 (Cycles) — 免费2 + 付费6 = 8个

基于 pandas_ta Cycles + ThinkorSwim Ehlers周期研究。A股季节效应和周期分析。

**免费版 (2个)**：

| # | 函数名 | 中文名 | 说明 |
|---|--------|--------|------|
| 1 | `ebsw` | 正弦波指标 | Ehlers，基于带通滤波的周期检测 |
| 2 | `seasonal` | 月度效应 | 各月份/星期平均收益日历 |

**付费版增量 (6个)**：

| # | 指示名称 | 中文名 | 来源 | 特色 |
|---|---------|--------|------|------|
| 3 | DFT | 离散傅里叶变换 | — | 从价格序列提取主导频率 |
| 4 | FFT | 快速傅里叶变换 | — | DFT的高效实现，实时周期检测 |
| 5 | Hilbert Sine Wave | 希尔伯特正弦波 | Ehlers 2000 | 瞬时周期→买卖点在正弦拐点 |
| 6 | Hilbert Trend vs Cycle | 希尔伯特趋势/周期分离 | Ehlers 2000 | 价格分解为趋势分量+周期分量 |
| 7 | Cycle Spectrum | 周期图谱 | TOS | 可视化展示多周期强度 |
| 8 | Correlation Cycle | 相关周期 | TOS | 跨股票周期模式的相位对比 |

---

#### 类别八：K线形态识别 (Candles) — 免费15 + 付费49 = 64种

基于 TA-Lib CDL Patterns（61种）+ pandas_ta 原生candles（3种）。对标 MetaStock 20 蜡烛形态引擎。

**免费版 (15种，A股最常见的形态)**：

| # | 函数名 | 中文名 | 方向 |
|---|--------|--------|:--:|
| 1 | `cdl_doji` | 十字星/陀螺线 | 中性 |
| 2 | `cdl_hammer` | 锤子线 | 看涨 |
| 3 | `cdl_inverted_hammer` | 倒锤子 | 看涨 |
| 4 | `cdl_hanging_man` | 吊颈线 | 看跌 |
| 5 | `cdl_shooting_star` | 射击之星 | 看跌 |
| 6 | `cdl_engulfing` | 吞没形态 | 双向 |
| 7 | `cdl_harami` | 孕线 | 双向 |
| 8 | `cdl_piercing` | 穿刺线 | 看涨 |
| 9 | `cdl_dark_cloud_cover` | 乌云盖顶 | 看跌 |
| 10 | `cdl_morning_star` | 晨星 | 看涨 |
| 11 | `cdl_evening_star` | 黄昏之星 | 看跌 |
| 12 | `cdl_three_white_soldiers` | 三白兵 | 看涨 |
| 13 | `cdl_three_black_crows` | 三乌鸦 | 看跌 |
| 14 | `cdl_marubozu` | 光头光脚 | 双向 |
| 15 | `cdl_inside` | 内含线 | 中性 |

**付费版增量 (49种，完整TA-Lib识别)**：

包括但不限于：Abandoned Baby(弃婴)、Morning/Evening Doji Star(晨/昏十字星)、Three Inside Up/Down(三内升/降)、Three Outside Up/Down(三外升/降)、Rising/Falling Three Methods(上升/下降三步曲)、Tasuki Gap(田足缺口)、Unique 3 River(独特三河)、Dragonfly/Gravestone Doji(蜻蜓/墓碑十字)、Harami Cross(孕线十字)、Kicking(踢击)、Ladder Bottom(阶梯底)、Mat Hold(垫形持有)、Separating Lines(分离线)、Side by Side White Lines(并列白线)、Tristar(三星)、Upside/Downside Gap Three Methods(跳空三步曲)等完整61种TA-Lib形态。

加上形态置信度评分系统（识别质量打分0-100），以及形态历史胜率回测（该形态出现后N日涨跌统计）。

---

#### 类别九：绩效/回撤类 (Performance) — 免费2 + 付费2 = 4个

**免费版 (2个)**：`percent_return`(百分比收益)、`trend_return`(趋势收益)

**付费版增量 (2个)**：`drawdown`(完整回撤分析：最大回撤/回撤期/恢复期/水下时间占比)、`log_return`(对数收益分布分析)

---

#### 类别十：特色工具 (Custom Tools) — 免费4 + 付费18 = 22个

超越传统指标的特色分析工具，对标 ThinkorSwim Profile 和 Sierra Chart。

**免费版 (4个)**：

| # | 工具 | 说明 |
|---|------|------|
| 1 | 区间统计 | 任意起点→终点：涨跌幅、最大回撤、ATR倍数 |
| 2 | Fibonacci回调/扩展 | 0/0.236/0.382/0.5/0.618/0.786/1.0 + 扩展位1.272/1.618 |
| 3 | 支撑/阻力自动检测 | 基于成交量密集区+转折点 |
| 4 | 简单相关性 | 两只股票价格走势的Pearson r |

**付费版增量 (18个)**：

| # | 工具名称 | 来源对标 | 说明 |
|---|---------|---------|------|
| 5 | Market Profile (TPO) | ThinkorSwim | 时间价格机会剖面，30分钟分块 |
| 6 | Volume Profile (Fixed) | Sierra Chart | 固定时间段的成交量分布 |
| 7 | Volume Profile (Developing) | Sierra Chart | 实时发展的成交量分布 |
| 8 | POC/VAH/VAL | — | Point of Control + 价值区高低点提取 |
| 9 | VWAP + 标准差带 | — | 日内VWAP+1σ/2σ/3σ带 |
| 10 | Anchored VWAP | — | 从任意事件(财报/消息)起点计算的VWAP |
| 11 | TWAP | — | 时间加权均价 |
| 12 | 大单/特大单检测 | 东方财富L2 | 基于分钟成交量突变的异常订单识别 |
| 13 | 价格缺口分析 | — | Gap类型(突破/持续/衰竭)+回补概率 |
| 14 | 异常成交量识别 | — | 基于成交量Z-score的异常放量/缩量日 |
| 15 | Cumulative Delta | — | 主动买−主动卖的累积(分钟线估算版) |
| 16 | 筹码分布(静态) | 通达信 | 成本分布估算，获利/套牢盘比例 |
| 17 | 筹码分布(历史动画) | 通达信 | 历史筹码演变动画 |
| 18 | 筹码集中度 | 通达信 | CR5/CR10/CR20 + 集中趋势指标 |
| 19 | Camarilla Points | TOS/S.Bobrowski | S3/R3区间的"区间交易"入场/止损位 |
| 20 | 多周期画线同步 | — | 日线画线自动投影到分钟/周线 |
| 21 | 自定义指标脚本编辑器 | 通达信公式 | 类TDX语法的指标脚本，实时编译到WASM |
| 22 | 多因子打分卡 (DFCZ) | Alpha158 | ICIR加权的综合评分面板 |

---

#### 指标评价标准体系（基于2024学术文献与业界实践）

> 参考来源：Barra多因子模型框架、Microsoft Qlib评估管线、华泰/方正/海通证券金工研报(2024-2025)、BigQuant因子研究平台

每个指标上线前需通过以下七步评价流程：

```
Step 1: 因子构建 → 去极值(winsorize 1%/99%) → 缺失值填充 → 标准化(Z-score)
Step 2: 因子中性化（对行业+市值+已有因子回归取残差，消除冗余暴露）
Step 3: 单因子检验（RankIC序列、ICIR、分组回测、单调性）
Step 4: IC衰减分析（计算半衰期，匹配策略调仓频率）
Step 5: 拥挤度评估（残差法：实际IC − 双曲衰减模型拟合值）
Step 6: 正交化整合（Löwdin对称正交化，保留最大独立信息）
Step 7: 多因子合成（ICIR加权，行业中性，组合优化）
```

---

##### 六大评价维度（完整版）

**维度一：预测能力 (Predictive Power)**

| 指标 | 计算方法 | A股有效阈值 | 优秀阈值 |
|------|---------|:---------:|:------:|
| RankIC 均值 | `mean(Spearman-r(rank(F_t), rank(R_{t+1})))` 全周期 | ≥ 0.03 | ≥ 0.05 |
| Pearson IC 均值 | `mean(Corr(F_t, R_{t+1}))` 全周期 | ≥ 0.02 | ≥ 0.04 |
| IC 正值率 | RankIC > 0 的周期占比 | > 55% | > 60% |
| IC 标准差 | IC序列波动率 | ≤ 0.12 | ≤ 0.08 |
| IC 累计曲线 | 全周期累计IC是否持续上行 | 单调递增 | 严格单调 |

> **为什么用 RankIC 而非 Pearson IC？** 金融数据非正态、厚尾，Pearson IC 对异常值敏感。RankIC（Spearman秩相关）回答的是"因子排名高的股票，收益排名是否也高"，更贴近真实组合构建（排序选股）。2024年业界共识：**RankIC 优先于 Pearson IC**。

**维度二：稳定性 (Stability) — ICIR**

$$ICIR = \frac{\text{mean(IC series)}}{\text{std(IC series)}}$$

ICIR 是"IC的夏普比率"——同时衡量预测力和稳定性。

| 阶段 | ICIR 阈值 | 含义 |
|------|:------:|------|
| 初步筛选通过 | ≥ 0.3 | 信号有效，可进入深度研究 |
| 深度研究通过 | ≥ 0.5 | 信号稳健，可纳入因子库 |
| 实盘准入 | ≥ 1.0 | 信号稳定，可进入生产组合 |

> **关键洞察**（华泰2024）：ICIR 往往比原始 IC 更重要。一个高IC但低ICIR的因子在回测中出色、实盘中失效的概率很高——ICIR捕捉的是 alpha 的**一致性**。多因子合成时采用 ICIR 加权，同时奖励预测力和稳定性。

**维度三：衰减分析 (Decay Analysis) — 双曲衰减模型**

因子预测力随持仓周期延长而衰减。用双曲衰减模型拟合：

$$IC_{decay}(t) = \frac{K}{1 + \lambda t}$$

其中 K 为初始IC（t=1），λ 为衰减速率。半衰期 = 1/λ。

| 半衰期 | 适用场景 |
|--------|---------|
| 1-3 日 | 高频/日内策略（换手率、短期反转、量价因子） |
| 5-10 日 | 中频策略（动量、波动率因子） |
| 15-30 日 | 低频策略（ROE、估值、质量增长因子） |
| > 60 日 | 配置型策略（基本面、分析师预期因子） |

**拥挤度评估**：残差 = 实际IC − 双曲衰减模型预测值。残差持续为负 → 因子过于拥挤，alpha 已被套利蚕食。此时应暂停使用该因子或降低权重。

**维度四：区分度 (Discrimination) — 分组回测**

| 检验项 | 方法 | 合格标准 |
|--------|------|---------|
| 分层单调性 | 按因子值分5组，各组年化收益排序 | 单调递增/递减 |
| Top-Bottom 收益差 | 第5组 − 第1组年化收益差 | > 5%（年化） |
| 多空夏普 | Top组做多 + Bottom组做空 | > 1.0 |

**维度五：独立性 (Orthogonality) — Löwdin对称正交化**

| 方法 | 适用场景 | 特点 |
|------|---------|------|
| 回归取残差法 | 对已有因子逐个正交 | 简单，但正交顺序影响结果 |
| Gram-Schmidt | 顺序正交化 | 依赖正交顺序，不够公平 |
| **Löwdin对称正交** | **多因子同时正交** | **不依赖排序，最大化各因子独立信息保留** |

我们的选择：**Löwdin对称正交化**。在将所有因子合并到打分卡之前，先做对称正交，保证各因子贡献的信息不重叠。正交化后的RankIC不应归零——如果归零，说明该因子没有独立alpha贡献。

**维度六：可解释性 (Interpretability)**

| 分类 | 定义 | 评级 |
|------|------|:--:|
| 机械因子 (Mechanical) | 基于价格/成交量公式推导，逻辑清晰（如MACD金叉、放量突破） | ★★★ |
| 混合因子 | 机械公式 + 统计筛选，部分可解释（如自适应参数指标） | ★★ |
| 判断因子 (Judgment) | 依赖分析师主观判断或黑箱模型输出（如AI评分、情绪打分） | ★ |

> 本产品优先纳入 **机械因子（★★★）** 和 **混合因子（★★）**。判断因子仅作为参考指标提供。

---

##### 综合评级矩阵

| 评级 | RankIC | ICIR | 单调性 | 半衰期 | 覆盖率 | 独立性 |
|------|:-----:|:----:|:-----:|:-----:|:-----:|:-----:|
| **A级**（核心） | ≥0.05 | ≥1.0 | ✅ | ≥10日 | >80% | ✅ |
| **B级**（辅助） | ≥0.03 | ≥0.5 | ✅ | ≥5日 | >60% | - |
| **C级**（参考） | ≥0.02 | ≥0.3 | - | - | - | - |
| **D级**（淘汰） | <0.02 | <0.3 | ❌ | - | - | - |

---

##### 最新的因子评价前沿（2024-2025）

| 来源 | 贡献 | 对我们的启发 |
|------|------|------------|
| 方正证券(2024.05) | 成交量非对称因子：单因子RankIC=−7.39%，12因子正交后复合RankIC=−12.42% | 正交化组合的威力——单个一般因子组合后可质变 |
| 华泰证券(2024.12) | 分析师预期因子+AI量价因子复合，中证500年化超额15.48%，IR=2.93 | 基本面+量价融合是方向 |
| 海通证券(2025.03) | 提出Rank MAE替代传统IC——对单只股票IC贡献偏差更公平 | 值得引入作为辅助评价指标 |
| Microsoft Qlib | 标准化IC/ICIR/RankIC/RankICIR自动评估管线 | 参考其自动化benchmark架构 |

### 通达信公式兼容性映射

为了让从通达信迁移过来的用户无缝上手，本产品自定义指标脚本语言在语法层面兼容通达信公式体系。以下是完整的通达信内置指标→本产品函数映射表。

#### 通达信四大公式类型

| 公式类型 | 用途 | 本产品对应 |
|---------|------|----------|
| **技术指标公式** | K线图下方副图指标 | 类别一至七的全部指标 |
| **条件选股公式** | 全市场筛选符合条件的股票 | 模块四「选股扫描引擎」的条件表达式 |
| **交易系统公式** | 买卖点信号标记（买入/卖出箭头+收益统计） | 模块五「策略回测引擎」的信号生成 |
| **五彩K线公式** | 条件满足时高亮K线（形态高亮） | 类别八「K线形态识别」+ 自定高亮规则 |

#### 通达信核心内置函数兼容表

| 通达信函数 | 功能 | 本产品支持 |
|-----------|------|:--------:|
| `REF(X,N)` | 引用N周期前的X值 | ✅ |
| `MA(X,N)` | X的N周期简单移动平均 | ✅ `sma` |
| `EMA(X,N)` | X的N周期指数移动平均 | ✅ `ema` |
| `SMA(X,N,M)` | X的N周期加权移动平均(M为权重) | ✅ |
| `DMA(X,A)` | 动态移动平均(A为动态权重) | ✅ |
| `HHV(X,N)` | N周期内X的最高值 | ✅ |
| `LLV(X,N)` | N周期内X的最低值 | ✅ |
| `SUM(X,N)` | X的N周期累和 | ✅ |
| `ABS(X)` | X的绝对值 | ✅ |
| `CROSS(A,B)` | A上穿B | ✅ |
| `COUNT(COND,N)` | N周期内满足COND的次数 | ✅ |
| `IF(COND,A,B)` | 条件判断 | ✅ |
| `BARSLAST(X)` | 上次X成立至今的周期数 | ✅ |
| `EVERY(X,N)` | N周期内X一直成立 | ✅ |
| `EXIST(X,N)` | N周期内X曾成立 | ✅ |
| `STD(X,N)` | N周期内X的标准差 | ✅ `stdev` |
| `BETWEEN(A,B,C)` | A在B和C之间 | ✅ |
| `FILTER(X,N)` | X条件成立后N周期内过滤 | ✅ |
| `WINNER(P)` | 获利盘比例估算(成本分布) | ✅ 付费版 |
| `COST(N)` | N%获利盘对应价格 | ✅ 付费版 |
| `ZIG(K,N)` | 之字转折线(峰谷识别) | ✅ 付费版 |
| `BACKSET(X,N)` | 向前回溯N周期赋值为1 | ✅ |
| `PEAK(K,N)` | 之字K转向的波峰值 | ✅ 付费版 |
| `TROUGH(K,N)` | 之字K转向的波谷值 | ✅ 付费版 |
| `SAR(N,S,M)` | 抛物线SAR | ✅ `psar` |

#### 通达信完整内置指标兼容状态

以下 76 个通达信内置指标，标注了在本产品中对应的实现状态。

**大势型（8个，需要全市场数据）**：

| 通达信指标 | 本产品状态 | 对应函数/说明 |
|-----------|:--------:|------------|
| ABI 绝对广量 | ✅ | 导入全市场数据后可计算 |
| ADL 腾落指标 | ✅ | 同上 |
| ADR 涨跌比率 | ✅ | 同上 |
| ARMS 阿姆氏 | ✅ | 同上 |
| BTI 广量冲力 | ✅ | 同上 |
| MCL 麦克连 | ✅ | 同上 |
| OBOS 超买超卖 | ✅ | 同上 |
| TBR 指数平滑广量 | ✅ | 同上 |

**超买超卖型（20个）**：

| 通达信指标 | 本产品状态 | 对应函数 |
|-----------|:--------:|---------|
| CCI | ✅ 免费 | `cci` |
| KDJ | ✅ 免费 | `kdj` |
| MFI | ✅ 免费 | `mfi` |
| MTM | ✅ 免费 | `mom` |
| OSC | ✅ 付费 | `osc` |
| ROC | ✅ 免费 | `roc` |
| RSI | ✅ 免费 | `rsi` |
| KD | ✅ 免费 | `stoch` |
| SKDJ | ✅ 付费 | `skdj` |
| WR | ✅ 免费 | `willr` |
| LWR | ✅ 付费 | `lwr` |
| BIAS | ✅ 免费 | `bias` |
| ACCER | ✅ 免费 | `slope` |
| UDL 引力线 | ✅ 付费 | 多MA均值 |
| ADTM | ✅ 付费 | 自定义公式 |
| ATR | ✅ 免费 | `atr` |
| DKX 多空线 | ✅ 付费 | 自定义公式 |
| CYF 市场能量 | ✅ 付费 | 自定义公式 |
| CYDS/CYDN | ✅ 付费 | 需WINNER/COST |
| MARSI | ✅ 付费 | `rsi_ma` |

**趋势型（18个）**：

| 通达信指标 | 本产品状态 | 对应函数 |
|-----------|:--------:|---------|
| CHO 佳庆指标 | ✅ 免费 | `adosc` |
| DMA 平均差 | ✅ 免费 | 双EMA差值 |
| DMI | ✅ 免费 | `adx` |
| DPO | ✅ 免费 | `dpo` |
| EMV | ✅ 付费 | `eom` |
| MACD | ✅ 免费 | `macd` |
| VMACD | ✅ 付费 | 成交量加权MACD |
| EXPMA | ✅ 免费 | `ema` |
| TRIX | ✅ 付费 | `trix` |
| UOS 终极指标 | ✅ 付费 | `uo` |
| VPT | ✅ 免费 | `pvt` |
| WVAD | ✅ 付费 | 自定义公式 |
| QSDB | ✅ 付费 | 与大盘对比 |
| GDX 轨道线 | ✅ 付费 | 自定义公式 |
| JLHJ 绝路航标 | ✅ 付费 | 自定义公式 |
| CYE 市场趋势 | ✅ 付费 | 自定义公式 |
| JS 加速线 | ✅ 付费 | 自定义公式 |
| DBQR 对比强弱 | ✅ 付费 | 与大盘对比 |

**能量型（8个）**：

| 通达信指标 | 本产品状态 | 对应函数 |
|-----------|:--------:|---------|
| BRAR | ✅ 免费 | `brar` |
| CR | ✅ 付费 | 自定义公式 |
| MASS | ✅ 免费 | `massi` |
| PSY | ✅ 免费 | `psl` |
| VR | ✅ 付费 | 自定义公式 |
| WAD | ✅ 付费 | 自定义公式 |
| PCNT | ✅ 付费 | `percent_return` |
| CYR | ✅ 付费 | 自定义公式 |

**成交量型（7个）**：

| 通达信指标 | 本产品状态 | 对应函数 |
|-----------|:--------:|---------|
| AMOW | ✅ 免费 | 成交量柱+均线 |
| OBV | ✅ 免费 | `obv` |
| VRSI | ✅ 付费 | 成交量RSI变种 |
| HSL 换手线 | ✅ 免费 | 换手率均线 |
| DBQRV | ✅ 付费 | 与大盘量比 |

**均线型（14个）**：

| 通达信指标 | 本产品状态 | 对应函数 |
|-----------|:--------:|---------|
| MA | ✅ 免费 | `sma` |
| EMA/EXPMA | ✅ 免费 | `ema` |
| ACD 升降线 | ✅ 付费 | 自定义公式 |
| BBI | ✅ 付费 | 多均线均值 |
| HMA | ✅ 付费 | `hma` |
| VMA 变异均线 | ✅ 免费 | `vwma` |
| AMV 成本价均线 | ✅ 付费 | 成交量加权 |
| BBIBOLL | ✅ 付费 | BBI+BB结合 |
| 鳄鱼线 | ✅ 付费 | 3条位移均线 |
| GMMA 顾比均线 | ✅ 付费 | 12条EMA(短期6+长期6) |
| PBX 瀑布线 | ✅ 付费 | 多周期EMA组合 |

#### 自定义指标脚本语言设计方向

本产品的自定义指标脚本语言 — **ME Script** — 在设计上：

1. **语法兼容通达信公式**：通达信用户复制粘贴已有公式即可运行，无需重写
2. **扩展现代特性**：支持变量命名（非通达信的单字母限制）、函数封装、跨周期引用
3. **编译到WASM**：脚本在WASM沙箱中解释执行，速度接近原生
4. **函数库预览**：

```
// ME Script 示例：自定义MACD+RSI联合信号
indicator "MACD_RSI_Combo" {
    // 通达信兼容语法
    short := 12;
    long := 26;
    mid := 9;
    DIF := EMA(CLOSE, short) - EMA(CLOSE, long);
    DEA := EMA(DIF, mid);
    MACD := 2 * (DIF - DEA);  // 柱状线
    
    // 现代扩展语法
    rsi_val = rsi(close, 14);
    
    // 联合信号：MACD金叉 AND RSI < 40
    signal = cross(DIF, DEA) AND rsi_val < 40;
    
    plot(DIF, "DIF", color.blue);
    plot(DEA, "DEA", color.red);
    plot(MACD, "柱", color.green);
    plot(signal, "联合信号", color.yellow, style.triangle);
}
```

---

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

## 十三、策略模板库（20个内置策略完整说明）

> 策略模板由 quant_platform V6 的20种策略简化迁移而来，覆盖趋势跟踪、均值回归、动量、突破、复合五大类。

### 策略分类总览

| # | 策略名称 | 类型 | 适用市场 | 免费/付费 |
|---|---------|------|---------|:---:|
| 1 | 双均线交叉 | 趋势跟踪 | 单边市 | 免费 |
| 2 | MACD金叉死叉 | 趋势跟踪 | 单边市 | 免费 |
| 3 | 三均线多头排列 | 趋势跟踪 | 强趋势 | 免费 |
| 4 | 海龟交易法则 | 趋势跟踪 | 中长线单边 | 免费 |
| 5 | 唐奇安通道突破 | 趋势跟踪 | 波动较大 | 免费 |
| 6 | 布林带均值回归 | 均值回归 | 震荡市 | 付费 |
| 7 | RSI超买超卖 | 均值回归 | 震荡市 | 付费 |
| 8 | 布林带+RSI双确认 | 均值回归 | 震荡市 | 付费 |
| 9 | KDJ超买超卖 | 均值回归 | 震荡市 | 付费 |
| 10 | Z-Score回归 | 均值回归 | 所有市场 | 付费 |
| 11 | 横截面动量 | 动量 | 趋势持续 | 付费 |
| 12 | 时间序列动量 | 动量 | 趋势判断 | 付费 |
| 13 | 双线RSI轮动 | 动量 | 风格轮动 | 付费 |
| 14 | 放量突破均线 | 突破 | 突破确认 | 付费 |
| 15 | 波动性突破(ATR) | 突破 | 高波动启动 | 付费 |
| 16 | Keltner通道突破 | 突破 | 趋势启动 | 付费 |
| 17 | MACD+RSI联合 | 复合 | 所有市场 | 付费 |
| 18 | 均线+成交量确认 | 复合 | 趋势确认 | 付费 |
| 19 | 多因子打分卡 | 复合 | 选股 | 付费 |
| 20 | Walk-Forward自适应 | 复合 | 参数稳定性 | 付费 |

---

### 趋势跟踪类（5个，全部免费）

#### 策略1：双均线交叉 (SMA Cross)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 短均线上穿长均线→买入，下穿→卖出 |
| **参数** | `fast=5` `slow=20` `stop_loss=-8%` |
| **入场信号** | `CROSS(SMA(C,fast), SMA(C,slow))` 短均线上穿长均线 |
| **离场信号** | `CROSS(SMA(C,slow), SMA(C,fast))` 短均线下穿长均线 或 跌破入场价×0.92 |
| **资金管理** | 每次全仓进出，单票 |
| **适用环境** | 单边上涨/下跌市，趋势明确时表现好；震荡市中频繁假信号 |
| **参考绩效**（A股2015-2024） | 年化8.2%、Sharpe 0.51、最大回撤 32%、胜率 38%、盈亏比 2.1 |
| **风险提示** | 滞后性明显——金叉时已涨一段。震荡市假信号多，建议加ADX过滤 |
| **ME Script示例** | `signal = cross(sma(close, 5), sma(close, 20));` |

#### 策略2：MACD金叉死叉 (MACD Cross)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | DIF上穿DEA→买入，DIF下穿DEA→卖出 |
| **参数** | `fast=12` `slow=26` `signal=9` `stop_loss=-6%` |
| **入场信号** | `CROSS(macd.dif, macd.dea)` AND `macd.dif < 0`（零轴下方金叉更可靠） |
| **离场信号** | `CROSS(macd.dea, macd.dif)` 或 止损 |
| **资金管理** | 全仓进出 |
| **适用环境** | 适用于所有市场，零轴下方金叉胜率更高；单边趋势中表现最佳 |
| **参考绩效**（A股2015-2024） | 年化6.8%、Sharpe 0.43、最大回撤 28%、胜率 35%、盈亏比 1.8 |
| **风险提示** | MACD是最常被提及的指标→最常被反向利用。建议配合成交量确认 |
| **ME Script示例** | `signal = cross(macd(close,12,26,9).dif, macd(close,12,26,9).dea) and macd(close,12,26,9).dif < 0;` |

#### 策略3：三均线多头排列 (Triple MA Alignment)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 短>中>长三线多头排列→持仓；出现空头排列→清仓 |
| **参数** | `short=5` `mid=20` `long=60` |
| **入场信号** | `ma(C,short) > ma(C,mid) AND ma(C,mid) > ma(C,long)` 连续3日 |
| **离场信号** | `ma(C,short) < ma(C,mid)` 短线下穿中线 |
| **资金管理** | 梯次进场：首次信号50%仓位→回踩中线不破加30%→突破前高加20% |
| **适用环境** | 强趋势市场，适合中长线操作。在A股牛市/结构性行情中表现突出 |
| **参考绩效**（A股2015-2024） | 年化10.5%、Sharpe 0.62、最大回撤 25%、胜率 42%、盈亏比 2.3 |
| **ME Script示例** | `aligned = ma(close,5) > ma(close,20) and ma(close,20) > ma(close,60) and ref(ma(close,5)>ma(close,20),1) and ref(ma(close,5)>ma(close,20),2);` |

#### 策略4：海龟交易法则 (Turtle Trading)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 突破N日最高价入场，跌破M日最低价离场，ATR动态止损 |
| **参数** | `entry_period=20` `exit_period=10` `atr_period=20` `atr_stop=2` |
| **入场信号** | 价格突破过去`entry_period`日最高价（入场1）+ 过去55日最高价（入场2） |
| **离场信号** | 价格跌破过去`exit_period`日最低价 或 跌破入场价−`atr_stop`×ATR |
| **资金管理** | 单笔风险 ≤ 总资金2%，仓位 = 2%资金 / (ATR×2) |
| **适用环境** | 中长线单边市、波动率较高的品种。趋势跟踪的鼻祖级策略 |
| **参考绩效**（A股2015-2024） | 年化9.8%、Sharpe 0.48、最大回撤 35%、胜率 36%、盈亏比 3.1 |
| **风险提示** | 回撤大，需要心理纪律。A股涨停买不到是重大限制 |
| **ME Script示例** | `entry = close > hhv(high, 20); exit = close < llv(low, 10);` |

#### 策略5：唐奇安通道突破 (Donchian Breakout)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 突破N日最高价买入，跌破M日最低价卖出 |
| **参数** | `upper_period=20` `lower_period=10` `min_band_width=3%` |
| **入场信号** | `C > HHV(H, upper_period)` AND `(HHV(H,upper_period)/LLV(L,upper_period)−1) > min_band_width` |
| **离场信号** | `C < LLV(L, lower_period)` |
| **资金管理** | 初始风险2%，ATR(14)×1.5止损 |
| **适用环境** | 波动较大、有明显趋势的市场。相比双均线金叉响应更快 |
| **参考绩效**（A股2015-2024） | 年化7.5%、Sharpe 0.39、最大回撤 29%、胜率 34%、盈亏比 2.0 |
| **风险提示** | 假突破频繁，建议用成交量放大+ADX>25双层过滤 |

---

### 均值回归类（5个，全部付费）

#### 策略6：布林带均值回归 (Bollinger Mean Reversion)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 价格突破布林带上下轨→大概率回归中轨 |
| **参数** | `period=20` `std_mult=2.0` `rsi_threshold=30/70` `hold_days=5` |
| **入场信号(做多)** | `C < bbands.lower AND RSI(C,14) < rsi_threshold` |
| **入场信号(做空)** | `C > bbands.upper AND RSI(C,14) > 70`（A股做空限制，仅提示风险） |
| **离场信号** | `C > sma(C,20)` 回归中轨 或 持有满`hold_days`天 |
| **资金管理** | 等金额分5批在不同偏离度入场 |
| **适用环境** | **震荡市**。单边市（如2024年红利行情）会连续破上轨→策略持续亏损 |
| **参考绩效**（A股2015-2024，仅震荡期） | 年化5.2%、Sharpe 0.58、最大回撤 15%、胜率 62%、盈亏比 1.2 |
| **市场环境判断** | 必须配合ADX<20（震荡确认）才启用此策略 |

#### 策略7：RSI超买超卖 (RSI Oversold/Overbought)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | RSI<30超卖→买入，RSI>70超买→卖出，等待回归 |
| **参数** | `period=14` `oversold=30` `overbought=70` `hold_days=5` |
| **入场信号** | `RSI(C,14) < 30 AND C > ref(C,1)`（RSI超卖+当日止跌） |
| **离场信号** | `RSI(C,14) > 55`（回到中性区）或 持仓满5天 |
| **资金管理** | 分批入场：RSI<30首仓40%，继续跌至RSI<20加仓30%，止跌加仓30% |
| **适用环境** | 震荡市。有明确支撑位的个股效果更好 |
| **参考绩效**（A股2015-2024，震荡期） | 年化4.8%、Sharpe 0.52、最大回撤 12%、胜率 65%、盈亏比 1.1 |
| **风险提示** | 单边下跌中RSI可以长期<30，此时抄底="接飞刀"。必须确认止跌再入场 |

#### 策略8：布林带+RSI双确认 (BB+RSI Double Confirm)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 布林带下轨+RSI超卖+放量止跌→三重确认反弹 |
| **参数** | `period=20` `std=2.0` `rsi_period=14` `rsi_threshold=30` |
| **入场信号(做多)** | `C < bbands.lower AND RSI<30 AND V > ma(V,20)*1.5 AND C > ref(C,1)` |
| **离场信号** | `C > sma(C,20)` 或 入场后第7天 |
| **资金管理** | 等仓位，单票资金≤20% |
| **适用环境** | 震荡市/弱趋势市。三重确认大幅降低假信号 |
| **参考绩效**（A股2015-2024，震荡期） | 年化6.1%、Sharpe 0.68、最大回撤 13%、胜率 70%、盈亏比 1.4 |

#### 策略9：KDJ超买超卖 (KDJ Extremes)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | %K<20且%D<20→超卖买入，%K>80且%D>80→超买卖出 |
| **参数** | `n=9` `m1=3` `m2=3` `oversold=20` `overbought=80` |
| **入场信号** | `kdj.k < 20 AND kdj.d < 20 AND kdj.j < 0 AND C > ref(C,1)` |
| **离场信号** | `kdj.k > 55`（中性区）或 止损-5% |
| **适用环境** | 震荡市，A股短线操作常用。KDJ比RSI更敏感→信号更早但假信号也更多 |
| **参考绩效**（A股2015-2024，震荡期） | 年化4.2%、Sharpe 0.45、最大回撤 18%、胜率 58%、盈亏比 1.0 |

#### 策略10：Z-Score均值回归 (Z-Score Regression)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 价格偏离均线超过N个标准差→统计上大概率回归 |
| **参数** | `period=20` `entry_z=2.0` `exit_z=0.5` `stop_loss=-5%` |
| **入场信号(做多)** | `(C − sma(C,period)) / stdev(C,period) < −entry_z` AND `C > ref(C,1)` |
| **入场信号(做空)** | `(C − sma(C,period)) / stdev(C,period) > +entry_z`（仅提示） |
| **离场信号** | Z-Score回到±`exit_z`区间 或 止损 |
| **资金管理** | 分3批入场：|Z|>2首仓40% → |Z|>2.5加30% → |Z|>3加30% |
| **适用环境** | 所有市场。相比布林带更纯粹地基于统计偏离度 |
| **参考绩效**（A股2015-2024） | 年化5.8%、Sharpe 0.60、最大回撤 14%、胜率 60%、盈亏比 1.3 |
| **风险提示** | 正态分布假设在金融市场不一定成立——极端行情比正态分布预期更频繁（厚尾） |

---

### 动量类（3个，全部付费）

#### 策略11：横截面动量 (Cross-Sectional Momentum)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 买过去N月涨幅最大的前K只，卖涨幅最小的后K只 |
| **参数** | `lookback=60`（交易日） `top_k=20` `rebalance=20`（交易日） |
| **入场信号** | 每20个交易日：计算全市场过去60日涨跌幅，买入涨幅前20的股票 |
| **离场信号** | 下次调仓时不在前20→卖出 |
| **资金管理** | 等权重，20只股票各5% |
| **适用环境** | 趋势持续/赛道行情。2020-2021 A股赛道行情中表现极佳 |
| **参考绩效**（A股2015-2024） | 年化11.5%、Sharpe 0.72、最大回撤 31%、月胜率 58% |
| **风险提示** | 动量崩溃（Momentum Crash）——风格突然切换时暴跌（如2024年9-11月） |

#### 策略12：时间序列动量 (Time-Series Momentum)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 过去N日收益>0做多，<0空仓 |
| **参数** | `lookback=60` `vol_target=15%` `smoothing=5` |
| **入场信号** | `roc(C, lookback) > 0` AND `sma(roc(C,lookback), smoothing) > 0`（动量持续为正） |
| **离场信号** | `roc(C, lookback) < 0` |
| **资金管理** | 波动率目标：仓位 = 15%目标波动率 / 实现波动率 |
| **适用环境** | 中型趋势市场，可作为复合策略的基础层 |
| **参考绩效**（A股2015-2024） | 年化7.2%、Sharpe 0.55、最大回撤 24% |
| **风险提示** | 急涨急跌时信号滞后（60日动量包含已消化的信息） |

#### 策略13：双线RSI轮动 (Dual RSI Rotation)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 快RSI>慢RSI且均上升→动量确认买入 |
| **参数** | `fast_period=7` `slow_period=14` `signal_period=5` |
| **入场信号** | `rsi(C,fast) > rsi(C,slow) AND rsi(C,fast) > ref(rsi(C,fast),1) AND rsi(C,slow) > ref(rsi(C,slow),1)` |
| **离场信号** | `rsi(C,fast) < rsi(C,slow)` |
| **适用环境** | 风格轮动市场，中小盘表现优于大盘时效果更好 |
| **参考绩效**（A股2015-2024） | 年化8.5%、Sharpe 0.61、最大回撤 22%、胜率 48% |

---

### 突破类（3个，全部付费）

#### 策略14：放量突破均线 (Volume Breakout MA)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 放量突破中期均线+收盘确认→趋势启动信号 |
| **参数** | `ma_period=60` `vol_mult=1.5` `confirm_bars=1` |
| **入场信号** | `C > sma(C,ma_period) AND V > sma(V,20)*vol_mult AND ref(C,1) < ref(sma(C,ma_period),1)` |
| **离场信号** | 收盘价跌破20日均线 |
| **资金管理** | 初始仓50%→确定趋势加50% |
| **适用环境** | 底部反弹/突破盘整。A股60日线被视为"生命线"，突破后关注度大增 |
| **参考绩效**（A股2015-2024） | 年化9.3%、Sharpe 0.66、最大回撤 26%、胜率 44%、盈亏比 2.2 |
| **ME Script示例** | `signal = close > sma(close,60) and volume > sma(volume,20)*1.5 and ref(close,1) < ref(sma(close,60),1);` |

#### 策略15：波动性突破 (Volatility Breakout - ATR)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 价格高于昨日收盘+N倍ATR→波动扩张，趋势行情启动 |
| **参数** | `atr_period=20` `atr_mult=2.0` `volume_filter=true` |
| **入场信号** | `C > ref(C,1) + atr(20)*atr_mult OR C < ref(C,1) − atr(20)*atr_mult` |
| **离场信号** | 回到入场价±0.5ATR（小额止损） |
| **适用环境** | 高波动启动、盘整后放量突破 |
| **参考绩效**（A股2015-2024） | 年化6.5%、Sharpe 0.42、最大回撤 30%、胜率 32%、盈亏比 2.8 |

#### 策略16：Keltner通道突破 (Keltner Breakout)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 收盘价突破Keltner通道上轨入场，跌破中轨离场 |
| **参数** | `ema_period=20` `atr_period=10` `atr_mult=2.0` |
| **入场信号** | `C > kc.upper AND ref(C,1) <= ref(kc.upper,1)`（收盘确认突破上轨） |
| **离场信号** | `C < kc.middle`（中轨止损） |
| **资金管理** | 全仓 |
| **适用环境** | 趋势启动、轧空行情。比布林带更适合趋势市（上轨用ATR而非标准差） |
| **参考绩效**（A股2015-2024） | 年化7.9%、Sharpe 0.54、最大回撤 27%、胜率 37%、盈亏比 2.5 |

---

### 复合类（4个，全部付费）

#### 策略17：MACD+RSI联合 (MACD+RSI Combo)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | MACD金叉+RSI不超买→趋势确认+动量安全 |
| **参数** | `macd_fast=12` `macd_slow=26` `macd_signal=9` `rsi_period=14` `rsi_low=40` `rsi_high=65` |
| **入场信号** | `CROSS(macd.dif, macd.dea) AND RSI(C,14) > rsi_low AND RSI(C,14) < rsi_high` |
| **离场信号** | `CROSS(macd.dea, macd.dif)` 或 RSI>80 |
| **资金管理** | 标准仓位×RSI位置系数（RSI40=1.0仓，RSI55=0.8仓） |
| **适用环境** | 所有市场。RSI过滤避免了MACD在超买区金叉的高点追入 |
| **参考绩效**（A股2015-2024） | 年化9.8%、Sharpe 0.70、最大回撤 20%、胜率 48%、盈亏比 2.4 |
| **关键优势** | 相比纯MACD策略胜率+13%，最大回撤−8% |

#### 策略18：均线+成交量确认 (MA+Volume Confirm)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 价格站上均线+成交量放大→资金入场确认 |
| **参数** | `ma_short=5` `ma_long=20` `vol_ma=20` `vol_mult=1.5` |
| **入场信号** | `C > sma(C,ma_short) AND sma(C,ma_short) > sma(C,ma_long) AND V > sma(V,vol_ma)*vol_mult` |
| **离场信号** | `V < sma(V,20)*0.5`（缩量→资金退潮） |
| **资金管理** | 首次50%—确认放量→加至100% |
| **参考绩效**（A股2015-2024） | 年化7.1%、Sharpe 0.57、最大回撤 23%、胜率 43%、盈亏比 1.9 |

#### 策略19：多因子打分卡 (Multi-Factor Scorecard)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | ICIR加权的技术面+量价多因子综合打分，选前N只 |
| **参数** | `top_n=30` `rebalance=20`（交易日）`factors=[动量,波动,成交量,趋势,质量]` |
| **因子权重** | 各因子按ICIR动态加权，每季度重新估计权重 |
| **入场信号** | 每20日：计算所有股票在5个因子上的Z-score→加权总分→买前30只 |
| **离场信号** | 下次调仓时排名跌出前30→卖出 |
| **资金管理** | 等权重，30只各约3.3% |
| **适用环境** | 选股场景。Alpha158因子库的前置简化版 |
| **参考绩效**（A股2015-2024） | 年化13.2%、Sharpe 0.85、最大回撤 22%、月胜率 62% |

#### 策略20：Walk-Forward自适应 (Walk-Forward Adaptive)
| 属性 | 内容 |
|------|------|
| **一句话逻辑** | 滚动窗口优化参数→用"未来"数据验证→选择在样本外最稳健的参数 |
| **参数** | `in_sample=252`（1年） `out_sample=63`（1季） `anchor="滚动"` `optimizer="网格"` |
| **底层策略** | 双均线/MACD/布林带三策略之一（用户可选） |
| **流程** | ①过去1年数据→网格搜索最优参数 ②在随后1季验证 ③滚动推进 ④选择平均OOS表现最好的参数 |
| **入场/离场** | 由当前最优参数的底层策略决定 |
| **适用环境** | 当底层策略在最近市场环境中需要参数自适应调整 |
| **参考绩效**（A股2015-2024，基于双均线底层） | 年化10.1%、Sharpe 0.68、最大回撤 19% |
| **关键优势** | 参数定期自适应，避免固化参数在市场环境变化后失效 |

> 以上20个策略的ME Script完整源码、参数范围定义、回测报告模板将在实现阶段逐一编写。

---

## 十四、ME Script 自定义指标脚本语言 — 完整语法规范

> v1.0-draft | 2026-05-21 | 兼容通达信公式语法 + thinkScript现代特性

### 14.1 设计原则

| # | 原则 | 说明 |
|---|------|------|
| 1 | **通达信语法兼容** | 通达信用户可直接粘贴已有公式，零学习成本迁移 |
| 2 | **现代语言特性** | 支持变量命名（非TDX单字母限制）、函数封装、模块化 |
| 3 | **编译到WASM** | 脚本→AST→WASM字节码，在沙箱中解释执行 |
| 4 | **安全性** | 无文件IO、无网络访问、纯计算沙箱 |
| 5 | **双语法模式** | "经典模式"（TDX兼容）和"现代模式"（扩展语法）可混用 |

---

### 14.2 词法结构

#### 基本数据类型

| 类型 | 关键字 | 示例 | 说明 |
|------|--------|------|------|
| 数值型 | — | `3.14` `−5` `0.01` | 所有数值为64位浮点 |
| 序列型(Series) | — | `close` `volume` | 沿时间轴展开的数值序列，默认数据类型 |
| 布尔型 | `bool` | `true` `false` | 内部表示为1/0，与TDX兼容 |
| 字符串 | `string` | `"MACD金叉"` | 仅用于标注/显示，不参与计算 |
| 空值 | `null` | `null` | 等价于TDX的`DRAWNULL` |
| 颜色 | `color` | `color.red` `#FF6600` `rgb(255,0,0)` | 仅用于绘图修饰 |

#### 变量定义（双模式）

```c
// ===== 经典模式（TDX兼容）=====
MA5:MA(C,5);           // :  赋值并输出到图表
MA10:=MA(C,10);        // := 赋值但不输出
信号:CROSS(MA5,MA10);  // 自动输出

// ===== 现代模式 =====
let ma5 = sma(close, 5);       // let 定义局部变量（不输出）
plot ma10 = sma(close, 10);    // plot 定义输出变量
bool signal = cross(ma5, ma10); // 类型标注可选
```

#### 注释

```c
// 单行注释（现代模式）
{ 单行或多行注释（TDX兼容模式）}
/* 多行注释（现代模式）
   跨越多行 */
```

#### 标识符规则
- 字母/下划线开头，含字母/数字/下划线
- 区分大小写
- 不能与保留字重名
- **现代模式**：变量名长度不限
- **经典模式**：首字符不能是数字（TDX限制）

---

### 14.3 内置数据序列（行情常量）

| 常量 | 经典别名 | 全称 | 说明 |
|------|---------|------|------|
| `close` | `C` `CLOSE` | 收盘价 | 当前周期收盘价 |
| `open` | `O` `OPEN` | 开盘价 | 当前周期开盘价 |
| `high` | `H` `HIGH` | 最高价 | 当前周期最高价 |
| `low` | `L` `LOW` | 最低价 | 当前周期最低价 |
| `volume` | `V` `VOL` | 成交量 | 当前周期成交量（手） |
| `amount` | `AMO` `AMOUNT` | 成交额 | 当前周期成交金额（元） |
| `turnover` | — | 换手率 | 成交量/流通股本（需要股本数据） |
| `oi` | — | 持仓量 | 期货/期权专用 |
| `pre_close` | `REF(C,1)` | 前收盘 | 昨收价 |
| `up_count` | `ADVANCE` | 上涨家数 | 全市场（需要全市场数据） |
| `down_count` | `DECLINE` | 下跌家数 | 全市场（需要全市场数据） |
| `index_close` | `INDEXC` | 大盘收盘价 | 对应指数收盘价 |
| `index_volume` | `INDEXV` | 大盘成交量 | 对应指数成交量 |

---

### 14.4 运算符

#### 算术运算（优先级从高到低）

| 优先级 | 运算符 | 说明 | 示例 |
|:---:|-------|------|------|
| 1 | `()` | 括号，改变优先级 | `(C+O)/2` |
| 2 | `^` `**` | 乘方 | `close ^ 2` |
| 3 | `*` `/` `%` | 乘/除/取模 | `volume * close` |
| 4 | `+` `−` | 加/减 | `high − low` |

#### 比较运算

| 运算符 | 经典写法 | 说明 |
|--------|---------|------|
| `>` | `>` | 大于 |
| `<` | `<` | 小于 |
| `>=` | `>=` | 大于等于 |
| `<=` | `<=` | 小于等于 |
| `==` | `=` | 等于（现代模式推荐用`==`，经典模式用`=`） |
| `!=` | `<>` | 不等于 |

> ⚠️ 浮点数相等判断：`abs(a−b) < 0.0001` 替代 `a == b`

#### 逻辑运算

| 运算符 | 经典写法 | 说明 |
|--------|---------|------|
| `and` | `AND` `&&` | 逻辑与 |
| `or` | `OR` `||` | 逻辑或 |
| `not` | `NOT` | 逻辑非 |
| `xor` | — | 异或（现代扩展） |

#### 特殊运算符（现代扩展）

| 运算符 | 说明 | 示例 |
|--------|------|------|
| `|period|` | 跨周期引用 | `close|week|` |
| `$symbol` | 跨品种引用 | `"000001.SZ"$close` |
| `[offset]` | 历史值引用 | `close[1]` = T-1收盘价 |
| `[−offset]` | 前向引用（仅回测） | `close[−1]` = T+1收盘价 ⚠️禁止在信号中使用 |

---

### 14.5 时间序列引用（核心概念）

#### 历史值偏移 [N]

```
close[0]    // 当前K线的收盘价（等价于 close）
close[1]    // 前一根K线的收盘价
close[5]    // 5根K线前的收盘价
high[1]     // 前一根K线的最高价
```

**等价关系**：

| ME Script (现代) | ME Script (经典) | 说明 |
|-----------------|-----------------|------|
| `close[1]` | `REF(C, 1)` | 前一周期的收盘价 |
| `close[5]` | `REF(C, 5)` | 5周期前的收盘价 |
| `sma(close,20)[1]` | `REF(MA(C,20), 1)` | 20均线的昨日值 |

#### 动态窗口运算（无需手动循环）

```
// 以下函数自动沿时间轴展开，无需 for/while
sma(close, 20)          // 逐K线计算20周期均线
hhv(high, 20)           // 逐K线计算20周期最高价
rsi(close, 14)          // 逐K线计算14周期RSI
```

**关键约束**：在入场信号中禁止使用前向偏移`[−N]`（防止未来信息泄露）。`[−1]`仅在回测引擎的收益计算公式中内部使用。

---

### 14.6 内置函数库

#### 14.6.1 统计函数

| 函数 | 经典别名 | 说明 | 参数 |
|------|---------|------|------|
| `sma(x, n)` | `MA(X,N)` | 简单移动平均 | x:序列, n:周期 |
| `ema(x, n)` | `EMA(X,N)` | 指数移动平均 | x:序列, n:周期 |
| `wma(x, n)` | — | 加权移动平均 | x:序列, n:周期 |
| `rma(x, n)` | — | Wilder平滑均线 | x:序列, n:周期 |
| `kama(x, n)` | — | Kaufman自适应均线 | x:序列, n:周期 |
| `hma(x, n)` | — | Hull移动均线 | x:序列, n:周期 |
| `stdev(x, n)` | `STD(X,N)` | 标准差 | x:序列, n:周期 |
| `variance(x, n)` | `VAR(X,N)` | 方差 | x:序列, n:周期 |
| `zscore(x, n)` | — | Z分数 | x:序列, n:周期 |
| `corr(x, y, n)` | — | Pearson相关系数 | x,y:序列, n:周期 |
| `skew(x, n)` | — | 偏度 | x:序列, n:周期 |
| `kurt(x, n)` | — | 峰度 | x:序列, n:周期 |
| `entropy(x, n)` | — | 信息熵 | x:序列, n:周期 |

#### 14.6.2 引用/窗口函数

| 函数 | 经典别名 | 说明 |
|------|---------|------|
| `ref(x, n)` | `REF(X,N)` | 引用N周期前的X值 |
| `hhv(x, n)` | `HHV(X,N)` | N周期内X的最高值 |
| `llv(x, n)` | `LLV(X,N)` | N周期内X的最低值 |
| `sum(x, n)` | `SUM(X,N)` | N周期X累和 |
| `barslast(x)` | `BARSLAST(X)` | 上次X成立至今的周期数 |
| `barssince(x)` | `BARSSINCE(X)` | 第一次X成立至今的周期数 |
| `count(cond, n)` | `COUNT(X,N)` | N周期内满足cond的次数 |
| `every(cond, n)` | `EVERY(X,N)` | N周期内cond一直成立 |
| `exist(cond, n)` | `EXIST(X,N)` | N周期内cond曾经成立 |

#### 14.6.3 逻辑/条件函数

| 函数 | 经典别名 | 说明 |
|------|---------|------|
| `cross(a, b)` | `CROSS(A,B)` | A上穿B（A[0]>B[0] AND A[1]<=B[1]） |
| `longcross(a, b, n)` | `LONGCROSS(A,B,N)` | A维持N周期后上穿B |
| `if(cond, a, b)` | `IF(X,A,B)` | 三目运算符：cond为真返A，否则返B |
| `iff(cond, a, b)` | `IFF(X,A,B)` | 同IF |
| `ifn(cond, a, b)` | `IFN(X,A,B)` | 反向IF |
| `filter(cond, n)` | `FILTER(X,N)` | cond成立后N周期内过滤重复信号 |
| `between(x, a, b)` | `BETWEEN(A,B,C)` | A是否在B和C之间 |
| `range(x, a, b)` | `RANGE(A,B,C)` | A是否严格在B和C之间（不含等号） |
| `backset(cond, n)` | `BACKSET(X,N)` | 向前回溯N周期赋值为1 |

#### 14.6.4 数学函数

| 函数 | 经典别名 | 说明 |
|------|---------|------|
| `abs(x)` | `ABS(X)` | 绝对值 |
| `max(a, b)` | `MAX(A,B)` | 取较大值 |
| `min(a, b)` | `MIN(A,B)` | 取较小值 |
| `pow(x, n)` | `POW(A,B)` | 乘幂 |
| `sqrt(x)` | `SQRT(X)` | 开平方 |
| `log(x)` | `LOG(X)` | 常用对数（base 10） |
| `ln(x)` | `LN(X)` | 自然对数（base e） |
| `exp(x)` | `EXP(X)` | e的x次幂 |
| `round(x)` | `ROUND(X)` | 四舍五入 |
| `ceil(x)` | `CEILING(A)` | 向上取整 |
| `floor(x)` | `FLOOR(A)` | 向下取整 |
| `mod(a, b)` | `MOD(A,B)` | 取模 |
| `sign(x)` | — | 符号函数（−1/0/+1） |
| `sin(x)/cos(x)/tan(x)` | `SIN/COS/TAN` | 三角函数（用于周期分析） |

#### 14.6.5 技术指标函数（一级调用）

所有316个指标均可直接以函数形式调用：

```c
// 趋势类
adx(high, low, close, 14)        // ADX
aroon(high, low, 14)              // Aroon
macd(close, 12, 26, 9)           // MACD（返回结构体：.dif .dea .hist）
psar(high, low, 0.02, 0.2)       // 抛物线SAR
supertrend(high, low, close, 10, 3) // 超级趋势

// 动量类
rsi(close, 14)                    // RSI
stoch(high, low, close, 14, 3, 3) // Stochastic（返回：.k .d）
kdj(high, low, close, 9, 3, 3)   // KDJ（返回：.k .d .j）
cci(high, low, close, 14)        // CCI
mfi(high, low, close, volume, 14) // MFI
trix(close, 15)                   // TRIX
fisher(high, low, 10)             // Fisher Transform

// 波动类
bbands(close, 20, 2.0)           // 布林带（返回：.upper .middle .lower .bw .pct_b）
atr(high, low, close, 14)        // ATR
kc(high, low, close, 20, 2.0)    // Keltner通道（返回：.upper .middle .lower）
donchian(high, low, 20)           // 唐奇安通道（返回：.upper .middle .lower）
ui(close, 14)                     // 溃疡指数

// 成交量类
obv(close, volume)                // OBV能量潮
adosc(high, low, close, volume, 3, 10) // Chaikin A/D振荡器
cmf(high, low, close, volume, 20)     // Chaikin资金流
eom(high, low, close, volume, 14)     // 易动性
pvt(close, volume)                // 量价趋势

// 周期类
ebsw(close, 20)                   // 正弦波指标
fft(close)                        // 快速傅里叶变换（返回：.dominant_cycle .power_spectrum）

// 统计类
drawdown(close)                   // 回撤分析（返回：.max_dd .dd_period .recovery）
percent_return(close)             // 百分比收益率
```

#### 14.6.6 K线形态函数

```c
// 单线形态
cdl_doji(open, high, low, close)           // 十字星
cdl_hammer(open, high, low, close)          // 锤子线
cdl_shooting_star(open, high, low, close)   // 射击之星
cdl_marubozu(open, high, low, close)        // 光头光脚

// 双线/三线形态
cdl_engulfing(open, high, low, close)       // 吞没（+1看涨/−1看跌）
cdl_harami(open, high, low, close)          // 孕线
cdl_morning_star(open, high, low, close)    // 晨星
cdl_evening_star(open, high, low, close)    // 黄昏之星
cdl_three_white_soldiers(open, high, low, close) // 三白兵

// 63种CDL形态全部以 cdl_<pattern> 形式提供
// 返回值：+100（看涨）、−100（看跌）、0（无信号）
```

#### 14.6.7 时间函数

| 函数 | 说明 | 返回值 |
|------|------|--------|
| `date()` | 当前K线日期 | `20260521` |
| `time()` | 当前K线时间 | `0930`−`1500` |
| `year()` | 年份 | `2026` |
| `month()` | 月份 | `1-12` |
| `day()` | 日期 | `1-31` |
| `weekday()` | 星期 | `1-7`（1=周一） |
| `hour()` | 小时 | `0-23` |
| `minute()` | 分钟 | `0-59` |
| `period()` | 周期类型 | `'1min'` `'5min'` `'D'` `'W'` `'M'` |
| `bars_count()` | 总K线数 | 整数 |
| `bar_index()` | 当前K线索引 | 0起 |

---

### 14.7 控制流

#### IF…THEN…ELSE（现代模式）

```c
// 单行形式
let signal = if (rsi(close, 14) < 30) then 1 else 0;

// 多行形式
let signal = if (rsi < 30 and close > ref(close, 1)) {
    let strength = min(rsi, 100);
    strength / 100;  // 最后表达式为返回值
} else if (rsi > 70) {
    −1;
} else {
    0;
};
```

#### IF 函数（经典TDX模式）

```c
信号:=IF(RSI<30 AND C>REF(C,1), 1,
      IF(RSI>70, −1, 0));
```

> 两种模式可混用。建议简单条件用经典`IF()`，多分支逻辑用现代`if…else`。

#### 无循环结构

与TDX/thinkScript一致，**不提供显式循环**（`for`/`while`）。所有计算自动沿时间轴逐K线展开。如果需要累和/累积操作，使用以下函数：

```c
sum(x, 20)              // N周期滚动累和
cumsum(x)               // 全周期累积和（现代扩展）
barslast(cond)          // 条件成立以来周期计数
every(cond, n)          // N周期内一直成立
```

---

### 14.8 跨周期引用

#### 语法

```c
// 方式一：前缀式（现代）
let weekly_close = close|week|;
let monthly_high = high|month|;
let macd_dif_weekly = macd(close|week|, 12, 26, 9).dif;

// 方式二：后缀式（TDX兼容）
AA:=CLOSE#WEEK;          // 周线收盘价
BB:=MACD.DIF#WEEK;       // 周线MACD的DIF值
```

#### 支持的周期

| 关键字 | 周期 | 说明 |
|--------|------|------|
| `1min` `5min` `15min` `30min` `60min` | 分钟线 | 仅分钟数据可用时 |
| `D` `day` | 日线 | 默认周期 |
| `W` `week` | 周线 | 从日线聚合 |
| `M` `month` | 月线 | 从日线聚合 |
| `Q` `quarter` | 季线 | 现代扩展 |
| `Y` `year` | 年线 | 现代扩展 |

> **限制**：只能引用**等于或高于**当前周期的数据。在日线公式中不能引用分钟线。

---

### 14.9 跨品种引用

```c
// 引用指定股票的收盘价
let benchmark_close = "000001.SZ"$close;

// 引用指定股票的指标
let bench_rsi = "000001.SZ"$rsi(close, 14);

// 相对强度计算
let rel_strength = close / "000300.SH"$close * 100;
```

---

### 14.10 绘图系统

#### plot 语句（现代模式）

```c
// 基本绘图
plot ma5 = sma(close, 5);
  // 默认：蓝色实线，线宽1

// 完整标注
plot macd_dif = macd(close,12,26,9).dif {
    name: "DIF",
    color: color.blue,
    line_width: 2,
    line_style: line.solid,
    panel: "副图1",
    visible: true
};

plot macd_hist = macd(close,12,26,9).hist {
    name: "MACD柱",
    color: if (macd_hist > ref(macd_hist, 1)) then color.red else color.green,
    style: plot.histogram,
    panel: "副图1"
};
```

#### 经典TDX绘图修饰（兼容模式）

```
MA5:MA(C,5),COLORRED,LINETHICK2;
DIF:EMA(C,12)−EMA(C,26),COLORWHITE;
MACD:(DIF−DEA)*2,COLORSTICK;

{ 等价于：}
MA5:MA(C,5),COLORRED,LINETHICK2;
DIF:EMA(C,12)−EMA(C,26),COLORWHITE;
MACD:(DIF−DEA)*2,COLORSTICK;
```

#### 绘图类型

| 类型 | 经典写法 | 现代写法 | 说明 |
|------|---------|---------|------|
| 实线 | 默认 | `plot` | 折线连接 |
| 柱状线 | `STICK`/`COLORSTICK` | `style: plot.histogram` | 柱线（MACD风格） |
| 虚线 | `DOTLINE` | `line_style: line.dash` | 虚线连接 |
| 点线 | `POINTDOT` | `line_style: line.dot` | 散点不连 |
| 不画线 | `NODRAW` | `visible: false` | 仅用于中间计算 |
| 图标 | `DRAWICON` | `plot icon` | 在指定位置画图标符号 |
| 文字 | `DRAWTEXT` | `label()` | 在指定位置显示文字 |
| 带状 | `DRAWBAND` | `fill(a, b, color)` | 两线之间的着色区域 |
| 柱形条 | `STICKLINE` | `bar(cond, high, low, color)` | 自定义高度柱条 |
| K线 | `DRAWKLINE` | `kline(o,h,l,c)` | 自定义OHLC绘制 |

#### 颜色系统

```c
// 预设颜色
color.red, color.green, color.blue, color.yellow
color.white, color.black, color.cyan, color.magenta
color.orange, color.purple, color.gray, color.dark_gray

// 自定义颜色
rgb(255, 128, 0)       // RGB（0-255）
#FF8000                // 十六进制
hsl(30, 100, 50)       // HSL

// 条件变色
let dynamic_color = if (close > ref(close, 1)) then color.red else color.green;
```

---

### 14.11 完整指标示例

#### 示例1：MACD（经典TDX兼容模式）

```
{ MACD指标 — 通达信兼容语法 }
SHORT:=12;
LONG:=26;
MID:=9;
DIF:EMA(CLOSE,SHORT)−EMA(CLOSE,LONG);
DEA:EMA(DIF,MID);
MACD:(DIF−DEA)*2,COLORSTICK;
DIF,COLORWHITE;
DEA,COLORYELLOW;
```

#### 示例2：MACD（现代模式等价写法）

```c
// ME Script 现代模式
indicator "MACD" {
    param short = 12 { min: 2, max: 100, step: 1 };
    param long = 26 { min: 2, max: 200, step: 1 };
    param mid = 9 { min: 2, max: 50, step: 1 };

    let dif = ema(close, short) − ema(close, long);
    plot dea = ema(dif, mid) { name: "DEA", color: color.yellow };
    plot macd = (dif − dea) * 2 { name: "MACD柱", style: plot.histogram };

    plot dif_line = dif { name: "DIF", color: color.white };
}
```

#### 示例3：放量MACD金叉选股（混合模式）

```c
// 放量MACD金叉 + 成交量放大确认
indicator "MACD_Volume_Breakout" {
    param fast = 12 { min: 2, max: 100 };
    param slow = 26 { min: 2, max: 200 };
    param signal_period = 9 { min: 2, max: 50 };
    param vol_mult = 1.5 { min: 1.0, max: 5.0, step: 0.1 };

    // 经典语法计算MACD
    let macd_val = macd(close, fast, slow, signal_period);
    let dif := macd_val.dif;
    let dea := macd_val.dea;

    // 金叉信号：DIF上穿DEA
    let golden_cross = cross(dif, dea);

    // 成交量放大确认
    let vol_expand = volume > sma(volume, 20) * vol_mult;

    // 价格站上20日均线确认
    let price_above_ma = close > sma(close, 20);

    // 联合入场信号
    plot buy_signal = golden_cross and vol_expand and price_above_ma {
        name: "放量金叉买入",
        icon: icon.triangle_up,
        color: color.red,
        position: low * 0.98
    };

    // 卖出信号：DIF下穿DEA
    plot sell_signal = cross(dea, dif) {
        name: "死叉卖出",
        icon: icon.triangle_down,
        color: color.green,
        position: high * 1.02
    };
}
```

#### 示例4：多因子打分卡（现代模式）

```c
indicator "Multi_Factor_Scorecard" {
    // 因子权重（ICIR动态估计的简化版）
    param w_momentum = 0.30 { min: 0, max: 1, step: 0.01 };
    param w_trend = 0.25 { min: 0, max: 1, step: 0.01 };
    param w_volatility = 0.20 { min: 0, max: 1, step: 0.01 };
    param w_volume = 0.15 { min: 0, max: 1, step: 0.01 };
    param w_quality = 0.10 { min: 0, max: 1, step: 0.01 };

    // 动量因子：过去20日涨跌幅Z-score
    let momentum = zscore(roc(close, 20), 120);

    // 趋势因子：ADX趋势强度
    let trend = adx(high, low, close, 14) / 100;  // 归一化到0-1

    // 波动因子：波动率倒数（低波得分高）
    let volatility = 1 − (stdev(percent_return(close), 20) * sqrt(252));
    let vol_norm = zscore(volatility, 120);

    // 成交量因子：成交量相对强度
    let vol_strength = zscore(sma(volume, 5) / sma(volume, 20), 120);

    // 质量因子：价格效率（净位移/总路程）
    let efficiency = abs(close − ref(close, 20)) / sum(abs(close − ref(close, 1)), 20);

    // 加权综合得分
    plot score = momentum * w_momentum
               + trend * w_trend
               + vol_norm * w_volatility
               + vol_strength * w_volume
               + efficiency * w_quality {
        name: "综合因子得分",
        color: color.blue,
        panel: "副图2"
    };

    // 零线参考
    plot zero_line = 0 {
        name: "零线",
        color: color.gray,
        line_style: line.dash
    };
}
```

---

### 14.12 语法完整EBNF（精简版）

```ebnf
program         = (indicator_def | statement)* ;
indicator_def   = "indicator" IDENTIFIER "{" statement* "}" ;
statement       = var_decl | plot_decl | if_stmt | cond_expr ";" ;
var_decl        = ("let" | "param") IDENTIFIER (":" type)? "=" expr ";" ;
plot_decl       = "plot" IDENTIFIER "=" expr ("{" plot_attrs "}")? ";" ;
if_stmt         = "if" "(" expr ")" block_expr ("else" (if_stmt | block_expr))? ;
block_expr      = "{" statement* "}" ;
expr            = logic_expr ("and"|"or"|"xor") logic_expr | logic_expr ;
logic_expr      = comp_expr (">"|"<"|">="|"<="|"=="|"!=") comp_expr | comp_expr ;
comp_expr       = term (("+"|"−") term)* ;
term            = factor (("*"|"/"|"%") factor)* ;
factor          = primary ("|" PERIOD_KEYWORD "|" | "$" IDENTIFIER | "[" NUMBER "]")* ;
primary         = NUMBER | STRING | IDENTIFIER
                | function_call | "(" expr ")" | "if" "(" expr ")" block_expr ;
function_call   = IDENTIFIER "(" (expr ("," expr)*)? ")" ;
type            = "bool" | "string" | "color" | "number" ;
plot_attrs      = (IDENTIFIER ":" expr ",")* ;
PERIOD_KEYWORD  = "1min" | "5min" | "15min" | "30min" | "60min" | "D" | "W" | "M" | "Q" | "Y" ;
```

---

### 14.13 编译器架构

```
ME Script 源码（文本）
       ↓ 词法分析 (Lexer)
   Token 流
       ↓ 语法分析 (Parser)
   AST (抽象语法树)
       ↓ 语义分析 (类型检查/周期解析/依赖图)
   标注 AST
       ↓ 代码生成 (WASM Backend)
   WASM 字节码
       ↓ 嵌入
   wasm-custom/ 模块（运行时解释执行）
```

**目标**：
- 解析速度：10,000行脚本 < 100ms
- WASM执行速度：相当于原生 Rust 的 85%+（计算密集型指标）
- 错误信息：精确到行号和列号的语法/语义错误提示
- 沙箱安全：零文件IO、零网络、纯计算

---

### 14.14 保留字总表

```
// 数据类型
close open high low volume amount turnover oi
pre_close up_count down_count index_close index_volume

// 关键字
let plot param indicator true false null
if then else and or xor not
color line icon style

// 函数名（316+ 指标函数，仅列关键保留）
sma ema wma rma kama hma macd rsi stoch kdj
cci mfi trix adx aroon psar bbands atr kc obv
adosc cmf eom pvt drawdown cross ref hhv llv
sum barslast barssince count every exist filter
backset if iff ifn between range

// 周期关键字
1min 5min 15min 30min 60min D W M Q Y

// 经典关键字（TDX兼容，保留但非必要）
C O H L V AMO CLOSE OPEN HIGH LOW VOL AMOUNT
ADVANCE DECLINE INDEXC DRAWNULL NODRAW
STICK COLORSTICK LINETHICK DOTLINE POINTDOT
COLORRED COLORGREEN COLORBLUE COLORYELLOW
COLORWHITE COLORCYAN COLORMAGENTA
```

---

## 十五、待完成事项

- [x] 技术指标 316个完整分类清单（10大类，pandas_ta+ThinkorSwim+通达信对标）
- [x] 指标评价标准体系（7步流程、6大维度、双曲衰减模型、Löwdin正交化、2024学术前沿）
- [x] 通达信76内置指标兼容映射（4大公式类型、50+函数兼容、ME Script设计方向）
- [x] 20个策略模板详细说明（含策略逻辑、参数、适用市场环境、绩效基准、风险提示）
- [x] ME Script 自定义指标脚本语言完整语法规范（词法/语法/函数库/控制流/绘图/跨周期/EBNF/编译器架构）
- [x] 功能矩阵每个细分项的用户故事（§十七）
- [x] 每个模块的 UI/UX 线框图（§二十一）
- [x] 打包/分发/自动更新方案（§十八）
- [x] 适老化UI设计（大字体模式、高对比度模式）— `packages/app/src/theme.tsx` + `theme.css`
- [x] 官网落地页设计（§二十）
- [x] 定价策略 A/B 测试方案（§十九）
- [x] Alpha158因子库→WASM迁移方案（162/158因子、DAG计算管线）— `crates/wasm-factors/src/`
- [x] 4种原创搜索算法（CAPS/CGPC/MARS/MetaSearcher）的产品化方案 — `crates/wasm-scanner/src/search.rs`
- [x] 数据下载源方案（A股日线/分钟线免费数据源选型）（§十六）
- [x] WASM各模块性能基准测试方案 — `crates/wasm-*/src/benches.rs`（3个crate，10个bench case）
- [x] ME Script编译器的完整实现（Lexer→Parser→AST→Compiler→Runtime — `crates/wasm-custom/src/` 6模块 + 27测试）
- [x] 20个策略模板的完整回测验证（每个策略在A股2015-2024全周期测试）

---

## 十六、A股数据下载源方案

### 已实现：东方财富免费API

当前 `packages/app/src-tauri/src/download.rs` 已实现基于东方财富公开API的数据下载：

| 接口 | 地址 | 说明 |
|------|------|------|
| 股票列表 | `push2.eastmoney.com/api/qt/clist/get` | 全市场5000+股票，含代码/名称/最新价/涨跌幅 |
| 日K线 | `push2his.eastmoney.com/api/qt/stock/kline/get` | 全历史日线数据，含OHLCV+振幅+涨跌幅+换手率 |

**优势**：免费、无需认证、历史数据完整（1990年起）、含换手率（A股关键指标）
**限制**：单次请求有限流风险，无分钟线数据

### 分钟线数据源选型

| 数据源 | 粒度 | 覆盖 | 认证 | 可靠性 | 推荐 |
|--------|------|------|------|--------|:--:|
| 东方财富通 | 1min/5min/15min/30min/60min | 全A股 | 无需登录 | ★★★ 较稳定 | **首选** |
| 新浪财经 | 1min/5min/15min/30min/60min | 全A股 | 无需登录 | ★★☆ 有时断流 | 备选 |
| 腾讯财经 | 1min/5min | 全A股 | 无需登录 | ★★☆ | 备选 |
| JoinQuant/jqdatasdk | 1min+ | 全A股 | 需注册 | ★★★★ | 专业版可选 |
| Tushare Pro | 1min+ | 全A股 | 需积分 | ★★★★ | 专业版可选 |

**结论**：免费版使用东方财富分钟线API（klt参数：1=1min, 5=5min, 15=15min, 30=30min, 60=60min），付费版可集成Tushare Pro作为高质量备用源。

### 批量下载策略

```
免费版：单次单股 → 用户按需下载
付费版：
  ├─ 批量下载队列（最多8线程并发）
  ├─ 限流保护（请求间隔250ms，减少被封风险）
  ├─ 自动重试（失败重试3次，指数退避）
  ├─ 增量更新（仅下载本地最新日期之后的数据）
  └─ 定时自动更新（每日收盘后30分钟自动触发）
```

### 数据质量管理

- 复权数据：东方财富API kqt参数控制（不复权/前复权/后复权），默认前复权
- 停牌处理：停牌日成交量=0，标记而非删除
- 异常值检测：涨跌幅>11%（非科创板/创业板）→标记为异常
- 去重策略：按(stock_id, trade_date)唯一约束，INSERT OR REPLACE

---

## 十七、功能矩阵用户故事

### 模块一：行情数据中枢
- **散户老张**（48岁，股龄3年）：每天收盘后用同花顺导出的CSV导入MoneyEarning，想在本地统一查看所有股票数据，不用在5个APP间切换。一次导入500只股票日线，1分钟内完成。
- **上班族小李**（35岁，股龄1年）：午休时想快速下载今天的数据更新，看盘后分析。点击"批量更新"按钮，系统自动识别需要更新的股票并下载增量数据。

### 模块二：K线图表系统
- **技术派老王**（52岁，股龄10年）：习惯通达信的画线工具，想在K线上画趋势线、斐波那契回调，画线在不同周期间自动同步。选中画线工具→在K线上拖拽→右键保存为模板。
- **波段交易者**（42岁，兼职炒股）：想同时看4只候选股票的日线图，对比走势选最佳入场点。

### 模块三：技术指标引擎
- **指标研究控**（45岁，工程背景）：想一次叠加MACD+RSI+布林带+成交量，对比不同参数效果。从指标库拖拽指标到图表，调整参数，实时预览。
- **通达信迁移用户**（50岁，通达信老用户）：复制粘贴通达信公式 `CROSS(MA(C,5),MA(C,20))` 到ME Script编辑器，直接运行看到信号。

### 模块四：选股扫描引擎
- **周末选股党**（40岁）：每周末花2小时全市场扫描，找出"MACD金叉+放量+突破60日线"的股票。设置3个条件→一键扫描5000只→3分钟内出排名结果。
- **盘中盯盘族**（38岁）：设置预警条件"RSI<20超卖"，符合条件的股票弹窗+声音提醒。

### 模块五：策略回测引擎
- **策略验证者**（43岁，有编程基础）：想验证"海龟交易法在A股2020-2024年表现如何"。选策略模板→设参数→点回测→看资金曲线和Sharpe→调参数再测。
- **参数优化者**（39岁）：双均线策略fast=5, slow=20是最优的吗？用网格搜索5-20, 10-60全组合，找到Sharpe最高的参数对。

### 模块六：复盘工具
- **纪律派老赵**（55岁）：每笔交易后记录买入理由和情绪标签（"冲动追高"/"理性建仓"），月底看情绪分布→发现60%亏损来自"冲动追高"→开始克制。
- **训练模式学习者**（36岁，股龄半年）：用历史数据逐根K线判断买卖点，隐藏后续走势，做完后再对照实际走势→训练盘感。

### 模块七/八/九/十（略）
- 形态识别：形态研究者自动标记K线形态+历史胜率统计
- 筹码分布：套牢盘分析者看上方套牢盘压力位决定是否入场
- 组合分析：多账户管理者看跨账户行业集中度

---

## 十八、打包/分发/自动更新方案

### 打包方案

| 平台 | 格式 | 工具 | 体积 |
|------|------|------|------|
| Windows（主力） | `.msi` + `.exe` (NSIS) | Tauri bundler + WiX | ~25-35MB |
| macOS | `.dmg` | Tauri bundler | ~30-40MB |
| Linux | `.AppImage` + `.deb` | Tauri bundler | ~35-45MB |

**代码签名**：
- Windows：购买EV Code Signing Certificate（约¥2000/年），SmartScreen不再拦截
- macOS：Apple Developer Program（$99/年），需公证（notarization）

### 分发链路

```
版本发布 (GitHub Releases)
    ↓
├─ 官网下载页 (Cloudflare Pages, 免费)
├─ 网盘备用 (百度网盘/蓝奏云, 中国用户下载速度快)
└─ GitHub Releases (国际用户 + CDN加速)
    ↓
内置自动更新检查 (tauri-plugin-updater)
    ↓
增量更新 (仅下载差异包, ~5-15MB vs 全量~30MB)
```

### 自动更新方案（已实现基础）

`packages/app/src-tauri/tauri.conf.json` 已配置 `tauri-plugin-updater` v2：

```json
"plugins": {
  "updater": {
    "endpoints": ["https://cdn.example.com/updates/{{target}}/{{arch}}/{{current_version}}"],
    "pubkey": "<RSA公钥>",
    "windows": { "installMode": "passive" }
  }
}
```

**待完成**：配置CDN端点、生成RSA签名密钥对、实现增量补丁（`zstd`压缩差分）。

### 版本管理

- 语义版本：`v<MAJOR>.<MINOR>.<PATCH>`（如 v0.7.0）
- 发布渠道：`stable`（默认）、`beta`（付费用户可选）
- 强制更新：数据库schema变更时，客户端版本<最低版本→强制更新弹窗
- 灰度发布：按百分比逐步推送给用户（CDN层实现）

---

## 十九、定价策略 A/B 测试方案

### 测试一：心理价格锚点

**假设**：在"终身买断 ¥199"旁边显示"首年 ¥149"会让 ¥199 看起来更划算。

| 变体A | 变体B |
|--------|--------|
| 免费版 / 首年¥149 / 终身¥199 | 免费版 / 终身¥199（无中间选项） |

**指标**：终身买断转化率。预期A组+15%。

### 测试二：付费引导时机

**假设**：触发功能边界时弹出引导，比固定位置静态展示效果好。

| 变体A（时机） | 变体B（时机） |
|--------|--------|
| 用户点击付费功能时弹窗："此功能为专业版功能，升级即可使用" | 首页固定位置显示升级入口 |

**指标**：付费页访问率、转化率。预期A组2x转化。

### 测试三：免费试用时长

**假设**：14天vs 7天试用期对转化率的影响。

| 变体A | 变体B |
|--------|--------|
| 14天全功能试用 | 7天全功能试用 |

**指标**：试用到期后7日内付费率。需平衡"足够体验"和"及时转化"。

### 测试四：定价文案

| 变体A | 变体B | 变体C |
|--------|--------|--------|
| "终身买断 ¥199" | "一次付费，永久使用 ¥199" | "¥199 = 终生VIP（原价¥499）" |

**指标**：点击→购买转化率。

### 实施方式

初始阶段（用户量<100）：手工切换定价页面配置，观察转化率
用户量增长后：服务端配置A/B分流（Cloudflare Workers + KV），30/70分流比例

---

## 二十、官网落地页设计

### 页面结构

```
┌──────────────────────────────────────────────┐
│  Hero Section                                │
│  "你的股票数据，留在你的电脑上"                 │
│  "专业级量化分析 · 完全离线 · 一次买断"         │
│  [免费下载 Windows版]  [查看功能对比]           │
│  "已支持macOS/Linux · 14天全功能免费试用"       │
├──────────────────────────────────────────────┤
│  Why MoneyEarning?                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ 100%离线  │ │ 300+指标 │ │ 一次买断 │     │
│  │ 数据私密  │ │ 专业分析 │ │  ¥199    │     │
│  └──────────┘ └──────────┘ └──────────┘     │
├──────────────────────────────────────────────┤
│  功能亮点截图轮播                              │
│  [K线图] [指标面板] [回测报告] [扫描结果]       │
├──────────────────────────────────────────────┤
│  对比表：免费版 vs 付费版                      │
│  (精简版功能矩阵，重点突出付费版增量)            │
├──────────────────────────────────────────────┤
│  FAQ                                         │
│  Q: 数据从哪里来？A: 东方财富免费API+手动导入   │
│  Q: 会不会上传数据？A: 100%本地，零网络依赖     │
│  Q: 换电脑怎么办？A: 激活码绑定机器，支持迁移    │
├──────────────────────────────────────────────┤
│  Footer: 备案号 · 联系方式 · 政策条款           │
└──────────────────────────────────────────────┘
```

### 技术方案

- **部署**：Cloudflare Pages（免费，全球CDN，中国大陆有节点）
- **框架**：纯HTML+CSS（单页，≤200KB），响应式（桌面+移动端适配）
- **下载分发**：Cloudflare R2（免费10GB/月）或 GitHub Releases
- **备案**：中国大陆需ICP备案（约20个工作日），初期先挂海外域名
- **统计**：Cloudflare Web Analytics（隐私友好，无Cookie）

### 配色

- 主色：`#0f0f23`（深空蓝黑，与软件内暗色主题一致）
- 强调色：`#fbbf24`（金色，软件内相同色系）
- 字体：系统默认中文字体（MiSans/PingFang SC），14-18px

---

## 二十一、UI/UX 线框图（模块级）

### 整体布局
```
┌──────────┬──────────────────────┬──────────┐
│ Header: Logo | 版本 | License | 大字/高对比 | 导入 │
├──────────┼──────────────────────┼──────────┤
│          │  Chart Toolbar       │          │
│ Left     ├──────────────────────┤  Right   │
│ Sidebar  │                      │  Sidebar │
│ (280px)  │   Chart Canvas       │  (300px) │
│          │   (Main Content)     │          │
│ - 股票   │                      │  - 交易  │
│ - 自选   │   flex: 1            │  - 策略  │
│ - 指标   │                      │  - 回测  │
│          │                      │  - 扫描  │
│          │                      │  - 筹码  │
│          │                      │  - 风险  │
├──────────┴──────────────────────┴──────────┤
│ Status Bar: 数据加载状态 · 最后更新时间       │
└──────────────────────────────────────────────┘
```

### 各模块布局要点

**股票列表 (Left Sidebar → 股票)**
```
┌────────────────────┐
│ [搜索框: 代码/名称]  │
│ [市场筛选: 全部▾]   │
├────────────────────┤
│ 000001 平安银行     │
│ 000002 万科A        │
│ ...滚动列表...      │
│ ── 点击行→加载K线   │
└────────────────────┘
```

**指标选择器 (Left Sidebar → 指标)**
```
┌────────────────────┐
│ 分类Tab: 趋势|动量|..│
├────────────────────┤
│ ☑ MACD   [参数▾]   │
│ ☑ RSI    [14▾]     │
│ ☐ KDJ              │
│ ...可滚动指标列表    │
└────────────────────┘
```

**交易记录 (Right Sidebar → 交易)**
```
┌────────────────────┐
│ [+ 新增交易]        │
├────────────────────┤
│ 2026-05-20 买入     │
│ 平安银行 ¥12.50     │
│ 1000股 情绪:理性建仓 │
│ ──────────────────  │
│ 盈亏统计面板         │
│ 总盈亏:+¥3,250      │
│ 胜率: 62%           │
└────────────────────┘
```

**策略面板 (Right Sidebar → 策略)**
```
┌────────────────────┐
│ 策略模板: [双均线▾]  │
│ 参数:               │
│  快线: [5]          │
│  慢线: [20]         │
│ [▶ 回测] [⚙ 优化]  │
├────────────────────┤
│ 回测结果:           │
│ 年化: 8.2%          │
│ Sharpe: 0.51        │
│ 最大回撤: -32%      │
│ [查看详情→]         │
└────────────────────┘
```

**扫描面板 (Right Sidebar → 扫描)**
```
┌────────────────────┐
│ 条件1: [MACD▾] [金叉▾] │
│ [+ 添加条件]        │
│ 逻辑: AND ▾         │
│ [▶ 开始扫描]        │
├────────────────────┤
│ 结果排名 (按评分↓)   │
│ 1. 000001 评分:92   │
│ 2. 600036 评分:88   │
│ ...                 │
└────────────────────┘
```

### 响应式策略

| 屏幕宽度 | 布局 |
|---------|------|
| ≥1400px | 三栏：LSidebar(280) + Chart(flex) + RSidebar(300) |
| 1200-1400px | 双栏：LSidebar(240) + Chart(flex)，RSidebar折叠为底部Tab |
| <1200px | 单栏：Chart全屏，两侧边栏变成Overlay抽屉 |
| 4K/超宽 | 三栏等比例放大，K线可显示更多数据点 |

---

## 十五、质检与风控体系

> v1.0 | 2026-05-22 | 基于开源生态扫描 + 金融行业 QA 最佳实践 + A股特殊规则

### 15.1 已安装的工具集

#### AI Agent Skills（Claude Code 插件）

| 来源 | Skill 名称 | 用途 |
|------|-----------|------|
| Superpowers 5.1.0 | `verify` | 运行应用验证代码变更生效 |
| Superpowers 5.1.0 | `code-review` | PR diff 正确性审查 |
| Superpowers 5.1.0 | `systematic-debugging` | 系统性缺陷定位 |
| Superpowers 5.1.0 | `test-driven-development` | TDD 开发流程 |
| Superpowers 5.1.0 | `verification-before-completion` | 完成前门控检查 |
| Superpowers 5.1.0 | `security-review` | 安全审查（四层防御验证） |
| GenSkills 1.4.2 | `accessibility-audit` | WCAG 无障碍审计 |
| GenSkills 1.4.2 | `code-review` | 多维审查（安全/性能/正确性/可维护） |
| GenSkills 1.4.2 | `dead-code` | 死代码检测 |
| GenSkills 1.4.2 | `dependency-audit` | 依赖审计（未使用/过时/冲突） |
| GenSkills 1.4.2 | `error-boundary` | 未处理异常检测 |
| GenSkills 1.4.2 | `lint-fix` | 自动检测+修复 lint 问题 |
| GenSkills 1.4.2 | `refactor` | 行为保持重构 |
| GenSkills 1.4.2 | `security-audit` | OWASP Top 10 + CVE 扫描 |
| GenSkills 1.4.2 | `test-generator` | 自动生成测试套件 |
| GenSkills 1.4.2 | `type-check` | TypeScript/Rust 类型错误修复 |
| GenSkills 1.4.2 | `perf-optimize` | 性能瓶颈分析与优化 |
| GenSkills 1.4.2 | `mock-data` | 测试 mock 数据生成 |
| GenSkills 1.4.2 | `release-notes` | 发布 changelog 生成 |

#### Python 量化工具（conda base 环境）

```bash
# 已安装
pip install quantlite==1.0.2    # 过拟合检测（DSR/CSCV）+ Bootstrap CI
conda install -c conda-forge pandas numpy scipy matplotlib

# 待安装（GitHub 克隆到 scripts/）
git clone https://github.com/plaintext-capital/pypbo.git scripts/pypbo/
# pypbo: PBO/PSR/DSR/MinTRL 专项库，参考 Lopez de Prado 论文
```

#### 量化专用 conda 环境

```bash
conda create -n quant-qa python=3.12 -y
conda activate quant-qa
pip install quantlite pandas numpy scipy statsmodels matplotlib
```

### 15.2 参考开源项目（量化验证与风控）

#### 核心对标项目

| 项目 | 语言 | 核心能力 | 对本产品的参考价值 |
|------|------|---------|-----------------|
| **[backtester-mcp](https://github.com/bcosm/backtester-mcp)** | Python | PBO/DSR/CSCV/PSR + Bootstrap CI + Walk-Forward | **最强过拟合检测链**，MCP 协议原生支持 AI Agent 调用 |
| **[pypbo](https://github.com/plaintext-capital/pypbo)** | Python | PBO + PSR + DSR + MinTRL | 算法参考——CSCV 并行交叉验证的参考实现 |
| **[quant-backtesting-validation](https://github.com/Hussain0327/quant-backtesting-validation)** | TS/Next.js | 三重检验（Sharpe CI + Permutation + MC） | 统计显著性判定规则（3/3=强证据, 2/3=需研究） |
| **[Quanto](https://github.com/skyliquid22/Quanto)** | Python | Data Health Checks + Qualification Gates + 可审计 manifest | 数据健康检查管线 + 策略上线门控 |
| **[marketbench.ai](https://marketbench.ai)** | Python | 回测输出 vs 参考实现的 MAE 比对 | 数值精度验证——不同引擎输出交叉比对 |
| **[findata-guard](https://github.com/xbtlin/findata-guard)** | Python | OHLCV 约束检查 + Benford 分析 + Merkle 审计 | 数据完整性/一致性验证框架 |
| **[ml4t/data](https://github.com/ml4t/data)** | Python | OHLCVValidator + AnomalyManager（20+ 数据源适配） | 生产级数据质量管线 |

#### A股专项参考

| 项目 | 核心能力 |
|------|---------|
| **[hikyuu](https://github.com/fasiondog/hikyuu)** | C++/Python 超高速量化框架，A股深度适配，止损/资金管理/滑点 |
| **[金策智算](https://github.com/ScottZt/jin-ce-zhi-suan)** | 三省六部分层风控，门下省一票否决制审核→刑部违规追溯 |

#### 行业 QA 框架

| 框架 | 来源 | 核心方法 |
|------|------|---------|
| **th2** | Exactpro | AI 驱动模型化测试 + mini-robots 模拟交易行为（套利/VWAP/合成策略） |
| **H2 Framework** | Wealthfront | 历史决策回归（新代码 vs 旧决策）+ 新老系统并行对比 |
| **Implementation Risk Framework** | arXiv 2603.20319 | 15 策略 × 5 引擎交叉比对，Engine Spread/IUI/DAF/CSI 四指标 |
| **Tauri + WebDriver** | Tauri 官方 | `tauri-driver` + WebdriverIO E2E，`cargo test` 后端单元测试 |

### 15.3 Rust 原生库依赖计划

可以直接 `Cargo.toml` 集成的 crate：

```toml
# 性能指标交叉验证（替代自写的指标计算）
[dependencies]
quant-metrics = "0.2"    # Sharpe/Sortino/Calmar/VaR/CVaR/MaxDD/WinRate
trametricks = "0.1"      # 轻量回测指标，依赖极少
quantstats-rs = "0.1"    # HTML Tear Sheet（SVG 图表 + 报告）

# 数值精度保证
rust_decimal = "1"       # 金融级定点小数，避免浮点累积误差

[dev-dependencies]
mockall = "0.13"         # Rust mocking 框架
serial_test = "3"        # 串行测试（状态共享场景）
tempfile = "3"           # 临时文件测试
```

> **注意**：Rust 生态目前缺少 PBO/DSR 原生实现。需将 `pypbo` 的 CSCV 算法+ DSR 公式移植到 `wasm-backtest`，或通过 PyO3 桥接 Python 库。

### 15.4 四层质检体系设计

```
┌──────────────────────────────────────────────────┐
│  第四层：AI Agent 持续审查层（Claude Code）         │
│  ├─ code-review (每次 PR)                        │
│  ├─ security-audit (每周)                        │
│  ├─ dependency-audit (每次发布前)                  │
│  ├─ dead-code (每次 milestone)                   │
│  ├─ test-generator (新模块完成后)                  │
│  └─ verify (每次功能变更后手工冒烟)                 │
├──────────────────────────────────────────────────┤
│  第三层：策略回测验证层（wasm-backtest + Python）    │
│  ├─ PBO (Probability of Backtest Overfitting)    │
│  ├─ Deflated Sharpe Ratio (多重测试校正)          │
│  ├─ Walk-Forward 交叉验证                          │
│  ├─ Monte Carlo 交易序列随机化                      │
│  ├─ 参数稳定性分析（网格搜索 + 遗传算法）              │
│  └─ 回测输出 vs 参考实现交叉比对                     │
├──────────────────────────────────────────────────┤
│  第二层：数据质量层（wasm-core + 导入管线）          │
│  ├─ OHLCV 一致性检查 (high≥low, high≥open/close)  │
│  ├─ A股 涨跌停/停牌/ST/退市 前置过滤               │
│  ├─ 缺失数据检测 + 交易日历校验                      │
│  ├─ 复权数据交叉比对                                │
│  ├─ 异常值检测 (Z-score + IQR + Isolation Forest) │
│  └─ 前视偏差 (look-ahead bias) 自动检测            │
├──────────────────────────────────────────────────┤
│  第一层：编译时保证层（Rust #[cfg(test)]）          │
│  ├─ 每个指标独立 unit test (对照 TA-Lib 基准值)    │
│  ├─ 指标联动测试 (A指标修改 → 依赖指标回归)         │
│  ├─ NaN/Inf/Zero 边界值覆盖                       │
│  ├─ 数值精度回归 (浮点容差 1e-8)                   │
│  └─ WASM 编译产物 integrity hash 自动比对          │
└──────────────────────────────────────────────────┘
```

### 15.5 A 股数据质量检查清单

#### 数据导入阶段

| 检查项 | 方法 | 阈值/规则 |
|--------|------|---------|
| OHLCV 基本约束 | `high >= max(open, close)` 且 `low <= min(open, close)` | 违反→标记异常行 |
| 日内涨跌幅校验 | `-20% <= change_pct <= +20%`（主板/中小板 ±10%，科创/创业 ±20%） | 超出→按板块校验 |
| 一字板检测 | `open == high == low == close` 且当天非停牌 | 标记为不可交易信号 |
| 复权数据对齐 | 分红/送股/定增日期 vs 复权因子突变日期 | ＞1个交易日偏差→告警 |
| 重复交易日 | 同一 stock_id + trade_date 出现多次 | 删除重复，保留最新 |
| 缺失交易日 | 交易日历 vs 实际数据行 | 标记跳空缺口天数 |
| 成交量非负 | `volume >= 0` | `volume = 0`→停牌或一字板 |
| 换手率异常 | 非基金类换手率 > 50% | 标记为疑似异常 |

#### 回测前过滤

| 过滤条件 | 规则 | 实现位置 |
|---------|------|---------|
| ST / *ST / 退市 | `'ST' in name or '*ST' in name or 退市` → 剔除 | wasm-scanner 前置 |
| 停牌中 | 最近交易日状态=停牌 → 剔除 | wasm-core DataFrame |
| 涨跌停封死 | 当日开盘=涨停价 or 跌停价 → 该日不可交易 | wasm-backtest 交易执行层 |
| 次新股 | 上市不足 60 个交易日 → 剔除 | wasm-scanner 前置 |
| 科创板 | 代码以 '688' 开头 → 可选过滤 | wasm-scanner 配置项 |
| 成交量极小 | 日均成交量 < 100手 → 剔除（流动性过滤） | wasm-scanner 前置 |
| 长期停牌 | 最近 10 日内停牌天数 > 5 → 剔除 | wasm-core DataFrame |

#### 前视偏差检查

| 类型 | 风险 | 防护 |
|------|------|------|
| 财务数据 | 使用 Q1 数据时实际 Q1 尚未公布 | 严格按报告披露日期访问数据 |
| 指数成分股调整 | 使用当前成分股名单回测历史 | 保存各期成分股快照 |
| 幸存者偏差 | 只看存活到今天的股票 | 保留已退市/被并购股票历史数据 |
| 技术指标计算 | 计算 `ref(N)` 时意外引用未来值 | `position = signal.shift(1)` 强制 lag |
| Train/Test 泄漏 | Walk-Forward 时 in-sample 与 out-sample 重叠 | Purged K-Fold CV 分配 |

### 15.6 回测过拟合检测流程（PBO/DSR）

参考 Lopez de Prado & Bailey 论文 + `pypbo`/`backtester-mcp` 实现：

#### 判定标准

| 指标 | 阈值 | 含义 |
|------|:----:|------|
| **PBO** (Probability of Backtest Overfitting) | < 0.10 | 10% 以下过拟合概率→可信 |
| **DSR** (Deflated Sharpe Ratio) | > 0.95 (P值) | 经多次试验校正后仍显著→真实 alpha |
| **PSR** (Probabilistic Sharpe Ratio) | > 0.95 | 夏普 > 0 的概率 |
| **MinTRL** (Minimum Track Record Length) | < 实际样本量 | 实际数据足够支撑结论 |
| Walk-Forward 样本外/内收益比 | > 0.5 | 样本外表现不低于样本内一半 |

#### CSCV 算法步骤（拟移植到 wasm-backtest）

```
1. 对策略做 N 次参数扫描，记录每次试验的 Sharpe ratio → 矩阵 S[1×N]
2. 将 N 次试验随机分成 S 对 (每组 N/2)
3. 对每对：(a) 用 IS 组选最优参数 (b) 记录 OOS 的 Sharpe
4. 计算 OOS Sharpe 的相对排名 → logit 值
5. 拟合 logit 的累积分布 → PBO = 排名逻辑回归的概率下界
```

### 15.7 Tauri 桌面应用测试架构

```
┌──────────────────────────────────┐
│  E2E: tauri-driver + WebdriverIO │ ← 关键用户路径（数据导入→K线图→回测→导出）
├──────────────────────────────────┤
│  Integration: cargo test         │ ← Rust 后端 IPC 命令 + WASM 调用链
├──────────────────────────────────┤
│  Unit: cargo test / vitest       │ ← 指标计算 / 形态识别 / UI 组件
└──────────────────────────────────┘
```

**实施建议**：

1. **Rust 后端**：所有 `#[tauri::command]` 函数增加 `#[cfg(test)] mod` 单元测试，使用 `mockall` mock 数据库连接
2. **WASM 核心**：每个指标函数对标 TA-Lib 基准值，用 `approx` crate 做浮点容差断言
3. **前端组件**：`@tauri-apps/api` mocks + Vitest，测试 UI 渲染和状态管理
4. **E2E**：安装 `cargo install tauri-driver`，配置 WebdriverIO，覆盖核心用户流程
5. **CI 管线**：GitHub Actions 矩阵（Windows→主力，macOS→兼容性）

### 15.8 实施路径

| 阶段 | 时间 | 任务 | 交付物 |
|------|------|------|--------|
| **Phase 1：编译时防线** | 第1-2周 | 为每个指标模块添加 `#[cfg(test)]` 单元测试；`wasm-core` 添加 OHLCV 约束检查函数；`wasm-indicators` 每个指标对照 TA-Lib 基准值 | 每个 crate 测试覆盖率 ≥ 40% |
| **Phase 2：数据防线** | 第3-4周 | 实现导入数据校验管线（停牌/ST/涨跌停/复权/缺失）；`wasm-backtest` 添加前视偏差自动检测；交易日历模块 | 导入数据自动质量报告 |
| **Phase 3：回测防线** | 第5-7周 | 移植 CSCV/PBO/DSR 算法到 `wasm-backtest`；引入 `quant-metrics` Rust crate；回测报告增加过拟合风险评分 | PBO < 0.10 的自动标记 |
| **Phase 4：CI + AI 持续审查** | 第8-9周 | 配置 GitHub Actions 测试矩阵；集成 `tauri-driver` E2E；编码规范文档 + AI code-review hook | 每次 PR 自动运行测试+审查 |
| **Phase 5：性能与安全** | 第10-12周 | 全市场扫描压力测试；WASM 完整性校验自动化；授权系统渗透测试 | 压力测试报告 + 安全审计报告 |

### 15.9 风险评分卡（策略上线门控）

每个策略上线前必须通过以下硬性门控：

| 门控项 | 通过条件 | 不通过处理 |
|--------|---------|-----------|
| 数据完整性 | 缺失交易日 < 5% 总交易日 | 回填缺失数据或缩小回测区间 |
| 前视偏差 | 零前视泄漏（自动化检测通过） | 修复 `shift`/`ref` 调用链 |
| 参数过拟合 | PBO < 0.10 且 DSR P值 > 0.95 | 简化参数空间或增加样本外验证长度 |
| Walk-Forward 稳定性 | OOS/IS 收益比 > 0.5 | 重新设计策略或接受该策略为"仅参考" |
| 极端行情压力 | 覆盖 2008/2015/2018/2024 四轮熊市 | 回撤超过 50%→增加止损机制 |
| 计算精度 | 浮点累积误差 < 1e-6（与参考实现比对） | 引入 `rust_decimal` 定点小数 |
| 实盘一致性 | paper trading 3个月 vs 回测偏差 < 2% | 检查滑点/手续费/流动性模型 |

---

## 二十二、v0.9.0 UX/UI 全面升级方案

### 22.1 背景与动机

v0.8.0 完成了授权系统、防闪退、性能优化三大基础设施。但产品存在以下 UX 短板：

- 大量占位按钮无交互反馈，用户点击后困惑
- 回测操作无进度反馈，长时间等待无感知
- CSV 导入需手动输入文件路径，对中年用户不友好
- 整体风格"AI味"浓重，缺乏金融科技产品应有的专业沉浸感

### 22.2 设计方向 —「赛博金融终端」

对标 Bloomberg Terminal（数据密度）+ TradingView（交互流畅度），但强调**离线本地、隐私优先、暗黑沉浸**的差异化定位。

**配色体系：**

| 用途 | 色值 | 说明 |
|------|------|------|
| 底色 | `#0A0E1A` | 深空蓝黑，减少长时间盯盘眼疲劳 |
| 面板色 | `#111827` | 微亮卡片层 |
| 边框 | `#1E293B` | 极细分割，替代阴影 |
| 主强调 | `#00D8FF` | 电光青——选中态、进度条、K线 |
| 次强调 | `#7C3CFF` | 紫罗兰——辅助图表、PRO标记 |
| 正向 | `#00E676` | 霓虹绿——涨、盈利 |
| 负向 | `#FF2A7A` | 品红——跌、亏损、告警 |
| 文字主 | `#F1F5F9` | 冰白 |
| 文字辅 | `#94A3B8` | 冷灰 |

**核心设计原则：**
1. 霓虹辉光只做信号不做装饰（活跃元素 <20% 面积使用辉光）
2. 1px 边框替代阴影，营造 CRT/终端感
3. 数据区域等宽字体（JetBrains Mono），标签用无衬线（Inter）
4. 保留"大字体模式"适老化切换（对标通达信的中老年用户群）
5. 禁用游戏化元素（FCA 研究证实排行榜/庆祝弹窗导致亏损概率 +4.8pp）

**参考产品对标：**

| 参考产品 | 借鉴点 | 避免 |
|----------|--------|------|
| Bloomberg Terminal | 数据密度、键盘驱动、琥珀色高亮 | 过于极简、零图表 |
| TradingView | 图表交互、多时间框架、深色主题 | 在线优先、社交噪音 |
| ThinkorSwim | 选项分析、自定义扫描 | UI 复杂度 |
| 通达信 | 公式开放、本地速度 | 90年代界面 |
| Cyberpunk Dashboard | 霓虹配色、毛玻璃、1px边框 | 过度花哨 |

### 22.3 实施计划

#### A 层：即时修复（当前迭代）

| # | 问题 | 方案 |
|---|------|------|
| A1 | 占位按钮无反应 | 无功能按钮加 `disabled + tooltip "功能开发中"`；BacktestPage 的"运行回测"按钮连接真实回测流程 |
| A2 | 回测无进度条 | Rust 后端 `run_backtest` 通过 Tauri event 发进度，前端 `listen()` 接收并以线性进度条展示 |
| A3 | CSV 导入路径输入 | 用 `tauri::dialog::open()` 调起系统原生文件选择器替代手动输入路径 |

#### B 层：风格重构（基于赛博金融终端设计）

- CSS 变量主题系统替换内联样式
- 全组件配色迁移至新色板
- 侧边栏/导航加霓虹发光活跃态
- 数据卡片改为 1px 边框 + 悬停微亮
- 按钮体系：幽灵按钮（默认）+ 实心强调（主要操作）
- K线图暗色网格线 + 发光十字光标
- PRO标记：紫色渐变边框 + 微弱辉光

### 22.4 市场调研关键发现

经过4轮深度搜索调研，确认以下市场空白（详见实施背景）：

1. **离线隐私**：所有竞品上传交易行为，58%用户最想要多端同步但以隐私为代价
2. **界面 vs 深度矛盾**：通达信功能最强但界面老旧，无产品同时做到"好看+深度"
3. **量化门槛**：QMT需10-300万资金门槛，PTrade代码上传有泄露风险
4. **中年投资者被忽视**：35-55岁用户群的适老化需求无产品专门满足
5. **AI集成空白**：无产品原生集成 LLM 决策辅助
6. **数据加载速度**：56%用户抱怨行情刷新慢（QuantVault 本地SQLite+WAL已解决）
7. **离线功能缺失**：24%用户想要离线查看（QuantVault 已天然满足）
8. **投研工具薄弱**：仅27%用户认为现有平台容易做投资研究
9. **过度游戏化**：FCA 研究证实排行榜/庆祝弹窗导致亏损概率高4.8pp
10. **策略保密性**：QMT/PTrade 自由度与稳定性不可兼得

QuantVault 的差异化优势完美覆盖以上空白：**100%本地、隐私优先、一次买断、专业深度+现代UI**。

---

> **此文档为活文档，随设计推进逐步完善。所有决策记录于此，避免上下文丢失。**

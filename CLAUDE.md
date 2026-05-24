# MoneyEarning — 本地量化分析工作站

一个面向中国大陆中年股票投资者的完全本地离线桌面量化分析工具。从选股→技术分析→策略回测→交易复盘→组合管理的一站式工作流。

产品定位详见 `GOAL_SPEC.md`。

## 技术栈

- **桌面框架**：Tauri 2.x（Rust + React/TypeScript）
- **核心计算**：Rust → WASM（技术指标、回测、选股扫描全部在 WASM 沙箱执行）
- **数据存储**：SQLite（本地单文件数据库）
- **图表渲染**：自研 Canvas + WebGL 引擎（对标 TradingView 交互体验）
- **代码保护**：四层防御（Rust原生二进制 → JS混淆 → WASM加密嵌入 → RSA离线授权）
- **平台支持**：Windows（主力）+ macOS + Linux（暂缓）

## Agent skills 配置

### Issue tracker（问题追踪器）

所有 Issue 和 PRD 存放在本项目的 **私有** GitHub 仓库中。使用 `gh` CLI 进行所有操作。详见 `docs/agents/issue-tracker.md`。

### Triage labels（分类标签）

使用默认五标签体系：`needs-triage`、`needs-info`、`ready-for-agent`、`ready-for-human`、`wontfix`。详见 `docs/agents/triage-labels.md`。

### Domain docs（领域文档）

单上下文布局——根目录一个 `CONTEXT.md` + `docs/adr/`。详见 `docs/agents/domain.md`。

## 踩坑经验

### ICO 图标格式（Windows RC 编译器兼容性）

用 Python Pillow 生成 .ico 时，**必须使用 PNG 压缩格式（Vista-style）**，不能写原始 DIB BMP 数据。
Windows RC 编译器 (`rc.exe`) 遇到旧式 DIB 会报 `RC2176: old DIB in icon.ico; pass it through SDKPAINT`。

正确做法：手动构造 ICO 容器，每个条目嵌入完整的 PNG 数据流（带头 `\x89PNG`），而不是 BITMAPINFOHEADER+像素数据。
Pillow 的 `save(format='ICO', sizes=[...])` 只输出单分辨率，不可用。
参考实现：`create_ico_proper()` 函数，手动写 ICO 文件头 + 目录项 + PNG 数据块。

### Landing 页面下载按钮

面向中国大陆用户的下载，必须考虑：
- **直链 .exe**：用 JS 下载处理器触发下载，国产浏览器兼容性已足够
- **用绝对 URL**：`https://zn070515.github.io/MoneyEarning/xxx.exe`，不用相对路径，避免路径歧义
- **加 JS 下载处理器**：拦截点击 → 显示 spinner + "下载中..." 反馈 → 程序化创建 `<a download>` 触发下载。国产浏览器（QQ、360、搜狗等）对 HTML `download` 属性支持参差不齐，JS 触发更可靠
- **加 Toast 提示**："下载已开始，请查看浏览器下载列表"，避免用户以为没反应
- **安装包同步更新**：每次 CI 构建新版本后，必须手动下载产物 → 替换 landing 目录的 .exe/.msi → 删除旧版 → commit push

### Git 远程：SSH 不用 HTTPS（中国大陆）

GitHub HTTPS (443) 在大陆经常超时/被重置。用 SSH：
```bash
git remote set-url origin git@github.com:Zn070515/MoneyEarning.git
```

### Tauri CI 路径（workspace 结构）

pnpm workspace 下 Tauri 2 的产物在 `<repo_root>/target/release/bundle/`，不是 `packages/app/src-tauri/target/`。
CI 里 `pnpm tauri build` 的 `working-directory` 必须设为 `packages/app`。

### Tauri updater 插件

除非有真实的更新服务器和有效 RSA 密钥对，否则**不要加** `tauri-plugin-updater`。
占位符 pubkey + 不存在的 endpoint 会导致应用启动后几秒闪退（auto-check 失败）。如果暂时不用，必须：
1. 从 `Cargo.toml` 移除依赖
2. 从 `lib.rs` 移除 `.plugin(tauri_plugin_updater::...)` 
3. 从 `capabilities/default.json` 移除 `updater:*` 权限
4. 从 `tauri.conf.json` 移除 `plugins.updater` 配置块

### SQLite WAL 模式 + 优化 PRAGMA

启动时在 migration 中设置以下 PRAGMA 可大幅提升并发性能：
- `journal_mode=WAL` — 读写不再互斥，多读者+一写者并发
- `busy_timeout=5000` — 遇到锁自动等待5秒而非立即报错
- `synchronous=NORMAL` — 写入性能 2-3x 提升（WAL 模式下安全）
- `cache_size=-8000` — 8MB 页缓存，减少磁盘 I/O
- `foreign_keys=ON` — 外键约束必须显式开启

### 授权缓存设计（防并行竞态）

`LICENSE_CACHE` 必须是 `Mutex<Option<LicenseStatus>>` 而不能只存 pro 状态。原因：
- 多个 Tauri 命令并行调用时各自获取 DB 锁检查授权
- 如果缓存只存 pro 状态，trial/free 每次都走 DB → 锁竞争导致超时/崩溃
- **所有状态（pro/trial/free/expired）都要缓存**，且 `drop(guard)` 必须在 cache 写入之前

### Professional Terminal 主题迁移注意事项

从霓虹风格迁到专业终端风格时，批量替换 CSS 变量容易遗漏：
- `rgba(0, 216, 255` 带空格的情况 regex 匹配不到，需单独检查
- 删除发光效果比改颜色更重要（`boxShadow`、`glow`、`gradient` 全部移除）
- 等宽字体全局统一后，行高需要微调（monospace 默认行高比 Inter 大）
- 保留无障碍模式的所有变量不变

### Demo 数据预置模式

`seed_demo_data` 函数放在 `run_migrations` 末尾，migration 之后执行：
- 先 `SELECT COUNT(*) FROM stocks`，为 0 才写入（幂等）
- 用 sin/cos 伪随机游走生成合成 OHLCV，确定性强且可重现
- 只写 stocks + daily_prices 两张表，不写 trades/watchlists
- 用 `println!` 而非 `log::info!`（log crate 未引入）

### 预警系统架构

SQLite `alert_rules` 表 + Rust 条件评估函数的组合模式：
- 三种条件类型：`price_breakout`（价格阈值）、`ma_cross`（快慢线交叉）、`volume_spike`（量能异常）
- 参数存 JSON（`params TEXT`），Rust 端 `serde_json::from_str` 解析
- `check_alerts()` 逐条加载 enabled 规则，查询该股票最近100条日线，chronological sort 后评估
- 已触发规则写入 `last_triggered` 防重复通知
- 前端通过 Tauri `app.emit("alert:triggered", ...)` 接收实时事件

### Textarea 内嵌自动完成

在 monospace textarea 之上做 autocomplete dropdown 的要点：
- 监听 `onChange` 用 regex `([A-Za-z_]{2,})$` 从光标位置前文本提取前缀
- dropdown 用 `position: absolute` 相对 textarea 容器定位（`top = row * lineHeight`，`left ≈ col * charWidth`）
- 拦截 `onKeyDown`：Tab/Enter 接受，Escape 关闭，Arrow Up/Down 导航
- `acceptAutocomplete()` 计算 wordStart 位置做字符串替换，`setTimeout` 恢复光标位置

### 多渠道图表栅格（2×2 Grid）

多图表布局采用 `gridCells: GridCellData[]` 状态数组 + `activeCellIdx` 模式：
- 每个 cell 独立存储 stockId/stockCode/data/indicators/drawings
- 点击 cell 设其为 active（金色边框），左侧 sidebar 的选股操作自动填充 active cell
- `loadGridCellData()` 异步加载数据并合并到对应 index 的 cell
- 单图/网格模式切换通过 chartStore 的 `gridMode` boolean，不影响已有的 single-chart 路径

### 前端 CSV 导入导出

面向中国大陆 Excel 用户的 CSV 处理注意事项：
- 写 BOM `"﻿"`（`﻿`）在 CSV 文件头以保证 Excel 正确识别 UTF-8 编码
- `blob.type` 设为 `"text/csv;charset=utf-8"`
- 导入时用 `\r?\n` split 兼容 Windows/Mac 换行
- 数值列用 `parseFloat` + `isNaN` 校验，跳过非法行而非整体失败

### 版本号规则

修改版本号时严格遵循 `docs/VERSIONING.md`。核心原则：**小改动（bug 修复、UI 调整、性能优化）只动第三位（PATCH），新功能才动第二位（MINOR）。** 每次发版需同步更新 `package.json`、`tauri.conf.json`、`Cargo.toml` 三处版本号。

### 变更后多Agent健康审计工作流

每次代码改动（新功能、bug修复、重构）完成后，**必须**执行以下审计循环：

1. **启动 3 个 Agent 并行审计**：
   - Agent 1：WASM 审计 — 扫描 `crates/` 下所有 `.rs` 文件，逐函数检查 panic/unwrap/索引越界/除零/整数溢出
   - Agent 2：Rust 审计 — 扫描 `packages/app/src-tauri/src/` 下所有 `.rs` 文件，检查 unwrap/锁竞争/未处理 Result/命令注入/license 边界
   - Agent 3：前端审计 — 扫描 `packages/` 下所有 `.ts`/`.tsx` 文件，检查 try/catch 缺失/null 解引用/类型断言/内存泄漏

2. **汇总发现** → 按严重性排序（CRITICAL > HIGH > LOW）

3. **修复全部 CRITICAL 和 HIGH 问题**，LOW 问题也一并修复

4. **重跑审计** — 修复后再次启动 3 Agent 审计，确认 0 个 CRITICAL/HIGH/LOW 问题

5. **循环上限 3 轮** — 若 3 轮后仍有残余问题，记录到 `docs/known-issues.md` 并继续

6. **检查通过标准**：0 CRITICAL + 0 HIGH + 0 LOW，TypeScript 编译无错误，Rust `cargo check` 无错误

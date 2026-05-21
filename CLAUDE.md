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

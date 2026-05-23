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
- **用 .zip 包，不要直链 .exe**：浏览器和国产杀毒软件不拦截 .zip 文件
- **用绝对 URL**：`https://zn070515.github.io/MoneyEarning/xxx.zip`，不用相对路径，避免路径歧义
- **加 JS 下载处理器**：拦截点击 → 显示 spinner + "下载中..." 反馈 → 程序化创建 `<a download>` 触发下载。国产浏览器（QQ、360、搜狗等）对 HTML `download` 属性支持参差不齐，JS 触发更可靠
- **加 Toast 提示**："下载已开始，请查看浏览器下载列表"，避免用户以为没反应
- **安装包同步更新**：每次 CI 构建新版本后，必须手动下载产物 → 替换 landing 目录的 .exe/.msi → 重新 zip → commit push

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

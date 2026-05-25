# QuantVault 项目复盘

> 2026-05-25 | 从立项到开源，完整经验总结

---

## 一、做了什么

一个 Windows 桌面量化分析工作站。Tauri 2.x（Rust + React + WASM）+ SQLite，自研 Canvas/WebGL K 线引擎，316+ 技术指标，策略回测引擎，条件扫描，预警系统，ME Script 公式语言，通达信数据兼容。

**代码规模**：~40 个 Tauri 命令，11 个 Rust crate，3 个前端包，约 15 万行代码。

**最终状态**：功能完整但未获得外部用户验证。决定开源。

---

## 二、架构决策：哪些对，哪些错

### 对的决定

1. **Tauri 2.x 选型** — 打包体积小（安装包 ~3.5MB），性能好，Rust 生态成熟。比 Electron 强太多。Windows 上 WebView2 覆盖率已经足够高。

2. **SQLite 本地存储** — 单文件数据库，零配置，零运维。WAL 模式 + busy_timeout + 优化 PRAGMA 的组合足够支撑桌面级并发。

3. **Rust → WASM 计算引擎** — 技术指标、回测、扫描全部在 WASM 沙箱执行。性能远超纯 JS 实现，且天然隔离了计算逻辑和 UI。

4. **Zustand 状态管理** — 三个 store（app/chart/backtest），职责清晰，无 boilerplate。persist 中间件 + partialize 选择性持久化授权字段的设计很好。

5. **增量数据库迁移** — 幂等 CREATE TABLE IF NOT EXISTS + ALTER TABLE 增量，简单可靠。

6. **Canvas/WebGL 自研图表** — 对比 TradingView 的交互体验，自定义绘制工具、多周期、2×2 网格都实现了。这是产品的核心差异化能力。

### 错的决策

1. **"wasm-" crate 命名欺诈** — 11 个 crate 中 9 个以 `wasm-` 开头，但没有一个真正编译为 wasm32 目标。它们都作为原生 rlib 链接到 Tauri 二进制。命名误导了所有人（包括我自己）。应该叫 `quant-core`、`quant-indicators` 等。

2. **lib.rs 变成 God file** — 1738 行，40 个 Tauri 命令全部堆在一个文件里。struct 定义、命令处理、业务逻辑混杂。应该拆成 `commands/backtest.rs`、`commands/scanner.rs` 等模块。

3. **错误处理全是 String** — 所有 Tauri 命令返回 `Result<_, String>`。没有 `thiserror` 枚举，没有结构化错误码。前端只能拿到一串文本，无法做差异化处理。

4. **全局 DB Mutex** — `Mutex<Option<Connection>>` 全局单例。并发命令多时锁竞争明显。应该用连接池或者至少把读写分离。

5. **CSS 方案混乱** — 部分用 CSS 变量主题系统，部分用内联 style。主题切换和维护成本高。应该从一开始就统一用 CSS modules 或 Tailwind。

6. **没有测试** — 0 个单元测试，0 个集成测试。"策略回测结果对不对"全靠手工验证小样本。这是最大的技术债务。

---

## 三、产品决策：哪些对，哪些错

### 对的决定

1. **完全离线定位** — 数据在用户硬盘上，不需要账号，不需要联网。这是和同花顺/通达信/BigQuant 最核心的差异。

2. **通达信数据兼容** — 直接导入 .day 格式，降低用户迁移成本。这是获客的关键功能。

3. **买断制定价** — ¥199 永久，符合中国用户厌恶订阅的心理。但前提是有人愿意买。

4. **合规先行** — 提前做完了禁止词扫描、免责声明覆盖、高危功能改名。虽然看起来是"过度准备"，但如果真有人用，这些能避免致命的法律风险。

### 错的决策

1. **先做功能，后找用户** — 316 个指标、20 个策略模板、ME Script 公式语言、筹码分布动画……大量的工程投入在"用户可能想要什么"的猜测上。但到开源为止，0 个外部用户验证过任何一个功能。

2. **过早优化"防盗版"** — RSA-4096 离线授权、机器指纹、激活码生成工具……对于一个还没验证过用户需求的产品，花费大量精力在防破解上是方向错误。

3. **在合规上花太多时间** — 39 个验证文档、6 个平台 go/no-go 分析、168 个敏感词清单。这些对未来产品有参考价值，但对于"先验证有没有人用"这个核心问题，是过度投入。

4. **Landing page 写了一周** — 3D 背景、SVG 图标、动画交互、响应式布局……但对于一个还没验证的产品，一个简单的 GitHub README + 下载链接就够了。

5. **不敢放出去** — 一直在"准备"，一直在"还差一点"。v0.12.1 时功能已经完整可用，但因为没有外部测试数据，不敢发。实际上，功能全正常的 v0.12.1 应该在 v0.9.0 的时候就扔出去让人用。

---

## 四、技术经验清单

### Rust/Tauri

- Tauri 2.x NSIS 安装包不支持 `/S` 静默参数（需要自己处理）
- `tauri-plugin-updater` 没服务器的话会导致启动后闪退——不要加占位配置
- Windows 上未签名 exe 必然触发 Defender，这是正常的，在文档里诚实说明即可
- `println!` 只在 seed_demo_data 中用（log crate 未引入时的妥协）
- 授权缓存必须用 `Mutex<Option<LicenseStatus>>` 而不是只缓存 pro 状态——否则 trial/free 每次都走 DB 导致锁竞争

### 前端

- Textarea 内嵌 autocomplete 的定位公式：`top = row × lineHeight`，`left ≈ col × charWidth`
- CSV 导入写 BOM `"﻿"` 保证 Excel 正确识别 UTF-8
- 国产浏览器对 HTML `download` 属性支持参差不齐，JS 程序化触发更可靠
- 多渠道图表栅格（2×2 Grid）用 `gridCells[]` + `activeCellIdx` 比条件渲染更干净
- zustand persist + partialize 可以精确控制哪些字段写 localStorage

### WASM

- `wasm-pack build --target web` 产出的 JS 胶水代码可以直接被 Vite 打包
- WASM 函数调用开销极低（微秒级），大量计算放 WASM 完全不担心性能
- 但 `wasm-bindgen` 的序列化开销需要注意——大数组传参用指针而非拷贝

### 构建与分发

- Cargo workspace + pnpm workspace 嵌套时，Tauri 产物在 `<repo_root>/target/` 而不是 `packages/app/src-tauri/target/`
- NSIS 安装包 ~3.5MB，MSI ~4.8MB（含 WebView2 bootstrapper）
- ICO 图标必须用 PNG 压缩格式（Vista-style），不能用旧式 DIB BMP
- GitHub Pages 部署时注意绝对 URL（`https://zn070515.github.io/MoneyEarning/xxx.exe`）

---

## 五、给下一个产品的建议

### 原则

1. **第 0 天就放出去** — 功能只要"能用"就发。哪怕只有 K 线图 + 一个指标 + 数据导入。

2. **不要猜用户要什么** — 做一个最小版本→发给别人→看反馈→决定下一步。不要堆功能。

3. **不要做防盗版** — 在产品验证 PMF 之前，盗版是你最不重要的问题。被破解说明有人用。

4. **不要写文档写到完美** — 一个简短的 README + 下载链接 > 39 个完美文档。

5. **合规要有，但不要过度** — 确保没有"荐股""保证收益"禁止词，做好风险提示，就够了。不需要 6 个平台的 go/no-go 分析。

### 技术

6. **Crate 命名不要骗自己** — 如果不会编译成 WASM，就不要叫 `wasm-xxx`。

7. **从第一天就写测试** — 特别是回测引擎这种"计算结果对不对"直接决定产品可信度的模块。

8. **Tauri 命令按领域拆分** — `commands/backtest.rs`、`commands/data.rs`、`commands/scanner.rs`。不要一个 lib.rs 塞 40 个命令。

9. **用 thiserror 做错误枚举** — 每层一个 Error enum，不要到处返回 String。

10. **CSS 方案一开始就统一** — 要么全用 CSS modules，要么全用 Tailwind，不要一半 CSS 变量一半内联 style。

### 分发

11. **GitHub Release + Landing page 就够了** — 不需要考虑淘宝/闲鱼/小红书/知乎/CSDN 全渠道覆盖。先让 GitHub 上有人 star。

12. **诚实说明未签名的后果** — Defender 警告是正常的，在 README 里写清楚就行，不需要花 ¥2000 买证书。

---

## 六、保留价值

以下资产对未来产品有复用价值：

| 资产 | 价值 | 复用难度 |
|------|------|:---:|
| 316+ 技术指标实现 (Rust) | 可直接在新项目中作为指标库使用 | 低 |
| 策略回测引擎 (WASM) | 事件驱动回测框架，支持滑点/手续费/止损 | 中 |
| Canvas/WebGL K 线引擎 | 自研图表渲染，对标 TradingView | 高 |
| 通达信 .day 解析器 | 完整二进制格式解析 + 批量导入 | 低 |
| K 线形态识别 (61 种) | TA-Lib 对标实现 | 中 |
| ME Script 公式解析器 | 通达信公式语法兼容的解释器 | 中 |
| 数据库迁移模式 | 幂等建表 + 增量 ALTER 的范式 | 低 |
| 合规文档体系 | 禁止词清单、免责声明模板、风险评估框架 | 低 |

---

> **一句话总结**：产品功能做得太多，用户验证做得太少。下一个产品，先发再改，不要先改再发。

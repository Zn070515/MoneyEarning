# 安全回归检查清单

> 版本 v1.0 | 2026-05-24

---

## 检查范围

确保 v0.12 新代码不引入安全回归。涵盖 Rust 后端、WASM 核心、前端三个层面。

---

## 一、Rust 后端安全

### 1.1 命令注入

- [ ] 所有 Tauri command 参数经过验证
- [ ] 无 `std::process::Command` 拼接用户输入
- [ ] 文件路径操作使用 `std::path::Path`，不拼接字符串
- [ ] 无 `eval` / `exec` 等代码执行

### 1.2 数据访问

- [ ] SQLite 查询使用参数化查询（prepared statement），不拼接 SQL 字符串
- [ ] 文件读写仅限应用数据目录（`app_data_dir`）
- [ ] 无路径穿越漏洞（`../` 检查）
- [ ] 备份/恢复不覆盖系统文件

### 1.3 错误处理

- [ ] 无裸 `unwrap()` — 所有 `unwrap()` 已有审计结论
- [ ] `Result` 在 Tauri command 中正确返回错误（非 panic）
- [ ] 错误信息不泄露敏感信息（文件路径、密钥、内部状态）

### 1.4 并发安全

- [ ] `LICENSE_CACHE: Mutex<Option<LicenseStatus>>` 使用正确
- [ ] `drop(guard)` 在 cache 写入之前
- [ ] SQLite 连接无并行竞态（WAL 模式下降低但仍需注意）
- [ ] 无 deadlock 风险（锁获取顺序一致）

### 1.5 授权边界

- [ ] Pro 功能在 Rust 端有授权检查（非仅前端检查）
- [ ] 激活码验证在本地完成（RSA 公钥嵌入）
- [ ] 无私钥信息泄露（字符串、注释、日志）
- [ ] 机器指纹获取有 fallback（WMI 失败时）

---

## 二、WASM 核心安全

### 2.1 Panic 防护

- [ ] WASM 函数无 `unwrap()` / `expect()` 残留
- [ ] `period == 0` 检查（已修复：atr/adx/supertrend 等）
- [ ] `pool_size == 0` 检查（已修复：search.rs）
- [ ] `n < 2` 协方差检查（已修复：stats.rs）
- [ ] `n == 0` 特征值检查（已修复：stats.rs）
- [ ] 排序 `unwrap_or(Ordering::Equal)`（已修复：median.rs）
- [ ] `ad_line unwrap` 已转 `Result`（已修复：adosc.rs）
- [ ] `rvgi unwrap` 已转 `Result`（已修复：inertia.rs）

### 2.2 数值安全

- [ ] 除法操作：除零有防护或返回 NaN
- [ ] 数组索引：不越界（偏移量检查）
- [ ] 超大参数：有上限检查（如 period < 10000）
- [ ] 超大序列：有上限检查（如 bar_count < 50000）
- [ ] 浮点数 NaN/Inf 传播有处理

### 2.3 内存

- [ ] 超大数组分配：有上限检查（防止 WASM 内存溢出）
- [ ] 无显式内存泄漏（WASM 线性内存模型下较安全）
- [ ] AST 深度限制（防止栈溢出，max_depth ≤ 50）

---

## 三、前端安全

### 3.1 XSS

- [ ] 用户输入（股票名称、备注等）在渲染前转义
- [ ] ME Script 编辑器中不执行用户代码为 JS
- [ ] CSV 导入的数据不直接插 innerHTML
- [ ] 无 `dangerouslySetInnerHTML`

### 3.2 数据泄漏

- [ ] localStorage 不存储激活码明文
- [ ] 调试日志中无敏感信息
- [ ] 错误提示不暴露文件路径

### 3.3 类型安全

- [ ] 无新增 `// @ts-ignore`
- [ ] 无新增 `as any` 强制类型转换
- [ ] API 响应有类型校验/防护

---

## 四、第三方依赖

- [ ] `Cargo.toml` 无新增未审查依赖
- [ ] `package.json` 无新增未审查依赖
- [ ] `cargo audit` 无已知漏洞（或已知且无影响）
- [ ] `npm audit` 无 CRITICAL/HIGH 漏洞（或已知且无影响）

---

## 五、数据安全

### 5.1 本地存储

- [ ] SQLite 数据库文件权限正常（用户目录下）
- [ ] 备份文件不包含授权私密信息（激活码哈希除外）
- [ ] 日志文件不包含完整激活码

### 5.2 网络

- [ ] 仅行情下载时有网络请求（东方财富公开 API）
- [ ] 无后台数据上传
- [ ] 无遥测/统计上报
- [ ] 无第三方 SDK 数据收集

---

## 六、审计 Agent 检查项

### A. WASM 审计

扫描：`crates/wasm-*/src/**/*.rs`

- [ ] 无 unwrap
- [ ] 无 除零
- [ ] 无 索引越界
- [ ] 无 整数溢出

### B. Rust 审计

扫描：`packages/app/src-tauri/src/**/*.rs`

- [ ] 无 unwrap
- [ ] 无 锁竞争
- [ ] 无 未处理 Result
- [ ] 无 命令注入
- [ ] 无 授权边界绕过

### C. 前端审计

扫描：`packages/*/src/**/*.{ts,tsx}`

- [ ] 无 try/catch 缺失
- [ ] 无 null 解引用
- [ ] 无 类型断言滥用
- [ ] 无 内存泄漏

---

## 安全回归日志

| 日期 | 版本 | 审计者 | WASM | Rust | 前端 | 发现 | 状态 |
|------|------|:---:|:---:|:---:|:---:|------|:---:|
| | | | ⬜ | ⬜ | ⬜ | | |

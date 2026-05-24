# v0.12.0 变更日志

> 发布日期：______

---

## 概述

v0.12.0 是"可卖 Alpha 闸门"阶段的首个版本。本版本的核心工作不是堆功能，而是：

1. **合规边界重构**：全面审查产品定位，移除荐股暗示，统一合规表述
2. **安装信任建设**：正面处理 Windows 未签名安装包的用户信任问题
3. **体验闭环**：后台预警、系统托盘、多周期 K 线、数据备份恢复、首次使用引导
4. **质量审计**：回测可信度审计、ME Script 兼容性验证
5. **销售准备**：销售页面草案、FAQ、授权交付 SOP、发版检查清单

---

## 新功能

### 后台预警 + Windows 通知
- 应用最小化到托盘后继续执行本地条件检查
- 价格突破/均线交叉/成交量异常三种条件触发通知
- Windows 原生通知（tauri-plugin-notification）
- 通知点击跳转对应股票图表
- 用户可配置扫描频率、启用/禁用后台扫描
- 所有触发记录本地存储，可查看/清除

### 系统托盘
- 托盘图标 + 右键菜单
- 显示/隐藏主窗口
- 暂停/恢复预警扫描
- 查看最近 5 条预警
- 关闭窗口默认隐藏到托盘（可配置）

### 多周期 K 线
- 日线 / 周线 / 月线 / 60分钟 / 30分钟 / 15分钟 / 5分钟
- 周线/月线通过日线 resample 生成
- 分钟线使用 minute_prices 表
- 2×2 网格模式下每个画布独立周期

### 数据备份/恢复
- 一键备份 SQLite 数据库到用户选择目录
- 恢复前自动备份当前数据库
- 显示备份文件大小、时间
- 每 7 天提醒备份（可关闭）

### 首次使用引导
- 3 步入门向导
- 加载演示数据：3 分钟内看到第一张 K 线图
- 导入通达信数据入口
- 快速教程

---

## 修改

### 合规改造
- "智能选股" → "条件扫描"
- "买入信号" → "条件触发标记(上)"
- "卖出信号" → "退出条件标记"
- "牛股评分" → "技术条件匹配度"
- 全产品文案移除荐股/收益承诺暗示（109 个禁词）
- 所有 L3 功能入口增加强制免责声明
- 回测报告/导出增加详细风险提示
- 新增 6 个合规文档、109 条禁止表达库

### 安装信任
- 新增安装教程（面向用户）
- 新增 SmartScreen 处理指南
- 新增杀软误报处理说明
- 新增安装包检查清单
- 新增首次使用体验设计

### 品质提升
- 错误状态和空状态优化
- 策略模板名移除"高收益""稳赚"字眼
- 回测报告免责声明强化

---

## 文档新增

### 合规（6 文档）
- `compliance/securities-compliance-audit.md`
- `compliance/prohibited-copywriting.md`（109 个禁词）
- `compliance/allowed-copywriting.md`
- `compliance/risk-disclaimer.md`
- `compliance/feature-risk-classification.md`
- `compliance/product-positioning-boundary.md`

### 安装信任（6 文档）
- `distribution/install-trust-audit.md`
- `distribution/unsigned-windows-warning-guide.md`
- `distribution/release-package-checklist.md`
- `distribution/antivirus-false-positive-plan.md`
- `distribution/installer-user-guide.md`
- `distribution/first-run-experience.md`

### 回测审计（5 文档）
- `backtest-validation/backtest-correctness-audit.md`
- `backtest-validation/strategy-template-risk-audit.md`
- `backtest-validation/no-future-function-tests.md`
- `backtest-validation/benchmark-against-known-cases.md`
- `backtest-validation/backtest-report-disclaimer.md`

### ME Script 验证（5 文档）
- `me-script/tdx-compatibility-matrix.md`
- `me-script/formula-parser-tests.md`
- `me-script/formula-error-message-guide.md`
- `me-script/sample-formula-library.md`
- `me-script/dangerous-formula-audit.md`

### 销售 + 授权（14 文档）
- `sales/landing-page-v0.12.md`
- `sales/taobao-listing.md`
- `sales/xianyu-listing.md`
- `sales/zhihu-article.md`
- `sales/xiaohongshu-posts.md`
- `sales/faq.md`（52 个问题）
- `sales/refund-policy.md`
- `sales/after-sales-policy.md`
- `license/offline-license-flow.md`
- `license/activation-code-generation-guide.md`
- `license/license-user-guide.md`
- `license/license-failure-cases.md`（25 种失败情况）
- `license/manual-delivery-sop.md`
- `license/anti-piracy-lite.md`

### 测试 + 发版（13 文档）
- `alpha-test/alpha-user-profile.md`
- `alpha-test/alpha-test-script.md`
- `alpha-test/alpha-feedback-form.md`
- `alpha-test/alpha-test-record.md`
- `alpha-test/alpha-bug-triage.md`
- `alpha-test/alpha-success-metrics.md`
- `release/v0.12-release-gate.md`（18 项闸门）
- `release/smoke-test-checklist.md`
- `release/regression-test-checklist.md`
- `release/security-regression-checklist.md`
- `release/performance-benchmark.md`
- `release/known-issues.md`
- `release/changelog-v0.12.md`

---

## 已知问题

见 `release/known-issues.md`

---

## 升级说明

从 v0.11.0 升级：
1. 下载最新安装包
2. 覆盖安装（数据文件不受影响）
3. 启动后自动运行数据库迁移（如需要）

---

## 下一步

- Alpha 用户测试（5-10 人）
- 收集反馈 → v0.13.0 迭代
- 小范围销售（≤ 20 份/月）

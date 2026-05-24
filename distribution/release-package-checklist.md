# 发版打包检查清单

> 版本 v1.0 | 2026-05-24 | 每次发版前逐项检查

---

## 发版前检查（构建前）

### 版本号

- [ ] `packages/app/package.json` — `version` 字段已更新
- [ ] `packages/app/src-tauri/Cargo.toml` — `version` 字段已更新
- [ ] `packages/app/src-tauri/tauri.conf.json` — `version` 字段已更新
- [ ] 三个文件的版本号一致
- [ ] 版本号遵循语义化版本规范（见 `docs/VERSIONING.md`）

### 代码质量

- [ ] `pnpm typecheck` — TypeScript 零错误
- [ ] `cargo check` — Rust 零错误（`packages/app/src-tauri/`）
- [ ] ESLint 无新错误
- [ ] 合规文案扫描通过（无 `prohibited-copywriting.md` 中的禁止词）

### 安全审计（来自 CLAUDE.md 工作流）

- [ ] WASM 审计通过（0 CRITICAL, 0 HIGH）
- [ ] Rust 审计通过（0 CRITICAL, 0 HIGH）
- [ ] 前端审计通过（0 CRITICAL, 0 HIGH）

---

## 构建产物检查

### 文件清单

| 文件 | 预期大小 | 检查 |
|------|:------:|:----:|
| `QuantVault_X.Y.Z_x64-setup.exe` (NSIS) | 25-50 MB | [ ] |
| `QuantVault_X.Y.Z_x64_zh-CN.msi` (WiX) | 25-50 MB | [ ] |
| `QuantVault_X.Y.Z_x64-setup.exe.sha256` | ~100 B | [ ] |
| `QuantVault_X.Y.Z_x64_zh-CN.msi.sha256` | ~100 B | [ ] |

### 关键信息

| 项目 | 值 |
|------|-----|
| 产品名称 | QuantVault |
| 版本号 | X.Y.Z |
| 构建日期 | YYYY-MM-DD |
| 平台 | Windows x64 |
| 安装包格式 | .exe (NSIS) + .msi (WiX) |
| 文件大小 (.exe) | ___ MB |
| 文件大小 (.msi) | ___ MB |
| SHA256 (.exe) | `___` |
| SHA256 (.msi) | `___` |
| 构建环境 | Windows 11 + pnpm + Tauri 2 |

---

## 安装测试

- [ ] 全新安装（无历史版本的虚拟机/干净机器上）
- [ ] NSIS 安装向导正常显示
- [ ] 安装语言选择正常（简体中文/English）
- [ ] 安装路径可自定义
- [ ] 桌面快捷方式创建成功
- [ ] 开始菜单文件夹创建成功
- [ ] 以当前用户安装（不需要管理员权限）

---

## 首次启动测试

- [ ] 应用正常启动无闪退
- [ ] 窗口标题显示 "QuantVault — 量化分析工作站"
- [ ] 数据库自动初始化
- [ ] 演示数据可加载
- [ ] 首页正常渲染
- [ ] 无崩溃/白屏

---

## 权限与隐私检查

| 检查项 | 结果 |
|--------|:----:|
| 是否请求管理员权限？ | 否（currentUser 安装） |
| 是否开机自启？ | 否（默认不开启） |
| 是否修改注册表（除卸载信息外）？ | 否 |
| 是否在后台运行（关闭窗口后）？ | v0.12 起可选 |
| 是否联网？ | 仅在用户手动下载数据时 |
| 是否上传用户数据？ | 否 |
| 是否包含第三方遥测/统计？ | 否 |
| 数据存储位置 | `%APPDATA%/com.quantvault.app/` |

---

## 卸载测试

- [ ] 通过 Windows 设置 → 应用 → 卸载正常
- [ ] 通过开始菜单 → QuantVault → 卸载正常
- [ ] 卸载后数据库文件保留（不自动删除用户数据）
- [ ] 桌面快捷方式已移除
- [ ] 开始菜单文件夹已移除
- [ ] 手动删除残留数据文件夹可正常工作

---

## 公开放出前

- [ ] 更新 `landing/index.html` — 版本号、下载链接、SHA256
- [ ] 更新 `GOAL_SPEC.md` — 版本标题和变更日志
- [ ] 替换 landing 目录下的 .exe/.msi 文件
- [ ] VirusTotal 扫描通过（误报<3个引擎）
- [ ] 更新 CHANGELOG
- [ ] Git commit + tag + push
- [ ] GitHub Release 创建

---

## 已知未解决问题（本版本）

在此记录本版本已知但不阻塞发版的问题：
- 
- 
- 

---

> **发版负责人确认**：_______ | **日期**：_______ | **版本**：_______

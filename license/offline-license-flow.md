# 离线授权流程

> 版本 v1.0 | 2026-05-24

---

## 一、整体流程

```
用户安装软件 → 自动获得14天全功能试用 → 试用满意 → 付款 →
提供机器码 → 开发者生成激活码 → 用户输入激活 → 永久解锁付费功能
```

所有步骤均在本地完成，无需服务器验证。

---

## 二、阶段一：试用期

### 2.1 试用开始

用户首次启动 QuantVault 时：
1. 读取机器指纹（CPU+主板+BIOS UUID 组合）
2. 检查 SQLite `licenses` 表，无记录则自动创建 trial 记录
3. `trial_start_date` = 今天，`trial_end_date` = 今天 + 14 天
4. 软件以**全功能模式**运行

### 2.2 试用期内

- 所有付费功能可用
- 主窗口标题栏显示"试用版 - X 天剩余"
- 每次启动检查试用剩余天数

### 2.3 试用期结束

- 付费功能锁定（回测模板限制、扫描器禁用等）
- 免费版功能继续可用
- 显示升级提示（不弹窗，信息栏常驻）

---

## 三、阶段二：购买与激活

### 3.1 用户侧操作

1. 用户打开：帮助 → 关于 → 查看**机器码**
2. 机器码格式：`XXXX-XXXX-XXXX-XXXX`（16位十六进制，基于机器指纹的简化展示）
3. 用户将机器码发给开发者（邮件/微信/其他渠道）

### 3.2 开发者侧操作

1. 确认收款
2. 运行激活码生成工具：`cargo run --bin gen_license -- --machine-id "<机器码>"`
3. 生成激活码：格式 `QV-XXXXXXXX-XXXXXXXX-XXXXXXXX-XXXXXXXX`
4. 将激活码发给用户
5. 记录：日期、机器码、激活码、支付凭证ID

### 3.3 激活码结构

```
QV-<签名段>-<机器码段>-<时间戳段>-<校验段>

签名段：RSA-4096 私钥签名（前8字节 hex）
机器码段：机器码哈希（前8字节 hex）
时间戳段：激活时间（Unix timestamp 十六进制）
校验段：以上三段 XOR + CRC32
```

### 3.4 用户侧激活

1. 帮助 → 输入激活码
2. 软件本地验证：
   - RSA-4096 公钥验签
   - 机器码匹配检查
   - 校验段检查
3. 验证通过 → `licenses` 表写入 `pro` 状态
4. 所有付费功能永久解锁

---

## 四、阶段三：激活后

### 4.1 正常运行

- 每次启动检查 `licenses` 表，确认 pro 状态有效
- `LICENSE_CACHE: Mutex<Option<LicenseStatus>>` 缓存状态，避免重复 DB 查询
- 无网络检查，无远程验证

### 4.2 机器码变化检测

如果检测到当前机器码与激活时绑定的机器码不匹配：
- 显示"硬件变更检测"提示
- 付费功能锁定
- 提示用户联系更换授权

---

## 五、换电脑/重装系统流程

### 5.1 用户换电脑

1. 备份旧电脑数据：`%APPDATA%/com.quantvault.app/` 整个目录
2. 在新电脑安装 QuantVault
3. 恢复备份到相同目录（或导入数据）
4. 在新电脑打开软件 → 获取新机器码
5. 联系开发者，提供：旧机器码 + 新机器码 + 原激活码
6. 开发者验证后签发新激活码（免费，每用户每半年限 1 次）

### 5.2 重装系统

重装系统可能导致机器码变化（取决于是否格式化、BIOS 是否变等）。

处理同换电脑流程。

### 5.3 限制

- 每用户每半年免费更换 1 次
- 超过频率需要人工审核，防止一码多用
- 如发现明显滥用（同一激活码在多个不同机器频繁切换），停止售后

---

## 六、技术实现

### 6.1 机器指纹提取

```
指纹来源（Windows）：
├── CPU ID（WMI: Win32_Processor → ProcessorId）
├── 主板序列号（WMI: Win32_BaseBoard → SerialNumber）
└── BIOS UUID（WMI: Win32_BIOS → SerialNumber / UUID）

最终指纹 = SHA256(CPU_ID + MB_SN + BIOS_UUID)
简化展示码 = 最终指纹[0..8] hex → XXXX-XXXX-XXXX-XXXX
```

### 6.2 RSA 密钥管理

```
私钥：
├── 位置：开发者本地，不在仓库、不在 CI
├── 格式：PEM，RSA-4096
└── 保护：文件权限 600，不提交到 git

公钥：
├── 位置：嵌入编译到 Rust 二进制（license.rs 常量）
└── 用途：激活码验签
```

### 6.3 激活码生成（gen_license 工具）

```rust
// 输入：机器码字符串
// 输出：激活码字符串 QV-XXXX-XXXX-XXXX-XXXX
// 步骤：
// 1. 解析机器码 → 计算 hash
// 2. 取当前 timestamp
// 3. 拼接 payload: machine_hash(8B) + timestamp(8B)
// 4. RSA-SHA256 签名
// 5. 组装激活码：QV + 签名 + 机器hash + timestamp + crc32
// 6. Base32 编码（避免容易混淆的字符 0/O/I/L/1）
```

### 6.4 本地验证

```rust
// license.rs 中的验证逻辑
fn verify_activation_code(code: &str, machine_id: &str) -> Result<(), LicenseError> {
    // 1. 解码激活码
    // 2. 用嵌入公钥验签
    // 3. 验证机器码匹配
    // 4. 验证校验段
    // 5. 验证时间戳合理性（不能是未来时间超过24h）
}
```

---

## 七、数据库 Schema

```sql
CREATE TABLE IF NOT EXISTS licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    machine_id TEXT NOT NULL,
    license_type TEXT NOT NULL CHECK(license_type IN ('trial','pro','expired')),
    trial_start_date TEXT,
    trial_end_date TEXT,
    activation_code TEXT,
    activated_at TEXT,
    replaced_at TEXT,
    replace_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
);
```

---

## 八、安全边界

### 现阶段不做的

- ❌ 定时联网验证（违背离线原则）
- ❌ 远程吊销激活码
- ❌ 绑定 IP/MAC 地址（换网络不影响）
- ❌ 虚拟机检测（不追求绝对防破解）
- ❌ 代码虚拟化/混淆授权模块（暂不需要）

### 安全目标

- 阻止普通用户"一码多人用"（机器指纹绑定）
- 不追求防专业逆向（Rust 二进制已有效防护）
- 如果用户有能力破解 RSA-4096，说明不是目标用户群

---

> **核心原则**：离线授权必须简单可靠。不做过度复杂的防破解，信任用户但不放任滥用。

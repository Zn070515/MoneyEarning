# 激活码生成步骤

> 版本 v1.0 | 2026-05-24 | 仅限开发者内部使用

---

## 一、前置条件

### 1.1 需要的文件

| 文件 | 位置 | 说明 |
|------|------|------|
| RSA 私钥 | `~/.quantvault/license_private.pem` | 4096-bit，不提交到 git |
| RSA 公钥 | 嵌入在 `license.rs` 中 | 随二进制编译 |
| 生成工具 | `crates/license-tool/` | 命令行工具 |

### 1.2 如果没有私钥（首次设置）

```bash
# 生成 RSA-4096 私钥
openssl genrsa -out ~/.quantvault/license_private.pem 4096

# 提取公钥
openssl rsa -in ~/.quantvault/license_private.pem -pubout -out ~/.quantvault/license_public.pem

# 将公钥内容更新到 packages/app/src-tauri/src/license.rs 的 PUBKEY 常量
```

---

## 二、生成步骤

### Step 1：确认收款

- 检查支付记录，确认款项到账
- 记录：支付平台、金额、时间、支付ID

### Step 2：获取用户机器码

- 用户从软件 "帮助→关于" 获取机器码
- 格式：`XXXX-XXXX-XXXX-XXXX`（16位十六进制）

### Step 3：运行生成命令

```bash
cd crates/license-tool
cargo run -- --private-key ~/.quantvault/license_private.pem generate \
    --machine-id "ABCD-1234-EF56-7890"
```

输出示例：
```
激活码：QV-K7X2M9NP-R3WQ8V5T-Y1B4D6F8-H2J9L0N3
生成时间：2026-05-24 15:30:00
绑定机器码：ABCD-1234-EF56-7890
```

### Step 4：测试验证（可选）

```bash
# 用公钥验证激活码（确保正确生成）
cargo run -- --public-key <(echo "PUBLIC_KEY_CONTENT") verify \
    --machine-id "ABCD-1234-EF56-7890" \
    --activation-code "QV-K7X2M9NP-R3WQ8V5T-Y1B4D6F8-H2J9L0N3"
```

### Step 5：发送给用户

```
您好，您的 QuantVault 激活码如下：

激活码：QV-XXXX-XXXX-XXXX-XXXX

请在软件中依次点击：帮助 → 输入激活码 → 粘贴激活码 → 激活。

注意：
· 激活码绑定当前电脑，请勿分享给他人
· 如需更换电脑，请联系我们
· 请妥善保管此激活码

感谢您的购买！
```

### Step 6：记录交付

在 `~/.quantvault/sales_log.csv` 记录：

```csv
日期,支付平台,金额,支付ID,机器码,激活码,备注
2026-05-24,微信,199,xxx,ABCD-1234-EF56-7890,QV-xxxx-xxxx-xxxx-xxxx,已发送
```

---

## 三、激活码格式

```
QV-<A>-<B>-<C>-<D>

总长：31 字符（含分隔符）

编码：Base32（排除 O/0/I/1/L 避免混淆）

段A（8字符）：RSA 签名前 40bit → 5字节 → 8字符 Base32
段B（8字符）：机器码哈希前 40bit → 5字节 → 8字符 Base32
段C（8字符）：时间戳 40bit → 5字节 → 8字符 Base32
段D（8字符）：CRC32 校验 + 填充 → 5字节 → 8字符 Base32
```

---

## 四、常见问题

### Q: 生成失败怎么办？

检查：
1. 私钥文件路径是否正确
2. 机器码格式是否正确（16位 hex，含或不含分隔符均可）
3. `cargo build` 是否通过

### Q: 用户说激活码无效怎么办？

1. 先确认用户输入的机器码与你生成时使用的一致（大小写不敏感，分隔符自动处理）
2. 检查用户是否在正确的软件版本（v0.10.0+ 支持激活）
3. 如果换了电脑，需要用新机器码重新生成
4. 用 `verify` 子命令重新验证激活码

### Q: 可以批量生成吗？

可以。创建 `machines.txt`（每行一个机器码）：

```bash
cargo run -- batch-generate \
    --private-key ~/.quantvault/license_private.pem \
    --input machines.txt \
    --output activation_codes.csv
```

---

## 五、安全提醒

- ⚠️ 私钥丢失 = 所有已发出的激活码无法换绑，必须重新签发
- ⚠️ 私钥泄露 = 任何人可以生成有效激活码
- ⚠️ 私钥不要放在云盘、git 仓库、聊天记录中
- ⚠️ 建议私钥做 2-3 份离线备份（加密U盘）
- ⚠️ 生成工具不需要联网

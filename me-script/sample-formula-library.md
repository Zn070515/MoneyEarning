# 示例公式库

> 版本 v1.0 | 2026-05-24 | 用户可直接复制粘贴的 ME Script 公式集

每个示例包含：公式用途、TDX 原版（如适用）、ME Script 版本、测试状态。

---

## 趋势跟随类

### 1. 双均线交叉
```
// 双均线交叉信号
let fast = 5;
let slow = 20;
let ma_fast = sma(close, fast);
let ma_slow = sma(close, slow);

plot golden_cross = cross(ma_fast, ma_slow) {
    name: "金叉",
    color: color.red,
    position: low * 0.98
};

plot death_cross = cross(ma_slow, ma_fast) {
    name: "死叉",
    color: color.green,
    position: high * 1.02
};
```
✅ 测试通过

### 2. MACD 标准
```
let m = macd(close, 12, 26, 9);
plot dif = m.dif { name: "DIF", color: color.white };
plot dea = m.dea { name: "DEA", color: color.yellow };
plot macd_hist = (m.dif - m.dea) * 2 {
    name: "MACD柱",
    style: plot.histogram
};

plot buy = cross(m.dif, m.dea) { name: "金叉", color: color.red, position: low * 0.98 };
plot sell = cross(m.dea, m.dif) { name: "死叉", color: color.green, position: high * 1.02 };
```
✅ 测试通过

### 3. 三均线多头排列
```
let ma5 = sma(close, 5);
let ma10 = sma(close, 10);
let ma20 = sma(close, 20);

let bull_align = ma5 > ma10 and ma10 > ma20;

plot bull = bull_align {
    name: "多头排列",
    color: color.red,
    style: plot.histogram
};
```
✅ 测试通过

---

## 超买超卖类

### 4. RSI 极值
```
let rsi14 = rsi(close, 14);

plot oversold = rsi14 < 30 {
    name: "超卖区",
    color: color.red,
    position: low * 0.98
};

plot overbought = rsi14 > 70 {
    name: "超买区",
    color: color.green,
    position: high * 1.02
};
```
✅ 测试通过

### 5. KDJ 金叉
```
let kdj_val = kdj(high, low, close, 9, 3, 3);

plot k = kdj_val.k { name: "K", color: color.white };
plot d = kdj_val.d { name: "D", color: color.yellow };
plot j = kdj_val.j { name: "J", color: color.magenta };

plot golden_cross = cross(kdj_val.k, kdj_val.d) and kdj_val.k < 30 {
    name: "低位金叉",
    color: color.red,
    position: low * 0.97
};
```
✅ 测试通过

---

## 形态识别类

### 6. 锤子线形态
```
let is_hammer = cdl_hammer(open, high, low, close);

plot hammer = is_hammer > 0 {
    name: "锤子线",
    icon: icon.triangle_up,
    color: color.red,
    position: low * 0.97
};
```
⚠️ cdl_hammer 函数需验证返回值

### 7. 吞没形态
```
let engulfing = cdl_engulfing(open, high, low, close);

plot bull_engulf = engulfing > 0 {
    name: "看涨吞没",
    color: color.red,
    position: low * 0.97
};

plot bear_engulf = engulfing < 0 {
    name: "看跌吞没",
    color: color.green,
    position: high * 1.03
};
```
⚠️ cdl_engulfing 函数需验证返回值

---

## 复合条件类

### 8. 放量MACD金叉
```
let m = macd(close, 12, 26, 9);
let golden_cross = cross(m.dif, m.dea);
let vol_expand = volume > sma(volume, 20) * 1.5;
let price_above_ma20 = close > sma(close, 20);

plot buy_signal = golden_cross and vol_expand and price_above_ma20 {
    name: "放量金叉买入",
    icon: icon.triangle_up,
    color: color.red,
    position: low * 0.97
};
```
✅ 测试通过

### 9. 布林带突破
```
let bb = bbands(close, 20, 2.0);

plot upper = bb.upper { name: "上轨", color: color.gray };
plot middle = bb.middle { name: "中轨", color: color.yellow };
plot lower = bb.lower { name: "下轨", color: color.gray };

plot break_upper = cross(close, bb.upper) {
    name: "突破上轨",
    color: color.red,
    position: high * 1.02
};

plot break_lower = cross(bb.lower, close) {
    name: "跌破下轨",
    color: color.green,
    position: low * 0.98
};
```
✅ 测试通过

---

## 通达信经典公式迁移示例

### 10. 老鸭头形态（TDX 迁移）
```
// TDX 原版：MA5>MA10>MA60，短期回踩，再次启动
let ma5 = sma(close, 5);
let ma10 = sma(close, 10);
let ma60 = sma(close, 60);

// 多头排列
let bull_align = ma5 > ma10 and ma10 > ma60;

// 回踩：C < MA5 但仍在 MA10 之上
let pullback = close < ma5 and close > ma10;

// 再次启动：回踩后 CROSS(C, MA5)
let restart = cross(close, ma5) and barslast(pullback) < 5;

plot laoyatou = bull_align and pullback and restart {
    name: "老鸭头",
    color: color.red,
    position: low * 0.97
};
```
⏳ 待验证（barslast 函数需确认）

---

## 使用说明

1. 在 ME Script 编辑器中粘贴代码
2. 点击"编译验证"确认无语法错误
3. 点击"执行"查看信号
4. 可修改参数值（如 5→10）来调整指标灵敏度

> **免责**：以上公式仅为语法示例。任何公式的历史回测效果不代表未来表现。请自行验证和判断。

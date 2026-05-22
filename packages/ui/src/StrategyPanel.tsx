import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Strategy {
  id: number;
  name: string;
  script: string | null;
  params: string | null;
  template_type: string | null;
  created_at: string;
}

const TEMPLATES: Record<string, { label: string; script: string }> = {
  ma_cross: {
    label: "双均线交叉",
    script: `// 双均线交叉策略 — 短均线上穿长均线买入
indicator "SMA_Cross" {
    params: { fast: 5, slow: 20 }

    // 计算双均线
    ma_fast := MA(CLOSE, fast);
    ma_slow := MA(CLOSE, slow);

    // 金叉买入 | 死叉卖出
    buy := CROSS(ma_fast, ma_slow);
    sell := CROSS(ma_slow, ma_fast);

    plot(ma_fast, "快线", COLORRED);
    plot(ma_slow, "慢线", COLORBLUE);
}`,
  },
  macd: {
    label: "MACD金叉死叉",
    script: `// MACD策略 — DIF上穿DEA买入
indicator "MACD_Cross" {
    params: { fast: 12, slow: 26, signal: 9 }

    DIF := EMA(CLOSE, fast) - EMA(CLOSE, slow);
    DEA := EMA(DIF, signal);
    MACD := 2 * (DIF - DEA);

    // 零轴下方金叉更可靠
    buy := CROSS(DIF, DEA) AND DIF < 0;
    sell := CROSS(DEA, DIF);

    plot(DIF, "DIF", COLORWHITE);
    plot(DEA, "DEA", COLORYELLOW);
    plot(MACD, "柱", COLORSTICK);
}`,
  },
  triple_ma: {
    label: "三均线多头排列",
    script: `// 三均线多头排列 — 短>中>长持仓
indicator "Triple_MA" {
    params: { short: 5, mid: 20, long: 60 }

    ma_short := MA(CLOSE, short);
    ma_mid := MA(CLOSE, mid);
    ma_long := MA(CLOSE, long);

    // 连续3日多头排列确认
    aligned := ma_short > ma_mid AND ma_mid > ma_long;
    buy := aligned AND REF(aligned, 1) AND REF(aligned, 2);
    sell := ma_short < ma_mid;

    plot(ma_short, "MA5", COLORRED);
    plot(ma_mid, "MA20", COLORBLUE);
    plot(ma_long, "MA60", COLORGREEN);
}`,
  },
  bb_rsi: {
    label: "布林带+RSI双确认",
    script: `// 布林带+RSI均值回归 — 下轨+超卖+放量止跌
indicator "BB_RSI" {
    params: { period: 20, std: 2, rsi_period: 14, vol_mult: 1.5 }

    mid := MA(CLOSE, period);
    sd := STD(CLOSE, period);
    upper := mid + std * sd;
    lower := mid - std * sd;

    rsi_val := RSI(CLOSE, rsi_period);
    vol_ma := MA(VOL, 20);

    // 三重确认：破下轨+RSI超卖+放量止跌
    buy := CLOSE < lower AND rsi_val < 30
           AND VOL > vol_ma * vol_mult
           AND CLOSE > REF(CLOSE, 1);
    sell := CLOSE > mid;

    plot(upper, "上轨", COLORGREEN);
    plot(mid, "中轨", COLORWHITE);
    plot(lower, "下轨", COLORRED);
}`,
  },
  donchian: {
    label: "唐奇安通道突破",
    script: `// 唐奇安通道突破 — 突破N日最高价买入
indicator "Donchian" {
    params: { period: 20, exit_period: 10 }

    upper := HHV(HIGH, period);
    lower := LLV(LOW, period);
    exit_low := LLV(LOW, exit_period);

    buy := CLOSE > REF(upper, 1);
    sell := CLOSE < exit_low;

    plot(upper, "上轨", COLORRED);
    plot(lower, "下轨", COLORGREEN);
}`,
  },
  turtle: {
    label: "海龟交易法则",
    script: `// 海龟交易法则 — 突破入场+ATR动态止损
indicator "Turtle" {
    params: { entry: 20, exit: 10, atr_period: 20, atr_stop: 2 }

    entry_high := HHV(HIGH, entry);
    exit_low := LLV(LOW, exit);
    atr_val := ATR(atr_period);

    buy := CLOSE > REF(entry_high, 1);
    // 动态止损离场
    sell := CLOSE < exit_low OR CLOSE < REF(CLOSE, 1) - atr_stop * atr_val;

    plot(entry_high, "入场通道", COLORRED);
    plot(exit_low, "离场通道", COLORGREEN);
}`,
  },
  kdj: {
    label: "KDJ超买超卖",
    script: `// KDJ超买超卖 — %K<20超卖买入
indicator "KDJ_Extremes" {
    params: { n: 9, oversold: 20, overbought: 80 }

    RSV := (CLOSE - LLV(LOW, n)) / (HHV(HIGH, n) - LLV(LOW, n)) * 100;
    K := SMA(RSV, 3, 1);
    D := SMA(K, 3, 1);
    J := 3 * K - 2 * D;

    buy := K < oversold AND D < oversold AND J < 0 AND CLOSE > REF(CLOSE, 1);
    sell := K > 55;

    plot(K, "%K", COLORWHITE);
    plot(D, "%D", COLORYELLOW);
    plot(J, "%J", COLORMAGENTA);
}`,
  },
  vol_breakout: {
    label: "放量突破均线",
    script: `// 放量突破60日均线 — 趋势启动确认
indicator "Vol_Breakout" {
    params: { ma_period: 60, vol_mult: 1.5, vol_period: 20 }

    ma60 := MA(CLOSE, ma_period);
    vol_ma := MA(VOL, vol_period);

    // 收盘站上60日线+成交量放大
    buy := CLOSE > ma60 AND VOL > vol_ma * vol_mult
           AND REF(CLOSE, 1) < REF(ma60, 1);
    sell := CLOSE < MA(CLOSE, 20);

    plot(ma60, "MA60", COLORYELLOW);
    plot(MA(CLOSE, 20), "MA20", COLORWHITE);
}`,
  },
  custom: {
    label: "自定义策略",
    script: `// 自定义策略 — 在此编写您的交易规则
indicator "MyStrategy" {
    params: {}

    // 通达信兼容语法:
    //   MA(CLOSE, N)  — 均线
    //   EMA(CLOSE, N) — 指数均线
    //   RSI(CLOSE, N) — 相对强弱
    //   MACD(CLOSE, S, L, M) — MACD
    //   CROSS(A, B)   — A上穿B
    //   REF(X, N)     — N周期前的X
    //   HHV(HIGH, N)  — N周期最高价
    //   LLV(LOW, N)   — N周期最低价
    //   STD(CLOSE, N) — 标准差
    //   ATR(N)        — 平均真实波幅

    // 买入条件
    buy := false;

    // 卖出条件
    sell := false;

    plot(CLOSE, "收盘", COLORWHITE);
}`,
  },
};

interface StrategyPanelProps {
  onSelectStrategy?: (s: Strategy) => void;
  selectedStockId?: number | null;
}

export function StrategyPanel({ onSelectStrategy, selectedStockId }: StrategyPanelProps) {
  const [strategies, setStrategies] = useState<Strategy[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [editing, setEditing] = useState<Strategy | null>(null);
  const [showNew, setShowNew] = useState(false);

  const [formName, setFormName] = useState("");
  const [formTemplate, setFormTemplate] = useState("custom");
  const [formScript, setFormScript] = useState("");
  const [formParams, setFormParams] = useState("{}");
  const [validateResult, setValidateResult] = useState<string | null>(null);
  const [validating, setValidating] = useState(false);

  const handleValidate = async () => {
    if (!formScript.trim() || !selectedStockId) return;
    setValidating(true);
    setValidateResult(null);
    try {
      const result = await invoke<{
        buy_count: number;
        sell_count: number;
        params: Record<string, number>;
        errors: string[];
      }>("execute_custom_script", {
        script: formScript,
        stockId: selectedStockId,
      });
      const parts: string[] = [];
      if (result.errors.length > 0) {
        parts.push("解析错误:\n" + result.errors.map((e) => "  ✗ " + e).join("\n"));
      } else {
        parts.push("✓ 脚本编译通过");
      }
      parts.push("买入信号: " + result.buy_count + " 次");
      parts.push("卖出信号: " + result.sell_count + " 次");
      if (Object.keys(result.params).length > 0) {
        parts.push("参数: " + JSON.stringify(result.params, null, 2));
      }
      setValidateResult(parts.join("\n"));
    } catch (e) {
      setValidateResult("验证失败: " + String(e));
    }
    setValidating(false);
  };

  const load = useCallback(async () => {
    try {
      const data = await invoke<Strategy[]>("strategy_list");
      setStrategies(data);
    } catch (e) {
      console.error("Strategy load:", e);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleNew = () => {
    setShowNew(true);
    setEditing(null);
    setFormName("");
    setFormTemplate("custom");
    setFormScript(TEMPLATES.custom.script);
    setFormParams("{}");
    setValidateResult(null);
  };

  const handleEdit = (s: Strategy) => {
    setEditing(s);
    setShowNew(true);
    setFormName(s.name);
    setFormTemplate(s.template_type ?? "custom");
    setFormScript(s.script ?? "");
    setFormParams(s.params ?? "{}");
    setValidateResult(null);
  };

  const handleSave = async () => {
    if (!formName.trim()) return;
    try {
      if (editing) {
        await invoke("strategy_update", {
          id: editing.id,
          name: formName.trim(),
          script: formScript || null,
          params: formParams || null,
          templateType: formTemplate || null,
        });
      } else {
        await invoke("strategy_create", {
          name: formName.trim(),
          script: formScript || null,
          params: formParams || null,
          templateType: formTemplate || null,
        });
      }
      setShowNew(false);
      setEditing(null);
      load();
    } catch (e) {
      console.error("Save strategy:", e);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm("确认删除此策略？")) return;
    try {
      await invoke("strategy_delete", { id });
      if (selectedId === id) setSelectedId(null);
      load();
    } catch (e) {
      console.error("Delete strategy:", e);
    }
  };

  const handleTemplate = (t: string) => {
    setFormTemplate(t);
    setFormScript(TEMPLATES[t]?.script ?? TEMPLATES.custom.script);
  };

  const lines = formScript.split("\n");
  const maxLineNum = Math.max(lines.length, 15);

  return (
    <div style={{
      background: "#16213e", color: "#ccc", fontFamily: "monospace",
      fontSize: 13, height: "100%", display: "flex", flexDirection: "column",
      overflow: "hidden",
    }}>
      <div style={{
        padding: "10px 12px", borderBottom: "1px solid #2a2a4a",
        fontWeight: 600, color: "#fff", fontSize: 14,
        display: "flex", justifyContent: "space-between", alignItems: "center",
      }}>
        <span>ME Script 编辑器</span>
        {!showNew && (
          <button onClick={handleNew} style={{
            background: "#fbbf24", color: "#000", border: "none",
            padding: "3px 10px", borderRadius: 4, cursor: "pointer",
            fontSize: 12, fontWeight: 600,
          }}>
            + 新建
          </button>
        )}
      </div>

      {showNew ? (
        <div style={{ flex: 1, overflow: "auto", padding: 12 }}>
          {/* Name */}
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 11, color: "#888", marginBottom: 3, display: "block" }}>
              策略名称
            </label>
            <input value={formName} onChange={(e) => setFormName(e.target.value)}
              placeholder="输入策略名称..."
              style={inputStyle} />
          </div>

          {/* Template */}
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 11, color: "#888", marginBottom: 3, display: "block" }}>
              模板
            </label>
            <select value={formTemplate} onChange={(e) => handleTemplate(e.target.value)}
              style={inputStyle}>
              {Object.entries(TEMPLATES).map(([key, t]) => (
                <option key={key} value={key}>{t.label}</option>
              ))}
            </select>
            <div style={{ color: "#666", fontSize: 10, marginTop: 4 }}>
              支持通达信兼容语法：MA/EMA/RSI/MACD/CROSS/REF/HHV/LLV/STD/ATR 等40+函数
            </div>
          </div>

          {/* Script editor with line numbers */}
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 11, color: "#888", marginBottom: 3, display: "block" }}>
              策略脚本
            </label>
            <div style={{
              display: "flex", background: "#1a1a2e",
              border: "1px solid #3a3a5a", borderRadius: 4,
              overflow: "hidden",
            }}>
              {/* Line numbers */}
              <div style={{
                padding: "8px 0", background: "#111122",
                borderRight: "1px solid #2a2a4a", minWidth: 36,
                textAlign: "right", userSelect: "none",
                fontSize: 12, lineHeight: "19px", color: "#555",
                fontFamily: "monospace",
              }}>
                {Array.from({ length: maxLineNum }, (_, i) => (
                  <div key={i} style={{ paddingRight: 8 }}>
                    {(i + 1).toString().padStart(2, " ")}
                  </div>
                ))}
              </div>
              {/* Textarea synced with line numbers */}
              <textarea
                value={formScript}
                onChange={(e) => setFormScript(e.target.value)}
                rows={Math.max(lines.length, 15)}
                style={{
                  ...inputStyle, width: "100%", resize: "vertical",
                  fontFamily: '"JetBrains Mono", "Consolas", monospace',
                  fontSize: 12, lineHeight: "19px", border: "none",
                  borderRadius: 0, boxSizing: "border-box",
                  padding: "8px 10px",
                  color: "#e0e0e0",
                  background: "transparent",
                  tabSize: 2,
                }}
              />
            </div>
          </div>

          {/* Params */}
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 11, color: "#888", marginBottom: 3, display: "block" }}>
              参数覆盖 (JSON)
            </label>
            <input value={formParams} onChange={(e) => setFormParams(e.target.value)}
              style={{ ...inputStyle, width: "100%", boxSizing: "border-box" }} />
          </div>

          {/* Validate */}
          <div style={{ marginBottom: 12 }}>
            <button onClick={handleValidate} disabled={validating || !formScript.trim()}
              style={{
                background: validating ? "#555" : "#3b82f6",
                color: "#fff", border: "none", padding: "5px 14px",
                borderRadius: 4, cursor: "pointer", fontSize: 12,
                fontFamily: "monospace", fontWeight: 600,
                opacity: validating || !selectedStockId ? 0.6 : 1,
              }}>
              {validating ? "编译中..." : "编译并验证"}
            </button>
            {!selectedStockId && (
              <span style={{ color: "#666", fontSize: 10, marginLeft: 8 }}>
                请先在图表中选择一只股票
              </span>
            )}
            {validateResult !== null && (
              <div style={{
                marginTop: 8, padding: 8, borderRadius: 4,
                background: validateResult.includes("✓") ? "#0a2a1a" : "#2a1a1a",
                border: "1px solid #3a3a5a",
                color: validateResult.includes("✓") ? "#22c55e" : "#ef4444",
                fontSize: 11, whiteSpace: "pre-wrap", lineHeight: "18px",
              }}>
                {validateResult}
              </div>
            )}
          </div>

          {/* Actions */}
          <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
            <button onClick={() => { setShowNew(false); setEditing(null); setValidateResult(null); }} style={{
              background: "transparent", border: "1px solid #3a3a5a",
              color: "#ccc", padding: "6px 16px", borderRadius: 4,
              cursor: "pointer", fontSize: 12, fontFamily: "monospace",
            }}>
              取消
            </button>
            <button onClick={handleSave} disabled={!formName.trim()} style={{
              background: formName.trim() ? "#fbbf24" : "#8a7a3a",
              color: "#000", border: "none", padding: "6px 16px",
              borderRadius: 4, cursor: "pointer", fontSize: 12,
              fontWeight: 600, fontFamily: "monospace",
            }}>
              {editing ? "更新" : "创建"}
            </button>
          </div>
        </div>
      ) : (
        <div style={{ flex: 1, overflow: "auto" }}>
          {strategies.length === 0 ? (
            <div style={{ padding: 24, color: "#666", fontSize: 12, textAlign: "center" }}>
              <div style={{ fontSize: 32, marginBottom: 12, color: "#3a3a5a" }}>{ }</div>
              <div style={{ marginBottom: 8 }}>暂无策略</div>
              <div style={{ fontSize: 10, color: "#555", lineHeight: 1.8 }}>
                点击"+ 新建"编写您的第一个策略脚本<br />
                支持通达信兼容语法：MA/EMA/RSI/MACD/CROSS/REF 等
              </div>
            </div>
          ) : (
            strategies.map((s) => (
              <div key={s.id} style={{
                padding: "8px 12px", cursor: "pointer",
                background: selectedId === s.id ? "#2a3a5e" : "transparent",
                borderBottom: "1px solid #1a1a2e",
              }} onClick={() => {
                setSelectedId(s.id);
                onSelectStrategy?.(s);
              }}>
                <div style={{
                  display: "flex", justifyContent: "space-between",
                  alignItems: "center", marginBottom: 4,
                }}>
                  <span style={{
                    color: selectedId === s.id ? "#fbbf24" : "#ccc",
                    fontWeight: 600,
                  }}>
                    {s.name}
                  </span>
                  <span style={{ display: "flex", gap: 4 }}>
                    {s.template_type && (
                      <span style={{ color: "#888", fontSize: 10 }}>
                        {s.template_type}
                      </span>
                    )}
                    <button onClick={(e) => { e.stopPropagation(); handleEdit(s); }}
                      style={{
                        background: "none", border: "none", color: "#666",
                        cursor: "pointer", fontSize: 12,
                      }}>
                      ✎
                    </button>
                    <button onClick={(e) => { e.stopPropagation(); handleDelete(s.id); }}
                      style={{
                        background: "none", border: "none", color: "#666",
                        cursor: "pointer", fontSize: 12,
                      }}>
                      ×
                    </button>
                  </span>
                </div>
                {s.params && (
                  <div style={{ color: "#a78bfa", fontSize: 11 }}>
                    {s.params}
                  </div>
                )}
                <div style={{ color: "#555", fontSize: 10 }}>
                  创建于 {s.created_at}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  width: "100%", background: "#1a1a2e", border: "1px solid #3a3a5a",
  color: "#fff", padding: "6px 8px", borderRadius: 4, fontSize: 12,
  fontFamily: "monospace", outline: "none",
};

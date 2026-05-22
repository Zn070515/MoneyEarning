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

const TEMPLATES: Record<string, string> = {
  ma_cross: `// 双均线交叉策略
// 当短周期均线上穿长周期均线时买入，下穿时卖出

params: { fast: 5, slow: 20 }

rule buy:
  cross(ema(close, fast), ema(close, slow), 1)

rule sell:
  cross(ema(close, fast), ema(close, slow), -1)`,
  breakout: `// 突破策略
// 价格突破N日最高价时买入，跌破N日最低价时卖出

params: { period: 20 }

rule buy:
  close > ref(highest(high, period), 1)

rule sell:
  close < ref(lowest(low, period), 1)`,
  rsi_mean: `// RSI均值回归策略
// RSI超卖时买入，超买时卖出

params: { period: 14, oversold: 30, overbought: 70 }

rule buy:
  rsi(close, period) < oversold

rule sell:
  rsi(close, period) > overbought`,
  custom: `// 自定义策略
// 在此编写您的交易规则

params: {}

rule buy:
  // 买入条件

rule sell:
  // 卖出条件`,
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

  // New/edit form
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
        parts.push("解析错误:\n" + result.errors.map(e => "  ✗ " + e).join("\n"));
      } else {
        parts.push("脚本验证通过");
      }
      parts.push(`买入信号: ${result.buy_count} 次`);
      parts.push(`卖出信号: ${result.sell_count} 次`);
      if (Object.keys(result.params).length > 0) {
        parts.push("参数: " + JSON.stringify(result.params));
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
    setFormScript(TEMPLATES.custom);
    setFormParams("{}");
  };

  const handleEdit = (s: Strategy) => {
    setEditing(s);
    setShowNew(true);
    setFormName(s.name);
    setFormTemplate(s.template_type ?? "custom");
    setFormScript(s.script ?? "");
    setFormParams(s.params ?? "{}");
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
    setFormScript(TEMPLATES[t] ?? "");
  };

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
        <span>策略管理</span>
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
            <input value={formName} onChange={e => setFormName(e.target.value)}
              placeholder="输入策略名称..."
              style={inputStyle} />
          </div>

          {/* Template */}
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 11, color: "#888", marginBottom: 3, display: "block" }}>
              模板
            </label>
            <select value={formTemplate} onChange={e => handleTemplate(e.target.value)}
              style={inputStyle}>
              <option value="custom">自定义</option>
              <option value="ma_cross">双均线交叉</option>
              <option value="breakout">通道突破</option>
              <option value="rsi_mean">RSI均值回归</option>
            </select>
          </div>

          {/* Script */}
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 11, color: "#888", marginBottom: 3, display: "block" }}>
              策略脚本
            </label>
            <textarea value={formScript} onChange={e => setFormScript(e.target.value)}
              rows={15}
              style={{
                ...inputStyle, resize: "vertical", fontFamily: "monospace",
                width: "100%", boxSizing: "border-box",
              }} />
          </div>

          {/* Params */}
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 11, color: "#888", marginBottom: 3, display: "block" }}>
              参数 (JSON)
            </label>
            <input value={formParams} onChange={e => setFormParams(e.target.value)}
              style={{ ...inputStyle, width: "100%", boxSizing: "border-box" }} />
          </div>

          {/* Validate */}
          {selectedStockId && (
            <div style={{ marginBottom: 12 }}>
              <button onClick={handleValidate} disabled={validating || !formScript.trim()}
                style={{
                  background: validating ? "#555" : "#3b82f6",
                  color: "#fff", border: "none", padding: "5px 14px",
                  borderRadius: 4, cursor: "pointer", fontSize: 12,
                  fontFamily: "monospace", fontWeight: 600,
                  opacity: validating ? 0.6 : 1,
                }}>
                {validating ? "验证中..." : "验证脚本"}
              </button>
              {validateResult !== null && (
                <div style={{
                  marginTop: 8, padding: 8, borderRadius: 4,
                  background: validateResult.startsWith("脚本验证通过") ? "#0a2a1a" : "#2a1a1a",
                  border: "1px solid #3a3a5a",
                  color: validateResult.startsWith("脚本验证通过") ? "#22c55e" : "#ef4444",
                  fontSize: 11, whiteSpace: "pre-wrap", lineHeight: "18px",
                }}>
                  {validateResult}
                </div>
              )}
            </div>
          )}

          {/* Actions */}
          <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
            <button onClick={() => { setShowNew(false); setEditing(null); setValidateResult(null); }} style={{
              background: "transparent", border: "1px solid #3a3a5a",
              color: "#ccc", padding: "6px 16px", borderRadius: 4,
              cursor: "pointer", fontSize: 12,
            }}>
              取消
            </button>
            <button onClick={handleSave} disabled={!formName.trim()} style={{
              background: formName.trim() ? "#fbbf24" : "#8a7a3a",
              color: "#000", border: "none", padding: "6px 16px",
              borderRadius: 4, cursor: "pointer", fontSize: 12, fontWeight: 600,
            }}>
              {editing ? "更新" : "创建"}
            </button>
          </div>
        </div>
      ) : (
        <div style={{ flex: 1, overflow: "auto" }}>
          {strategies.length === 0 ? (
            <div style={{ padding: 16, color: "#666", fontSize: 12, textAlign: "center" }}>
              暂无策略，点击"+ 新建"创建
            </div>
          ) : (
            strategies.map(s => (
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
                    <button onClick={e => { e.stopPropagation(); handleEdit(s); }}
                      style={{
                        background: "none", border: "none", color: "#666",
                        cursor: "pointer", fontSize: 12,
                      }}>
                      ✎
                    </button>
                    <button onClick={e => { e.stopPropagation(); handleDelete(s.id); }}
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

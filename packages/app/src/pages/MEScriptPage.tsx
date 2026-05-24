import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/appStore";

interface StockInfo {
  id: number;
  code: string;
  name: string;
}

const FUNCTION_REF: Record<string, string> = {
  "MA(CLOSE, N)": "N周期简单移动平均",
  "EMA(CLOSE, N)": "N周期指数移动平均",
  "WMA(CLOSE, N)": "N周期加权移动平均",
  "SMA(X, N, M)": "N周期加权移动平均(M为权重)",
  "RSI(CLOSE, N)": "N周期相对强弱指数(0-100)",
  "MACD(CLOSE, S, L, M)": "MACD指标，返回.dif/.dea/.macd",
  "KDJ(N, M1, M2)": "随机指标，返回.k/.d/.j",
  "CROSS(A, B)": "A上穿B，返回布尔值",
  "REF(X, N)": "N周期前的X值",
  "HHV(HIGH, N)": "N周期内最高价的最大值",
  "LLV(LOW, N)": "N周期内最低价的最小值",
  "STD(CLOSE, N)": "N周期标准差",
  "ATR(N)": "N周期平均真实波幅",
  "ROC(CLOSE, N)": "N周期变动率(%)",
  "CCI(N)": "N周期商品通道指数",
  "OBV()": "能量潮(累积成交量)",
  "SUM(X, N)": "X的N周期累和",
  "COUNT(COND, N)": "N周期内COND成立的次数",
  "ABS(X)": "X的绝对值",
  "MAX(A, B)": "取A和B的最大值",
  "MIN(A, B)": "取A和B的最小值",
  "BETWEEN(X, A, B)": "X是否在A和B之间",
  "EVERY(COND, N)": "N周期内COND一直成立",
  "EXIST(COND, N)": "N周期内COND曾成立",
  "BARSLAST(COND)": "上次COND成立至今的周期数",
  "FILTER(COND, N)": "COND成立后N周期内过滤",
  "IF(COND, A, B)": "条件判断",
};

const REFERENCE_CATEGORIES: Record<string, string[]> = {
  "均线/统计": ["MA", "EMA", "WMA", "SMA", "STD", "SUM", "ABS", "MAX", "MIN"],
  "技术指标": ["RSI", "MACD", "KDJ", "CCI", "ATR", "ROC", "OBV"],
  "逻辑判断": ["CROSS", "REF", "HHV", "LLV", "COUNT", "EVERY", "EXIST", "BARSLAST", "BETWEEN", "FILTER", "IF"],
  "K线数据": ["OPEN", "HIGH", "LOW", "CLOSE", "VOL", "AMOUNT"],
};

const QUICK_TEMPLATES: { label: string; script: string }[] = [
  {
    label: "MA金叉死叉",
    script: `indicator "MA_Cross" {
    params: { fast: 5, slow: 20 }
    ma_fast := MA(CLOSE, fast);
    ma_slow := MA(CLOSE, slow);
    buy := CROSS(ma_fast, ma_slow);
    sell := CROSS(ma_slow, ma_fast);
    plot(ma_fast, "快线", COLORRED);
    plot(ma_slow, "慢线", COLORBLUE);
}`,
  },
  {
    label: "MACD零轴金叉",
    script: `indicator "MACD_Zero" {
    params: { fast: 12, slow: 26, signal: 9 }
    DIF := EMA(CLOSE, fast) - EMA(CLOSE, slow);
    DEA := EMA(DIF, signal);
    buy := CROSS(DIF, DEA) AND DIF < 0;
    sell := CROSS(DEA, DIF);
    plot(DIF, "DIF", COLORWHITE);
    plot(DEA, "DEA", COLORYELLOW);
}`,
  },
  {
    label: "RSI超买超卖",
    script: `indicator "RSI_Extreme" {
    params: { period: 14, low: 30, high: 70 }
    rsi_val := RSI(CLOSE, period);
    buy := rsi_val < low AND CLOSE > REF(CLOSE, 1);
    sell := rsi_val > high;
    plot(rsi_val, "RSI", COLORWHITE);
}`,
  },
  {
    label: "放量突破",
    script: `indicator "Vol_Breakout" {
    params: { period: 60, vol_mult: 1.5 }
    ma60 := MA(CLOSE, period);
    vol_ma := MA(VOL, 20);
    buy := CLOSE > ma60 AND VOL > vol_ma * vol_mult AND REF(CLOSE, 1) < REF(ma60, 1);
    sell := CLOSE < MA(CLOSE, 20);
    plot(ma60, "MA60", COLORYELLOW);
}`,
  },
];

export default function MEScriptPage() {
  const selectedStockId = useAppStore((s) => s.selectedStockId);
  const selectedStockCode = useAppStore((s) => s.selectedStockCode);
  const selectedStockName = useAppStore((s) => s.selectedStockName);
  const licenseTier = useAppStore((s) => s.licenseTier);
  const isPro = licenseTier === "pro";

  const [script, setScript] = useState(QUICK_TEMPLATES[0].script);
  const [testStockId, setTestStockId] = useState<number | null>(selectedStockId);
  const [testStockCode, setTestStockCode] = useState(selectedStockCode ?? "");
  const [compiling, setCompiling] = useState(false);
  const [compileResult, setCompileResult] = useState<string | null>(null);
  const [rightTab, setRightTab] = useState<"templates" | "reference">("reference");

  // Sync test stock with app store selection
  useState(() => {
    if (selectedStockId) {
      setTestStockId(selectedStockId);
      setTestStockCode(selectedStockCode ?? "");
    }
  });

  const handleCompile = async () => {
    if (!script.trim()) return;
    setCompiling(true);
    setCompileResult(null);
    try {
      const stockId = testStockId ?? selectedStockId;
      if (!stockId) {
        setCompileResult("请先在K线图中选择一只股票或输入股票代码进行测试");
        setCompiling(false);
        return;
      }

      const result = await invoke<{
        buy_count: number;
        sell_count: number;
        params: Record<string, number>;
        errors: string[];
      }>("execute_custom_script", {
        script,
        stockId,
      });

      const parts: string[] = [];
      if (result.errors.length > 0) {
        parts.push("编译错误:");
        result.errors.forEach((e) => parts.push("  ✗ " + e));
      } else {
        parts.push("✓ 脚本编译通过，执行成功");
      }
      parts.push("");
      parts.push("买入信号: " + result.buy_count + " 次");
      parts.push("卖出信号: " + result.sell_count + " 次");
      if (Object.keys(result.params).length > 0) {
        parts.push("");
        parts.push("参数:");
        parts.push(JSON.stringify(result.params, null, 2));
      }
      setCompileResult(parts.join("\n"));
    } catch (e) {
      setCompileResult("执行失败: " + String(e));
    }
    setCompiling(false);
  };

  const handleLookupStock = useCallback(async () => {
    if (!testStockCode.trim()) return;
    try {
      const s = await invoke<StockInfo | null>("query_stock_by_code", { code: testStockCode.trim() });
      if (s) {
        setTestStockId(s.id);
        setCompileResult("已找到: " + s.code + " " + s.name);
      } else {
        setCompileResult("未找到股票: " + testStockCode);
      }
    } catch (e) {
      setCompileResult("查找失败: " + String(e));
    }
  }, [testStockCode]);

  const lines = script.split("\n");
  const maxLineNum = Math.max(lines.length, 20);

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden", fontFamily: "monospace" }}>
      {/* Header */}
      <div style={{
        padding: "10px 20px", background: "#161616", borderBottom: "1px solid #2A2A2A",
        display: "flex", justifyContent: "space-between", alignItems: "center",
      }}>
        <div style={{ display: "flex", gap: 16, alignItems: "center" }}>
          <span style={{ color: "#CCAA00", fontWeight: 700, fontSize: 15 }}>ME Script 编辑器</span>
          <span style={{ color: "#666666", fontSize: 11 }}>通达信兼容语法 · 40+内置函数 · WASM沙箱编译执行</span>
        </div>
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          {!isPro && (
            <span style={{ color: "#858585", fontSize: 10, background: "#121212", padding: "2px 8px", borderRadius: 3 }}>
              免费版·基础功能
            </span>
          )}
        </div>
      </div>

      {/* Body */}
      <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        {/* Editor panel */}
        <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
          {/* Toolbar */}
          <div style={{
            padding: "8px 16px", background: "#121212", borderBottom: "1px solid #2A2A2A",
            display: "flex", gap: 12, alignItems: "center",
          }}>
            <button onClick={handleCompile} disabled={compiling || !script.trim()}
              style={{
                background: compiling ? "#555" : "#3b82f6", color: "#fff",
                border: "none", padding: "6px 16px", borderRadius: 4,
                cursor: compiling ? "not-allowed" : "pointer",
                fontSize: 12, fontWeight: 600, fontFamily: "monospace",
              }}>
              {compiling ? "编译中..." : "▶ 编译并测试"}
            </button>
            <span style={{ color: "#666666", fontSize: 11 }}>
              测试股票:
            </span>
            <input
              value={testStockCode}
              onChange={(e) => setTestStockCode(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleLookupStock()}
              placeholder="如 000001.SZ"
              style={{
                width: 110, background: "#0C0C0C", border: "1px solid #2A2A2A",
                color: "#CCAA00", padding: "4px 8px", borderRadius: 3,
                fontSize: 12, fontFamily: "monospace", outline: "none",
              }}
            />
            <button onClick={handleLookupStock}
              style={{
                background: "#2A2A2A", color: "#D4D4D4", border: "none",
                padding: "4px 10px", borderRadius: 3, cursor: "pointer",
                fontSize: 11, fontFamily: "monospace",
              }}>
              查找
            </button>
            {selectedStockId && (
              <span style={{ color: "#858585", fontSize: 10 }}>
                (当前图表: {selectedStockCode} {selectedStockName})
              </span>
            )}
          </div>

          {/* Editor with line numbers */}
          <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
            <div style={{
              padding: "12px 0", background: "#0C0C0C",
              borderRight: "1px solid #2A2A2A", minWidth: 44,
              textAlign: "right", userSelect: "none",
              fontSize: 13, lineHeight: "22px", color: "#555",
              fontFamily: '"JetBrains Mono", "Consolas", monospace',
              overflow: "hidden",
            }}>
              {Array.from({ length: maxLineNum }, (_, i) => (
                <div key={i} style={{ paddingRight: 10 }}>
                  {i + 1}
                </div>
              ))}
            </div>
            <textarea
              value={script}
              onChange={(e) => setScript(e.target.value)}
              spellCheck={false}
              style={{
                flex: 1, background: "#111122", color: "#e0e0e0",
                border: "none", padding: "12px 16px",
                fontSize: 13, lineHeight: "22px",
                fontFamily: '"JetBrains Mono", "Consolas", monospace',
                resize: "none", outline: "none",
                tabSize: 2, whiteSpace: "pre",
              }}
            />
          </div>

          {/* Compile output */}
          {compileResult && (
            <div style={{
              padding: "10px 16px", maxHeight: 160, overflow: "auto",
              background: compileResult.includes("✓") ? "#0a1a1a" : "#1a0a0a",
              borderTop: "1px solid #2A2A2A",
              color: compileResult.includes("✓") ? "#26A69A" : "#EF5350",
              fontSize: 12, whiteSpace: "pre-wrap", lineHeight: "20px",
              fontFamily: "monospace",
            }}>
              {compileResult}
            </div>
          )}
        </div>

        {/* Right sidebar */}
        <div style={{
          width: 280, borderLeft: "1px solid #2A2A2A",
          display: "flex", flexDirection: "column", overflow: "hidden",
          background: "#161616",
        }}>
          {/* Tab selector */}
          <div style={{ display: "flex", borderBottom: "1px solid #2A2A2A" }}>
            {(["reference", "templates"] as const).map((tab) => (
              <button key={tab} onClick={() => setRightTab(tab)} style={{
                flex: 1, padding: "8px 12px", border: "none",
                background: rightTab === tab ? "#121212" : "transparent",
                color: rightTab === tab ? "#CCAA00" : "#858585",
                cursor: "pointer", fontSize: 12, fontWeight: rightTab === tab ? 600 : 400,
                fontFamily: "monospace",
                borderBottom: rightTab === tab ? "2px solid #CCAA00" : "2px solid transparent",
              }}>
                {tab === "reference" ? "函数参考" : "快速模板"}
              </button>
            ))}
          </div>

          <div style={{ flex: 1, overflow: "auto" }}>
            {rightTab === "reference" && (
              <div style={{ padding: 12 }}>
                {Object.entries(REFERENCE_CATEGORIES).map(([cat, funcs]) => (
                  <div key={cat} style={{ marginBottom: 16 }}>
                    <div style={{ color: "#CCAA00", fontSize: 12, fontWeight: 600, marginBottom: 8 }}>
                      {cat}
                    </div>
                    {funcs.map((name) => {
                      const fullKey = Object.keys(FUNCTION_REF).find((k) => k.startsWith(name + "(")) ?? name;
                      const desc = FUNCTION_REF[fullKey];
                      return (
                        <div key={name} style={{
                          padding: "4px 0", fontSize: 11, display: "flex",
                          justifyContent: "space-between", cursor: "pointer",
                        }} onClick={() => {
                          setScript((s) => {
                            const sig = name + "()";
                            if (!s.includes(sig)) return s.replace("buy := false;", `buy := ${sig};\n    sell := false;`);
                            return s;
                          });
                        }}>
                          <span style={{ color: "#CCAA00", fontFamily: "monospace" }}>{name}</span>
                          <span style={{ color: "#666666", textAlign: "right", maxWidth: 140 }}>
                            {desc ?? ""}
                          </span>
                        </div>
                      );
                    })}
                  </div>
                ))}
              </div>
            )}

            {rightTab === "templates" && (
              <div style={{ padding: 12 }}>
                {QUICK_TEMPLATES.map((t) => (
                  <div key={t.label} onClick={() => setScript(t.script)}
                    style={{
                      padding: "8px 10px", cursor: "pointer", borderRadius: 4,
                      marginBottom: 6, background: "#121212", border: "1px solid #2A2A2A",
                    }}>
                    <div style={{ color: "#D4D4D4", fontSize: 12, fontWeight: 600, marginBottom: 4 }}>
                      {t.label}
                    </div>
                    <div style={{ color: "#555", fontSize: 10, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {t.script.slice(0, 60)}...
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

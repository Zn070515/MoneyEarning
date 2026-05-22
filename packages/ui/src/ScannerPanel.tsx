import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StockInfo {
  id: number;
  code: string;
  name: string;
  exchange: string;
}

interface IndicatorMeta {
  name: string;
  name_cn: string;
  category: string;
  params: ParamDef[];
  is_free: boolean;
}

interface ParamDef {
  name: string;
  default: number;
  min?: number;
  max?: number;
}

interface ScanResultItem {
  stock_id: number;
  score: number;
  signals: string[];
}

type ScanOp = "COMPARE" | "CROSS";

export function ScannerPanel() {
  const [indicators, setIndicators] = useState<IndicatorMeta[]>([]);
  const [stocks, setStocks] = useState<StockInfo[]>([]);
  const [results, setResults] = useState<(ScanResultItem & { code?: string; name?: string })[]>([]);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState("");

  // Scan expression state
  const [scanOp, setScanOp] = useState<ScanOp>("COMPARE");
  const [indicator, setIndicator] = useState("rsi");
  const [compareOp, setCompareOp] = useState("<");
  const [threshold, setThreshold] = useState("30");
  const [crossDir, setCrossDir] = useState("above");
  const [params, setParams] = useState<Record<string, string>>({});

  useEffect(() => {
    invoke<IndicatorMeta[]>("list_indicators").then(setIndicators).catch(console.error);
    invoke<StockInfo[]>("query_stock_list").then(setStocks).catch(console.error);
  }, []);

  const selectedMeta = indicators.find(i => i.name === indicator);

  const handleIndicatorChange = (name: string) => {
    setIndicator(name);
    const meta = indicators.find(i => i.name === name);
    if (meta) {
      const p: Record<string, string> = {};
      meta.params.forEach(d => { p[d.name] = String(d.default); });
      setParams(p);
    }
  };

  const runScan = useCallback(async () => {
    setLoading(true);
    setStatus("扫描中...");
    try {
      const paramMap: Record<string, number> = {};
      Object.entries(params).forEach(([k, v]) => {
        const n = parseFloat(v);
        if (!isNaN(n)) paramMap[k] = n;
      });

      const expr: Record<string, unknown> = {
        op: scanOp,
        children: [],
        indicator,
        params: paramMap,
        compare_op: scanOp === "COMPARE" ? compareOp : crossDir,
        value: scanOp === "COMPARE" ? parseFloat(threshold) || 0 : null,
      };

      const raw = await invoke<ScanResultItem[]>("run_scanner", {
        stockIds: stocks.map(s => s.id),
        expr,
      });

      // Map stock codes
      const stockMap = new Map(stocks.map(s => [s.id, s]));
      const mapped = raw.map(r => ({
        ...r,
        code: stockMap.get(r.stock_id)?.code,
        name: stockMap.get(r.stock_id)?.name,
      }));
      setResults(mapped);
      setStatus(`找到 ${mapped.length} 个匹配结果`);
    } catch (e) {
      console.error("Scanner error:", e);
      setStatus(`扫描失败: ${e}`);
    }
    setLoading(false);
  }, [scanOp, indicator, compareOp, threshold, crossDir, params, stocks]);

  const categoryLabel: Record<string, string> = {
    trend: "趋势", momentum: "动量", volatility: "波动",
    volume: "成交量", moving_average: "均线", oscillator: "震荡",
    overlay: "叠加", pattern: "形态", candles: "蜡烛图", stats: "统计",
  };

  return (
    <div style={{
      display: "flex", flexDirection: "column", height: "100%",
      background: "#1a1a2e", color: "#ccc", fontFamily: "monospace", fontSize: 13,
    }}>
      {/* Condition Builder */}
      <div style={{
        padding: "12px", borderBottom: "1px solid #2a2a4a",
        background: "#16213e",
      }}>
        <div style={{ fontWeight: 600, marginBottom: 10, color: "#fbbf24" }}>
          选股条件
        </div>

        {/* Operation type */}
        <div style={{ display: "flex", gap: 8, marginBottom: 10 }}>
          <button onClick={() => setScanOp("COMPARE")} style={{
            ...opBtn, background: scanOp === "COMPARE" ? "#fbbf24" : "#2a2a4a",
            color: scanOp === "COMPARE" ? "#000" : "#888",
          }}>
            指标比较
          </button>
          <button onClick={() => setScanOp("CROSS")} style={{
            ...opBtn, background: scanOp === "CROSS" ? "#fbbf24" : "#2a2a4a",
            color: scanOp === "CROSS" ? "#000" : "#888",
          }}>
            交叉信号
          </button>
        </div>

        {/* Indicator select */}
        <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
          <select value={indicator} onChange={e => handleIndicatorChange(e.target.value)}
            style={selectStyle}>
            {indicators.map(ind => (
              <option key={ind.name} value={ind.name}>
                {ind.name_cn} ({ind.name.toUpperCase()})
              </option>
            ))}
          </select>

          {scanOp === "COMPARE" ? (
            <>
              <select value={compareOp} onChange={e => setCompareOp(e.target.value)}
                style={selectStyle}>
                <option value=">">{">"} 大于</option>
                <option value="<">{"<"} 小于</option>
                <option value=">=">{">="} 大于等于</option>
                <option value="<=">{"<="} 小于等于</option>
                <option value="==">== 等于</option>
                <option value="!=">!= 不等于</option>
              </select>
              <input type="number" value={threshold} onChange={e => setThreshold(e.target.value)}
                style={inputStyle} placeholder="阈值" step="any" />
            </>
          ) : (
            <>
              <select value={crossDir} onChange={e => setCrossDir(e.target.value)}
                style={selectStyle}>
                <option value="above">上穿 (金叉)</option>
                <option value="below">下穿 (死叉)</option>
              </select>
              <input type="number" value={threshold} onChange={e => setThreshold(e.target.value)}
                style={{ ...inputStyle, width: 80 }} placeholder="阈值(可选)" step="any" />
            </>
          )}

          <button onClick={runScan} disabled={loading} style={{
            ...opBtn, background: "#fbbf24", color: "#000", fontWeight: 600,
            opacity: loading ? 0.5 : 1,
          }}>
            {loading ? "扫描中..." : "开始扫描"}
          </button>
        </div>

        {/* Params */}
        {selectedMeta && selectedMeta.params.length > 0 && (
          <div style={{ display: "flex", gap: 8, marginTop: 8, flexWrap: "wrap" }}>
            <span style={{ color: "#888", fontSize: 12 }}>参数:</span>
            {selectedMeta.params.map(p => (
              <label key={p.name} style={{ fontSize: 12, color: "#aaa" }}>
                {p.name}
                <input type="number" value={params[p.name] ?? p.default}
                  onChange={e => setParams(prev => ({ ...prev, [p.name]: e.target.value }))}
                  style={{ ...inputStyle, width: 56, marginLeft: 4 }} step="any" />
              </label>
            ))}
          </div>
        )}
      </div>

      {/* Results */}
      <div style={{ flex: 1, overflow: "auto", padding: "8px" }}>
        <div style={{
          padding: "4px 8px", fontSize: 12, color: "#888",
          display: "flex", justifyContent: "space-between",
        }}>
          <span>{status}</span>
          <span>共 {stocks.length} 只股票</span>
        </div>

        {results.length > 0 && (
          <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}>
            <thead>
              <tr style={{ color: "#888", borderBottom: "1px solid #2a2a4a" }}>
                <th style={thStyle}>排名</th>
                <th style={thStyle}>代码</th>
                <th style={thStyle}>名称</th>
                <th style={{ ...thStyle, textAlign: "right" }}>评分</th>
                <th style={thStyle}>信号</th>
              </tr>
            </thead>
            <tbody>
              {results.map((r, i) => (
                <tr key={i} style={{
                  borderBottom: "1px solid #1f1f3a",
                  background: i < 5 ? "rgba(251,191,36,0.05)" : undefined,
                }}>
                  <td style={{ ...tdStyle, color: i < 3 ? "#fbbf24" : "#888" }}>
                    {i + 1}
                  </td>
                  <td style={{ ...tdStyle, color: "#aaa" }}>{r.code}</td>
                  <td style={tdStyle}>{r.name}</td>
                  <td style={{ ...tdStyle, textAlign: "right", color: "#22c55e" }}>
                    {r.score.toFixed(2)}
                  </td>
                  <td style={{ ...tdStyle, color: "#888", fontSize: 11 }}>
                    {r.signals.join(", ")}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {results.length === 0 && status && !loading && (
          <div style={{
            textAlign: "center", color: "#555", marginTop: 40,
            fontSize: 14,
          }}>
            未找到匹配结果
          </div>
        )}
      </div>
    </div>
  );
}

const opBtn: React.CSSProperties = {
  padding: "5px 14px", border: "none", borderRadius: 4,
  cursor: "pointer", fontSize: 13, fontFamily: "monospace",
};

const selectStyle: React.CSSProperties = {
  background: "#0f0f23", color: "#ccc", border: "1px solid #2a2a4a",
  padding: "4px 8px", borderRadius: 4, fontSize: 13, fontFamily: "monospace",
  maxWidth: 200,
};

const inputStyle: React.CSSProperties = {
  background: "#0f0f23", color: "#ccc", border: "1px solid #2a2a4a",
  padding: "4px 8px", borderRadius: 4, fontSize: 13, fontFamily: "monospace",
  width: 90,
};

const thStyle: React.CSSProperties = {
  padding: "6px 8px", textAlign: "left", fontWeight: 600,
};

const tdStyle: React.CSSProperties = {
  padding: "6px 8px",
};

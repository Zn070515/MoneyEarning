import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface RiskOutput {
  total_return: number;
  annual_return: number;
  annual_volatility: number;
  sharpe_ratio: number;
  sortino_ratio: number;
  max_drawdown: number;
  var_95: number;
  cvar_95: number;
  calmar_ratio: number;
  positive_days: number;
  negative_days: number;
  best_day: number;
  worst_day: number;
}

interface PatternResult {
  name: string;
  name_cn: string;
  start_idx: number;
  end_idx: number;
  confidence: number;
  direction: string;
  description: string;
}

export function RiskPanel({ stockId }: { stockId: number | null }) {
  const [risk, setRisk] = useState<RiskOutput | null>(null);
  const [patterns, setPatterns] = useState<PatternResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState("");

  const load = useCallback(async (sid: number) => {
    setLoading(true);
    setStatus("计算中...");
    try {
      const [r, p] = await Promise.all([
        invoke<RiskOutput>("compute_risk", { stockId: sid }),
        invoke<PatternResult[]>("scan_all_patterns", { stockId: sid }),
      ]);
      setRisk(r);
      setPatterns(p);
      setStatus("");
    } catch (e) {
      console.error("Risk panel error:", e);
      setStatus(`加载失败: ${e}`);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    if (stockId) {
      load(stockId);
    } else {
      setRisk(null);
      setPatterns([]);
    }
  }, [stockId, load]);

  if (!stockId) {
    return (
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "center",
        height: "100%", color: "#555", fontFamily: "monospace", fontSize: 14,
      }}>
        请先选择一只股票
      </div>
    );
  }

  return (
    <div style={{
      display: "flex", flexDirection: "column", height: "100%",
      background: "#1a1a2e", color: "#ccc", fontFamily: "monospace",
      fontSize: 13, overflow: "auto",
    }}>
      {loading && (
        <div style={{ padding: 12, color: "#fbbf24", textAlign: "center" }}>
          计算中...
        </div>
      )}
      {status && !loading && (
        <div style={{ padding: 12, color: "#ef4444", textAlign: "center" }}>
          {status}
        </div>
      )}

      {risk && (
        <>
          {/* Risk Metrics */}
          <div style={{ padding: "10px 12px", borderBottom: "1px solid #2a2a4a" }}>
            <div style={{ fontWeight: 600, color: "#fbbf24", marginBottom: 10 }}>
              风险指标
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "6px 16px" }}>
              <Metric label="总收益" value={fmtPct(risk.total_return)} color={risk.total_return >= 0 ? "#22c55e" : "#ef4444"} />
              <Metric label="年化收益" value={fmtPct(risk.annual_return)} color={risk.annual_return >= 0 ? "#22c55e" : "#ef4444"} />
              <Metric label="年化波动" value={fmtPct(risk.annual_volatility)} color="#888" />
              <Metric label="最大回撤" value={fmtPct(risk.max_drawdown)} color="#ef4444" />
              <Metric label="Sharpe" value={risk.sharpe_ratio.toFixed(2)} color={risk.sharpe_ratio >= 1 ? "#22c55e" : risk.sharpe_ratio >= 0 ? "#fbbf24" : "#ef4444"} />
              <Metric label="Sortino" value={risk.sortino_ratio.toFixed(2)} color="#888" />
              <Metric label="Calmar" value={risk.calmar_ratio.toFixed(2)} color="#888" />
              <Metric label="VaR 95%" value={fmtPct(risk.var_95)} color="#ef4444" />
              <Metric label="CVaR 95%" value={fmtPct(risk.cvar_95)} color="#ef4444" />
            </div>
          </div>

          {/* Day stats */}
          <div style={{ padding: "8px 12px", borderBottom: "1px solid #2a2a4a" }}>
            <div style={{ fontSize: 12, color: "#888" }}>
              <span style={{ color: "#22c55e" }}>阳线 {risk.positive_days}天</span>
              <span style={{ margin: "0 8px", color: "#555" }}>|</span>
              <span style={{ color: "#ef4444" }}>阴线 {risk.negative_days}天</span>
              <span style={{ margin: "0 8px", color: "#555" }}>|</span>
              <span>胜率 {(risk.positive_days / (risk.positive_days + risk.negative_days || 1) * 100).toFixed(1)}%</span>
            </div>
            <div style={{ fontSize: 12, color: "#888", marginTop: 4 }}>
              最佳日 <span style={{ color: "#22c55e" }}>{fmtPct(risk.best_day)}</span>
              <span style={{ margin: "0 8px", color: "#555" }}>|</span>
              最差日 <span style={{ color: "#ef4444" }}>{fmtPct(risk.worst_day)}</span>
            </div>
          </div>
        </>
      )}

      {/* Patterns */}
      {patterns.length > 0 && (
        <div style={{ padding: "10px 12px" }}>
          <div style={{ fontWeight: 600, color: "#fbbf24", marginBottom: 8 }}>
            形态识别 ({patterns.length})
          </div>
          {patterns.slice(0, 15).map((p, i) => (
            <div key={i} style={{
              display: "flex", justifyContent: "space-between", alignItems: "center",
              padding: "4px 8px", marginBottom: 3, borderRadius: 3,
              background: p.direction === "bullish" ? "rgba(34,197,94,0.08)"
                : p.direction === "bearish" ? "rgba(239,68,68,0.08)"
                : "rgba(255,255,255,0.03)",
              fontSize: 12,
            }}>
              <span>
                <span style={{
                  color: p.direction === "bullish" ? "#22c55e"
                    : p.direction === "bearish" ? "#ef4444" : "#888",
                  marginRight: 6,
                }}>
                  {p.direction === "bullish" ? "↑" : p.direction === "bearish" ? "↓" : "·"}
                </span>
                {p.name_cn}
              </span>
              <span style={{ color: "#888" }}>
                {(p.confidence * 100).toFixed(0)}%
              </span>
            </div>
          ))}
        </div>
      )}

      {patterns.length === 0 && risk && !loading && (
        <div style={{ padding: 16, color: "#555", textAlign: "center", fontSize: 12 }}>
          未检测到显著形态
        </div>
      )}
    </div>
  );
}

function Metric({ label, value, color }: { label: string; value: string; color: string }) {
  return (
    <div>
      <div style={{ fontSize: 10, color: "#666" }}>{label}</div>
      <div style={{ fontSize: 13, color, fontWeight: 600 }}>{value}</div>
    </div>
  );
}

function fmtPct(v: number): string {
  const pct = v * 100;
  if (Math.abs(pct) < 0.01) return "0.00%";
  return (pct >= 0 ? "+" : "") + pct.toFixed(2) + "%";
}

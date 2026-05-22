import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { OHLCV } from "@me/chart-engine";

interface BtResult {
  total_return: number;
  annual_return: number;
  max_drawdown: number;
  sharpe_ratio: number;
  sortino_ratio: number;
  calmar_ratio: number;
  win_rate: number;
  profit_loss_ratio: number;
  total_trades: number;
  equity_curve: [string, number][];
  monthly_returns: [string, number][];
}

interface Strategy {
  id: number;
  name: string;
  script: string | null;
  params: string | null;
  template_type: string | null;
  created_at: string;
}

interface BacktestPanelProps {
  data: OHLCV[];
}

const TEMPLATES: Record<string, string> = {
  ma_cross: "双均线交叉",
  breakout: "通道突破",
  rsi_mean: "RSI均值回归",
};

export function BacktestPanel({ data }: BacktestPanelProps) {
  const [strategies, setStrategies] = useState<Strategy[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState("ma_cross");
  const [params, setParams] = useState<Record<string, number>>({ fast: 5, slow: 20 });
  const [capital, setCapital] = useState(100000);
  const [commission, setCommission] = useState(0.0003);
  const [stampTax, setStampTax] = useState(0.001);
  const [slippage, setSlippage] = useState(0.001);
  const [positionPct, setPositionPct] = useState(1.0);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<BtResult | null>(null);
  const [error, setError] = useState("");

  const loadStrategies = useCallback(async () => {
    try {
      const data = await invoke<Strategy[]>("strategy_list");
      setStrategies(data);
    } catch (_) {}
  }, []);

  useEffect(() => { loadStrategies(); }, [loadStrategies]);

  const handleTemplate = (t: string) => {
    setSelectedTemplate(t);
    switch (t) {
      case "ma_cross": setParams({ fast: 5, slow: 20 }); break;
      case "breakout": setParams({ period: 20 }); break;
      case "rsi_mean": setParams({ period: 14, oversold: 30, overbought: 70 }); break;
    }
  };

  const handleRun = async () => {
    if (data.length === 0) return;
    setRunning(true);
    setError("");
    try {
      const res = await invoke<BtResult>("run_backtest", {
        data,
        template: selectedTemplate,
        params,
        config: {
          initial_capital: capital,
          commission_rate: commission,
          stamp_tax_rate: stampTax,
          slippage,
          position_pct: positionPct,
        },
      });
      setResult(res);
    } catch (e) {
      setError(String(e));
    }
    setRunning(false);
  };

  const pct = (v: number) => (v * 100).toFixed(2) + "%";
  const fmt = (v: number) => v.toFixed(4);
  const color = (v: number) => v >= 0 ? "#ef4444" : "#22c55e";

  return (
    <div style={{
      background: "#16213e", color: "#ccc", fontFamily: "monospace",
      fontSize: 13, height: "100%", display: "flex", flexDirection: "column",
      overflow: "hidden",
    }}>
      <div style={{
        padding: "10px 12px", borderBottom: "1px solid #2a2a4a",
        fontWeight: 600, color: "#fff", fontSize: 14,
      }}>
        策略回测
      </div>

      <div style={{ flex: 1, overflow: "auto", padding: 12 }}>
        {/* Template */}
        <div style={{ marginBottom: 12 }}>
          <label style={labelStyle}>策略模板</label>
          <select value={selectedTemplate} onChange={e => handleTemplate(e.target.value)}
            style={inputStyle}>
            {Object.entries(TEMPLATES).map(([k, v]) => (
              <option key={k} value={k}>{v}</option>
            ))}
            {strategies.filter(s => s.template_type && !TEMPLATES[s.template_type]).map(s => (
              <option key={s.id} value={s.template_type!}>{s.name}</option>
            ))}
          </select>
        </div>

        {/* Params */}
        <div style={{ marginBottom: 12 }}>
          <label style={labelStyle}>参数</label>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
            {Object.entries(params).map(([k, v]) => (
              <div key={k} style={{ display: "flex", alignItems: "center", gap: 6 }}>
                <span style={{ color: "#aaa", fontSize: 11, width: 80 }}>{k}</span>
                <input
                  type="number" value={v}
                  step={k === "oversold" || k === "overbought" ? 1 : 1}
                  onChange={e => setParams({ ...params, [k]: parseFloat(e.target.value) || 0 })}
                  style={{ ...inputStyle, width: 80, textAlign: "center" }} />
              </div>
            ))}
          </div>
        </div>

        {/* Config */}
        <div style={{ marginBottom: 12 }}>
          <label style={labelStyle}>回测配置</label>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
            <ConfigField label="初始资金" value={capital} onChange={setCapital} />
            <ConfigField label="仓位比例" value={positionPct} onChange={setPositionPct} step={0.1} />
            <ConfigField label="佣金率" value={commission} onChange={setCommission} step={0.0001} />
            <ConfigField label="印花税率" value={stampTax} onChange={setStampTax} step={0.0001} />
            <ConfigField label="滑点" value={slippage} onChange={setSlippage} step={0.0001} />
          </div>
        </div>

        {/* Run */}
        <button onClick={handleRun} disabled={running || data.length === 0}
          style={{
            width: "100%", background: running ? "#8a7a3a" : "#fbbf24",
            color: "#000", border: "none", padding: "8px 16px",
            borderRadius: 4, cursor: running ? "not-allowed" : "pointer",
            fontSize: 14, fontWeight: 600, marginBottom: 12,
          }}>
          {running ? "回测中..." : "开始回测"}
        </button>

        {error && (
          <div style={{
            padding: 8, background: "#3a1a2e", borderRadius: 4,
            color: "#ef4444", fontSize: 12, marginBottom: 12,
          }}>
            {error}
          </div>
        )}

        {/* Results */}
        {result && (
          <div>
            <div style={{
              display: "grid", gridTemplateColumns: "1fr 1fr",
              gap: 6, marginBottom: 12,
            }}>
              <MetricBox label="总收益率" value={pct(result.total_return)} color={color(result.total_return)} />
              <MetricBox label="年化收益" value={pct(result.annual_return)} color={color(result.annual_return)} />
              <MetricBox label="最大回撤" value={pct(result.max_drawdown)} color="#ef4444" />
              <MetricBox label="夏普比率" value={fmt(result.sharpe_ratio)} />
              <MetricBox label="索提诺比率" value={fmt(result.sortino_ratio)} />
              <MetricBox label="卡玛比率" value={fmt(result.calmar_ratio)} />
              <MetricBox label="胜率" value={pct(result.win_rate)} color="#fbbf24" />
              <MetricBox label="盈亏比" value={fmt(result.profit_loss_ratio)} />
            </div>

            <div style={{
              padding: 8, background: "#1a1a2e", borderRadius: 4,
              fontSize: 12, color: "#aaa", textAlign: "center",
            }}>
              总交易：{result.total_trades} 笔
              {result.equity_curve.length > 0 && (
                <span> · 权益曲线：{result.equity_curve.length}点</span>
              )}
            </div>

            {/* Equity mini chart */}
            {result.equity_curve.length > 1 && (
              <EquityMiniChart
                data={result.equity_curve.map(([, v]) => v)}
                initialCapital={capital}
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function ConfigField({ label, value, onChange, step = 1 }: {
  label: string; value: number; onChange: (v: number) => void; step?: number;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
      <span style={{ color: "#aaa", fontSize: 11, width: 70 }}>{label}</span>
      <input type="number" value={value} step={step}
        onChange={e => onChange(parseFloat(e.target.value) || 0)}
        style={{ ...inputStyle, width: 100, textAlign: "center" }} />
    </div>
  );
}

function MetricBox({ label, value, color: c }: {
  label: string; value: string; color?: string;
}) {
  return (
    <div style={{
      padding: "6px 8px", background: "#1a1a2e", borderRadius: 4,
      textAlign: "center",
    }}>
      <div style={{ color: "#888", fontSize: 10, marginBottom: 2 }}>{label}</div>
      <div style={{ color: c ?? "#fff", fontSize: 14, fontWeight: 600 }}>{value}</div>
    </div>
  );
}

function EquityMiniChart({ data, initialCapital }: { data: number[]; initialCapital: number }) {
  const h = 80;
  const min = Math.min(...data, initialCapital);
  const max = Math.max(...data, initialCapital);
  const rng = max - min || 1;
  const w = 280;

  const points = data.map((v, i) => {
    const x = (i / (data.length - 1)) * w;
    const y = h - ((v - min) / rng) * h;
    return `${x},${y}`;
  }).join(" ");

  return (
    <div style={{ marginTop: 8, padding: "6px", background: "#1a1a2e", borderRadius: 4 }}>
      <div style={{ color: "#888", fontSize: 10, marginBottom: 4 }}>权益曲线</div>
      <svg width="100%" height={h} style={{ display: "block" }}>
        {/* Baseline */}
        <line x1={0} y1={h - ((initialCapital - min) / rng) * h}
          x2={w} y2={h - ((initialCapital - min) / rng) * h}
          stroke="#3a3a5a" strokeDasharray="4,4" />
        {/* Equity line */}
        <polyline points={points} fill="none"
          stroke={data[data.length - 1] >= initialCapital ? "#ef4444" : "#22c55e"}
          strokeWidth={1.5} />
      </svg>
    </div>
  );
}

const labelStyle: React.CSSProperties = {
  fontSize: 11, color: "#888", marginBottom: 4, display: "block",
};

const inputStyle: React.CSSProperties = {
  background: "#1a1a2e", border: "1px solid #3a3a5a",
  color: "#fff", padding: "4px 8px", borderRadius: 4, fontSize: 12,
  fontFamily: "monospace", outline: "none",
};

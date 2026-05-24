import { useState, useEffect, useCallback } from "react";
import { BacktestPanel } from "@me/ui";
import { useBacktestStore, type TradeRecord } from "../stores/backtestStore";
import { useAppStore } from "../stores/appStore";
import { invoke } from "@tauri-apps/api/core";
import {
  generateBacktestReportMarkdown,
  downloadMarkdownReport,
} from "../utils/exportReport";

interface StrategyTemplate {
  name: string;
  name_cn: string;
  category: string;
  is_free: boolean;
  params: Record<string, number>;
  description: string;
}

const CATEGORY_ORDER = ["趋势跟踪", "均值回归", "动量", "突破", "复合"];

export default function BacktestPage() {
  const config = useBacktestStore((s) => s.config);
  const updateConfig = useBacktestStore((s) => s.updateConfig);
  const running = useBacktestStore((s) => s.running);
  const result = useBacktestStore((s) => s.result);
  const error = useBacktestStore((s) => s.error);
  const setRunning = useBacktestStore((s) => s.setRunning);
  const selectedStockId = useAppStore((s) => s.selectedStockId);
  const selectedStockCode = useAppStore((s) => s.selectedStockCode);
  const licenseTier = useAppStore((s) => s.licenseTier);

  const [templates, setTemplates] = useState<StrategyTemplate[]>([]);
  const [progress, setProgress] = useState({ current: 0, total: 0 });
  const setResult = useBacktestStore((s) => s.setResult);

  const loadTemplates = useCallback(async () => {
    try {
      const data = await invoke<StrategyTemplate[]>("list_strategy_templates");
      setTemplates(data);
    } catch (_) {}
  }, []);

  useEffect(() => { loadTemplates(); }, [loadTemplates]);

  const grouped: Record<string, StrategyTemplate[]> = {};
  templates.forEach((t) => {
    if (!grouped[t.category]) grouped[t.category] = [];
    grouped[t.category].push(t);
  });

  const handleRunBacktest = async () => {
    if (!selectedStockId) return;
    setRunning(true); setProgress({ current: 0, total: 100 });
    try {
      const prices = await invoke<Array<{
        trade_date: string; open: number; high: number; low: number;
        close: number; volume: number; amount: number; turnover?: number;
      }>>("query_daily_prices", {
        stockId: selectedStockId,
        startDate: "2020-01-01",
        endDate: "2099-12-31",
      });
      if (prices.length === 0) {
        setRunning(false);
        setResult(null, "该股票无日线数据，请先导入数据"); return;
      }
      setProgress({ current: 30, total: 100 });
      const data = prices.map((p) => ({
        time: new Date(p.trade_date).getTime() / 1000,
        open: p.open, high: p.high, low: p.low, close: p.close,
        volume: p.volume, amount: p.amount, turnover: p.turnover,
      }));
      const sel = templates.find((t) => t.name === config.template);
      setProgress({ current: 60, total: 100 });
      const raw = await invoke<Record<string, unknown>>("run_backtest", {
        data, template: config.template, params: sel?.params ?? {},
        config: {
          initial_capital: config.initialCapital,
          commission_rate: config.commissionRate,
          stamp_tax_rate: 0.001, slippage: config.slippage,
          position_pct: 0.95,
        },
      });
      setProgress({ current: 100, total: 100 });
      setResult({
        totalReturn: Number(raw.total_return ?? 0),
        annualReturn: Number(raw.annual_return ?? 0),
        maxDrawdown: Number(raw.max_drawdown ?? 0),
        sharpeRatio: Number(raw.sharpe_ratio ?? 0),
        sortinoRatio: Number(raw.sortino_ratio ?? 0),
        calmarRatio: Number(raw.calmar_ratio ?? 0),
        winRate: Number(raw.win_rate ?? 0),
        totalTrades: Number(raw.total_trades ?? 0),
        equityCurve: (Array.isArray(raw.equity_curve) ? raw.equity_curve : []) as [string, number][],
        trades: (Array.isArray(raw.trades) ? raw.trades : []).map((t: Record<string, unknown>) => ({
          buy_date: String(t.buy_date ?? ""),
          sell_date: String(t.sell_date ?? ""),
          buy_price: Number(t.buy_price ?? 0),
          sell_price: Number(t.sell_price ?? 0),
          pnl: Number(t.pnl ?? 0),
          pnl_pct: Number(t.pnl_pct ?? 0),
          holding_days: Number(t.holding_days ?? 0),
        })),
        maxDrawdownDuration: Number(raw.max_drawdown_duration ?? 0),
        annualVolatility: Number(raw.annual_volatility ?? 0),
      });
    } catch (e) { setRunning(false); setResult(null, String(e)); }
  };

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
      {/* Page header */}
      <div style={{
        padding: "12px 20px", background: "#161616",
        borderBottom: "1px solid #2A2A2A", display: "flex",
        alignItems: "center", gap: 16, flexShrink: 0,
      }}>
        <h2 style={{ color: "#CCAA00", fontSize: 16, fontFamily: "monospace", margin: 0 }}>
          策略回测
        </h2>
        {selectedStockCode && (
          <span style={{ color: "#858585", fontSize: 12, fontFamily: "monospace" }}>
            当前标的: {selectedStockCode}
          </span>
        )}
      </div>

      {/* Configuration bar */}
      <div style={{
        padding: "8px 20px", background: "#1A1A1A",
        borderBottom: "1px solid #2A2A2A", display: "flex",
        gap: 16, alignItems: "center", flexWrap: "wrap",
        fontSize: 12, fontFamily: "monospace", color: "#858585", flexShrink: 0,
      }}>
        <label>
          模板:
          <select value={config.template} onChange={(e) => updateConfig({ template: e.target.value })}
            style={selectStyle}>
            {CATEGORY_ORDER.map((cat) => {
              const group = grouped[cat];
              if (!group) return null;
              return (
                <optgroup key={cat} label={`── ${cat} ──`}>
                  {group.map((s) => (
                    <option key={s.name} value={s.name} disabled={!s.is_free && licenseTier !== "pro"}>
                      {s.is_free ? "▸" : "★"} {s.name_cn}
                    </option>
                  ))}
                </optgroup>
              );
            })}
          </select>
          {licenseTier !== "pro" && <span style={{ color: "#666666", fontSize: 10, marginLeft: 4 }}>(★ = PRO)</span>}
        </label>
        <label>
          初始资金:
          <input type="number" value={config.initialCapital}
            onChange={(e) => updateConfig({ initialCapital: Number(e.target.value) })}
            style={inputStyle} />
        </label>
        <label>
          佣金:
          <input type="number" step="0.0001" value={config.commissionRate}
            onChange={(e) => updateConfig({ commissionRate: Number(e.target.value) })}
            style={{ ...inputStyle, width: 70 }} />
        </label>
        <label>
          滑点:
          <input type="number" step="0.0001" value={config.slippage}
            onChange={(e) => updateConfig({ slippage: Number(e.target.value) })}
            style={{ ...inputStyle, width: 70 }} />
        </label>
        <button
          disabled={running || !selectedStockId}
          onClick={handleRunBacktest}
          style={{
            padding: "4px 16px",
            background: running || !selectedStockId ? "#2A2A2A" : "#CCAA00",
            color: running || !selectedStockId ? "#858585" : "#000",
            border: "none", borderRadius: 4,
            cursor: running || !selectedStockId ? "not-allowed" : "pointer",
            fontFamily: "monospace", fontSize: 12, fontWeight: 600,
          }}
        >
          {running ? "运行中..." : "▶ 运行回测"}
        </button>
        {result && (
          <button onClick={() => {
            const md = generateBacktestReportMarkdown(config, result, selectedStockCode ?? "");
            downloadMarkdownReport(md, `backtest-${config.template}-${Date.now()}.md`);
          }} style={{
            padding: "4px 16px", background: "#26A69A", color: "#000",
            border: "none", borderRadius: 4, cursor: "pointer",
            fontFamily: "monospace", fontSize: 12, fontWeight: 600,
          }}>
            导出报告 .md
          </button>
        )}
        {!selectedStockId && (
          <span style={{ color: "#666666", fontSize: 11 }}>请先在图表页面选择股票</span>
        )}
      </div>

      {/* Progress bar */}
      {running && progress.total > 0 && (
        <div style={{ padding: "0 20px", flexShrink: 0 }}>
          <div style={{
            height: 3, background: "#2A2A2A", borderRadius: 2,
            overflow: "hidden", marginTop: 2,
          }}>
            <div style={{
              height: "100%", width: `${(progress.current / progress.total) * 100}%`,
              background: "linear-gradient(90deg, #CCAA00, #7E57C2)",
              borderRadius: 2, transition: "width 0.3s ease",
            }} />
          </div>
          <div style={{
            color: "#858585", fontSize: 10, fontFamily: "monospace",
            textAlign: "right", marginTop: 1,
          }}>
            {Math.round((progress.current / progress.total) * 100)}%
          </div>
        </div>
      )}

      {/* Results summary */}
      {result && (
        <div style={{
          padding: "10px 20px", background: "#0C0C0C",
          borderBottom: "1px solid #2A2A2A", display: "flex",
          gap: 24, flexWrap: "wrap", fontSize: 12,
          fontFamily: "monospace", flexShrink: 0,
        }}>
          <Metric label="总收益" value={`${(result.totalReturn * 100).toFixed(2)}%`} positive={result.totalReturn > 0} />
          <Metric label="年化收益" value={`${(result.annualReturn * 100).toFixed(2)}%`} positive={result.annualReturn > 0} />
          <Metric label="最大回撤" value={`${(result.maxDrawdown * 100).toFixed(2)}%`} />
          <Metric label="回撤持续" value={`${result.maxDrawdownDuration}天`} />
          <Metric label="夏普比率" value={result.sharpeRatio.toFixed(2)} />
          <Metric label="年化波动率" value={`${(result.annualVolatility * 100).toFixed(2)}%`} />
          <Metric label="胜率" value={`${(result.winRate * 100).toFixed(1)}%`} />
          <Metric label="交易次数" value={String(result.totalTrades)} />
        </div>
      )}

      {/* Trades table */}
      {result && result.trades.length > 0 && (
        <div style={{
          margin: "0 20px 8px", padding: "8px 12px",
          background: "#0C0C0C", borderBottom: "1px solid #2A2A2A",
          flexShrink: 0,
        }}>
          <div style={{
            color: "#CCAA00", fontSize: 12, fontFamily: "monospace",
            fontWeight: 600, marginBottom: 6,
          }}>
            交易明细 ({result.trades.length}笔)
          </div>
          <TradeTable trades={result.trades} />
        </div>
      )}

      {/* Equity curve with trade markers */}
      {result && result.equityCurve.length > 1 && (
        <div style={{
          margin: "0 20px 8px", padding: "8px 12px",
          background: "#0C0C0C", flexShrink: 0,
        }}>
          <div style={{
            color: "#CCAA00", fontSize: 12, fontFamily: "monospace",
            fontWeight: 600, marginBottom: 6,
          }}>
            权益曲线
          </div>
          <EquityChart
            equityCurve={result.equityCurve}
            trades={result.trades}
            initialCapital={config.initialCapital}
          />
        </div>
      )}

      {error && (
        <div style={{
          padding: "8px 20px", background: "#2a1525", color: "#EF5350",
          fontSize: 12, fontFamily: "monospace", flexShrink: 0,
        }}>
          {error}
        </div>
      )}

      {/* Main content */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        <BacktestPanel data={[]} isPro={licenseTier === "pro"} />
      </div>
    </div>
  );
}

function Metric({
  label,
  value,
  positive,
}: {
  label: string;
  value: string;
  positive?: boolean;
}) {
  return (
    <div>
      <span style={{ color: "#858585" }}>{label}: </span>
      <span
        style={{
          color:
            positive === undefined
              ? "#D4D4D4"
              : positive
                ? "#26A69A"
                : "#EF5350",
          fontWeight: 600,
        }}
      >
        {value}
      </span>
    </div>
  );
}

function TradeTable({ trades }: { trades: TradeRecord[] }) {
  const [showAll, setShowAll] = useState(false);
  const visible = showAll ? trades : trades.slice(-10);
  return (
    <div>
      <div style={{
        display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr 90px 1fr 80px",
        gap: 2, fontSize: 10, fontFamily: "monospace", color: "#858585",
        padding: "3px 4px", borderBottom: "1px solid #2A2A2A",
      }}>
        <span>买入日期</span>
        <span>卖出日期</span>
        <span style={{ textAlign: "right" }}>买入价</span>
        <span style={{ textAlign: "right" }}>卖出价</span>
        <span style={{ textAlign: "right" }}>盈亏</span>
        <span style={{ textAlign: "right" }}>盈亏%</span>
        <span style={{ textAlign: "right" }}>持仓天数</span>
      </div>
      {visible.map((t, i) => {
        const pnlColor = t.pnl >= 0 ? "#26A69A" : "#EF5350";
        return (
          <div key={i} style={{
            display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr 90px 1fr 80px",
            gap: 2, fontSize: 11, fontFamily: "monospace", color: "#D4D4D4",
            padding: "2px 4px", borderBottom: "1px solid #1A1A1A",
          }}>
            <span>{t.buy_date}</span>
            <span>{t.sell_date}</span>
            <span style={{ textAlign: "right" }}>{t.buy_price.toFixed(2)}</span>
            <span style={{ textAlign: "right" }}>{t.sell_price.toFixed(2)}</span>
            <span style={{ textAlign: "right", color: pnlColor, fontWeight: 600 }}>{t.pnl >= 0 ? "+" : ""}{t.pnl.toFixed(0)}</span>
            <span style={{ textAlign: "right", color: pnlColor }}>{t.pnl_pct >= 0 ? "+" : ""}{(t.pnl_pct * 100).toFixed(2)}%</span>
            <span style={{ textAlign: "right" }}>{t.holding_days}天</span>
          </div>
        );
      })}
      {trades.length > 10 && (
        <button onClick={() => setShowAll(!showAll)} style={{
          width: "100%", padding: "4px", marginTop: 4,
          background: "#121212", border: "1px solid #2A2A2A",
          color: "#CCAA00", cursor: "pointer", fontSize: 11,
          fontFamily: "monospace", borderRadius: 3,
        }}>
          {showAll ? "收起" : `查看全部 ${trades.length} 笔交易`}
        </button>
      )}
    </div>
  );
}

function EquityChart({
  equityCurve,
  trades,
  initialCapital,
}: {
  equityCurve: [string, number][];
  trades: TradeRecord[];
  initialCapital: number;
}) {
  const h = 160;
  const w = 800;
  const pad = { top: 10, right: 40, bottom: 30, left: 50 };
  const chartW = w - pad.left - pad.right;
  const chartH = h - pad.top - pad.bottom;

  const gradId = `equityGrad-${Math.random().toString(36).slice(2, 8)}`;

  const values = equityCurve.map(([, v]) => v);
  const min = Math.min(...values, initialCapital * 0.9);
  const max = Math.max(...values, initialCapital * 1.1);
  const rng = max - min || 1;

  const toX = (i: number) => pad.left + (i / (values.length - 1)) * chartW;
  const toY = (v: number) => pad.top + chartH - ((v - min) / rng) * chartH;

  // Build area path and line path
  const areaPath = values.map((v, i) => {
    const x = toX(i);
    const y = toY(v);
    return `${i === 0 ? "M" : "L"} ${x} ${y}`;
  }).join(" ") + ` L ${toX(values.length - 1)} ${toY(initialCapital)} L ${toX(0)} ${toY(initialCapital)} Z`;

  const linePath = values.map((v, i) => {
    const x = toX(i);
    const y = toY(v);
    return `${i === 0 ? "M" : "L"} ${x} ${y}`;
  }).join(" ");

  const initY = toY(initialCapital);

  // Trade markers: find closest equity points
  const markers = trades.map((t) => {
    const buyIdx = equityCurve.findIndex(([d]) => d === t.buy_date);
    const sellIdx = equityCurve.findIndex(([d]) => d === t.sell_date);
    return { buyDate: t.buy_date, sellDate: t.sell_date, pnl: t.pnl, buyIdx, sellIdx };
  }).filter((m) => m.buyIdx >= 0 && m.sellIdx >= 0);

  // Estimate drawdown overlay — track peak per point for correct Y positioning
  const ddPath = (() => {
    let peak = values[0];
    const peaks: number[] = [];
    const ddValues = values.map((v) => {
      if (v > peak) peak = v;
      peaks.push(peak);
      return peak > 0 ? (v - peak) / peak : 0;
    });
    const scaledDD = ddValues.map((dd, i) => {
      const x = toX(i);
      const y = toY(peaks[i] * (1 + dd));
      return `${i === 0 ? "M" : "L"} ${x} ${y}`;
    }).join(" ");
    return scaledDD;
  })();

  // Y-axis labels
  const yTicks = 5;
  const yLabels = Array.from({ length: yTicks }, (_, i) => {
    const v = min + (rng / (yTicks - 1)) * i;
    return { v, y: toY(v) };
  });

  return (
    <svg viewBox={`0 0 ${w} ${h}`} style={{ width: "100%", display: "block", background: "#0C0C0C", borderRadius: 4 }}>
      {/* Grid lines */}
      {yLabels.map(({ v, y }) => (
        <g key={v}>
          <line x1={pad.left} y1={y} x2={w - pad.right} y2={y} stroke="#1A1A1A" strokeWidth={0.5} />
          <text x={pad.left - 6} y={y + 3} textAnchor="end" fill="#666666" fontSize={9} fontFamily="monospace">
            {v >= 10000 ? `${(v / 10000).toFixed(1)}万` : v.toFixed(0)}
          </text>
        </g>
      ))}

      {/* Initial capital reference line */}
      <line x1={pad.left} y1={initY} x2={w - pad.right} y2={initY} stroke="#CCAA00" strokeWidth={0.5} strokeDasharray="4,3" opacity={0.5} />
      <text x={w - pad.right + 4} y={initY + 3} fill="#CCAA00" fontSize={8} fontFamily="monospace" opacity={0.6}>初始</text>

      {/* Equity area fill */}
      <path d={areaPath} fill="url(#${gradId})" opacity={0.15} />
      <defs>
        <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={values[values.length - 1] >= initialCapital ? "#26A69A" : "#EF5350"} />
          <stop offset="100%" stopColor={values[values.length - 1] >= initialCapital ? "#26A69A" : "#EF5350"} stopOpacity={0} />
        </linearGradient>
      </defs>

      {/* Equity line */}
      <path d={linePath} fill="none" stroke={values[values.length - 1] >= initialCapital ? "#26A69A" : "#EF5350"} strokeWidth={1.5} />

      {/* Trade markers */}
      {markers.slice(-50).map((m, i) => {
        const isWin = m.pnl >= 0;
        return (
          <g key={`${m.buyDate}-${m.sellDate}`}>
            {/* Buy marker */}
            <circle cx={toX(m.buyIdx)} cy={toY(values[m.buyIdx])} r={2.5}
              fill={isWin ? "#26A69A" : "#EF5350"} stroke="#0C0C0C" strokeWidth={0.5} />
            <text x={toX(m.buyIdx)} y={toY(values[m.buyIdx]) - 6}
              textAnchor="middle" fill={isWin ? "#26A69A" : "#EF5350"} fontSize={7} fontFamily="monospace">B</text>
            {/* Sell marker */}
            <circle cx={toX(m.sellIdx)} cy={toY(values[m.sellIdx])} r={2.5}
              fill={isWin ? "#26A69A" : "#EF5350"} stroke="#0C0C0C" strokeWidth={0.5} />
            <text x={toX(m.sellIdx)} y={toY(values[m.sellIdx]) - 6}
              textAnchor="middle" fill={isWin ? "#26A69A" : "#EF5350"} fontSize={7} fontFamily="monospace">S</text>
          </g>
        );
      })}

      {/* X-axis label */}
      <text x={w / 2} y={h - 4} textAnchor="middle" fill="#666666" fontSize={9} fontFamily="monospace">
        {equityCurve[0][0]} ~ {equityCurve[equityCurve.length - 1][0]} ({equityCurve.length}采样点)
      </text>
    </svg>
  );
}

const selectStyle: React.CSSProperties = {
  marginLeft: 6,
  padding: "2px 6px",
  background: "#161616",
  color: "#D4D4D4",
  border: "1px solid #2A2A2A",
  borderRadius: 3,
  fontFamily: "monospace",
  fontSize: 12,
};

const inputStyle: React.CSSProperties = {
  marginLeft: 6,
  padding: "2px 6px",
  width: 80,
  background: "#161616",
  color: "#D4D4D4",
  border: "1px solid #2A2A2A",
  borderRadius: 3,
  fontFamily: "monospace",
  fontSize: 12,
};

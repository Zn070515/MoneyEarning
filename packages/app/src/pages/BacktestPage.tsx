import { useState, useEffect, useCallback } from "react";
import { BacktestPanel } from "@me/ui";
import { useBacktestStore } from "../stores/backtestStore";
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
        totalReturn: raw.total_return as number,
        annualReturn: raw.annual_return as number,
        maxDrawdown: raw.max_drawdown as number,
        sharpeRatio: raw.sharpe_ratio as number,
        sortinoRatio: raw.sortino_ratio as number,
        calmarRatio: raw.calmar_ratio as number,
        winRate: raw.win_rate as number,
        totalTrades: raw.total_trades as number,
        equityCurve: raw.equity_curve as [string, number][],
      });
    } catch (e) { setResult(null, String(e)); }
  };

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
      {/* Page header */}
      <div style={{
        padding: "12px 20px", background: "#16213e",
        borderBottom: "1px solid #2a2a4a", display: "flex",
        alignItems: "center", gap: 16, flexShrink: 0,
      }}>
        <h2 style={{ color: "#fbbf24", fontSize: 16, fontFamily: "monospace", margin: 0 }}>
          策略回测
        </h2>
        {selectedStockCode && (
          <span style={{ color: "#888", fontSize: 12, fontFamily: "monospace" }}>
            当前标的: {selectedStockCode}
          </span>
        )}
      </div>

      {/* Configuration bar */}
      <div style={{
        padding: "8px 20px", background: "#1a1a2e",
        borderBottom: "1px solid #2a2a4a", display: "flex",
        gap: 16, alignItems: "center", flexWrap: "wrap",
        fontSize: 12, fontFamily: "monospace", color: "#aaa", flexShrink: 0,
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
            background: running || !selectedStockId ? "#3a3a5a" : "#fbbf24",
            color: running || !selectedStockId ? "#888" : "#000",
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
            padding: "4px 16px", background: "#22c55e", color: "#000",
            border: "none", borderRadius: 4, cursor: "pointer",
            fontFamily: "monospace", fontSize: 12, fontWeight: 600,
          }}>
            导出报告 .md
          </button>
        )}
        {!selectedStockId && (
          <span style={{ color: "#666", fontSize: 11 }}>请先在图表页面选择股票</span>
        )}
      </div>

      {/* Progress bar */}
      {running && progress.total > 0 && (
        <div style={{ padding: "0 20px", flexShrink: 0 }}>
          <div style={{
            height: 3, background: "#2a2a4a", borderRadius: 2,
            overflow: "hidden", marginTop: 2,
          }}>
            <div style={{
              height: "100%", width: `${(progress.current / progress.total) * 100}%`,
              background: "linear-gradient(90deg, #00D8FF, #7C3CFF)",
              borderRadius: 2, transition: "width 0.3s ease",
            }} />
          </div>
          <div style={{
            color: "#888", fontSize: 10, fontFamily: "monospace",
            textAlign: "right", marginTop: 1,
          }}>
            {Math.round((progress.current / progress.total) * 100)}%
          </div>
        </div>
      )}

      {/* Results summary */}
      {result && (
        <div style={{
          padding: "10px 20px", background: "#0f0f23",
          borderBottom: "1px solid #2a2a4a", display: "flex",
          gap: 24, flexWrap: "wrap", fontSize: 12,
          fontFamily: "monospace", flexShrink: 0,
        }}>
          <Metric label="总收益" value={`${(result.totalReturn * 100).toFixed(2)}%`} positive={result.totalReturn > 0} />
          <Metric label="年化收益" value={`${(result.annualReturn * 100).toFixed(2)}%`} positive={result.annualReturn > 0} />
          <Metric label="最大回撤" value={`${(result.maxDrawdown * 100).toFixed(2)}%`} />
          <Metric label="夏普比率" value={result.sharpeRatio.toFixed(2)} />
          <Metric label="胜率" value={`${(result.winRate * 100).toFixed(1)}%`} />
          <Metric label="交易次数" value={String(result.totalTrades)} />
        </div>
      )}

      {error && (
        <div style={{
          padding: "8px 20px", background: "#3a1a1a", color: "#f87171",
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
      <span style={{ color: "#888" }}>{label}: </span>
      <span
        style={{
          color:
            positive === undefined
              ? "#ccc"
              : positive
                ? "#22c55e"
                : "#ef4444",
          fontWeight: 600,
        }}
      >
        {value}
      </span>
    </div>
  );
}

const selectStyle: React.CSSProperties = {
  marginLeft: 6,
  padding: "2px 6px",
  background: "#16213e",
  color: "#ccc",
  border: "1px solid #2a2a4a",
  borderRadius: 3,
  fontFamily: "monospace",
  fontSize: 12,
};

const inputStyle: React.CSSProperties = {
  marginLeft: 6,
  padding: "2px 6px",
  width: 80,
  background: "#16213e",
  color: "#ccc",
  border: "1px solid #2a2a4a",
  borderRadius: 3,
  fontFamily: "monospace",
  fontSize: 12,
};

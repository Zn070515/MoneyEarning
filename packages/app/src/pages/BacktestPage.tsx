import { BacktestPanel } from "@me/ui";
import { useBacktestStore } from "../stores/backtestStore";
import { useAppStore } from "../stores/appStore";
import {
  generateBacktestReportMarkdown,
  downloadMarkdownReport,
} from "../utils/exportReport";

export default function BacktestPage() {
  const config = useBacktestStore((s) => s.config);
  const updateConfig = useBacktestStore((s) => s.updateConfig);
  const running = useBacktestStore((s) => s.running);
  const result = useBacktestStore((s) => s.result);
  const error = useBacktestStore((s) => s.error);
  const setRunning = useBacktestStore((s) => s.setRunning);
  const selectedStockId = useAppStore((s) => s.selectedStockId);
  const selectedStockCode = useAppStore((s) => s.selectedStockCode);

  return (
    <div
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Page header */}
      <div
        style={{
          padding: "12px 20px",
          background: "#16213e",
          borderBottom: "1px solid #2a2a4a",
          display: "flex",
          alignItems: "center",
          gap: 16,
          flexShrink: 0,
        }}
      >
        <h2
          style={{
            color: "#fbbf24",
            fontSize: 16,
            fontFamily: "monospace",
            margin: 0,
          }}
        >
          策略回测
        </h2>
        {selectedStockCode && (
          <span
            style={{
              color: "#888",
              fontSize: 12,
              fontFamily: "monospace",
            }}
          >
            当前标的: {selectedStockCode}
          </span>
        )}
      </div>

      {/* Configuration bar */}
      <div
        style={{
          padding: "8px 20px",
          background: "#1a1a2e",
          borderBottom: "1px solid #2a2a4a",
          display: "flex",
          gap: 16,
          alignItems: "center",
          flexWrap: "wrap",
          fontSize: 12,
          fontFamily: "monospace",
          color: "#aaa",
          flexShrink: 0,
        }}
      >
        <label>
          模板:
          <select
            value={config.template}
            onChange={(e) => updateConfig({ template: e.target.value })}
            style={selectStyle}
          >
            <option value="ma_cross">MA 双均线</option>
            <option value="macd_signal">MACD 信号</option>
            <option value="rsi_mean_reversion">RSI 均值回归</option>
            <option value="bollinger_breakout">布林带突破</option>
            <option value="turtle_trend">海龟趋势</option>
          </select>
        </label>
        <label>
          初始资金:
          <input
            type="number"
            value={config.initialCapital}
            onChange={(e) =>
              updateConfig({ initialCapital: Number(e.target.value) })
            }
            style={inputStyle}
          />
        </label>
        <label>
          佣金:
          <input
            type="number"
            step="0.0001"
            value={config.commissionRate}
            onChange={(e) =>
              updateConfig({ commissionRate: Number(e.target.value) })
            }
            style={{ ...inputStyle, width: 70 }}
          />
        </label>
        <label>
          滑点:
          <input
            type="number"
            step="0.0001"
            value={config.slippage}
            onChange={(e) =>
              updateConfig({ slippage: Number(e.target.value) })
            }
            style={{ ...inputStyle, width: 70 }}
          />
        </label>
        <button
          disabled={running || !selectedStockId}
          onClick={() => {
            setRunning(true);
            // BacktestPanel handles the actual run internally
          }}
          style={{
            padding: "4px 16px",
            background:
              running || !selectedStockId ? "#3a3a5a" : "#fbbf24",
            color: running || !selectedStockId ? "#888" : "#000",
            border: "none",
            borderRadius: 4,
            cursor:
              running || !selectedStockId ? "not-allowed" : "pointer",
            fontFamily: "monospace",
            fontSize: 12,
            fontWeight: 600,
          }}
        >
          {running ? "运行中..." : "▶ 运行回测"}
        </button>
        {result && (
          <button
            onClick={() => {
              const md = generateBacktestReportMarkdown(
                config,
                result,
                selectedStockCode,
              );
              downloadMarkdownReport(
                md,
                `backtest-${config.template}-${Date.now()}.md`,
              );
            }}
            style={{
              padding: "4px 16px",
              background: "#22c55e",
              color: "#000",
              border: "none",
              borderRadius: 4,
              cursor: "pointer",
              fontFamily: "monospace",
              fontSize: 12,
              fontWeight: 600,
            }}
          >
            导出报告 .md
          </button>
        )}
        {!selectedStockId && (
          <span style={{ color: "#666", fontSize: 11 }}>
            请先在图表页面选择股票
          </span>
        )}
      </div>

      {/* Results summary */}
      {result && (
        <div
          style={{
            padding: "10px 20px",
            background: "#0f0f23",
            borderBottom: "1px solid #2a2a4a",
            display: "flex",
            gap: 24,
            flexWrap: "wrap",
            fontSize: 12,
            fontFamily: "monospace",
            flexShrink: 0,
          }}
        >
          <Metric
            label="总收益"
            value={`${(result.totalReturn * 100).toFixed(2)}%`}
            positive={result.totalReturn > 0}
          />
          <Metric
            label="年化收益"
            value={`${(result.annualReturn * 100).toFixed(2)}%`}
            positive={result.annualReturn > 0}
          />
          <Metric label="最大回撤" value={`${(result.maxDrawdown * 100).toFixed(2)}%`} />
          <Metric label="夏普比率" value={result.sharpeRatio.toFixed(2)} />
          <Metric label="胜率" value={`${(result.winRate * 100).toFixed(1)}%`} />
          <Metric label="交易次数" value={String(result.totalTrades)} />
        </div>
      )}

      {error && (
        <div
          style={{
            padding: "8px 20px",
            background: "#3a1a1a",
            color: "#f87171",
            fontSize: 12,
            fontFamily: "monospace",
            flexShrink: 0,
          }}
        >
          {error}
        </div>
      )}

      {/* Main content */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        <BacktestPanel data={[]} />
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

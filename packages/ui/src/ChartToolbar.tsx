import type { ChartType } from "@me/chart-engine";
import type { DrawingTool } from "./KLineChart";
import { drawingToolLabel } from "./KLineChart";

export type Period = "1min" | "5min" | "15min" | "30min" | "60min" | "D" | "W" | "M";

interface ChartToolbarProps {
  chartType: ChartType;
  onChartTypeChange: (t: ChartType) => void;
  activeTool: DrawingTool | null;
  onToolChange: (t: DrawingTool | null) => void;
  onClearDrawings: () => void;
  drawingCount: number;
  gridMode?: boolean;
  onToggleGridMode?: () => void;
  period?: Period;
  onPeriodChange?: (p: Period) => void;
}

const CHART_TYPES: Array<{ key: ChartType; label: string }> = [
  { key: "candlestick", label: "K线" },
  { key: "heikin_ashi", label: "Heikin Ashi" },
  { key: "line", label: "折线" },
];

const PERIODS: Array<{ key: Period; label: string }> = [
  { key: "1min", label: "1分" },
  { key: "5min", label: "5分" },
  { key: "15min", label: "15分" },
  { key: "30min", label: "30分" },
  { key: "60min", label: "60分" },
  { key: "D", label: "日" },
  { key: "W", label: "周" },
  { key: "M", label: "月" },
];

const DRAWING_TOOLS: Array<{ key: DrawingTool; label: string; icon: string }> = [
  { key: "trend_line", label: "趋势线", icon: "╱" },
  { key: "horiz_line", label: "水平线", icon: "—" },
  { key: "vert_line", label: "垂直线", icon: "│" },
  { key: "rect", label: "矩形", icon: "▭" },
  { key: "fib_retrace", label: "斐波回调", icon: "F↩" },
  { key: "fib_ext", label: "斐波扩展", icon: "F↗" },
];

export function ChartToolbar({
  chartType, onChartTypeChange,
  activeTool, onToolChange,
  onClearDrawings, drawingCount,
  gridMode, onToggleGridMode,
  period, onPeriodChange,
}: ChartToolbarProps) {
  return (
    <div style={{
      display: "flex", alignItems: "center", gap: 8,
      padding: "4px 12px", background: "#161616",
      borderBottom: "1px solid #2A2A2A",
      fontFamily: "monospace", fontSize: 12,
      flexShrink: 0, flexWrap: "wrap",
    }}>
      {/* Chart type */}
      <span style={{ color: "#858585", marginRight: 4 }}>图表:</span>
      {CHART_TYPES.map(ct => (
        <button key={ct.key} onClick={() => onChartTypeChange(ct.key)}
          style={{
            ...btnBase,
            background: chartType === ct.key ? "#CCAA00" : "#2A2A2A",
            color: chartType === ct.key ? "#000" : "#D4D4D4",
            fontWeight: chartType === ct.key ? 600 : 400,
          }}>
          {ct.label}
        </button>
      ))}

      {/* Period selector */}
      {period && onPeriodChange && (
        <>
          <span style={{ color: "#666666", margin: "0 4px" }}>|</span>
          <span style={{ color: "#858585", marginRight: 4 }}>周期:</span>
          {PERIODS.map(p => (
            <button key={p.key} onClick={() => onPeriodChange(p.key)}
              style={{
                ...btnBase,
                background: period === p.key ? "#CCAA00" : "#2A2A2A",
                color: period === p.key ? "#000" : "#D4D4D4",
                fontWeight: period === p.key ? 600 : 400,
                minWidth: 32, textAlign: "center", padding: "4px 6px",
              }}>
              {p.label}
            </button>
          ))}
        </>
      )}

      {onToggleGridMode && (
        <>
          <span style={{ color: "#666666", margin: "0 4px" }}>|</span>
          <button onClick={onToggleGridMode} style={{
            ...btnBase,
            background: gridMode ? "#7E57C2" : "#2A2A2A",
            color: gridMode ? "#fff" : "#858585",
          }} title="多图表 2×2 布局">
            {gridMode ? "⊞ 2×2 ✓" : "⊞ 2×2"}
          </button>
        </>
      )}

      <span style={{ color: "#666666", margin: "0 4px" }}>|</span>

      {/* Drawing tools */}
      <span style={{ color: "#858585", marginRight: 4 }}>绘图:</span>
      {DRAWING_TOOLS.map(dt => (
        <button key={dt.key} onClick={() => onToolChange(activeTool === dt.key ? null : dt.key)}
          title={dt.label}
          style={{
            ...btnBase,
            background: activeTool === dt.key ? "#CCAA00" : "#2A2A2A",
            color: activeTool === dt.key ? "#000" : "#858585",
            minWidth: 28, textAlign: "center",
          }}>
          {dt.icon}
        </button>
      ))}

      {drawingCount > 0 && (
        <>
          <span style={{ color: "#666666", margin: "0 4px" }}>|</span>
          <button onClick={onClearDrawings} style={{
            ...btnBase, background: "#3a1a1a", color: "#EF5350",
          }}>
            清除全部 ({drawingCount})
          </button>
        </>
      )}
    </div>
  );
}

const btnBase: React.CSSProperties = {
  border: "none", borderRadius: 3, cursor: "pointer",
  padding: "4px 10px", fontSize: 12, fontFamily: "monospace",
  lineHeight: "16px",
};

import type { ChartType } from "@me/chart-engine";
import type { DrawingTool } from "./KLineChart";
import { drawingToolLabel } from "./KLineChart";

interface ChartToolbarProps {
  chartType: ChartType;
  onChartTypeChange: (t: ChartType) => void;
  activeTool: DrawingTool | null;
  onToolChange: (t: DrawingTool | null) => void;
  onClearDrawings: () => void;
  drawingCount: number;
}

const CHART_TYPES: Array<{ key: ChartType; label: string }> = [
  { key: "candlestick", label: "K线" },
  { key: "heikin_ashi", label: "Heikin Ashi" },
  { key: "line", label: "折线" },
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

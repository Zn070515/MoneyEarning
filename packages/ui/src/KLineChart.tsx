import { useEffect, useRef, useState, useMemo } from "react";
import { ChartEngine, OHLCV, ChartType, IndicatorData, DrawingObject, formatVolumeCN, formatAmountCN } from "@me/chart-engine";

export type DrawingTool = "trend_line" | "horiz_line" | "vert_line" | "rect" | "fib_retrace" | "fib_ext";

interface PendingDraw {
  tool: DrawingTool;
  points: Array<{ x: number; y: number; index: number; price: number }>;
}

interface KLineChartProps {
  data: OHLCV[];
  indicators?: IndicatorData[];
  chartType?: ChartType;
  activeTool?: DrawingTool | null;
  drawings?: DrawingObject[];
  onDrawingAdd?: (obj: DrawingObject) => void;
  onDrawingDelete?: (id: string) => void;
  onDrawingSelect?: (id: string | null) => void;
  onToolCancel?: () => void;
  className?: string;
}

export function KLineChart({
  data, indicators = [], chartType = "candlestick",
  activeTool = null, drawings = [], onDrawingAdd, onDrawingDelete, onDrawingSelect, onToolCancel, className,
}: KLineChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engineRef = useRef<ChartEngine | null>(null);
  const [pending, setPending] = useState<PendingDraw | null>(null);
  const [selectedDrawingId, setSelectedDrawingId] = useState<string | null>(null);
  const [engineError, setEngineError] = useState(false);
  const pendingRef = useRef<PendingDraw | null>(null);
  const prevDrawingsRef = useRef<DrawingObject[]>([]);

  // Init engine
  useEffect(() => {
    if (!canvasRef.current) return;
    try {
      const engine = new ChartEngine(canvasRef.current);
      engineRef.current = engine;
      const ro = new ResizeObserver(() => engine.resize());
      if (containerRef.current) ro.observe(containerRef.current);
      return () => { ro.disconnect(); engine.destroy(); };
    } catch (_) {
      setEngineError(true);
    }
  }, []);

  // Data updates
  useEffect(() => { engineRef.current?.setData(data); }, [data]);
  useEffect(() => { engineRef.current?.setIndicators(indicators); }, [indicators]);
  useEffect(() => { engineRef.current?.setChartType(chartType); }, [chartType]);

  // Sync drawings to engine
  useEffect(() => {
    const engine = engineRef.current;
    if (!engine) return;
    if (prevDrawingsRef.current === drawings) return;
    prevDrawingsRef.current = drawings;
    engine.clearDrawings();
    for (const d of drawings) engine.addDrawing(d);
  }, [drawings]);

  // Mouse event handling for interactive drawing
  useEffect(() => {
    const canvas = canvasRef.current;
    const engine = engineRef.current;
    if (!canvas || !engine) return;

    const getPixelPos = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      return { px: e.clientX - rect.left, py: e.clientY - rect.top };
    };

    const handleClick = (e: MouseEvent) => {
      if (!activeTool) {
        const { px, py } = getPixelPos(e);
        const hit = engine.hitTestDrawing(px, py);
        if (hit) {
          engine.selectDrawing(hit.id);
          setSelectedDrawingId(hit.id);
          onDrawingSelect?.(hit.id);
        } else {
          engine.selectDrawing(null);
          setSelectedDrawingId(null);
          onDrawingSelect?.(null);
        }
        return;
      }

      const { px, py } = getPixelPos(e);
      const index = engine.pixelToIndex(px);
      const [price] = engine.pixelToPrice(py);

      const point = { x: px, y: py, index, price };

      const cur = pendingRef.current;
      if (!cur) {
        const p: PendingDraw = { tool: activeTool, points: [point] };
        pendingRef.current = p;
        setPending(p);
        if (activeTool === "horiz_line" || activeTool === "vert_line") {
          commitDrawing(p);
        }
      } else {
        cur.points.push(point);
        commitDrawing(cur);
      }
    };

    const commitDrawing = (p: PendingDraw) => {
      const id = `draw_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
      const obj: DrawingObject = {
        id,
        type: p.tool,
        points: p.points.map(pt => ({ x: pt.x, y: pt.y, index: pt.index, price: pt.price })),
        color: "#ffd700",
        lineWidth: 1.5,
      };
      onDrawingAdd?.(obj);
      pendingRef.current = null;
      setPending(null);
    };

    const handleMouseMove = (e: MouseEvent) => {
      const p = pendingRef.current;
      if (!p || p.points.length === 0) return;
      // Redraw happens on next frame
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        pendingRef.current = null;
        setPending(null);
        onToolCancel?.();
      }
      if ((e.key === "Delete" || e.key === "Backspace") && selectedDrawingId) {
        engine.removeDrawing(selectedDrawingId);
        onDrawingDelete?.(selectedDrawingId);
        setSelectedDrawingId(null);
        onDrawingSelect?.(null);
      }
    };

    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      pendingRef.current = null;
      setPending(null);
      onToolCancel?.();
    };

    canvas.addEventListener("click", handleClick);
    canvas.addEventListener("mousemove", handleMouseMove);
    canvas.addEventListener("contextmenu", handleContextMenu);
    window.addEventListener("keydown", handleKeyDown);

    return () => {
      canvas.removeEventListener("click", handleClick);
      canvas.removeEventListener("mousemove", handleMouseMove);
      canvas.removeEventListener("contextmenu", handleContextMenu);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [activeTool, selectedDrawingId, onDrawingAdd, onDrawingDelete, onDrawingSelect, onToolCancel]);

  // Update cursor based on active tool
  useEffect(() => {
    if (!canvasRef.current) return;
    canvasRef.current.style.cursor = activeTool ? "crosshair" : "default";
  }, [activeTool]);

  const lastBar = useMemo(() => {
    if (data.length === 0) return null;
    const last = data[data.length - 1];
    const prevClose = data.length > 1 ? data[data.length - 2].close : last.open;
    return { last, prevClose };
  }, [data]);

  return (
    <div ref={containerRef} className={className} style={{
      width: "100%", height: "100%", position: "relative",
      display: "flex", flexDirection: "column",
    }}>
      {lastBar && (
        <div style={{
          display: "flex", alignItems: "center", gap: 16,
          padding: "4px 8px", background: "#141414",
          borderBottom: "1px solid #2A2A2A", flexShrink: 0,
          fontFamily: "monospace", fontSize: 11, color: "#aaa",
          overflow: "hidden", flexWrap: "wrap",
        }}>
          <SnapShot label="开" value={lastBar.last.open} />
          <SnapShot label="高" value={lastBar.last.high} />
          <SnapShot label="低" value={lastBar.last.low} />
          <SnapShot label="收" value={lastBar.last.close}
            isUp={lastBar.last.close >= lastBar.prevClose} />
          <span style={{ color: "#555" }}>|</span>
          <span style={{ color: "#999" }}>
            量 {formatVolumeCN(lastBar.last.volume)}
          </span>
          <span style={{ color: "#999" }}>
            额 {formatAmountCN(lastBar.last.amount ?? 0)}
          </span>
          <span style={{
            color: lastBar.last.close >= lastBar.prevClose ? "#ef4444" : "#22c55e",
            fontWeight: 600,
          }}>
            {lastBar.last.close >= lastBar.prevClose ? "+" : ""}
            {lastBar.prevClose > 0
              ? ((lastBar.last.close - lastBar.prevClose) / lastBar.prevClose * 100).toFixed(2)
              : "0.00"}%
          </span>
        </div>
      )}
      {engineError ? (
        <div style={{
          flex: 1, display: "flex", alignItems: "center", justifyContent: "center",
          color: "#666666", fontFamily: "monospace", fontSize: 14,
        }}>
          图表引擎初始化失败，请刷新页面重试
        </div>
      ) : (
        <canvas ref={canvasRef} style={{ width: "100%", height: "100%" }} />
      )}
      {activeTool && (
        <div style={{
          position: "absolute", top: 8, left: 8,
          background: "rgba(204,170,0,0.12)", color: "#CCAA00",
          padding: "4px 10px", borderRadius: 4, fontSize: 12,
          fontFamily: "monospace", pointerEvents: "none",
          border: "1px solid rgba(204,170,0,0.3)",
          zIndex: 10,
        }}>
          {drawingToolLabel(activeTool)} — 点击图表放置 {pending ? "第二个点" : "第一个点"} · 右键/Esc 取消
        </div>
      )}
      {!activeTool && selectedDrawingId && (
        <div style={{
          position: "absolute", top: 8, right: 12,
          background: "rgba(255,215,0,0.12)", color: "#ffd700",
          padding: "4px 10px", borderRadius: 4, fontSize: 12,
          fontFamily: "monospace", pointerEvents: "none",
          border: "1px solid rgba(255,215,0,0.25)",
          zIndex: 10,
        }}>
          已选图形 · Delete 删除
        </div>
      )}
    </div>
  );
}

export function drawingToolLabel(t: DrawingTool): string {
  const map: Record<DrawingTool, string> = {
    trend_line: "趋势线", horiz_line: "水平线", vert_line: "垂直线",
    rect: "矩形", fib_retrace: "斐波那契回调", fib_ext: "斐波那契扩展",
  };
  return map[t];
}

function SnapShot({ label, value, isUp }: { label: string; value: number; isUp?: boolean }) {
  const color = isUp == null ? "#ccc" : isUp ? "#ef4444" : "#22c55e";
  return (
    <span>
      <span style={{ color: "#666" }}>{label}</span>{" "}
      <span style={{ color }}>{value.toFixed(2)}</span>
    </span>
  );
}

export type { DrawingObject, ChartType, IndicatorData, OHLCV };

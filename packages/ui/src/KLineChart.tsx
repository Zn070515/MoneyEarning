import { useEffect, useRef, useState } from "react";
import { ChartEngine, OHLCV, ChartType, IndicatorData, DrawingObject } from "@me/chart-engine";

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
  onToolCancel?: () => void;
  className?: string;
}

export function KLineChart({
  data, indicators = [], chartType = "candlestick",
  activeTool = null, drawings = [], onDrawingAdd, onDrawingDelete, onToolCancel, className,
}: KLineChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engineRef = useRef<ChartEngine | null>(null);
  const [pending, setPending] = useState<PendingDraw | null>(null);
  const pendingRef = useRef<PendingDraw | null>(null);

  // Init engine
  useEffect(() => {
    if (!canvasRef.current) return;
    const engine = new ChartEngine(canvasRef.current);
    engineRef.current = engine;
    const ro = new ResizeObserver(() => engine.resize());
    if (containerRef.current) ro.observe(containerRef.current);
    return () => { ro.disconnect(); };
  }, []);

  // Data updates
  useEffect(() => { engineRef.current?.setData(data); }, [data]);
  useEffect(() => { engineRef.current?.setIndicators(indicators); }, [indicators]);
  useEffect(() => { engineRef.current?.setChartType(chartType); }, [chartType]);

  // Sync drawings to engine
  useEffect(() => {
    const engine = engineRef.current;
    if (!engine) return;
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
        // Check if clicking on existing drawing
        const { px, py } = getPixelPos(e);
        const hit = engine.hitTestDrawing(px, py);
        if (hit) {
          engine.selectDrawing(hit.id);
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
  }, [activeTool, onDrawingAdd, onToolCancel]);

  // Update cursor based on active tool
  useEffect(() => {
    if (!canvasRef.current) return;
    canvasRef.current.style.cursor = activeTool ? "crosshair" : "default";
  }, [activeTool]);

  return (
    <div ref={containerRef} className={className} style={{
      width: "100%", height: "100%", position: "relative",
    }}>
      <canvas ref={canvasRef} style={{ width: "100%", height: "100%" }} />
      {activeTool && (
        <div style={{
          position: "absolute", top: 8, left: 8,
          background: "rgba(251,191,36,0.15)", color: "#fbbf24",
          padding: "4px 10px", borderRadius: 4, fontSize: 12,
          fontFamily: "monospace", pointerEvents: "none",
          border: "1px solid rgba(251,191,36,0.3)",
          zIndex: 10,
        }}>
          {drawingToolLabel(activeTool)} — 点击图表放置 {pending ? "第二个点" : "第一个点"} · 右键/Esc 取消
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

export type { DrawingObject, ChartType, IndicatorData, OHLCV };

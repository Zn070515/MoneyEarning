import { useEffect, useRef, useCallback } from "react";
import { ChartEngine, OHLCV, ChartType, IndicatorData, DrawingObject } from "@me/chart-engine";

interface KLineChartProps {
  data: OHLCV[];
  indicators?: IndicatorData[];
  chartType?: ChartType;
  drawings?: DrawingObject[];
  onDrawingAdd?: (obj: DrawingObject) => void;
  onDrawingSelect?: (id: string | null) => void;
  className?: string;
}

export function KLineChart({
  data, indicators = [], chartType = "candlestick",
  drawings = [], onDrawingAdd, onDrawingSelect, className,
}: KLineChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engineRef = useRef<ChartEngine | null>(null);

  useEffect(() => {
    if (!canvasRef.current) return;
    const engine = new ChartEngine(canvasRef.current);
    engineRef.current = engine;

    const ro = new ResizeObserver(() => engine.resize());
    if (containerRef.current) ro.observe(containerRef.current);

    return () => { ro.disconnect(); };
  }, []);

  useEffect(() => {
    engineRef.current?.setData(data);
  }, [data]);

  useEffect(() => {
    engineRef.current?.setIndicators(indicators);
  }, [indicators]);

  useEffect(() => {
    engineRef.current?.setChartType(chartType);
  }, [chartType]);

  // Expose engine ref (for imperative access by parent)
  const getEngine = useCallback(() => engineRef.current, []);

  return (
    <div ref={containerRef} className={className} style={{ width: "100%", height: "100%" }}>
      <canvas ref={canvasRef} style={{ width: "100%", height: "100%" }} />
    </div>
  );
}

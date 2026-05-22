import { OHLCV, ViewRect, LayoutZones, CrosshairMode } from "./types";
import { ViewportManager } from "./layout/ViewportManager";
import { CandleRenderer } from "./renderer/CandleRenderer";
import { HeikinAshiRenderer } from "./renderer/HeikinAshiRenderer";
import { LineRenderer } from "./renderer/LineRenderer";
import { VolumeRenderer } from "./renderer/VolumeRenderer";
import { GridRenderer } from "./renderer/GridRenderer";
import { CrosshairRenderer } from "./renderer/CrosshairRenderer";
import { IndicatorRenderer, IndicatorData } from "./renderer/IndicatorRenderer";
import { DrawingRenderer, DrawingObject } from "./renderer/DrawingRenderer";

export type ChartType = "candlestick" | "heikin_ashi" | "line" | "hlc3" | "ohlc4";

const LAYOUT_RATIO = { main: 0.6, volume: 0.15, indicator: 0.08 };

export class ChartEngine {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private data: OHLCV[] = [];
  private indicators: IndicatorData[] = [];
  private viewport: ViewportManager;

  private candleRenderer: CandleRenderer;
  private haRenderer: HeikinAshiRenderer;
  private lineRenderer: LineRenderer;
  private volumeRenderer: VolumeRenderer;
  private gridRenderer: GridRenderer;
  private crosshairRenderer: CrosshairRenderer;
  private indicatorRenderer: IndicatorRenderer;
  private drawingRenderer: DrawingRenderer;

  private chartType: ChartType = "candlestick";
  private crosshair: CrosshairMode = "none";
  private mouseX = 0;
  private mouseY = 0;
  private dpr: number;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.dpr = window.devicePixelRatio || 1;
    this.ctx = canvas.getContext("2d")!;
    this.viewport = new ViewportManager(
      { x: 0, y: 0, width: 0, height: 0 }, 0,
    );

    this.candleRenderer = new CandleRenderer(this.ctx);
    this.haRenderer = new HeikinAshiRenderer(this.ctx);
    this.lineRenderer = new LineRenderer(this.ctx);
    this.volumeRenderer = new VolumeRenderer(this.ctx);
    this.gridRenderer = new GridRenderer(this.ctx);
    this.crosshairRenderer = new CrosshairRenderer(this.ctx);
    this.indicatorRenderer = new IndicatorRenderer(this.ctx);
    this.drawingRenderer = new DrawingRenderer(this.ctx);

    this.bindEvents();
    this.resize();
  }

  // --- Data ---

  setData(data: OHLCV[]): void {
    this.data = data;
    this.viewport.setTotalCount(data.length);
    if (this.chartType === "heikin_ashi") this.haRenderer.compute(data);
    this.draw();
  }

  setIndicators(ind: IndicatorData[]): void {
    this.indicators = ind;
    this.draw();
  }

  setChartType(type: ChartType): void {
    this.chartType = type;
    if (type === "heikin_ashi") this.haRenderer.compute(this.data);
    this.draw();
  }

  // --- Drawings ---

  addDrawing(obj: DrawingObject): void { this.drawingRenderer.add(obj); this.draw(); }
  removeDrawing(id: string): void { this.drawingRenderer.remove(id); this.draw(); }
  clearDrawings(): void { this.drawingRenderer.clear(); this.draw(); }
  selectDrawing(id: string | null): void { this.drawingRenderer.select(id); this.draw(); }
  getSelectedDrawingId(): string | null { return this.drawingRenderer.getSelectedId(); }
  hitTestDrawing(x: number, y: number): DrawingObject | null { return this.drawingRenderer.hitTest(x, y); }

  // --- Coordinate conversion ---

  /// Convert canvas pixel X to data index (for placing vertical tools)
  pixelToIndex(px: number): number {
    return this.viewport.xToIndex(px);
  }

  /// Convert canvas pixel Y to price (for placing horizontal tools)
  /// Returns [price, minPrice, maxPrice]
  pixelToPrice(py: number): [number, number, number] {
    let minP = Infinity, maxP = -Infinity;
    const { from, to } = this.viewport.visibleRange;
    const src = this.chartType === "heikin_ashi"
      ? this.haRenderer.getBars()
      : this.data;
    for (let i = from; i <= to && i < src.length; i++) {
      const d = src[i];
      const low = this.chartType === "heikin_ashi" ? (d as any).haLow : d.low;
      const high = this.chartType === "heikin_ashi" ? (d as any).haHigh : d.high;
      if (isFinite(low) && low < minP) minP = low;
      if (isFinite(high) && high > maxP) maxP = high;
    }
    if (!isFinite(minP)) { minP = 0; maxP = 100; }
    const price = this.viewport.yToPrice(py, minP, maxP);
    return [price, minP, maxP];
  }

  /// Get main chart rect
  getMainRect(): ViewRect {
    return this.getLayout().main;
  }

  // --- Layout ---

  private getLayout(): LayoutZones {
    const w = this.canvas.width / this.dpr;
    const h = this.canvas.height / this.dpr;
    const pad = { top: 10, bottom: 10, left: 60, right: 20 };

    const usableH = h - pad.top - pad.bottom;
    const mainH = usableH * LAYOUT_RATIO.main;
    const volH = usableH * LAYOUT_RATIO.volume;

    const main: ViewRect = {
      x: pad.left, y: pad.top,
      width: w - pad.left - pad.right, height: mainH,
    };
    const volume: ViewRect = {
      x: pad.left, y: pad.top + mainH + 4,
      width: w - pad.left - pad.right, height: volH,
    };
    const indicatorRects: ViewRect[] = [];
    const indStart = volume.y + volH + 8;
    const remainingH = h - indStart - pad.bottom;
    const indCount = this.indicators.length;

    for (let i = 0; i < indCount; i++) {
      indicatorRects.push({
        x: pad.left, y: indStart + i * (remainingH / Math.max(1, indCount)),
        width: w - pad.left - pad.right,
        height: remainingH / Math.max(1, indCount),
      });
    }

    return { main, volume, indicator: indicatorRects };
  }

  resize(): void {
    const rect = this.canvas.getBoundingClientRect();
    this.canvas.width = rect.width * this.dpr;
    this.canvas.height = rect.height * this.dpr;
    this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    this.draw();
  }

  // --- Render ---

  draw(): void {
    const ctx = this.ctx;
    const w = this.canvas.width / this.dpr;
    const h = this.canvas.height / this.dpr;

    ctx.fillStyle = "#1a1a2e";
    ctx.fillRect(0, 0, w, h);

    if (this.data.length === 0) {
      ctx.fillStyle = "#666";
      ctx.font = "16px monospace";
      ctx.textAlign = "center";
      ctx.fillText("无数据 / No Data", w / 2, h / 2);
      return;
    }

    const layout = this.getLayout();
    this.viewport.updateMainRect(layout.main);

    const { from, to } = this.viewport.visibleRange;
    let minP = Infinity, maxP = -Infinity;

    // Use appropriate data source
    const src = this.chartType === "heikin_ashi"
      ? this.haRenderer.getBars()
      : this.data;

    for (let i = from; i <= to && i < src.length; i++) {
      const d = src[i];
      const low = this.chartType === "heikin_ashi" ? (d as any).haLow : d.low;
      const high = this.chartType === "heikin_ashi" ? (d as any).haHigh : d.high;
      if (isFinite(low) && low < minP) minP = low;
      if (isFinite(high) && high > maxP) maxP = high;
    }
    if (!isFinite(minP)) { minP = 0; maxP = 100; }

    const transform = {
      indexToX: (i: number) => this.viewport.indexToX(i),
      priceToY: (r: ViewRect, min: number, rng: number, price: number) =>
        r.y + r.height * (1 - (price - min) / rng),
    };

    // Grid
    this.gridRenderer.draw(layout.main, minP, maxP);

    // Main chart
    switch (this.chartType) {
      case "heikin_ashi":
        this.haRenderer.draw(layout.main, from, to, minP, maxP, transform);
        break;
      case "line":
      case "hlc3":
      case "ohlc4":
        this.lineRenderer.draw(
          this.data, layout.main, from, to, minP, maxP, transform,
          this.chartType as "close" | "hlc3" | "ohlc4",
        );
        break;
      default:
        this.candleRenderer.draw(this.data, this.viewport, layout.main);
    }

    // Drawings
    this.drawingRenderer.draw(layout.main, minP, maxP, transform);

    // Volume
    this.volumeRenderer.draw(this.data, this.viewport, layout.volume);

    // Indicators
    for (let i = 0; i < this.indicators.length && i < layout.indicator.length; i++) {
      this.indicatorRenderer.drawIndicator(
        layout.indicator[i], this.indicators[i],
        { indexToX: (idx: number) => this.viewport.indexToX(idx) },
        from, to,
      );
    }

    // Crosshair
    if (this.crosshair !== "none") {
      this.crosshairRenderer.draw(
        this.data, this.viewport, layout.main, this.mouseX, this.mouseY,
      );
    }
  }

  // --- Events ---

  private bindEvents(): void {
    this.canvas.addEventListener("mousemove", (e) => {
      const rect = this.canvas.getBoundingClientRect();
      this.mouseX = e.clientX - rect.left;
      this.mouseY = e.clientY - rect.top;
      if (this.crosshair !== "none") this.draw();
    });
    this.canvas.addEventListener("mouseleave", () => {
      this.crosshair = "none";
      this.draw();
    });
    this.canvas.addEventListener("mouseenter", () => {
      this.crosshair = "cross";
      this.draw();
    });
    this.canvas.addEventListener("wheel", (e) => {
      e.preventDefault();
      if (e.ctrlKey) {
        this.viewport.zoom(e.deltaY > 0 ? -1 : 1);
      } else {
        this.viewport.panH(e.deltaY > 0 ? 3 : -3);
      }
      this.draw();
    });
    window.addEventListener("resize", () => this.resize());
  }
}

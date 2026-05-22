import { OHLCV, ViewRect } from "../types";
import { ViewportManager } from "../layout/ViewportManager";

const BULL_COLOR = "#ef4444"; // red (Chinese convention: red = up)
const BEAR_COLOR = "#22c55e"; // green (Chinese convention: green = down)

export class CandleRenderer {
  private ctx: CanvasRenderingContext2D;

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  draw(
    data: OHLCV[],
    viewport: ViewportManager,
    rect: ViewRect,
  ): void {
    const ctx = this.ctx;
    const { from, to } = viewport.visibleRange;
    const bw = viewport.barWidth - 2; // candle body width minus gap

    // Calculate visible price range
    let minP = Infinity, maxP = -Infinity;
    for (let i = from; i <= to && i < data.length; i++) {
      const d = data[i];
      if (d.low < minP) minP = d.low;
      if (d.high > maxP) maxP = d.high;
    }
    if (!isFinite(minP)) { minP = 0; maxP = 100; }

    ctx.save();
    ctx.beginPath();
    ctx.rect(rect.x, rect.y, rect.width, rect.height);
    ctx.clip();

    for (let i = from; i <= to && i < data.length; i++) {
      const d = data[i];
      const x = viewport.indexToX(i);
      const openY = viewport.priceToY(d.open, minP, maxP);
      const closeY = viewport.priceToY(d.close, minP, maxP);
      const highY = viewport.priceToY(d.high, minP, maxP);
      const lowY = viewport.priceToY(d.low, minP, maxP);
      const isBull = d.close >= d.open;
      const color = isBull ? BULL_COLOR : BEAR_COLOR;

      // Wick
      ctx.strokeStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x, highY);
      ctx.lineTo(x, lowY);
      ctx.stroke();

      // Body
      const bodyH = Math.max(1, Math.abs(closeY - openY));
      const bodyY = isBull ? closeY : openY;
      ctx.fillStyle = isBull ? BULL_COLOR : BEAR_COLOR;
      ctx.fillRect(x - bw / 2, bodyY, bw, bodyH);

      // Hollow bear candle
      if (!isBull) {
        ctx.strokeStyle = BEAR_COLOR;
        ctx.strokeRect(x - bw / 2, bodyY, bw, bodyH);
      }
    }
    ctx.restore();
  }
}

import { OHLCV, ViewRect } from "../types";

interface XYPixel {
  indexToX(idx: number): number;
  priceToY(rect: ViewRect, minP: number, range: number, price: number): number;
}

export class LineRenderer {
  private ctx: CanvasRenderingContext2D;

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  draw(
    data: OHLCV[], rect: ViewRect,
    visibleFrom: number, visibleTo: number,
    minP: number, maxP: number, transform: XYPixel,
    type: "close" | "hlc3" | "ohlc4" = "close",
  ): void {
    const ctx = this.ctx;
    const range = maxP - minP || 1;

    ctx.save();
    ctx.beginPath();
    ctx.rect(rect.x, rect.y, rect.width, rect.height);
    ctx.clip();

    const color = type === "close" ? "#fbbf24" : "#60a5fa";
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    ctx.lineJoin = "round";
    ctx.beginPath();
    let started = false;

    for (let i = visibleFrom; i <= visibleTo && i < data.length; i++) {
      const d = data[i];
      let price: number;
      switch (type) {
        case "hlc3": price = (d.high + d.low + d.close) / 3; break;
        case "ohlc4": price = (d.open + d.high + d.low + d.close) / 4; break;
        default: price = d.close;
      }
      if (!isFinite(price)) { started = false; continue; }
      const x = transform.indexToX(i);
      const y = transform.priceToY(rect, minP, range, price);
      if (!started) { ctx.moveTo(x, y); started = true; }
      else { ctx.lineTo(x, y); }
    }
    ctx.stroke();

    // Label
    ctx.fillStyle = color + "88";
    ctx.font = "10px monospace";
    ctx.textAlign = "left";
    const label = type === "close" ? "CLOSE" : type.toUpperCase();
    ctx.fillText(label, rect.x + 4, rect.y + 12);

    ctx.restore();
  }
}

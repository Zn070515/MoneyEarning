import { ViewRect } from "../types";

export class GridRenderer {
  private ctx: CanvasRenderingContext2D;

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  draw(
    rect: ViewRect,
    minPrice: number,
    maxPrice: number,
    rows: number = 5,
  ): void {
    const ctx = this.ctx;
    const range = maxPrice - minPrice || 1;

    ctx.save();
    ctx.strokeStyle = "rgba(255,255,255,0.08)";
    ctx.lineWidth = 0.5;

    // Horizontal grid lines
    for (let i = 0; i <= rows; i++) {
      const y = rect.y + (rect.height / rows) * i;
      ctx.beginPath();
      ctx.moveTo(rect.x, y);
      ctx.lineTo(rect.x + rect.width, y);
      ctx.stroke();

      // Price label
      const price = maxPrice - (range / rows) * i;
      ctx.fillStyle = "rgba(255,255,255,0.5)";
      ctx.font = "11px monospace";
      ctx.textAlign = "left";
      ctx.fillText(price.toFixed(2), rect.x + 4, y - 4);
    }

    ctx.restore();
  }
}

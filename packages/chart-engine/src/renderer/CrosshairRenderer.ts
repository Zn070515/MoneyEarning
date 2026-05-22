import { OHLCV, ViewRect } from "../types";
import { ViewportManager } from "../layout/ViewportManager";

export class CrosshairRenderer {
  private ctx: CanvasRenderingContext2D;

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  draw(
    data: OHLCV[],
    viewport: ViewportManager,
    mainRect: ViewRect,
    mouseX: number,
    mouseY: number,
  ): void {
    const ctx = this.ctx;
    const idx = viewport.xToIndex(mouseX);
    if (idx < 0 || idx >= data.length) return;

    const px = viewport.indexToX(idx);

    ctx.save();
    ctx.strokeStyle = "rgba(255,255,255,0.2)";
    ctx.lineWidth = 0.5;
    ctx.setLineDash([4, 4]);

    // Vertical line
    ctx.beginPath();
    ctx.moveTo(px, mainRect.y);
    ctx.lineTo(px, mainRect.y + mainRect.height);
    ctx.stroke();

    // Horizontal line
    ctx.beginPath();
    ctx.moveTo(mainRect.x, mouseY);
    ctx.lineTo(mainRect.x + mainRect.width, mouseY);
    ctx.stroke();

    ctx.setLineDash([]);

    // OHLCV tooltip
    const d = data[idx];
    const text = [
      `O:${d.open.toFixed(2)}`,
      `H:${d.high.toFixed(2)}`,
      `L:${d.low.toFixed(2)}`,
      `C:${d.close.toFixed(2)}`,
      `V:${d.volume.toFixed(0)}`,
    ].join("  ");

    ctx.fillStyle = "rgba(0,0,0,0.8)";
    const tw = ctx.measureText(text).width + 8;
    const tx = px + 10;
    const ty = mainRect.y + 4;
    ctx.fillRect(tx, ty, tw, 18);

    ctx.fillStyle = "#fff";
    ctx.font = "11px monospace";
    ctx.textAlign = "left";
    ctx.fillText(text, tx + 4, ty + 13);

    ctx.restore();
  }
}

import { OHLCV, ViewRect } from "../types";
import { ViewportManager } from "../layout/ViewportManager";
import { formatVolumeCN } from "../utils/format";

export class VolumeRenderer {
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
    const bw = viewport.barWidth - 2;

    if (data.length === 0) return;

    let maxV = 0;
    for (let i = from; i <= to && i < data.length; i++) {
      if (data[i].volume > maxV) maxV = data[i].volume;
    }
    if (maxV === 0) return;

    ctx.save();
    ctx.beginPath();
    ctx.rect(rect.x, rect.y, rect.width, rect.height);
    ctx.clip();

    for (let i = from; i <= to && i < data.length; i++) {
      const d = data[i];
      const x = viewport.indexToX(i);
      const isBull = d.close >= d.open;
      const barH = (d.volume / maxV) * rect.height;
      const barY = rect.y + rect.height - barH;

      ctx.fillStyle = isBull
        ? "rgba(239,68,68,0.5)"
        : "rgba(34,197,94,0.5)";
      ctx.fillRect(x - bw / 2, barY, bw, barH);
    }

    // Max volume label at top-right of volume pane
    ctx.fillStyle = "rgba(255,255,255,0.4)";
    ctx.font = "9px monospace";
    ctx.textAlign = "right";
    ctx.fillText(formatVolumeCN(maxV), rect.x + rect.width - 4, rect.y + 10);

    ctx.restore();
  }
}

import { OHLCV, ViewRect } from "../types";
import { ViewportManager } from "../layout/ViewportManager";

export class DateAxisRenderer {
  private ctx: CanvasRenderingContext2D;

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  draw(
    data: OHLCV[],
    viewport: ViewportManager,
    volumeRect: ViewRect,
  ): void {
    const ctx = this.ctx;
    const { from, to } = viewport.visibleRange;
    if (data.length === 0) return;

    const visibleCount = to - from + 1;
    const maxLabels = Math.max(2, Math.floor(volumeRect.width / 90));
    const step = Math.max(1, Math.ceil(visibleCount / maxLabels));

    const y = volumeRect.y + volumeRect.height + 16;

    ctx.save();
    ctx.fillStyle = "rgba(255,255,255,0.4)";
    ctx.font = "10px monospace";
    ctx.textAlign = "center";

    for (let i = from; i <= to; i += step) {
      if (i >= data.length) break;
      const x = viewport.indexToX(i);
      const label = formatDateLabel(data[i].time);
      ctx.fillText(label, x, y);
    }

    ctx.restore();
  }
}

function formatDateLabel(unixSec: number): string {
  const d = new Date(unixSec * 1000);
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${m}-${day}`;
}

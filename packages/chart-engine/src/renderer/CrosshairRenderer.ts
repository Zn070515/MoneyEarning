import { OHLCV, ViewRect } from "../types";
import { ViewportManager } from "../layout/ViewportManager";
import { IndicatorData } from "./IndicatorRenderer";
import { formatVolumeCN, formatAmountCN, autoPrecision } from "../utils/format";

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
    indicators?: IndicatorData[],
    indicatorRects?: ViewRect[],
  ): void {
    const ctx = this.ctx;
    const idx = viewport.xToIndex(mouseX);
    if (idx < 0 || idx >= data.length) return;

    const px = viewport.indexToX(idx);
    const d = data[idx];
    const prevClose = idx > 0 ? data[idx - 1].close : d.open;
    const changePct = prevClose > 0 ? ((d.close - prevClose) / prevClose) * 100 : 0;
    const isUp = changePct >= 0;
    const precis = autoPrecision(Math.max(d.high, d.close));

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

    // Build multi-line tooltip
    const dateStr = formatDate(d.time);
    const line1 = dateStr;
    const line2 = [
      `O:${d.open.toFixed(precis)}`,
      `H:${d.high.toFixed(precis)}`,
      `L:${d.low.toFixed(precis)}`,
      `C:${d.close.toFixed(precis)}`,
      `${isUp ? "+" : ""}${changePct.toFixed(2)}%`,
    ].join("  ");
    const line3 = [
      `V:${formatVolumeCN(d.volume)}`,
      `额:${formatAmountCN(d.amount ?? 0)}`,
      d.turnover != null ? `换:${d.turnover.toFixed(2)}%` : "",
    ].filter(Boolean).join("  ");

    ctx.font = "11px monospace";
    const lineHeight = 15;
    const maxLineW = Math.max(
      ctx.measureText(line1).width,
      ctx.measureText(line2).width,
      ctx.measureText(line3).width,
    );
    const tw = maxLineW + 12;
    const boxH = lineHeight * 3 + 8;

    // Smart positioning: flip to left if near right edge
    let tx = px + 10;
    const rightEdge = mainRect.x + mainRect.width;
    if (tx + tw > rightEdge) {
      tx = px - tw - 10;
    }
    if (tx < mainRect.x) tx = mainRect.x + 4;

    const ty = mainRect.y + 4;

    // Background
    ctx.fillStyle = "rgba(0,0,0,0.85)";
    ctx.fillRect(tx, ty, tw, boxH);

    // Date line (dim)
    ctx.fillStyle = "rgba(255,255,255,0.55)";
    ctx.textAlign = "left";
    ctx.fillText(line1, tx + 6, ty + 13);

    // OHLCV + change% line
    ctx.fillStyle = "#fff";
    ctx.fillText(line2, tx + 6, ty + 13 + lineHeight);

    // Color the change% portion
    const changeStr = `${isUp ? "+" : ""}${changePct.toFixed(2)}%`;
    const changeW = ctx.measureText(changeStr).width;
    const line2beforeChange = line2.slice(0, -changeStr.length);
    const beforeW = ctx.measureText(line2beforeChange).width;
    ctx.fillStyle = isUp ? "#ef4444" : "#22c55e";
    ctx.fillText(changeStr, tx + 6 + beforeW, ty + 13 + lineHeight);

    // Volume / amount / turnover line
    ctx.fillStyle = "rgba(255,255,255,0.7)";
    ctx.fillText(line3, tx + 6, ty + 13 + lineHeight * 2);

    // Indicator values at crosshair position
    if (indicators && indicatorRects) {
      for (let i = 0; i < indicators.length && i < indicatorRects.length; i++) {
        const ind = indicators[i];
        const rect = indicatorRects[i];
        if (idx < ind.values.length && isFinite(ind.values[idx])) {
          ctx.fillStyle = "rgba(255,255,255,0.6)";
          ctx.font = "10px monospace";
          ctx.textAlign = "right";
          const valStr = ind.values[idx].toFixed(4);
          ctx.fillText(
            `${ind.name}: ${valStr}`,
            rect.x + rect.width - 4,
            rect.y + 12,
          );
        }
      }
    }

    ctx.restore();
  }
}

function formatDate(unixSec: number): string {
  const d = new Date(unixSec * 1000);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

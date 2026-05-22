import { ViewRect } from "../types";

export interface IndicatorData {
  name: string;
  values: Float64Array;
  style: "line" | "histogram" | "dots" | "band";
  upper?: Float64Array;
  lower?: Float64Array;
  color?: string;
  opacity?: number;
}

interface XYFn {
  indexToX(idx: number): number;
}

const PALETTE = [
  "#fbbf24", "#34d399", "#60a5fa", "#f472b6",
  "#a78bfa", "#fb923c", "#22d3ee", "#f87171",
];

export class IndicatorRenderer {
  private ctx: CanvasRenderingContext2D;
  private colorMap = new Map<string, string>();
  private paletteIdx = 0;

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  private getColor(name: string, fallback?: string): string {
    if (fallback) return fallback;
    if (!this.colorMap.has(name)) {
      this.colorMap.set(name, PALETTE[this.paletteIdx % PALETTE.length]);
      this.paletteIdx++;
    }
    return this.colorMap.get(name)!;
  }

  drawLine(
    rect: ViewRect, visibleFrom: number, visibleTo: number,
    values: Float64Array, color: string, xy: XYFn,
  ): void {
    const ctx = this.ctx;
    let minV = Infinity, maxV = -Infinity;
    for (let i = visibleFrom; i <= visibleTo && i < values.length; i++) {
      const v = values[i];
      if (isFinite(v)) {
        if (v < minV) minV = v;
        if (v > maxV) maxV = v;
      }
    }
    if (!isFinite(minV)) return;
    const range = maxV - minV || 1;

    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    ctx.lineJoin = "round";
    ctx.beginPath();
    let started = false;

    for (let i = visibleFrom; i <= visibleTo && i < values.length; i++) {
      const v = values[i];
      if (!isFinite(v)) { started = false; continue; }
      const x = xy.indexToX(i);
      const y = rect.y + rect.height * (1 - (v - minV) / range);
      if (!started) { ctx.moveTo(x, y); started = true; }
      else { ctx.lineTo(x, y); }
    }
    ctx.stroke();
  }

  drawHistogram(
    rect: ViewRect, visibleFrom: number, visibleTo: number,
    values: Float64Array, color: string, xy: XYFn,
  ): void {
    const ctx = this.ctx;
    const baseline = 0;

    let minV = Infinity, maxV = -Infinity;
    for (let i = visibleFrom; i <= visibleTo && i < values.length; i++) {
      const v = values[i];
      if (isFinite(v)) { if (v < minV) minV = v; if (v > maxV) maxV = v; }
    }
    if (!isFinite(minV)) return;
    const range = (maxV - minV) || 1;
    const zeroY = rect.y + rect.height * (1 - (baseline - minV) / range);

    for (let i = visibleFrom; i <= visibleTo && i < values.length; i++) {
      const v = values[i];
      if (!isFinite(v)) continue;
      const x = xy.indexToX(i);
      const barY = rect.y + rect.height * (1 - (v - minV) / range);
      const barH = barY - zeroY;

      ctx.fillStyle = v >= baseline ? (color + "cc") : (color + "66");
      const barW = Math.max(1, rect.width / (visibleTo - visibleFrom + 1) * 0.7);
      ctx.fillRect(x - barW / 2, barH > 0 ? zeroY : barY, barW, Math.abs(barH) || 1);
    }
  }

  drawDots(
    rect: ViewRect, visibleFrom: number, visibleTo: number,
    values: Float64Array, color: string, xy: XYFn,
  ): void {
    const ctx = this.ctx;
    let minV = Infinity, maxV = -Infinity;
    for (let i = visibleFrom; i <= visibleTo && i < values.length; i++) {
      const v = values[i];
      if (isFinite(v)) { if (v < minV) minV = v; if (v > maxV) maxV = v; }
    }
    if (!isFinite(minV)) return;
    const range = maxV - minV || 1;

    ctx.fillStyle = color;
    for (let i = visibleFrom; i <= visibleTo && i < values.length; i++) {
      const v = values[i];
      if (!isFinite(v)) continue;
      const x = xy.indexToX(i);
      const y = rect.y + rect.height * (1 - (v - minV) / range);
      ctx.beginPath();
      ctx.arc(x, y, 3, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  drawBand(
    rect: ViewRect, visibleFrom: number, visibleTo: number,
    mid: Float64Array, upper: Float64Array, lower: Float64Array, color: string, xy: XYFn,
  ): void {
    const ctx = this.ctx;
    let minV = Infinity, maxV = -Infinity;
    for (let i = visibleFrom; i <= visibleTo && i < upper.length; i++) {
      const u = upper[i], l = lower[i];
      if (isFinite(u)) { if (u > maxV) maxV = u; if (u < minV) minV = u; }
      if (isFinite(l)) { if (l < minV) minV = l; if (l > maxV) maxV = l; }
    }
    if (!isFinite(minV)) return;
    const range = maxV - minV || 1;

    // Fill band
    ctx.fillStyle = color + "22";
    ctx.beginPath();
    for (let i = visibleFrom; i <= visibleTo && i < upper.length; i++) {
      const u = upper[i], l = lower[i];
      if (!isFinite(u) || !isFinite(l)) continue;
      const x = xy.indexToX(i);
      const yu = rect.y + rect.height * (1 - (u - minV) / range);
      const yl = rect.y + rect.height * (1 - (l - minV) / range);
      if (i === visibleFrom || !isFinite(upper[i - 1])) ctx.moveTo(x, yu);
      ctx.lineTo(x, yu);
    }
    for (let i = Math.min(visibleTo, upper.length - 1); i >= visibleFrom; i--) {
      const l = lower[i];
      if (!isFinite(l)) continue;
      const x = xy.indexToX(i);
      const yl = rect.y + rect.height * (1 - (l - minV) / range);
      ctx.lineTo(x, yl);
    }
    ctx.closePath();
    ctx.fill();

    // Upper / lower lines
    ctx.strokeStyle = color;
    ctx.lineWidth = 1;
    ctx.setLineDash([4, 4]);
    ctx.stroke();
    ctx.setLineDash([]);

    // Mid line (solid)
    if (mid.length > 0) {
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      let started = false;
      for (let i = visibleFrom; i <= visibleTo && i < mid.length; i++) {
        const v = mid[i];
        if (!isFinite(v)) { started = false; continue; }
        const x = xy.indexToX(i);
        const y = rect.y + rect.height * (1 - (v - minV) / range);
        if (!started) { ctx.moveTo(x, y); started = true; }
        else { ctx.lineTo(x, y); }
      }
      ctx.stroke();
    }
  }

  drawIndicator(
    rect: ViewRect, indicator: IndicatorData, xy: XYFn,
    visibleFrom: number, visibleTo: number,
  ): void {
    const ctx = this.ctx;
    ctx.save();
    ctx.beginPath();
    ctx.rect(rect.x, rect.y, rect.width, rect.height);
    ctx.clip();

    const color = this.getColor(indicator.name, indicator.color);

    switch (indicator.style) {
      case "histogram":
        this.drawHistogram(rect, visibleFrom, visibleTo, indicator.values, color, xy);
        break;
      case "dots":
        this.drawDots(rect, visibleFrom, visibleTo, indicator.values, color, xy);
        break;
      case "band":
        this.drawBand(
          rect, visibleFrom, visibleTo,
          indicator.values,
          indicator.upper || new Float64Array(),
          indicator.lower || new Float64Array(),
          color, xy,
        );
        break;
      default:
        this.drawLine(rect, visibleFrom, visibleTo, indicator.values, color, xy);
    }

    // Label
    ctx.fillStyle = "rgba(255,255,255,0.55)";
    ctx.font = "10px monospace";
    ctx.textAlign = "left";
    ctx.fillText(indicator.name, rect.x + 4, rect.y + 12);

    ctx.restore();
  }
}

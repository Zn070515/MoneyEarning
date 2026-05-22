import { OHLCV, ViewRect } from "../types";

export interface HeikinAshiBar extends OHLCV {
  haOpen: number;
  haClose: number;
  haHigh: number;
  haLow: number;
}

interface XYPixel {
  indexToX(idx: number): number;
  priceToY(rect: ViewRect, minP: number, range: number, price: number): number;
}

const BULL_COLOR = "#ef4444";
const BEAR_COLOR = "#22c55e";
const WICK_COLOR = "#ffffff66";

export class HeikinAshiRenderer {
  private ctx: CanvasRenderingContext2D;
  private bars: HeikinAshiBar[] = [];

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  compute(source: OHLCV[]): HeikinAshiBar[] {
    this.bars = [];
    for (let i = 0; i < source.length; i++) {
      const s = source[i];
      if (i === 0) {
        this.bars.push({
          ...s,
          haOpen: s.open,
          haClose: s.close,
          haHigh: s.high,
          haLow: s.low,
        });
      } else {
        const prev = this.bars[i - 1];
        const haClose = (s.open + s.high + s.low + s.close) / 4;
        const haOpen = (prev.haOpen + prev.haClose) / 2;
        const haHigh = Math.max(s.high, haOpen, haClose);
        const haLow = Math.min(s.low, haOpen, haClose);
        this.bars.push({ ...s, haOpen, haClose, haHigh, haLow });
      }
    }
    return this.bars;
  }

  draw(
    rect: ViewRect, visibleFrom: number, visibleTo: number,
    minP: number, maxP: number, transform: XYPixel,
  ): void {
    const ctx = this.ctx;
    const range = maxP - minP || 1;
    const totalVisible = visibleTo - visibleFrom + 1;
    const candleW = Math.max(1, rect.width / totalVisible * 0.7);
    const gap = candleW * 0.3;

    ctx.save();
    ctx.beginPath();
    ctx.rect(rect.x, rect.y, rect.width, rect.height);
    ctx.clip();

    for (let i = visibleFrom; i <= visibleTo && i < this.bars.length; i++) {
      const b = this.bars[i];
      const isBull = b.haClose >= b.haOpen;
      const bodyTop = isBull ? b.haClose : b.haOpen;
      const bodyBot = isBull ? b.haOpen : b.haClose;

      const x = transform.indexToX(i);
      const yTop = transform.priceToY(rect, minP, range, Math.max(bodyTop, bodyBot));
      const yBot = transform.priceToY(rect, minP, range, Math.min(bodyTop, bodyBot));
      const yHigh = transform.priceToY(rect, minP, range, b.haHigh);
      const yLow = transform.priceToY(rect, minP, range, b.haLow);

      // Wick
      ctx.strokeStyle = WICK_COLOR;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x, yHigh);
      ctx.lineTo(x, yLow);
      ctx.stroke();

      // Body
      ctx.fillStyle = isBull ? BULL_COLOR : BEAR_COLOR;
      const bodyH = Math.max(1, yBot - yTop);
      ctx.fillRect(x - candleW / 2, yTop, candleW, bodyH);

      // Border on bodies too thin
      if (bodyH < 2) {
        ctx.strokeStyle = isBull ? BULL_COLOR : BEAR_COLOR;
        ctx.lineWidth = 1;
        ctx.strokeRect(x - candleW / 2, yTop, candleW, Math.max(1, bodyH));
      }
    }

    ctx.restore();
  }

  getBars(): HeikinAshiBar[] { return this.bars; }
}

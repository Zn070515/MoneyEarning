import { DataRange, ViewRect } from "../types";

export class ViewportManager {
  private _candleWidth: number = 8;
  private _gapWidth: number = 2;
  private _offsetX: number = 0; // pixel offset from right edge
  private _scaleY: number = 1.0; // vertical scale factor
  private _translateY: number = 0;
  private _mainRect: ViewRect;
  private _candleCount: number;
  private _totalCount: number;

  constructor(mainRect: ViewRect, totalCount: number) {
    this._mainRect = mainRect;
    this._totalCount = totalCount;
    this._candleCount = totalCount;
  }

  get candleWidth(): number {
    return this._candleWidth;
  }
  get barWidth(): number {
    return this._candleWidth + this._gapWidth;
  }
  get visibleRange(): DataRange {
    const visible = Math.floor(this._mainRect.width / this.barWidth);
    const from = Math.max(0, this._totalCount - visible - Math.floor(this._offsetX / this.barWidth));
    const to = Math.min(this._totalCount - 1, from + visible);
    return { from, to };
  }

  updateMainRect(rect: ViewRect): void {
    this._mainRect = rect;
  }
  setTotalCount(n: number): void {
    this._totalCount = n;
  }

  // Zoom by delta (positive = zoom in)
  zoom(delta: number, _centerX?: number): void {
    const factor = delta > 0 ? 1.1 : 0.9;
    this._candleWidth = Math.max(2, Math.min(40, this._candleWidth * factor));
  }

  // Pan horizontal (positive = move right/forward in time)
  panH(delta: number): void {
    this._offsetX = Math.max(0, this._offsetX + delta);
  }

  // Pan vertical
  panV(delta: number): void {
    this._translateY += delta;
  }

  // Convert data index to pixel X
  indexToX(index: number): number {
    const visible = this.visibleRange;
    const relIdx = index - visible.from;
    return this._mainRect.x + relIdx * this.barWidth + this.barWidth / 2;
  }

  // Convert price to pixel Y (main chart)
  priceToY(price: number, minPrice: number, maxPrice: number): number {
    const range = maxPrice - minPrice || 1;
    const ratio = (price - minPrice) / range;
    return this._mainRect.y + this._mainRect.height * (1 - ratio) * this._scaleY + this._translateY;
  }

  // Pixel X to data index (approximate)
  xToIndex(px: number): number {
    const visible = this.visibleRange;
    return visible.from + Math.floor((px - this._mainRect.x) / this.barWidth);
  }

  // Pixel Y to price
  yToPrice(py: number, minPrice: number, maxPrice: number): number {
    const range = maxPrice - minPrice || 1;
    const ratio = 1 - (py - this._mainRect.y - this._translateY) / (this._mainRect.height * this._scaleY);
    return minPrice + ratio * range;
  }
}

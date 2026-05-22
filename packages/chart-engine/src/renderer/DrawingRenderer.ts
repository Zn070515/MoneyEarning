import { ViewRect } from "../types";

export interface DrawingObject {
  id: string;
  type: "trend_line" | "horiz_line" | "vert_line" | "rect" | "fib_retrace" | "fib_ext";
  points: Array<{ x: number; y: number; index?: number; price?: number }>;
  color?: string;
  lineWidth?: number;
  dashed?: boolean;
  label?: string;
}

interface XYPrice {
  indexToX(i: number): number;
  priceToY(rect: ViewRect, minP: number, range: number, price: number): number;
}

export class DrawingRenderer {
  private ctx: CanvasRenderingContext2D;
  private objects: DrawingObject[] = [];
  private selectedId: string | null = null;
  private hoveredId: string | null = null;

  constructor(ctx: CanvasRenderingContext2D) {
    this.ctx = ctx;
  }

  add(obj: DrawingObject): void { this.objects.push(obj); }
  remove(id: string): void { this.objects = this.objects.filter(o => o.id !== id); }
  clear(): void { this.objects = []; }
  getAll(): DrawingObject[] { return [...this.objects]; }
  select(id: string | null): void { this.selectedId = id; }

  draw(mainRect: ViewRect, minPrice: number, maxPrice: number, transform: XYPrice): void {
    const ctx = this.ctx;
    const range = (maxPrice - minPrice) || 1;

    ctx.save();
    ctx.beginPath();
    ctx.rect(mainRect.x, mainRect.y, mainRect.width, mainRect.height);
    ctx.clip();

    for (const obj of this.objects) {
      const isSel = obj.id === this.selectedId;
      const isHov = obj.id === this.hoveredId;
      const color = obj.color || "#ffd700";
      const width = (obj.lineWidth || 1.5) * (isSel || isHov ? 1.5 : 1);

      ctx.strokeStyle = isHov ? "#ffffff" : color;
      ctx.fillStyle = color;
      ctx.lineWidth = width;
      if (obj.dashed) ctx.setLineDash([6, 3]);
      else ctx.setLineDash([]);

      this.drawObject(obj, mainRect, minPrice, range, transform);
    }

    ctx.setLineDash([]);
    ctx.restore();
  }

  private drawObject(
    obj: DrawingObject, rect: ViewRect,
    minP: number, range: number, t: XYPrice,
  ): void {
    const ctx = this.ctx;
    const pts = obj.points;

    switch (obj.type) {
      case "trend_line": {
        if (pts.length < 2) return;
        const x1 = t.indexToX(pts[0].index ?? 0);
        const y1 = t.priceToY(rect, minP, range, pts[0].price ?? 0);
        const x2 = t.indexToX(pts[1].index ?? 0);
        const y2 = t.priceToY(rect, minP, range, pts[1].price ?? 0);
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
        // Handles
        this.drawHandle(x1, y1);
        this.drawHandle(x2, y2);
        break;
      }
      case "horiz_line": {
        for (const pt of pts) {
          const y = t.priceToY(rect, minP, range, pt.price ?? 0);
          ctx.beginPath();
          ctx.moveTo(rect.x, y);
          ctx.lineTo(rect.x + rect.width, y);
          ctx.stroke();
          // Price label
          ctx.fillStyle = ctx.strokeStyle as string;
          ctx.font = "9px monospace";
          ctx.textAlign = "right";
          ctx.fillText((pt.price ?? 0).toFixed(2), rect.x + rect.width - 4, y - 3);
        }
        break;
      }
      case "vert_line": {
        for (const pt of pts) {
          const x = t.indexToX(pt.index ?? 0);
          ctx.beginPath();
          ctx.moveTo(x, rect.y);
          ctx.lineTo(x, rect.y + rect.height);
          ctx.stroke();
        }
        break;
      }
      case "rect": {
        if (pts.length < 2) return;
        const x1 = t.indexToX(pts[0].index ?? 0);
        const y1 = t.priceToY(rect, minP, range, pts[0].price ?? 0);
        const x2 = t.indexToX(pts[1].index ?? 0);
        const y2 = t.priceToY(rect, minP, range, pts[1].price ?? 0);
        ctx.fillStyle = (ctx.strokeStyle as string) + "18";
        ctx.fillRect(
          Math.min(x1, x2), Math.min(y1, y2),
          Math.abs(x2 - x1), Math.abs(y2 - y1),
        );
        ctx.strokeRect(
          Math.min(x1, x2), Math.min(y1, y2),
          Math.abs(x2 - x1), Math.abs(y2 - y1),
        );
        break;
      }
      case "fib_retrace": {
        if (pts.length < 2) return;
        const y1 = t.priceToY(rect, minP, range, pts[0].price ?? 0);
        const y2 = t.priceToY(rect, minP, range, pts[1].price ?? 0);
        const levels = [0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];
        for (const lv of levels) {
          const y = y1 + (y2 - y1) * lv;
          ctx.beginPath();
          ctx.moveTo(rect.x, y);
          ctx.lineTo(rect.x + rect.width, y);
          ctx.strokeStyle = "#ffd70066";
          ctx.stroke();
          ctx.fillStyle = "#ffd700";
          ctx.font = "9px monospace";
          ctx.textAlign = "right";
          ctx.fillText(
            `${(lv * 100).toFixed(1)}% — ${(pts[0].price! + (pts[1].price! - pts[0].price!) * lv).toFixed(2)}`,
            rect.x + rect.width - 4, y - 3,
          );
        }
        break;
      }
      case "fib_ext": {
        if (pts.length < 2) return;
        const y1 = t.priceToY(rect, minP, range, pts[0].price ?? 0);
        const y2 = t.priceToY(rect, minP, range, pts[1].price ?? 0);
        const extLevels = [-0.272, -0.618, 1.272, 1.618, 2.272, 2.618];
        for (const lv of extLevels) {
          const y = y1 + (y2 - y1) * (1 + Math.abs(lv > 0 ? lv - 1 : lv));
          ctx.beginPath();
          ctx.moveTo(rect.x, y);
          ctx.lineTo(rect.x + rect.width, y);
          ctx.strokeStyle = "#60a5fa66";
          ctx.setLineDash([2, 6]);
          ctx.stroke();
          ctx.setLineDash([]);
          ctx.fillStyle = "#60a5fa";
          ctx.font = "9px monospace";
          ctx.textAlign = "right";
          ctx.fillText(
            `${(lv * 100).toFixed(1)}%`,
            rect.x + rect.width - 4, y - 3,
          );
        }
        break;
      }
    }

    // Label
    if (obj.label) {
      const first = pts[0];
      const lx = t.indexToX(first.index ?? 0) + 8;
      const ly = t.priceToY(rect, minP, range, first.price ?? 0) - 6;
      ctx.fillStyle = ctx.strokeStyle as string;
      ctx.font = "10px monospace";
      ctx.textAlign = "left";
      ctx.fillText(obj.label, lx, ly);
    }
  }

  private drawHandle(x: number, y: number): void {
    const ctx = this.ctx;
    ctx.fillStyle = ctx.strokeStyle as string;
    ctx.beginPath();
    ctx.arc(x, y, 4, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = "#fff";
    ctx.lineWidth = 1;
    ctx.stroke();
  }

  hitTest(x: number, y: number, threshold = 8): DrawingObject | null {
    for (let i = this.objects.length - 1; i >= 0; i--) {
      const obj = this.objects[i];
      for (const pt of obj.points) {
        const dx = x - (pt.x ?? 0);
        const dy = y - (pt.y ?? 0);
        if (Math.sqrt(dx * dx + dy * dy) < threshold) return obj;
      }
    }
    return null;
  }
}

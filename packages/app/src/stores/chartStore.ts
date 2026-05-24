import { create } from "zustand";
import type { DrawingTool } from "@me/ui";

export type ChartType = "candlestick" | "heikin_ashi" | "line";
export type Period = "1min" | "5min" | "15min" | "30min" | "60min" | "D" | "W" | "M";

export interface ChartState {
  chartType: ChartType;
  setChartType: (type: ChartType) => void;

  period: Period;
  setPeriod: (p: Period) => void;

  activeIndicators: string[];
  addIndicator: (name: string) => void;
  removeIndicator: (name: string) => void;
  clearIndicators: () => void;

  drawingTool: DrawingTool | null;
  setDrawingTool: (tool: DrawingTool | null) => void;

  crosshair: boolean;
  toggleCrosshair: () => void;

  gridMode: boolean;
  toggleGridMode: () => void;
}

export const useChartStore = create<ChartState>()((set) => ({
  chartType: "candlestick",
  setChartType: (chartType) => set({ chartType }),

  period: "D",
  setPeriod: (period) => set({ period }),

  activeIndicators: [],
  addIndicator: (name) =>
    set((s) => ({
      activeIndicators: s.activeIndicators.includes(name)
        ? s.activeIndicators
        : [...s.activeIndicators, name],
    })),
  removeIndicator: (name) =>
    set((s) => ({ activeIndicators: s.activeIndicators.filter((n) => n !== name) })),
  clearIndicators: () => set({ activeIndicators: [] }),

  drawingTool: null,
  setDrawingTool: (drawingTool) => set({ drawingTool }),

  crosshair: true,
  toggleCrosshair: () => set((s) => ({ crosshair: !s.crosshair })),

  gridMode: false,
  toggleGridMode: () => set((s) => ({ gridMode: !s.gridMode })),
}));

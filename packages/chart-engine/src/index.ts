// @me/chart-engine — K线图渲染引擎

export { ChartEngine } from "./ChartEngine";
export type { ChartType } from "./ChartEngine";
export { ViewportManager } from "./layout/ViewportManager";
export { CandleRenderer } from "./renderer/CandleRenderer";
export { HeikinAshiRenderer } from "./renderer/HeikinAshiRenderer";
export type { HeikinAshiBar } from "./renderer/HeikinAshiRenderer";
export { LineRenderer } from "./renderer/LineRenderer";
export { VolumeRenderer } from "./renderer/VolumeRenderer";
export { GridRenderer } from "./renderer/GridRenderer";
export { CrosshairRenderer } from "./renderer/CrosshairRenderer";
export { IndicatorRenderer } from "./renderer/IndicatorRenderer";
export type { IndicatorData } from "./renderer/IndicatorRenderer";
export { DrawingRenderer } from "./renderer/DrawingRenderer";
export type { DrawingObject } from "./renderer/DrawingRenderer";

export { DateAxisRenderer } from "./renderer/DateAxisRenderer";
export { formatVolumeCN, formatAmountCN, autoPrecision, formatPriceCN } from "./utils/format";

export type {
  OHLCV, ViewRect, DataRange, IndicatorLine, LayoutZones, CrosshairMode,
} from "./types";

export const version = "0.3.0";

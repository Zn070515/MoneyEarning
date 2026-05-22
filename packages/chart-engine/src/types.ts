// OHLCV data point
export interface OHLCV {
  time: number; // unix timestamp in seconds
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  amount?: number;
  turnover?: number;
}

// Pixel-space rectangle
export interface ViewRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

// Visible data range
export interface DataRange {
  from: number; // first visible index
  to: number; // last visible index
}

// Single indicator output
export interface IndicatorLine {
  name: string;
  values: Float64Array;
  style: "line" | "histogram" | "dots" | "band";
  color?: string;
}

// Chart layout zones
export interface LayoutZones {
  main: ViewRect; // K-line area
  volume: ViewRect; // volume area
  indicator: ViewRect[]; // indicator panels (0-N)
}

export type CrosshairMode = "none" | "cross" | "vertical";

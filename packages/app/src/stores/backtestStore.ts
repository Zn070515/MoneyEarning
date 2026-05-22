import { create } from "zustand";

export interface BacktestConfig {
  template: string;
  initialCapital: number;
  commissionRate: number;
  stampTaxRate: number;
  slippage: number;
  startDate: string;
  endDate: string;
  params: Record<string, number>;
}

export interface BacktestResult {
  totalReturn: number;
  annualReturn: number;
  maxDrawdown: number;
  sharpeRatio: number;
  sortinoRatio: number;
  calmarRatio: number;
  winRate: number;
  totalTrades: number;
  equityCurve: [string, number][];
}

export interface BacktestState {
  config: BacktestConfig;
  updateConfig: (patch: Partial<BacktestConfig>) => void;
  updateParam: (key: string, value: number) => void;

  running: boolean;
  result: BacktestResult | null;
  error: string | null;
  setResult: (r: BacktestResult | null, err?: string) => void;
  setRunning: (v: boolean) => void;
}

export const useBacktestStore = create<BacktestState>()((set) => ({
  config: {
    template: "ma_cross",
    initialCapital: 100_000,
    commissionRate: 0.0003,
    stampTaxRate: 0.001,
    slippage: 0.001,
    startDate: "2020-01-01",
    endDate: "2025-12-31",
    params: { fast: 5, slow: 20 },
  },
  updateConfig: (patch) =>
    set((s) => ({ config: { ...s.config, ...patch } })),
  updateParam: (key, value) =>
    set((s) => ({ config: { ...s.config, params: { ...s.config.params, [key]: value } } })),

  running: false,
  result: null,
  error: null,
  setResult: (result, error) => set({ result, error: error ?? null, running: false }),
  setRunning: (running) => set({ running, error: null }),
}));

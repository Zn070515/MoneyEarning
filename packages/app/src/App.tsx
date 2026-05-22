import { useState, useEffect, useCallback } from "react";
import { Routes, Route } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import {
  KLineChart, WatchlistPanel, StockListPanel,
  TradeJournalPanel, ImportDialog, LicensePanel,
  IndicatorSelector, StrategyPanel, BacktestPanel,
  ScannerPanel, DistributionPanel, ChartToolbar,
} from "@me/ui";
import type { DrawingTool } from "@me/ui";
import { OHLCV, IndicatorData, ChartType, DrawingObject } from "@me/chart-engine";

interface StockInfo {
  id: number;
  code: string;
  name: string;
  exchange: string;
  ipo_date: string | null;
  total_rows: number;
  first_date: string | null;
  last_date: string | null;
}

interface DailyPrice {
  id: number;
  stock_id: number;
  trade_date: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  amount: number;
  turnover: number | null;
}

interface LicenseStatus {
  valid: boolean;
  tier: string;
  expiry: string | null;
  trial_days_left: number | null;
}

type SidebarTab = "stocks" | "watchlist" | "indicators";
type RightPanelTab = "trades" | "strategies" | "backtest" | "scanner" | "distribution" | "license";

function tierLabel(t: string) {
  switch (t) {
    case "pro": return "专业版";
    case "trial": return "试用版";
    default: return "免费版";
  }
}

function Home() {
  const [sidebarTab, setSidebarTab] = useState<SidebarTab>("stocks");
  const [rightTab, setRightTab] = useState<RightPanelTab>("trades");
  const [selectedStock, setSelectedStock] = useState<StockInfo | null>(null);
  const [selectedWatchlistId, setSelectedWatchlistId] = useState<number | null>(null);
  const [chartData, setChartData] = useState<OHLCV[]>([]);
  const [loading, setLoading] = useState(false);
  const [dataStatus, setDataStatus] = useState("");
  const [showImport, setShowImport] = useState(false);
  const [license, setLicense] = useState<LicenseStatus | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [indicators, setIndicators] = useState<IndicatorData[]>([]);
  const [chartType, setChartType] = useState<ChartType>("candlestick");
  const [activeTool, setActiveTool] = useState<DrawingTool | null>(null);
  const [drawings, setDrawings] = useState<DrawingObject[]>([]);

  const loadLicense = useCallback(async () => {
    try {
      const s = await invoke<LicenseStatus>("check_license");
      setLicense(s);
    } catch (e) {
      console.error("License check:", e);
    }
  }, []);

  useEffect(() => { loadLicense(); }, [loadLicense]);

  const loadChartData = useCallback(async (stockId: number) => {
    setLoading(true);
    setDataStatus("加载中...");
    try {
      const data = await invoke<DailyPrice[]>("query_daily_prices", {
        stockId: stockId,
        startDate: "2020-01-01",
        endDate: "2099-12-31",
      });
      const ohlcv: OHLCV[] = data.map(d => ({
        time: Math.floor(new Date(d.trade_date).getTime() / 1000),
        open: d.open, high: d.high, low: d.low, close: d.close,
        volume: d.volume, amount: d.amount, turnover: d.turnover ?? undefined,
      }));
      setChartData(ohlcv);
      setDataStatus(`${ohlcv.length} 条K线数据`);
    } catch (e) {
      console.error("Failed to load chart data:", e);
      setDataStatus("加载失败");
    }
    setLoading(false);
  }, []);

  const handleSelectStock = (stock: StockInfo) => {
    setSelectedStock(stock);
    loadChartData(stock.id);
  };

  const handleSelectWatchlist = (id: number) => {
    setSelectedWatchlistId(id);
  };

  const handleAddToWatchlist = async (stockId: number) => {
    if (!selectedWatchlistId) return;
    try {
      await invoke("watchlist_add_item", { watchlistId: selectedWatchlistId, stockId });
    } catch (e) {
      console.error("Failed to add to watchlist:", e);
    }
  };

  const handleImportSuccess = () => {
    setRefreshKey(k => k + 1);
  };

  return (
    <div style={{
      display: "flex", flexDirection: "column", height: "100vh",
      background: "#0f0f23", color: "#ccc",
    }}>
      {/* Header */}
      <header style={{
        padding: "6px 16px", background: "#16213e",
        borderBottom: "1px solid #2a2a4a",
        display: "flex", justifyContent: "space-between",
        alignItems: "center", fontFamily: "monospace",
        fontSize: 13, flexShrink: 0,
      }}>
        <div style={{ display: "flex", gap: 16, alignItems: "center" }}>
          <span style={{ color: "#fbbf24", fontWeight: 700, fontSize: 15 }}>
            MoneyEarning
          </span>
          <span style={{ color: "#888" }}>v0.6.0</span>
          {license && (
            <span style={{
              color: license.tier === "pro" ? "#fbbf24" : "#22c55e",
              fontSize: 11, background: "#1a1a2e",
              padding: "2px 8px", borderRadius: 3,
            }}>
              {tierLabel(license.tier)}
            </span>
          )}
          <span style={{ color: "#666" }}>|</span>
          <span style={{ color: "#aaa" }}>
            {selectedStock
              ? `${selectedStock.code} ${selectedStock.name}`
              : "选择股票开始分析"}
          </span>
        </div>
        <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
          <button onClick={() => setShowImport(true)} style={headerBtn}>
            + 导入数据
          </button>
          {loading && <span style={{ color: "#fbbf24" }}>⏳</span>}
          <span style={{ color: "#888", fontSize: 12 }}>{dataStatus}</span>
          {selectedStock && (
            <span style={{ color: "#666", fontSize: 11 }}>
              {selectedStock.exchange} · {selectedStock.total_rows}条
              {selectedStock.first_date && ` · ${selectedStock.first_date}~${selectedStock.last_date}`}
            </span>
          )}
        </div>
      </header>

      {/* Chart Toolbar */}
      <ChartToolbar
        chartType={chartType}
        onChartTypeChange={setChartType}
        activeTool={activeTool}
        onToolChange={setActiveTool}
        onClearDrawings={() => setDrawings([])}
        drawingCount={drawings.length}
      />

      {/* Body */}
      <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        {/* Left Sidebar */}
        <div style={{
          width: 280, display: "flex", flexDirection: "column",
          borderRight: "1px solid #2a2a4a", flexShrink: 0,
        }}>
          <div style={{
            display: "flex", borderBottom: "1px solid #2a2a4a",
            background: "#16213e",
          }}>
            <TabBtn active={sidebarTab === "stocks"} onClick={() => setSidebarTab("stocks")}>
              股票
            </TabBtn>
            <TabBtn active={sidebarTab === "watchlist"} onClick={() => setSidebarTab("watchlist")}>
              自选
            </TabBtn>
            <TabBtn active={sidebarTab === "indicators"} onClick={() => setSidebarTab("indicators")}>
              指标
            </TabBtn>
          </div>

          <div style={{ flex: 1, overflow: "hidden" }}>
            {sidebarTab === "stocks" && (
              <StockListPanel
                key={refreshKey}
                onSelectStock={handleSelectStock}
                selectedWatchlistId={selectedWatchlistId}
                onAddToWatchlist={handleAddToWatchlist}
              />
            )}
            {sidebarTab === "watchlist" && (
              <WatchlistPanel
                key={refreshKey}
                onSelectStock={handleSelectStock}
                onSelectWatchlist={handleSelectWatchlist}
              />
            )}
            {sidebarTab === "indicators" && (
              <IndicatorSelector
                data={chartData}
                activeIndicators={indicators}
                onChange={setIndicators}
              />
            )}
          </div>
        </div>

        {/* Main Chart */}
        <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
          {chartData.length > 0 ? (
            <KLineChart
              data={chartData}
              indicators={indicators}
              chartType={chartType}
              activeTool={activeTool}
              drawings={drawings}
              onDrawingAdd={(obj) => setDrawings(prev => [...prev, obj])}
              onToolCancel={() => setActiveTool(null)}
            />
          ) : (
            <EmptyChart onImport={() => setShowImport(true)} />
          )}
        </div>

        {/* Right Panel */}
        <div style={{
          width: 300, borderLeft: "1px solid #2a2a4a",
          flexShrink: 0, overflow: "hidden", display: "flex", flexDirection: "column",
        }}>
          <div style={{
            display: "flex", borderBottom: "1px solid #2a2a4a",
            background: "#16213e", flexWrap: "wrap",
          }}>
            <TabBtn active={rightTab === "trades"} onClick={() => setRightTab("trades")}>
              交易
            </TabBtn>
            <TabBtn active={rightTab === "strategies"} onClick={() => setRightTab("strategies")}>
              策略
            </TabBtn>
            <TabBtn active={rightTab === "backtest"} onClick={() => setRightTab("backtest")}>
              回测
            </TabBtn>
            <TabBtn active={rightTab === "scanner"} onClick={() => setRightTab("scanner")}>
              扫描
            </TabBtn>
            <TabBtn active={rightTab === "distribution"} onClick={() => setRightTab("distribution")}>
              筹码
            </TabBtn>
            <TabBtn active={rightTab === "license"} onClick={() => setRightTab("license")}>
              授权
            </TabBtn>
          </div>
          <div style={{ flex: 1, overflow: "hidden" }}>
            {rightTab === "trades" && (
              <TradeJournalPanel
                selectedStockId={selectedStock?.id ?? null}
                compact
              />
            )}
            {rightTab === "strategies" && (
              <StrategyPanel />
            )}
            {rightTab === "backtest" && (
              <BacktestPanel data={chartData} />
            )}
            {rightTab === "scanner" && (
              <ScannerPanel />
            )}
            {rightTab === "distribution" && (
              <DistributionPanel stockId={selectedStock?.id ?? null} />
            )}
            {rightTab === "license" && (
              <LicensePanel onActivated={loadLicense} />
            )}
          </div>
        </div>
      </div>

      {/* Import Dialog */}
      <ImportDialog
        visible={showImport}
        onClose={() => setShowImport(false)}
        onSuccess={handleImportSuccess}
      />
    </div>
  );
}

function TabBtn({ active, onClick, children }: {
  active: boolean; onClick: () => void; children: React.ReactNode;
}) {
  return (
    <button onClick={onClick} style={{
      flex: 1, padding: "8px 12px", border: "none",
      background: active ? "#1a1a2e" : "transparent",
      color: active ? "#fbbf24" : "#888",
      cursor: "pointer", fontSize: 13,
      fontFamily: "monospace", fontWeight: active ? 600 : 400,
      borderBottom: active ? "2px solid #fbbf24" : "2px solid transparent",
    }}>
      {children}
    </button>
  );
}

function EmptyChart({ onImport }: { onImport: () => void }) {
  return (
    <div style={{
      flex: 1, display: "flex", alignItems: "center", justifyContent: "center",
      color: "#666", fontFamily: "monospace", fontSize: 16,
      background: "#1a1a2e",
    }}>
      <div style={{ textAlign: "center" }}>
        <div style={{ fontSize: 48, marginBottom: 16, color: "#3a3a5a" }}>📈</div>
        <div>从左侧选择一个股票开始分析</div>
        <div style={{ fontSize: 12, marginTop: 16, color: "#555" }}>
          <button onClick={onImport} style={{
            background: "#fbbf24", color: "#000", border: "none",
            padding: "6px 16px", borderRadius: 4, cursor: "pointer",
            fontFamily: "monospace", fontSize: 13, fontWeight: 600,
          }}>
            导入CSV数据
          </button>
        </div>
      </div>
    </div>
  );
}

const headerBtn: React.CSSProperties = {
  background: "#3a3a5a", color: "#ccc", border: "none",
  padding: "4px 12px", borderRadius: 4, cursor: "pointer",
  fontSize: 12, fontFamily: "monospace",
};

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
    </Routes>
  );
}

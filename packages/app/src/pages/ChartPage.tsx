import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  KLineChart,
  WatchlistPanel,
  StockListPanel,
  TradeJournalPanel,
  ImportDialog,
  LicensePanel,
  IndicatorSelector,
  StrategyPanel,
  BacktestPanel,
  ScannerPanel,
  DistributionPanel,
  ChartToolbar,
  RiskPanel,
  PatternPanel,
  DownloadPanel,
} from "@me/ui";
import { OHLCV, IndicatorData, ChartType, DrawingObject } from "@me/chart-engine";
import { useAppStore, type LicenseStatus } from "../stores/appStore";
import { useChartStore } from "../stores/chartStore";

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

type SidebarTab = "stocks" | "watchlist" | "indicators";
type RightPanelTab =
  | "trades"
  | "strategies"
  | "backtest"
  | "scanner"
  | "distribution"
  | "patterns"
  | "risk"
  | "download"
  | "license";

function tierLabel(t: string) {
  switch (t) {
    case "pro":
      return "专业版";
    case "trial":
      return "试用版";
    default:
      return "免费版";
  }
}

export default function ChartPage() {
  // --- Zustand stores ---
  const selectedStockId = useAppStore((s) => s.selectedStockId);
  const selectedStockCode = useAppStore((s) => s.selectedStockCode);
  const selectedStockName = useAppStore((s) => s.selectedStockName);
  const selectStock = useAppStore((s) => s.selectStock);
  const largeFont = useAppStore((s) => s.largeFont);
  const highContrast = useAppStore((s) => s.highContrast);
  const toggleLargeFont = useAppStore((s) => s.toggleLargeFont);
  const toggleHighContrast = useAppStore((s) => s.toggleHighContrast);

  const chartType = useChartStore((s) => s.chartType);
  const setChartType = useChartStore((s) => s.setChartType);
  const drawingTool = useChartStore((s) => s.drawingTool);
  const setDrawingTool = useChartStore((s) => s.setDrawingTool);
  const gridMode = useChartStore((s) => s.gridMode);
  const toggleGridMode = useChartStore((s) => s.toggleGridMode);

  // --- Page-local state ---
  const [sidebarTab, setSidebarTab] = useState<SidebarTab>("stocks");
  const [rightTab, setRightTab] = useState<RightPanelTab>("trades");
  const [selectedWatchlistId, setSelectedWatchlistId] = useState<number | null>(null);
  const [chartData, setChartData] = useState<OHLCV[]>([]);
  const [loading, setLoading] = useState(false);
  const [dataStatus, setDataStatus] = useState("");
  const [showImport, setShowImport] = useState(false);
  const [selectedStrategyTemplate, setSelectedStrategyTemplate] = useState<string | undefined>();
  const refreshLicense = useAppStore((s) => s.refreshLicense);
  const [license, setLicense] = useState<LicenseStatus | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [indicatorsData, setIndicatorsData] = useState<IndicatorData[]>([]);
  const [drawings, setDrawings] = useState<DrawingObject[]>([]);
  const [selectedDrawingId, setSelectedDrawingId] = useState<string | null>(null);

  // ── Multi-chart grid state ──
  interface GridCellData {
    stockId: number | null;
    stockCode: string | null;
    stockName: string | null;
    data: OHLCV[];
    indicators: IndicatorData[];
    drawings: DrawingObject[];
  }
  const emptyCell = (): GridCellData => ({
    stockId: null, stockCode: null, stockName: null,
    data: [], indicators: [], drawings: [],
  });
  const [gridCells, setGridCells] = useState<GridCellData[]>([emptyCell(), emptyCell(), emptyCell(), emptyCell()]);
  const [activeCellIdx, setActiveCellIdx] = useState(0);
  const mountedRef = useRef(true);
  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  const loadGridCellData = useCallback(async (cellIdx: number, stockId: number, code: string, name: string) => {
    setLoading(true);
    try {
      const data = await invoke<DailyPrice[]>("query_daily_prices", {
        stockId, startDate: "2020-01-01", endDate: "2099-12-31",
      });
      if (!mountedRef.current) return;
      const ohlcv: OHLCV[] = data.map((d) => ({
        time: Math.floor(new Date(d.trade_date).getTime() / 1000),
        open: d.open, high: d.high, low: d.low, close: d.close,
        volume: d.volume, amount: d.amount, turnover: d.turnover ?? undefined,
      }));
      setGridCells(prev => {
        const next = [...prev];
        next[cellIdx] = { ...next[cellIdx], stockId, stockCode: code, stockName: name, data: ohlcv };
        return next;
      });
      if (cellIdx === activeCellIdx) {
        setChartData(ohlcv);
      }
    } catch (e) {
      if (mountedRef.current) console.error("Grid cell load failed:", e);
    }
    if (mountedRef.current) setLoading(false);
  }, [activeCellIdx]);

  // When stock is selected from sidebar, assign to active grid cell
  const handleSelectStock = (stock: StockInfo) => {
    selectStock(stock.id, stock.code, stock.name);
    if (gridMode) {
      setGridCells(prev => {
        const next = [...prev];
        next[activeCellIdx] = { ...next[activeCellIdx], stockId: stock.id, stockCode: stock.code, stockName: stock.name };
        return next;
      });
      loadGridCellData(activeCellIdx, stock.id, stock.code, stock.name);
    }
  };

  const loadLicense = useCallback(async () => {
    try {
      const s = await refreshLicense();
      setLicense(s);
    } catch (e) {
      console.error("License check:", e);
    }
  }, [refreshLicense]);

  useEffect(() => {
    loadLicense();
  }, [loadLicense]);

  const loadChartData = useCallback(async (stockId: number) => {
    setLoading(true);
    setDataStatus("加载中...");
    try {
      const data = await invoke<DailyPrice[]>("query_daily_prices", {
        stockId,
        startDate: "2020-01-01",
        endDate: "2099-12-31",
      });
      if (!mountedRef.current) return;
      const ohlcv: OHLCV[] = data.map((d) => ({
        time: Math.floor(new Date(d.trade_date).getTime() / 1000),
        open: d.open,
        high: d.high,
        low: d.low,
        close: d.close,
        volume: d.volume,
        amount: d.amount,
        turnover: d.turnover ?? undefined,
      }));
      setChartData(ohlcv);
      setDataStatus(`${ohlcv.length} 条K线数据`);
    } catch (e) {
      if (mountedRef.current) {
        console.error("Failed to load chart data:", e);
        setDataStatus("加载失败");
      }
    }
    if (mountedRef.current) setLoading(false);
  }, []);

  // Watchlist→chart linkage: auto-load when selectedStockId changes (skip in grid mode)
  useEffect(() => {
    if (selectedStockId != null && !gridMode) {
      loadChartData(selectedStockId);
    }
  }, [selectedStockId, loadChartData, gridMode]);

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
    setRefreshKey((k) => k + 1);
  };

  // ChartType from @me/chart-engine is a superset of the store's ChartType;
  // only pass through supported values.
  const handleChartTypeChange = (ct: ChartType) => {
    if (ct === "candlestick" || ct === "heikin_ashi" || ct === "line") {
      setChartType(ct);
    }
  };

  const selectedStock: StockInfo | null =
    selectedStockId != null && selectedStockCode != null
      ? {
          id: selectedStockId,
          code: selectedStockCode,
          name: selectedStockName ?? "",
          exchange: "",
          ipo_date: null,
          total_rows: chartData.length,
          first_date: null,
          last_date: null,
        }
      : null;

  return (
    <div
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Header */}
      <header
        style={{
          padding: "6px 16px",
          background: "#161616",
          borderBottom: "1px solid #2A2A2A",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          fontFamily: "monospace",
          fontSize: 13,
          flexShrink: 0,
        }}
      >
        <div style={{ display: "flex", gap: 16, alignItems: "center" }}>
          <span style={{ color: "#CCAA00", fontWeight: 700, fontSize: 15 }}>
            QuantVault
          </span>
          {license && (
            <span
              style={{
                color: license.tier === "pro" ? "#CCAA00" : "#26A69A",
                fontSize: 11,
                background: "#121212",
                padding: "2px 8px",
                borderRadius: 3,
              }}
            >
              {tierLabel(license.tier)}
            </span>
          )}
          <span style={{ color: "#666666" }}>|</span>
          <span style={{ color: "#858585" }}>
            {selectedStock
              ? `${selectedStock.code} ${selectedStock.name}`
              : "选择股票开始分析"}
          </span>
        </div>
        <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
          <button onClick={() => setShowImport(true)} style={headerBtn}>
            + 导入数据
          </button>
          <span style={{ color: "#666666" }}>|</span>
          <button
            onClick={toggleLargeFont}
            style={{
              ...headerBtn,
              background: largeFont ? "#CCAA00" : "transparent",
              color: largeFont ? "#000" : "#858585",
              border: largeFont ? "1px solid #CCAA00" : "1px solid #555",
            }}
            title="大字体模式"
          >
            大字
          </button>
          <button
            onClick={toggleHighContrast}
            style={{
              ...headerBtn,
              background: highContrast ? "#CCAA00" : "transparent",
              color: highContrast ? "#000" : "#858585",
              border: highContrast ? "1px solid #CCAA00" : "1px solid #555",
            }}
            title="高对比度模式"
          >
            高对比
          </button>
          {loading && <span style={{ color: "#CCAA00" }}>⏳</span>}
          <span style={{ color: "#858585", fontSize: 12 }}>{dataStatus}</span>
        </div>
      </header>

      {/* Chart Toolbar */}
      <ChartToolbar
        chartType={chartType}
        onChartTypeChange={handleChartTypeChange}
        activeTool={drawingTool}
        onToolChange={setDrawingTool}
        onClearDrawings={() => {
          if (gridMode) {
            setGridCells((prev) => {
              const next = [...prev];
              next[activeCellIdx] = { ...next[activeCellIdx], drawings: [] };
              return next;
            });
          } else {
            setDrawings([]);
          }
        }}
        drawingCount={drawings.length}
        gridMode={gridMode}
        onToggleGridMode={toggleGridMode}
      />

      {/* Body */}
      <div style={{ flex: 1, display: "flex", overflow: "hidden" }}>
        {/* Left Sidebar */}
        <div
          style={{
            width: 280,
            display: "flex",
            flexDirection: "column",
            borderRight: "1px solid #2A2A2A",
            flexShrink: 0,
          }}
        >
          <div
            style={{
              display: "flex",
              borderBottom: "1px solid #2A2A2A",
              background: "#161616",
            }}
          >
            <TabBtn active={sidebarTab === "stocks"} onClick={() => setSidebarTab("stocks")}>
              股票
            </TabBtn>
            <TabBtn
              active={sidebarTab === "watchlist"}
              onClick={() => setSidebarTab("watchlist")}
            >
              自选
            </TabBtn>
            <TabBtn
              active={sidebarTab === "indicators"}
              onClick={() => setSidebarTab("indicators")}
            >
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
                activeIndicators={indicatorsData}
                onChange={setIndicatorsData}
              />
            )}
          </div>
        </div>

        {/* Main Chart */}
        <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
          {gridMode ? (
            /* 2×2 Grid Layout */
            <div style={{ flex: 1, display: "grid", gridTemplateColumns: "1fr 1fr", gridTemplateRows: "1fr 1fr", gap: 2, background: "#2A2A2A" }}>
              {gridCells.map((cell, idx) => (
                <div
                  key={idx}
                  onClick={() => {
                    setActiveCellIdx(idx);
                    if (cell.stockId != null) {
                      selectStock(cell.stockId, cell.stockCode ?? "", cell.stockName ?? "");
                      if (cell.data.length > 0) setChartData(cell.data);
                    }
                  }}
                  style={{
                    background: "#121212",
                    display: "flex",
                    flexDirection: "column",
                    overflow: "hidden",
                    border: activeCellIdx === idx ? "2px solid #CCAA00" : "2px solid transparent",
                    cursor: "pointer",
                  }}
                >
                  {/* Cell header */}
                  <div style={{
                    padding: "4px 8px", background: activeCellIdx === idx ? "#1A1A0A" : "#161616",
                    display: "flex", justifyContent: "space-between", alignItems: "center",
                    borderBottom: "1px solid #2A2A2A", flexShrink: 0,
                  }}>
                    <span style={{
                      color: cell.stockCode ? "#CCAA00" : "#666666",
                      fontSize: 11, fontFamily: "monospace", fontWeight: 600,
                    }}>
                      {cell.stockCode ? `${cell.stockCode} ${cell.stockName || ""}` : `画布 ${idx + 1}`}
                    </span>
                    {activeCellIdx === idx && (
                      <span style={{ color: "#CCAA00", fontSize: 9, fontFamily: "monospace" }}>● 激活</span>
                    )}
                  </div>
                  {/* Chart content */}
                  <div style={{ flex: 1, overflow: "hidden" }}>
                    {cell.data.length > 0 ? (
                      <KLineChart
                        data={cell.data}
                        indicators={cell.indicators}
                        chartType={chartType}
                        activeTool={activeCellIdx === idx ? drawingTool : null}
                        drawings={cell.drawings}
                        onDrawingAdd={(obj) => {
                          setGridCells(prev => {
                            const next = [...prev];
                            next[idx] = { ...next[idx], drawings: [...next[idx].drawings, obj] };
                            return next;
                          });
                        }}
                        onDrawingDelete={(id) => {
                          setGridCells(prev => {
                            const next = [...prev];
                            next[idx] = { ...next[idx], drawings: next[idx].drawings.filter((d) => d.id !== id) };
                            return next;
                          });
                        }}
                        onDrawingSelect={(_id) => {}}
                        onToolCancel={() => {}}
                      />
                    ) : (
                      <div style={{
                        display: "flex", alignItems: "center", justifyContent: "center",
                        height: "100%", color: "#555", fontSize: 11, fontFamily: "monospace",
                      }}>
                        点击左侧股票 → 自动加载到激活画布
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            /* Single chart */
            chartData.length > 0 ? (
              <KLineChart
                data={chartData}
                indicators={indicatorsData}
                chartType={chartType}
                activeTool={drawingTool}
                drawings={drawings}
                onDrawingAdd={(obj) => setDrawings((prev) => [...prev, obj])}
                onDrawingDelete={(id) => { setDrawings((prev) => prev.filter((d) => d.id !== id)); setSelectedDrawingId(null); }}
                onDrawingSelect={(id) => setSelectedDrawingId(id)}
                onToolCancel={() => setDrawingTool(null)}
              />
            ) : (
              <EmptyChart onImport={() => setShowImport(true)} />
            )
          )}
        </div>

        {/* Right Panel */}
        <div
          style={{
            width: 300,
            borderLeft: "1px solid #2A2A2A",
            flexShrink: 0,
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
          }}
        >
          <div
            style={{
              display: "flex",
              borderBottom: "1px solid #2A2A2A",
              background: "#161616",
              flexWrap: "wrap",
            }}
          >
            <TabBtn active={rightTab === "trades"} onClick={() => setRightTab("trades")}>
              交易
            </TabBtn>
            <TabBtn active={rightTab === "strategies"} onClick={() => setRightTab("strategies")}>
              策略
            </TabBtn>
            <TabBtn active={rightTab === "backtest"} onClick={() => setRightTab("backtest")}>
              回测 <ProBadge />
            </TabBtn>
            <TabBtn active={rightTab === "scanner"} onClick={() => setRightTab("scanner")}>
              扫描 <ProBadge />
            </TabBtn>
            <TabBtn active={rightTab === "distribution"} onClick={() => setRightTab("distribution")}>
              筹码 <ProBadge />
            </TabBtn>
            <TabBtn active={rightTab === "patterns"} onClick={() => setRightTab("patterns")}>
              形态 <ProBadge />
            </TabBtn>
            <TabBtn active={rightTab === "risk"} onClick={() => setRightTab("risk")}>
              风控 <ProBadge />
            </TabBtn>
            <TabBtn active={rightTab === "download"} onClick={() => setRightTab("download")}>
              下载
            </TabBtn>
            <TabBtn active={rightTab === "license"} onClick={() => setRightTab("license")}>
              授权
            </TabBtn>
          </div>
          <div style={{ flex: 1, overflow: "hidden" }}>
            {rightTab === "trades" && (
              <TradeJournalPanel selectedStockId={selectedStockId} compact />
            )}
            {rightTab === "strategies" && (
              <StrategyPanel
                selectedStockId={selectedStockId}
                onSelectStrategy={(s) => {
                  setSelectedStrategyTemplate(s.template_type ?? s.name);
                  setRightTab("backtest");
                }}
              />
            )}
            {rightTab === "backtest" && (
              <BacktestPanel
                data={chartData}
                isPro={license?.tier === "pro"}
                initialTemplate={selectedStrategyTemplate}
              />
            )}
            {rightTab === "scanner" && <ScannerPanel />}
            {rightTab === "distribution" && (
              <DistributionPanel stockId={selectedStockId} />
            )}
            {rightTab === "patterns" && (
              <PatternPanel stockId={selectedStockId} />
            )}
            {rightTab === "risk" && <RiskPanel stockId={selectedStockId} />}
            {rightTab === "download" && <DownloadPanel />}
            {rightTab === "license" && <LicensePanel onActivated={loadLicense} />}
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

function TabBtn({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        flex: 1,
        padding: "8px 12px",
        border: "none",
        background: active ? "#121212" : "transparent",
        color: active ? "#CCAA00" : "#858585",
        cursor: "pointer",
        fontSize: 13,
        fontFamily: "monospace",
        fontWeight: active ? 600 : 400,
        borderBottom: active ? "2px solid #CCAA00" : "2px solid transparent",
      }}
    >
      {children}
    </button>
  );
}

function EmptyChart({ onImport }: { onImport: () => void }) {
  return (
    <div
      style={{
        flex: 1,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        color: "#666666",
        fontFamily: "monospace",
        fontSize: 16,
        background: "#121212",
      }}
    >
      <div style={{ textAlign: "center" }}>
        <div style={{ fontSize: 48, marginBottom: 16, color: "#2A2A2A" }}>📈</div>
        <div>从左侧选择一个股票开始分析</div>
        <div style={{ fontSize: 12, marginTop: 16, color: "#555" }}>
          <button
            onClick={onImport}
            style={{
              background: "#CCAA00",
              color: "#000",
              border: "none",
              padding: "6px 16px",
              borderRadius: 4,
              cursor: "pointer",
              fontFamily: "monospace",
              fontSize: 13,
              fontWeight: 600,
            }}
          >
            导入CSV数据
          </button>
        </div>
      </div>
    </div>
  );
}

const headerBtn: React.CSSProperties = {
  background: "#2A2A2A",
  color: "#D4D4D4",
  border: "none",
  padding: "4px 12px",
  borderRadius: 4,
  cursor: "pointer",
  fontSize: 12,
  fontFamily: "monospace",
};

function ProBadge() {
  return (
    <span style={{
      fontSize: 8, padding: "1px 4px", marginLeft: 2,
      background: "rgba(126,87,194,0.15)", color: "#7E57C2",
      borderRadius: 2, verticalAlign: "middle",
      fontFamily: "monospace",
    }}>
      PRO
    </span>
  );
}

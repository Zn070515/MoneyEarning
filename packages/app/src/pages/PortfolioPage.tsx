import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RiskPanel } from "@me/ui";
import { useAppStore } from "../stores/appStore";

export default function PortfolioPage() {
  const selectedStockId = useAppStore((s) => s.selectedStockId);
  const selectedStockCode = useAppStore((s) => s.selectedStockCode);
  const [activeSection, setActiveSection] = useState<"risk" | "analysis">("risk");

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
      {/* Header */}
      <div
        style={{
          padding: "12px 20px",
          background: "#161616",
          borderBottom: "1px solid #2A2A2A",
          display: "flex",
          alignItems: "center",
          gap: 16,
          flexShrink: 0,
        }}
      >
        <h2 style={{ color: "#CCAA00", fontSize: 16, fontFamily: "monospace", margin: 0 }}>
          组合与风险
        </h2>
        {selectedStockCode && (
          <span style={{ color: "#858585", fontSize: 12, fontFamily: "monospace" }}>
            当前标的: {selectedStockCode}
          </span>
        )}
      </div>

      {/* Section nav */}
      <div
        style={{
          padding: "4px 20px",
          background: "#121212",
          borderBottom: "1px solid #2A2A2A",
          display: "flex",
          gap: 4,
          flexShrink: 0,
        }}
      >
        {([
          ["risk", "风险指标"],
          ["analysis", "组合分析"],
        ] as [typeof activeSection, string][]).map(([sec, label]) => (
          <button
            key={sec}
            onClick={() => setActiveSection(sec)}
            style={{
              padding: "6px 16px",
              background: activeSection === sec ? "#CCAA00" : "transparent",
              color: activeSection === sec ? "#000" : "#858585",
              border: "none",
              borderRadius: "4px 4px 0 0",
              cursor: "pointer",
              fontFamily: "monospace",
              fontSize: 12,
              fontWeight: activeSection === sec ? 600 : 400,
            }}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {activeSection === "risk" && (
          <div style={{ padding: 16 }}>
            {selectedStockId ? (
              <RiskPanel stockId={selectedStockId} />
            ) : (
              <EmptyState message="请在图表页面选择股票后查看风险指标" />
            )}
          </div>
        )}
        {activeSection === "analysis" && (
          <PortfolioAnalysisPanel selectedStockCode={selectedStockCode} />
        )}
      </div>
    </div>
  );
}

function EmptyState({ message }: { message: string }) {
  return (
    <div
      style={{
        textAlign: "center",
        color: "#666666",
        fontFamily: "monospace",
        fontSize: 14,
        padding: 60,
      }}
    >
      <div style={{ fontSize: 48, marginBottom: 16, color: "#2A2A2A" }}>📊</div>
      <div>{message}</div>
    </div>
  );
}

// ── Portfolio Analysis Panel ──

interface PortfolioHolding {
  code: string;
  name: string;
  shares: number;
  avgCost: number;
  currentPrice: number;
  marketValue: number;
  pnl: number;
  pnlPct: number;
  weight: number;
}

function PortfolioAnalysisPanel({ selectedStockCode }: { selectedStockCode: string | null }) {
  const [holdings, setHoldings] = useState<PortfolioHolding[]>(() => {
    try { return JSON.parse(localStorage.getItem("me-portfolio") || "[]"); }
    catch { return []; }
  });
  const [showForm, setShowForm] = useState(false);
  const [editIdx, setEditIdx] = useState(-1);
  const [simAmount, setSimAmount] = useState("");
  const [simPrice, setSimPrice] = useState("");
  const [simResult, setSimResult] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Form state
  const [formCode, setFormCode] = useState(selectedStockCode || "");
  const [formName, setFormName] = useState("");
  const [formShares, setFormShares] = useState("");
  const [formCost, setFormCost] = useState("");
  const [formPrice, setFormPrice] = useState("");

  const totalValue = holdings.reduce((s, h) => s + h.marketValue, 0);
  const totalPnl = holdings.reduce((s, h) => s + h.pnl, 0);
  const totalCost = holdings.reduce((s, h) => s + h.avgCost * h.shares, 0);
  const totalPnlPct = totalCost > 0 ? (totalPnl / totalCost) * 100 : 0;
  const maxWeight = Math.max(...holdings.map((h) => h.weight).concat([0]));

  const saveHoldings = (h: PortfolioHolding[]) => {
    setHoldings(h);
    localStorage.setItem("me-portfolio", JSON.stringify(h));
  };

  const addHolding = () => {
    if (!formCode || !formShares || !formCost || !formPrice) return;
    const shares = parseFloat(formShares);
    const avgCost = parseFloat(formCost);
    const currentPrice = parseFloat(formPrice);
    const marketValue = shares * currentPrice;
    const pnl = (currentPrice - avgCost) * shares;
    const pnlPct = avgCost > 0 ? ((currentPrice - avgCost) / avgCost) * 100 : 0;
    const h: PortfolioHolding = {
      code: formCode, name: formName || formCode, shares, avgCost,
      currentPrice, marketValue, pnl, pnlPct, weight: 0,
    };
    const newList = [...holdings, h];
    // Recalculate weights
    const tv = newList.reduce((s, x) => s + x.marketValue, 0);
    newList.forEach((x) => { x.weight = tv > 0 ? (x.marketValue / tv) * 100 : 0; });
    saveHoldings(newList);
    resetForm();
  };

  const removeHolding = (idx: number) => {
    const newList = holdings.filter((_, i) => i !== idx);
    const tv = newList.reduce((s, x) => s + x.marketValue, 0);
    newList.forEach((x) => { x.weight = tv > 0 ? (x.marketValue / tv) * 100 : 0; });
    saveHoldings(newList);
  };

  const resetForm = () => {
    setFormCode(""); setFormName(""); setFormShares(""); setFormCost(""); setFormPrice("");
    setShowForm(false); setEditIdx(-1);
  };

  const runSim = () => {
    const amt = parseFloat(simAmount);
    const price = parseFloat(simPrice);
    if (!amt || !price) return;
    const shares = amt / price;
    const newTV = totalValue + amt;
    const newCost = totalCost + amt;
    setSimResult(`买入 ¥${amt.toLocaleString()}（约${shares.toFixed(0)}股@${price.toFixed(2)}），持仓市值→¥${newTV.toLocaleString(undefined, { maximumFractionDigits: 0 })}，成本→¥${newCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}`);
  };

  const handleExportCSV = () => {
    if (holdings.length === 0) return;
    const header = "代码,名称,持仓股数,成本价,现价,市值,盈亏,盈亏%,占比%";
    const rows = holdings.map(h =>
      `${h.code},${h.name},${h.shares},${h.avgCost.toFixed(3)},${h.currentPrice.toFixed(3)},${h.marketValue.toFixed(2)},${h.pnl.toFixed(2)},${h.pnlPct.toFixed(2)},${h.weight.toFixed(1)}`
    );
    const csv = [header, ...rows].join("\n");
    const blob = new Blob(["﻿" + csv], { type: "text/csv;charset=utf-8" }); // BOM for Excel
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `portfolio_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImportCSV = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onerror = () => { alert("文件读取失败，请检查文件是否损坏"); };
    reader.onload = (ev) => {
      const text = ev.target?.result as string;
      if (!text) { alert("文件内容为空"); return; }
      const lines = text.split(/\r?\n/).filter(l => l.trim());
      if (lines.length < 2) return;
      const newHoldings: PortfolioHolding[] = [];
      for (let i = 1; i < lines.length; i++) {
        const cols = lines[i].split(",");
        if (cols.length < 5) continue;
        const code = cols[0].trim();
        const name = cols[1].trim();
        const shares = parseFloat(cols[2]);
        const avgCost = parseFloat(cols[3]);
        const currentPrice = parseFloat(cols[4]);
        if (!code || isNaN(shares) || isNaN(avgCost) || isNaN(currentPrice)) continue;
        const marketValue = shares * currentPrice;
        const pnl = (currentPrice - avgCost) * shares;
        const pnlPct = avgCost > 0 ? ((currentPrice - avgCost) / avgCost) * 100 : 0;
        newHoldings.push({ code, name, shares, avgCost, currentPrice, marketValue, pnl, pnlPct, weight: 0 });
      }
      if (newHoldings.length > 0) {
        const tv = newHoldings.reduce((s, x) => s + x.marketValue, 0);
        newHoldings.forEach(x => { x.weight = tv > 0 ? (x.marketValue / tv) * 100 : 0; });
        saveHoldings(newHoldings);
      }
      // Reset file input
      if (fileInputRef.current) fileInputRef.current.value = "";
    };
    reader.readAsText(file, "UTF-8");
  };

  return (
    <div style={{ padding: 16 }}>
      {/* Summary cards */}
      <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginBottom: 20 }}>
        <SummaryCard label="持仓总市值" value={`¥${totalValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}`} />
        <SummaryCard label="总盈亏" value={`¥${totalPnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}`} positive={totalPnl >= 0} />
        <SummaryCard label="收益率" value={`${totalPnlPct.toFixed(2)}%`} positive={totalPnl >= 0} />
        <SummaryCard label="持仓数量" value={`${holdings.length} 只`} />
      </div>

      {/* Weight distribution bar */}
      {holdings.length > 0 && (
        <div style={{ marginBottom: 20, padding: "12px 16px", background: "#121212", borderRadius: 8, border: "1px solid #2A2A2A" }}>
          <div style={{ color: "#CCAA00", fontSize: 13, fontFamily: "monospace", marginBottom: 10 }}>
            仓位分布
          </div>
          {holdings.map((h, i) => (
            <div key={i} style={{ marginBottom: 6, display: "flex", alignItems: "center", gap: 8 }}>
              <span style={{ color: "#D4D4D4", fontSize: 11, fontFamily: "monospace", width: 70 }}>
                {h.code}
              </span>
              <div style={{ flex: 1, background: "#0C0C0C", borderRadius: 4, height: 16, overflow: "hidden" }}>
                <div style={{
                  width: `${(h.weight / Math.max(maxWeight, 1)) * 100}%`, height: "100%",
                  background: h.weight > 30 ? "#EF5350" : h.weight > 15 ? "#CCAA00" : "#26A69A",
                  borderRadius: 4, opacity: 0.8, transition: "width 0.3s",
                }} />
              </div>
              <span style={{ color: h.weight > 30 ? "#EF5350" : "#858585", fontSize: 11, fontFamily: "monospace", width: 45, textAlign: "right" }}>
                {h.weight.toFixed(1)}%
              </span>
              <button onClick={() => removeHolding(i)} style={{
                background: "transparent", color: "#EF5350", border: "none",
                cursor: "pointer", fontSize: 11, padding: "0 4px",
              }}>✕</button>
            </div>
          ))}
          {/* Concentration warning */}
          {maxWeight > 30 && (
            <div style={{ marginTop: 8, padding: "4px 8px", background: "rgba(239,83,80,0.1)", borderRadius: 4, color: "#EF5350", fontSize: 10, fontFamily: "monospace" }}>
              ⚠ 单只股票仓位超过30%，存在集中风险，建议分散配置
            </div>
          )}
        </div>
      )}

      {/* Add holding button + CSV import/export */}
      <div style={{ display: "flex", gap: 8, marginBottom: 16, alignItems: "center" }}>
        {!showForm && (
          <button onClick={() => setShowForm(true)} style={{
            padding: "6px 16px", background: "#CCAA00", color: "#000",
            border: "none", borderRadius: 4, cursor: "pointer",
            fontFamily: "monospace", fontSize: 12, fontWeight: 600,
          }}>
            + 添加持仓
          </button>
        )}
        <input
          ref={fileInputRef}
          type="file"
          accept=".csv"
          onChange={handleImportCSV}
          style={{ display: "none" }}
        />
        <button onClick={() => fileInputRef.current?.click()} style={{
          padding: "6px 14px", background: "transparent", color: "#858585",
          border: "1px solid #2A2A2A", borderRadius: 4, cursor: "pointer",
          fontFamily: "monospace", fontSize: 11,
        }}>
          📥 导入CSV
        </button>
        {holdings.length > 0 && (
          <button onClick={handleExportCSV} style={{
            padding: "6px 14px", background: "transparent", color: "#858585",
            border: "1px solid #2A2A2A", borderRadius: 4, cursor: "pointer",
            fontFamily: "monospace", fontSize: 11,
          }}>
            📤 导出CSV
          </button>
        )}
      </div>
      {showForm && (
        <div style={{ marginBottom: 16, padding: 12, background: "#121212", borderRadius: 8, border: "1px solid #2A2A2A" }}>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 8, marginBottom: 8 }}>
            <div>
              <label style={labelStyle}>代码</label>
              <input value={formCode} onChange={e => setFormCode(e.target.value)} style={miniInput} placeholder="000001.SZ" />
            </div>
            <div>
              <label style={labelStyle}>名称</label>
              <input value={formName} onChange={e => setFormName(e.target.value)} style={miniInput} placeholder="平安银行" />
            </div>
            <div>
              <label style={labelStyle}>持仓(股)</label>
              <input type="number" value={formShares} onChange={e => setFormShares(e.target.value)} style={miniInput} placeholder="1000" />
            </div>
            <div>
              <label style={labelStyle}>成本价</label>
              <input type="number" step="0.01" value={formCost} onChange={e => setFormCost(e.target.value)} style={miniInput} placeholder="10.00" />
            </div>
            <div>
              <label style={labelStyle}>现价</label>
              <input type="number" step="0.01" value={formPrice} onChange={e => setFormPrice(e.target.value)} style={miniInput} placeholder="11.00" />
            </div>
            <div style={{ display: "flex", alignItems: "flex-end", gap: 4 }}>
              <button onClick={addHolding} style={actionBtn("#CCAA00")}>添加</button>
              <button onClick={resetForm} style={actionBtn("#2A2A2A")}>取消</button>
            </div>
          </div>
        </div>
      )}

      {/* Holdings table */}
      {holdings.length > 0 && (
        <div style={{ overflow: "auto", marginBottom: 20 }}>
          <table style={tableStyle}>
            <thead>
              <tr>
                <th style={thStyle}>代码</th><th style={thStyle}>名称</th>
                <th style={{ ...thStyle, textAlign: "right" }}>持仓(股)</th>
                <th style={{ ...thStyle, textAlign: "right" }}>成本</th>
                <th style={{ ...thStyle, textAlign: "right" }}>现价</th>
                <th style={{ ...thStyle, textAlign: "right" }}>市值</th>
                <th style={{ ...thStyle, textAlign: "right" }}>盈亏</th>
                <th style={{ ...thStyle, textAlign: "right" }}>盈亏%</th>
                <th style={{ ...thStyle, textAlign: "right" }}>占比</th>
              </tr>
            </thead>
            <tbody>
              {holdings.map((h, i) => (
                <tr key={i}>
                  <td style={tdStyle}>{h.code}</td><td style={tdStyle}>{h.name}</td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>{h.shares.toLocaleString()}</td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>{h.avgCost.toFixed(3)}</td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>{h.currentPrice.toFixed(3)}</td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>¥{h.marketValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                  <td style={{ ...tdStyle, textAlign: "right", color: h.pnl >= 0 ? "#26A69A" : "#EF5350" }}>
                    ¥{h.pnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                  </td>
                  <td style={{ ...tdStyle, textAlign: "right", color: h.pnlPct >= 0 ? "#26A69A" : "#EF5350" }}>
                    {h.pnlPct.toFixed(2)}%
                  </td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>{h.weight.toFixed(1)}%</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* What-if sandbox */}
      <div style={{ marginBottom: 16, padding: "12px 16px", background: "#121212", borderRadius: 8, border: "1px solid #2A2A2A" }}>
        <div style={{ color: "#CCAA00", fontSize: 13, fontFamily: "monospace", marginBottom: 8 }}>
          沙盘推演
        </div>
        <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace", marginBottom: 8 }}>
          输入计划买入的金额和价格，预览对组合的影响
        </div>
        <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
          <input type="number" value={simAmount} onChange={e => setSimAmount(e.target.value)}
            placeholder="计划买入金额" style={miniInput} />
          <input type="number" step="0.01" value={simPrice} onChange={e => setSimPrice(e.target.value)}
            placeholder="预期买入价格" style={miniInput} />
          <button onClick={runSim} style={actionBtn("#CCAA00")}>模拟</button>
        </div>
        {simResult && (
          <div style={{ marginTop: 8, padding: "6px 10px", background: "#0C0C0C", borderRadius: 4, color: "#858585", fontSize: 11, fontFamily: "monospace" }}>
            {simResult}
          </div>
        )}
      </div>

      {/* PRO Analysis Modules */}
      <ProAnalysisSection holdings={holdings} />
    </div>
  );
}

// ── PRO Analysis Section ──

interface StockIdMap {
  [code: string]: number;
}

function ProAnalysisSection({ holdings }: { holdings: PortfolioHolding[] }) {
  const [stockIds, setStockIds] = useState<StockIdMap>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (holdings.length === 0) return;
    let cancelled = false;
    (async () => {
      const map: StockIdMap = {};
      for (const h of holdings) {
        try {
          const stock: { id: number; code: string } | null = await invoke("query_stock_by_code", { code: h.code });
          if (stock && !cancelled) {
            map[h.code] = stock.id;
          }
        } catch { /* stock not imported yet */ }
      }
      if (!cancelled) setStockIds(map);
    })();
    return () => { cancelled = true; };
  }, [holdings]);

  const ids = holdings.map(h => stockIds[h.code]).filter(Boolean);
  const weights = holdings.map(h => h.marketValue);

  if (holdings.length === 0) {
    return (
      <div style={{ padding: 16, color: "#666666", fontSize: 12, fontFamily: "monospace", textAlign: "center" }}>
        添加持仓后可查看专业组合分析
      </div>
    );
  }

  if (ids.length === 0) {
    return (
      <div style={{ padding: 16, color: "#666666", fontSize: 12, fontFamily: "monospace", textAlign: "center" }}>
        请先导入对应股票数据以启用组合分析
      </div>
    );
  }

  return (
    <div style={{ display: "flex", gap: 16, flexWrap: "wrap" }}>
      <CorrelationPanel stockIds={ids} codes={holdings.map(h => h.code)} />
      <IndustryPanel stockIds={ids} weights={weights} codes={holdings.map(h => h.code)} holdings={holdings} stockIdMap={stockIds} />
      <VaRPanel stockIds={ids} weights={weights} />
      <AttributionPanel holdings={holdings} />
    </div>
  );
}

// ── 1. Correlation Matrix ──

function CorrelationPanel({ stockIds, codes }: { stockIds: number[]; codes: string[] }) {
  const [matrix, setMatrix] = useState<number[][] | null>(null);
  const [labels, setLabels] = useState<string[]>([]);

  useEffect(() => {
    if (stockIds.length < 2) return;
    let cancelled = false;
    (async () => {
      try {
        const result = await invoke<{ codes: string[]; matrix: number[][] }>("portfolio_correlation", {
          stockIds, days: 250,
        });
        if (!cancelled) {
          setLabels(result.codes);
          setMatrix(result.matrix);
        }
      } catch (e) {
        console.error("相关性计算失败:", e);
      }
    })();
    return () => { cancelled = true; };
  }, [stockIds]);

  if (!matrix || labels.length < 2) {
    return (
      <div style={proPanelStyle}>
        <div style={proTitleStyle}>相关性矩阵</div>
        <div style={{ color: "#666666", fontSize: 11, fontFamily: "monospace", textAlign: "center", padding: 20 }}>
          {stockIds.length < 2 ? "需要至少2只股票" : "计算中..."}
        </div>
      </div>
    );
  }

  const n = labels.length;
  return (
    <div style={proPanelStyle}>
      <div style={proTitleStyle}>相关性矩阵</div>
      <div style={{ color: "#858585", fontSize: 10, fontFamily: "monospace", marginBottom: 8 }}>
        基于250日收益率 Pearson r，红色=正相关，绿色=负相关
      </div>
      <div style={{ overflow: "auto", maxHeight: 280 }}>
        <table style={{ borderCollapse: "collapse", fontFamily: "monospace", fontSize: 11 }}>
          <thead>
            <tr>
              <th style={{ ...cmTh }}></th>
              {labels.map((l, i) => <th key={i} style={cmTh}>{l}</th>)}
            </tr>
          </thead>
          <tbody>
            {matrix.map((row, ri) => (
              <tr key={ri}>
                <td style={{ ...cmTh, color: "#CCAA00" }}>{labels[ri]}</td>
                {row.map((v, ci) => (
                  <td key={ci} style={{
                    padding: "4px 8px", textAlign: "center",
                    background: ri === ci ? "#0C0C0C" : `rgba(${v > 0 ? "239,83,80" : "38,166,154"},${Math.abs(v) * 0.3})`,
                    color: ri === ci ? "#858585" : v > 0.5 ? "#EF5350" : v < -0.3 ? "#26A69A" : "#D4D4D4",
                  }}>
                    {v.toFixed(2)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

const cmTh: React.CSSProperties = {
  padding: "4px 8px", color: "#858585", borderBottom: "1px solid #2A2A2A",
  textAlign: "center", position: "sticky", top: 0, background: "#121212",
};

// ── 2. Industry Concentration ──

function IndustryPanel({ stockIds, weights, codes, holdings, stockIdMap }: {
  stockIds: number[]; weights: number[]; codes: string[]; holdings: PortfolioHolding[];
  stockIdMap: StockIdMap;
}) {
  const [items, setItems] = useState<{ industry: string; stock_count: number; weight: number }[]>([]);
  const [editMode, setEditMode] = useState(false);

  const loadData = useCallback(async () => {
    if (stockIds.length === 0) return;
    try {
      const result = await invoke<{ industry: string; stock_count: number; weight: number }[]>(
        "portfolio_concentration", { stockIds, weights }
      );
      setItems(result);
    } catch (e) {
      console.error("行业集中度计算失败:", e);
    }
  }, [stockIds, weights]);

  useEffect(() => {
    setEditMode(false);
    loadData();
  }, [loadData]);

  const maxWeight = Math.max(...items.map(i => i.weight).concat([1]));

  return (
    <div style={proPanelStyle}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 6 }}>
        <div style={proTitleStyle}>行业集中度</div>
        <button onClick={() => setEditMode(!editMode)} style={{
          background: "transparent", color: "#CCAA00", border: "1px solid #2A2A2A",
          borderRadius: 3, cursor: "pointer", fontSize: 10, fontFamily: "monospace", padding: "2px 8px",
        }}>
          {editMode ? "完成" : "编辑行业"}
        </button>
      </div>
      {items.length === 0 ? (
        <div style={{ color: "#666666", fontSize: 11, fontFamily: "monospace", textAlign: "center", padding: 20 }}>
          请在编辑模式中为持仓设置行业分类
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {items.map((item, i) => (
            <div key={i} style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <span style={{ color: "#D4D4D4", fontSize: 11, fontFamily: "monospace", width: 70, flexShrink: 0 }}>
                {item.industry}
              </span>
              <div style={{ flex: 1, background: "#0C0C0C", borderRadius: 3, height: 14, overflow: "hidden" }}>
                <div style={{
                  width: `${(item.weight / Math.max(maxWeight, 1)) * 100}%`,
                  height: "100%",
                  background: item.weight > 40 ? "#EF5350" : item.weight > 20 ? "#CCAA00" : "#26A69A",
                  borderRadius: 3, transition: "width 0.3s",
                }} />
              </div>
              <span style={{
                color: item.weight > 40 ? "#EF5350" : "#858585",
                fontSize: 11, fontFamily: "monospace", width: 40, textAlign: "right",
              }}>
                {item.weight.toFixed(0)}%
              </span>
              <span style={{ color: "#666666", fontSize: 10, fontFamily: "monospace" }}>
                ({item.stock_count}只)
              </span>
            </div>
          ))}
        </div>
      )}
      {editMode && (
        <div style={{ marginTop: 10, borderTop: "1px solid #2A2A2A", paddingTop: 8 }}>
          {holdings.filter(h => stockIdMap[h.code]).map(h => {
            const sid = stockIdMap[h.code];
            return (
              <div key={h.code} style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 4 }}>
                <span style={{ color: "#CCAA00", fontSize: 10, fontFamily: "monospace", width: 60 }}>
                  {h.code}
                </span>
                <IndustryPicker sid={sid} code={h.code} onSet={loadData} />
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

const INDUSTRIES = [
  "银行", "保险", "证券", "白酒", "医药", "新能源", "半导体",
  "汽车", "家电", "房地产", "建筑", "钢铁", "煤炭", "有色",
  "化工", "机械", "军工", "食品饮料", "纺织服装", "传媒",
  "计算机", "通信", "电子", "电力", "交通运输", "农林牧渔",
  "商贸零售", "社会服务", "未分类",
];

function IndustryPicker({ sid, code, onSet }: { sid: number; code: string; onSet: () => void }) {
  const [selected, setSelected] = useState("");

  const handleSet = async (industry: string) => {
    try {
      await invoke("stock_set_industry", { stockId: sid, industry });
      setSelected(industry);
      onSet();
    } catch (e) {
      console.error("设置行业失败:", e);
    }
  };

  return (
    <select value={selected} onChange={e => handleSet(e.target.value)}
      style={{
        background: "#0C0C0C", color: "#D4D4D4", border: "1px solid #2A2A2A",
        borderRadius: 3, fontSize: 10, fontFamily: "monospace", padding: "2px 4px", width: 80,
      }}>
      <option value="">选择行业</option>
      {INDUSTRIES.map(ind => <option key={ind} value={ind}>{ind}</option>)}
    </select>
  );
}

// ── 3. VaR Panel ──

function VaRPanel({ stockIds, weights }: { stockIds: number[]; weights: number[] }) {
  const [data, setData] = useState<{
    var_95: number; var_99: number; cvar_95: number; cvar_99: number;
    daily_volatility: number; period_days: number;
  } | null>(null);

  useEffect(() => {
    if (stockIds.length === 0) return;
    let cancelled = false;
    (async () => {
      try {
        const result = await invoke<{
          var_95: number; var_99: number; cvar_95: number; cvar_99: number;
          daily_volatility: number; period_days: number;
        }>("portfolio_var", { stockIds, weights, days: 250 });
        if (!cancelled) setData(result);
      } catch (e) {
        console.error("VaR计算失败:", e);
      }
    })();
    return () => { cancelled = true; };
  }, [stockIds, weights]);

  if (!data) {
    return (
      <div style={proPanelStyle}>
        <div style={proTitleStyle}>VaR 风险价值</div>
        <div style={{ color: "#666666", fontSize: 11, fontFamily: "monospace", textAlign: "center", padding: 20 }}>
          计算中...
        </div>
      </div>
    );
  }

  const totalWeight = weights.reduce((s, v) => s + v, 0);

  return (
    <div style={proPanelStyle}>
      <div style={proTitleStyle}>VaR 风险价值</div>
      <div style={{ color: "#858585", fontSize: 10, fontFamily: "monospace", marginBottom: 10 }}>
        历史模拟法 · {data.period_days}个交易日 · 组合市值 ¥{totalWeight.toLocaleString(undefined, { maximumFractionDigits: 0 })}
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}>
        <VaRCard label="VaR 95%" value={data.var_95 * totalWeight} pct={data.var_95} />
        <VaRCard label="VaR 99%" value={data.var_99 * totalWeight} pct={data.var_99} />
        <VaRCard label="CVaR 95%" value={data.cvar_95 * totalWeight} pct={data.cvar_95} />
        <VaRCard label="CVaR 99%" value={data.cvar_99 * totalWeight} pct={data.cvar_99} />
      </div>
      <div style={{
        marginTop: 10, padding: "6px 10px", background: "#0C0C0C", borderRadius: 4,
        color: "#858585", fontSize: 10, fontFamily: "monospace",
      }}>
        日波动率: {(data.daily_volatility * 100).toFixed(2)}%
      </div>
    </div>
  );
}

function VaRCard({ label, value, pct }: { label: string; value: number; pct: number }) {
  return (
    <div style={{ padding: "8px 10px", background: "#0C0C0C", borderRadius: 4, border: "1px solid #1A1A1A" }}>
      <div style={{ color: "#858585", fontSize: 10, fontFamily: "monospace", marginBottom: 2 }}>{label}</div>
      <div style={{ color: "#EF5350", fontSize: 13, fontFamily: "monospace", fontWeight: 600 }}>
        ¥{Math.abs(value).toLocaleString(undefined, { maximumFractionDigits: 0 })}
      </div>
      <div style={{ color: "#666666", fontSize: 10, fontFamily: "monospace" }}>
        {(pct * 100).toFixed(2)}% 日损失
      </div>
    </div>
  );
}

// ── 4. Return Attribution ──

function AttributionPanel({ holdings }: { holdings: PortfolioHolding[] }) {
  const totalPnl = holdings.reduce((s, h) => s + h.pnl, 0);
  const totalValue = holdings.reduce((s, h) => s + h.marketValue, 0);

  const sorted = [...holdings].sort((a, b) => Math.abs(b.pnl) - Math.abs(a.pnl));

  return (
    <div style={proPanelStyle}>
      <div style={proTitleStyle}>收益归因</div>
      <div style={{ color: "#858585", fontSize: 10, fontFamily: "monospace", marginBottom: 10 }}>
        持仓盈亏贡献度分析
      </div>
      {sorted.length === 0 ? (
        <div style={{ color: "#666666", fontSize: 11, fontFamily: "monospace", textAlign: "center", padding: 20 }}>
          暂无持仓
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {sorted.map((h, i) => (
            <div key={i} style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <span style={{ color: "#CCAA00", fontSize: 11, fontFamily: "monospace", width: 60, flexShrink: 0 }}>
                {h.code}
              </span>
              <div style={{ flex: 1 }}>
                <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 2 }}>
                  <span style={{ color: h.pnl >= 0 ? "#26A69A" : "#EF5350", fontSize: 12, fontFamily: "monospace" }}>
                    ¥{h.pnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                  </span>
                  <span style={{ color: "#858585", fontSize: 10, fontFamily: "monospace" }}>
                    {totalPnl !== 0 ? ((h.pnl / totalPnl) * 100).toFixed(0) : "0"}%
                  </span>
                </div>
                <div style={{ background: "#0C0C0C", borderRadius: 3, height: 6, overflow: "hidden" }}>
                  <div style={{
                    height: "100%",
                    width: `${Math.min(Math.abs(h.pnl) / Math.max(Math.abs(totalPnl), 1) * 100, 100)}%`,
                    background: h.pnl >= 0 ? "#26A69A" : "#EF5350",
                    borderRadius: 3,
                  }} />
                </div>
              </div>
            </div>
          ))}
          <div style={{ borderTop: "1px solid #2A2A2A", paddingTop: 8, marginTop: 4 }}>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <span style={{ color: "#D4D4D4", fontSize: 11, fontFamily: "monospace", fontWeight: 600 }}>总计</span>
              <span style={{
                color: totalPnl >= 0 ? "#26A69A" : "#EF5350",
                fontSize: 13, fontFamily: "monospace", fontWeight: 600,
              }}>
                ¥{totalPnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}
              </span>
            </div>
            <div style={{ color: "#858585", fontSize: 10, fontFamily: "monospace", marginTop: 2 }}>
              最大贡献: {sorted.length > 0 ? sorted[0].code : "—"} ·
              最大拖累: {sorted.length > 0 ? sorted[sorted.length - 1].code : "—"}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

const proPanelStyle: React.CSSProperties = {
  flex: "1 1 calc(50% - 8px)",
  minWidth: 280,
  padding: 12,
  background: "#121212",
  border: "1px solid #2A2A2A",
  borderRadius: 6,
};

const proTitleStyle: React.CSSProperties = {
  color: "#CCAA00", fontSize: 13, fontFamily: "monospace",
  fontWeight: 600, marginBottom: 6,
};

const labelStyle: React.CSSProperties = {
  fontSize: 10, color: "#858585", fontFamily: "monospace", display: "block", marginBottom: 2,
};

const miniInput: React.CSSProperties = {
  background: "#0C0C0C", border: "1px solid #2A2A2A", color: "#fff",
  padding: "4px 8px", borderRadius: 4, fontSize: 12,
  fontFamily: "monospace", outline: "none", width: "100%", boxSizing: "border-box",
};

function actionBtn(bg: string): React.CSSProperties {
  return {
    padding: "4px 12px", background: bg, color: bg === "#CCAA00" ? "#000" : "#fff",
    border: "none", borderRadius: 4, cursor: "pointer", fontFamily: "monospace", fontSize: 12,
  };
}

function SummaryCard({
  label,
  value,
  positive,
}: {
  label: string;
  value: string;
  positive?: boolean;
}) {
  return (
    <div
      style={{
        padding: "12px 20px",
        background: "#121212",
        border: "1px solid #2A2A2A",
        borderRadius: 8,
        minWidth: 140,
      }}
    >
      <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace", marginBottom: 4 }}>
        {label}
      </div>
      <div
        style={{
          color:
            positive === undefined
              ? "#D4D4D4"
              : positive
                ? "#26A69A"
                : "#EF5350",
          fontSize: 16,
          fontFamily: "monospace",
          fontWeight: 600,
        }}
      >
        {value}
      </div>
    </div>
  );
}

const tableStyle: React.CSSProperties = {
  width: "100%",
  borderCollapse: "collapse",
  fontFamily: "monospace",
  fontSize: 12,
};

const thStyle: React.CSSProperties = {
  padding: "6px 12px",
  color: "#858585",
  borderBottom: "1px solid #2A2A2A",
  textAlign: "left",
  position: "sticky",
  top: 0,
  background: "#0C0C0C",
};

const tdStyle: React.CSSProperties = {
  padding: "5px 12px",
  color: "#D4D4D4",
  borderBottom: "1px solid #1A1A1A",
};

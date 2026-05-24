import { useState } from "react";
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
          background: "#111827",
          borderBottom: "1px solid #1E293B",
          display: "flex",
          alignItems: "center",
          gap: 16,
          flexShrink: 0,
        }}
      >
        <h2 style={{ color: "#00D8FF", fontSize: 16, fontFamily: "monospace", margin: 0 }}>
          组合与风险
        </h2>
        {selectedStockCode && (
          <span style={{ color: "#94A3B8", fontSize: 12, fontFamily: "monospace" }}>
            当前标的: {selectedStockCode}
          </span>
        )}
      </div>

      {/* Section nav */}
      <div
        style={{
          padding: "4px 20px",
          background: "#141b2d",
          borderBottom: "1px solid #1E293B",
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
              background: activeSection === sec ? "#00D8FF" : "transparent",
              color: activeSection === sec ? "#000" : "#94A3B8",
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
        color: "#64748B",
        fontFamily: "monospace",
        fontSize: 14,
        padding: 60,
      }}
    >
      <div style={{ fontSize: 48, marginBottom: 16, color: "#1E293B" }}>📊</div>
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
        <div style={{ marginBottom: 20, padding: "12px 16px", background: "#141b2d", borderRadius: 8, border: "1px solid #1E293B" }}>
          <div style={{ color: "#00D8FF", fontSize: 13, fontFamily: "monospace", marginBottom: 10 }}>
            仓位分布
          </div>
          {holdings.map((h, i) => (
            <div key={i} style={{ marginBottom: 6, display: "flex", alignItems: "center", gap: 8 }}>
              <span style={{ color: "#F1F5F9", fontSize: 11, fontFamily: "monospace", width: 70 }}>
                {h.code}
              </span>
              <div style={{ flex: 1, background: "#0A0E1A", borderRadius: 4, height: 16, overflow: "hidden" }}>
                <div style={{
                  width: `${(h.weight / Math.max(maxWeight, 1)) * 100}%`, height: "100%",
                  background: h.weight > 30 ? "#FF2A7A" : h.weight > 15 ? "#00D8FF" : "#00E676",
                  borderRadius: 4, opacity: 0.8, transition: "width 0.3s",
                }} />
              </div>
              <span style={{ color: h.weight > 30 ? "#FF2A7A" : "#94A3B8", fontSize: 11, fontFamily: "monospace", width: 45, textAlign: "right" }}>
                {h.weight.toFixed(1)}%
              </span>
              <button onClick={() => removeHolding(i)} style={{
                background: "transparent", color: "#FF2A7A", border: "none",
                cursor: "pointer", fontSize: 11, padding: "0 4px",
              }}>✕</button>
            </div>
          ))}
          {/* Concentration warning */}
          {maxWeight > 30 && (
            <div style={{ marginTop: 8, padding: "4px 8px", background: "rgba(239,68,68,0.1)", borderRadius: 4, color: "#FF2A7A", fontSize: 10, fontFamily: "monospace" }}>
              ⚠ 单只股票仓位超过30%，存在集中风险，建议分散配置
            </div>
          )}
        </div>
      )}

      {/* Add holding button */}
      {!showForm ? (
        <button onClick={() => setShowForm(true)} style={{
          padding: "6px 16px", background: "#00D8FF", color: "#000",
          border: "none", borderRadius: 4, cursor: "pointer",
          fontFamily: "monospace", fontSize: 12, fontWeight: 600, marginBottom: 16,
        }}>
          + 添加持仓
        </button>
      ) : (
        <div style={{ marginBottom: 16, padding: 12, background: "#141b2d", borderRadius: 8, border: "1px solid #1E293B" }}>
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
              <button onClick={addHolding} style={actionBtn("#00D8FF")}>添加</button>
              <button onClick={resetForm} style={actionBtn("#1E293B")}>取消</button>
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
                  <td style={{ ...tdStyle, textAlign: "right", color: h.pnl >= 0 ? "#00E676" : "#FF2A7A" }}>
                    ¥{h.pnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                  </td>
                  <td style={{ ...tdStyle, textAlign: "right", color: h.pnlPct >= 0 ? "#00E676" : "#FF2A7A" }}>
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
      <div style={{ marginBottom: 16, padding: "12px 16px", background: "#141b2d", borderRadius: 8, border: "1px solid #1E293B" }}>
        <div style={{ color: "#00D8FF", fontSize: 13, fontFamily: "monospace", marginBottom: 8 }}>
          沙盘推演
        </div>
        <div style={{ color: "#94A3B8", fontSize: 11, fontFamily: "monospace", marginBottom: 8 }}>
          输入计划买入的金额和价格，预览对组合的影响
        </div>
        <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
          <input type="number" value={simAmount} onChange={e => setSimAmount(e.target.value)}
            placeholder="计划买入金额" style={miniInput} />
          <input type="number" step="0.01" value={simPrice} onChange={e => setSimPrice(e.target.value)}
            placeholder="预期买入价格" style={miniInput} />
          <button onClick={runSim} style={actionBtn("#00D8FF")}>模拟</button>
        </div>
        {simResult && (
          <div style={{ marginTop: 8, padding: "6px 10px", background: "#0A0E1A", borderRadius: 4, color: "#94A3B8", fontSize: 11, fontFamily: "monospace" }}>
            {simResult}
          </div>
        )}
      </div>

      {/* Pro feature placeholders */}
      <div style={{ display: "flex", gap: 16, flexWrap: "wrap" }}>
        <ProPlaceholder title="相关性矩阵" desc="跨持仓相关性分析，识别集中风险" />
        <ProPlaceholder title="行业集中度" desc="按行业维度分析持仓分布" />
        <ProPlaceholder title="VaR 风险价值" desc="95%/99%置信度VaR和CVaR估算" />
        <ProPlaceholder title="收益归因" desc="按策略/行业/时段的盈亏归因分析" />
      </div>
    </div>
  );
}

const labelStyle: React.CSSProperties = {
  fontSize: 10, color: "#94A3B8", fontFamily: "monospace", display: "block", marginBottom: 2,
};

const miniInput: React.CSSProperties = {
  background: "#0A0E1A", border: "1px solid #1E293B", color: "#fff",
  padding: "4px 8px", borderRadius: 4, fontSize: 12,
  fontFamily: "monospace", outline: "none", width: "100%", boxSizing: "border-box",
};

function actionBtn(bg: string): React.CSSProperties {
  return {
    padding: "4px 12px", background: bg, color: bg === "#00D8FF" ? "#000" : "#fff",
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
        background: "#141b2d",
        border: "1px solid #1E293B",
        borderRadius: 8,
        minWidth: 140,
      }}
    >
      <div style={{ color: "#94A3B8", fontSize: 11, fontFamily: "monospace", marginBottom: 4 }}>
        {label}
      </div>
      <div
        style={{
          color:
            positive === undefined
              ? "#F1F5F9"
              : positive
                ? "#00E676"
                : "#FF2A7A",
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

function ProPlaceholder({ title, desc }: { title: string; desc: string }) {
  return (
    <div
      title="功能开发中，敬请期待"
      onClick={() => alert(`${title}为专业版功能，正在开发中，敬请期待。`)}
      style={{
        flex: "1 1 200px",
        padding: 16,
        background: "#141b2d",
        border: "1px solid #1E293B",
        borderRadius: 8,
        opacity: 0.6,
        cursor: "pointer",
        transition: "border-color 0.2s",
      }}
    >
      <div style={{ color: "#00D8FF", fontSize: 13, fontFamily: "monospace", marginBottom: 4 }}>
        {title}
      </div>
      <div style={{ color: "#64748B", fontSize: 11, fontFamily: "monospace", marginBottom: 8 }}>
        {desc}
      </div>
      <span
        style={{
          padding: "2px 8px",
          background: "#3a2a0a",
          color: "#00D8FF",
          borderRadius: 4,
          fontSize: 10,
          fontFamily: "monospace",
        }}
      >
        专业版
      </span>
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
  color: "#94A3B8",
  borderBottom: "1px solid #1E293B",
  textAlign: "left",
  position: "sticky",
  top: 0,
  background: "#0A0E1A",
};

const tdStyle: React.CSSProperties = {
  padding: "5px 12px",
  color: "#F1F5F9",
  borderBottom: "1px solid #1a1a3e",
};

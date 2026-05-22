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
          background: "#16213e",
          borderBottom: "1px solid #2a2a4a",
          display: "flex",
          alignItems: "center",
          gap: 16,
          flexShrink: 0,
        }}
      >
        <h2 style={{ color: "#fbbf24", fontSize: 16, fontFamily: "monospace", margin: 0 }}>
          组合与风险
        </h2>
        {selectedStockCode && (
          <span style={{ color: "#888", fontSize: 12, fontFamily: "monospace" }}>
            当前标的: {selectedStockCode}
          </span>
        )}
      </div>

      {/* Section nav */}
      <div
        style={{
          padding: "4px 20px",
          background: "#1a1a2e",
          borderBottom: "1px solid #2a2a4a",
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
              background: activeSection === sec ? "#fbbf24" : "transparent",
              color: activeSection === sec ? "#000" : "#888",
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
        color: "#666",
        fontFamily: "monospace",
        fontSize: 14,
        padding: 60,
      }}
    >
      <div style={{ fontSize: 48, marginBottom: 16, color: "#3a3a5a" }}>📊</div>
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
    try {
      return JSON.parse(localStorage.getItem("me-portfolio") || "[]");
    } catch {
      return [];
    }
  });

  const totalValue = holdings.reduce((s, h) => s + h.marketValue, 0);
  const totalPnl = holdings.reduce((s, h) => s + h.pnl, 0);
  const totalCost = holdings.reduce((s, h) => s + h.avgCost * h.shares, 0);
  const totalPnlPct = totalCost > 0 ? (totalPnl / totalCost) * 100 : 0;

  return (
    <div style={{ padding: 16 }}>
      {/* Summary cards */}
      <div
        style={{
          display: "flex",
          gap: 16,
          flexWrap: "wrap",
          marginBottom: 20,
        }}
      >
        <SummaryCard label="持仓总市值" value={`¥${totalValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}`} />
        <SummaryCard label="总盈亏" value={`¥${totalPnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}`} positive={totalPnl >= 0} />
        <SummaryCard label="收益率" value={`${totalPnlPct.toFixed(2)}%`} positive={totalPnl >= 0} />
        <SummaryCard label="持仓数量" value={`${holdings.length} 只`} />
      </div>

      {/* Holdings table */}
      {holdings.length > 0 ? (
        <div style={{ overflow: "auto" }}>
          <table style={tableStyle}>
            <thead>
              <tr>
                <th style={thStyle}>代码</th>
                <th style={thStyle}>名称</th>
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
                  <td style={tdStyle}>{h.code}</td>
                  <td style={tdStyle}>{h.name}</td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>{h.shares.toLocaleString()}</td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>{h.avgCost.toFixed(3)}</td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>{h.currentPrice.toFixed(3)}</td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>¥{h.marketValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                  <td style={{ ...tdStyle, textAlign: "right", color: h.pnl >= 0 ? "#22c55e" : "#ef4444" }}>
                    ¥{h.pnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                  </td>
                  <td style={{ ...tdStyle, textAlign: "right", color: h.pnlPct >= 0 ? "#22c55e" : "#ef4444" }}>
                    {h.pnlPct.toFixed(2)}%
                  </td>
                  <td style={{ ...tdStyle, textAlign: "right" }}>{h.weight.toFixed(1)}%</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <EmptyState message="暂无持仓记录，请在交易复盘页面记录交易" />
      )}

      {/* Pro feature placeholders */}
      <div style={{ marginTop: 24, display: "flex", gap: 16, flexWrap: "wrap" }}>
        <ProPlaceholder title="相关性矩阵" desc="跨持仓相关性分析，识别集中风险" />
        <ProPlaceholder title="行业集中度" desc="按行业维度分析持仓分布" />
        <ProPlaceholder title="VaR 风险价值" desc="95%/99%置信度VaR和CVaR估算" />
        <ProPlaceholder title="沙盘推演" desc="'假如我在X日以Y价买入...' 情景模拟" />
      </div>
    </div>
  );
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
        background: "#1a1a2e",
        border: "1px solid #2a2a4a",
        borderRadius: 8,
        minWidth: 140,
      }}
    >
      <div style={{ color: "#888", fontSize: 11, fontFamily: "monospace", marginBottom: 4 }}>
        {label}
      </div>
      <div
        style={{
          color:
            positive === undefined
              ? "#ccc"
              : positive
                ? "#22c55e"
                : "#ef4444",
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
      style={{
        flex: "1 1 200px",
        padding: 16,
        background: "#1a1a2e",
        border: "1px solid #2a2a4a",
        borderRadius: 8,
        opacity: 0.6,
      }}
    >
      <div style={{ color: "#fbbf24", fontSize: 13, fontFamily: "monospace", marginBottom: 4 }}>
        {title}
      </div>
      <div style={{ color: "#666", fontSize: 11, fontFamily: "monospace", marginBottom: 8 }}>
        {desc}
      </div>
      <span
        style={{
          padding: "2px 8px",
          background: "#3a2a0a",
          color: "#fbbf24",
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
  color: "#888",
  borderBottom: "1px solid #2a2a4a",
  textAlign: "left",
  position: "sticky",
  top: 0,
  background: "#0f0f23",
};

const tdStyle: React.CSSProperties = {
  padding: "5px 12px",
  color: "#ccc",
  borderBottom: "1px solid #1a1a3e",
};

import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/appStore";

interface LicenseStatus {
  valid: boolean;
  tier: string;
  expiry: string | null;
  trial_days_left: number | null;
}

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

export default function DashboardPage() {
  const navigateTo = useAppStore((s) => s.navigate);
  const [license, setLicense] = useState<LicenseStatus | null>(null);

  const loadLicense = useCallback(async () => {
    try {
      const s = await invoke<LicenseStatus>("check_license");
      setLicense(s);
    } catch {
      // license check unavailable
    }
  }, []);

  useEffect(() => {
    loadLicense();
  }, [loadLicense]);

  const handleEnterChart = () => {
    navigateTo("/chart");
  };

  return (
    <div
      style={{
        flex: 1,
        padding: 32,
        overflow: "auto",
        fontFamily: "monospace",
        color: "#ccc",
      }}
    >
      <h1 style={{ color: "#fbbf24", marginBottom: 8, fontSize: 24 }}>
        MoneyEarning
      </h1>
      <p style={{ color: "#888", marginBottom: 32, fontSize: 13 }}>
        本地量化分析工作站 v0.6.0
      </p>

      {/* Quick actions */}
      <div style={{ display: "flex", gap: 16, marginBottom: 32, flexWrap: "wrap" }}>
        <QuickCard
          icon="📈"
          title="图表分析"
          desc="K线图、技术指标、绘图工具"
          onClick={() => handleEnterChart()}
        />
        <QuickCard
          icon="⚡"
          title="策略回测"
          desc="20+策略模板，完整回测报告"
          onClick={() => navigateTo("/backtest")}
        />
        <QuickCard
          icon="🔍"
          title="股票扫描"
          desc="CAPS/CGPC/MARS 智能选股"
          onClick={() => navigateTo("/scanner")}
        />
        <QuickCard
          icon="📊"
          title="组合管理"
          desc="持仓跟踪、收益分析"
          onClick={() => navigateTo("/portfolio")}
        />
        <QuickCard
          icon="📝"
          title="交易复盘"
          desc="交易记录、复盘模板、情绪标签"
          onClick={() => navigateTo("/review")}
        />
      </div>

      {/* License status */}
      {license && (
        <div
          style={{
            background: "#16213e",
            borderRadius: 8,
            padding: "16px 20px",
            marginBottom: 24,
            display: "flex",
            alignItems: "center",
            gap: 16,
            border: "1px solid #2a2a4a",
          }}
        >
          <span style={{ fontSize: 24 }}>
            {license.tier === "pro" ? "⭐" : "🆓"}
          </span>
          <div>
            <div style={{ color: "#fbbf24", fontWeight: 600 }}>
              {tierLabel(license.tier)}
            </div>
            <div style={{ color: "#888", fontSize: 12 }}>
              {license.expiry
                ? `有效期至 ${license.expiry}`
                : license.trial_days_left != null
                  ? `剩余试用 ${license.trial_days_left} 天`
                  : "未激活"}
            </div>
          </div>
        </div>
      )}

      {/* Getting started */}
      <div
        style={{
          background: "#1a1a2e",
          borderRadius: 8,
          padding: "20px 24px",
          border: "1px solid #2a2a4a",
        }}
      >
        <h3 style={{ color: "#aaa", marginBottom: 16, fontSize: 14 }}>
          快速开始
        </h3>
        <ol
          style={{
            color: "#888",
            fontSize: 13,
            lineHeight: 2,
            paddingLeft: 20,
          }}
        >
          <li>导入股票数据 — 点击&ldquo;导入数据&rdquo;按钮，支持 CSV 格式</li>
          <li>选择股票 — 在图表页面左侧面板浏览和搜索股票</li>
          <li>添加技术指标 — 从指标选择器添加 MA、MACD、RSI 等</li>
          <li>运行策略回测 — 选择策略模板，配置参数，查看回测结果</li>
          <li>使用扫描器 — 设定条件，批量筛选符合条件的股票</li>
        </ol>
      </div>
    </div>
  );
}

function QuickCard({
  icon,
  title,
  desc,
  onClick,
}: {
  icon: string;
  title: string;
  desc: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        width: 200,
        padding: "16px",
        background: "#16213e",
        border: "1px solid #2a2a4a",
        borderRadius: 8,
        cursor: "pointer",
        textAlign: "left",
        fontFamily: "monospace",
        color: "#ccc",
      }}
    >
      <div style={{ fontSize: 28, marginBottom: 8 }}>{icon}</div>
      <div style={{ fontWeight: 600, marginBottom: 4, fontSize: 14 }}>{title}</div>
      <div style={{ color: "#888", fontSize: 12 }}>{desc}</div>
    </button>
  );
}

import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/appStore";

interface LicenseStatus {
  valid: boolean;
  tier: string;
  expiry: string | null;
  trial_days_left: number | null;
}

interface DataSummary {
  total_stocks: number;
  total_rows: number;
  db_size_mb: number;
}

interface PnLSummary {
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
  win_rate: number;
  total_pnl: number;
  avg_win: number;
  avg_loss: number;
  max_win: number;
  max_loss: number;
  profit_factor: number;
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
  const setLicenseTier = useAppStore((s) => s.setLicenseTier);
  const [license, setLicense] = useState<LicenseStatus | null>(null);
  const [dataSummary, setDataSummary] = useState<DataSummary | null>(null);
  const [pnl, setPnl] = useState<PnLSummary | null>(null);

  const loadLicense = useCallback(async () => {
    try {
      const s = await invoke<LicenseStatus>("check_license");
      setLicense(s);
      setLicenseTier(s.tier);
    } catch {
      // license check unavailable
    }
  }, [setLicenseTier]);

  const loadStats = useCallback(async () => {
    try {
      const [ds, p] = await Promise.all([
        invoke<DataSummary>("get_data_summary"),
        invoke<PnLSummary>("trade_pnl", { stockId: null }),
      ]);
      setDataSummary(ds);
      setPnl(p);
    } catch {
      // stats unavailable
    }
  }, []);

  useEffect(() => {
    loadLicense();
    loadStats();
  }, [loadLicense, loadStats]);

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
      {/* Hero banner */}
      <div
        style={{
          background: "linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #1a1a2e 100%)",
          borderRadius: 12,
          padding: "28px 32px",
          marginBottom: 28,
          border: "1px solid #2a2a4a",
        }}
      >
        <h1 style={{ color: "#fbbf24", marginBottom: 8, fontSize: 22, fontWeight: 700 }}>
          QuantVault
        </h1>
        <p style={{ color: "#aaa", marginBottom: 4, fontSize: 14, fontWeight: 600 }}>
          先回测，再实盘 — 用策略代替感觉，用纪律战胜情绪
        </p>
        <p style={{ color: "#888", fontSize: 12, lineHeight: 1.8 }}>
          70% 的散户在亏钱，头号原因不是行情不好，而是缺乏交易系统。
          回测验证策略 → 扫描锁定标的 → 纪律执行交易 → 复盘持续改进，一站式闭环。
        </p>
      </div>

      {/* Quick actions — ordered: backtest first */}
      <div style={{ display: "flex", gap: 16, marginBottom: 28, flexWrap: "wrap" }}>
        <QuickCard
          icon="⚡"
          title="策略回测"
          desc="20+策略模板，完整回测报告，参数优化"
          primary
          onClick={() => navigateTo("/backtest")}
        />
        <QuickCard
          icon="🔍"
          title="股票扫描"
          desc="CAPS/CGPC/MARS 智能选股，锁定标的"
          onClick={() => navigateTo("/scanner")}
        />
        <QuickCard
          icon="📈"
          title="图表分析"
          desc="K线图、316+技术指标、绘图工具"
          onClick={() => navigateTo("/chart")}
        />
        <QuickCard
          icon="📝"
          title="交易复盘"
          desc="情绪标签、复盘模板，建立交易纪律"
          onClick={() => navigateTo("/review")}
        />
        <QuickCard
          icon="📊"
          title="组合管理"
          desc="持仓跟踪、VaR风险、收益归因分析"
          onClick={() => navigateTo("/portfolio")}
        />
      </div>

      {/* License status */}
      {license && (
        <div
          style={{
            background: "#16213e",
            borderRadius: 8,
            padding: "14px 18px",
            marginBottom: 24,
            display: "flex",
            alignItems: "center",
            gap: 14,
            border: "1px solid #2a2a4a",
          }}
        >
          <span style={{ fontSize: 22 }}>
            {license.tier === "pro" ? "⭐" : "🆓"}
          </span>
          <div>
            <div style={{ color: "#fbbf24", fontWeight: 600, fontSize: 13 }}>
              {tierLabel(license.tier)}
            </div>
            <div style={{ color: "#888", fontSize: 11 }}>
              {license.expiry
                ? `有效期至 ${license.expiry}`
                : license.trial_days_left != null
                  ? `剩余试用 ${license.trial_days_left} 天`
                  : "未激活"}
            </div>
          </div>
        </div>
      )}

      {/* Live data stats */}
      {dataSummary && (
        <div
          style={{
            display: "flex",
            gap: 16,
            flexWrap: "wrap",
            marginBottom: 28,
          }}
        >
          <StatBox
            label="数据库股票数"
            value={String(dataSummary.total_stocks)}
            sub={`${(dataSummary.total_rows / 10000).toFixed(1)}万条日线`}
            color="#fbbf24"
          />
          <StatBox
            label="数据库大小"
            value={
              dataSummary.db_size_mb < 1
                ? `${(dataSummary.db_size_mb * 1024).toFixed(0)} KB`
                : `${dataSummary.db_size_mb.toFixed(1)} MB`
            }
            color="#60a5fa"
          />
          {pnl && (
            <>
              <StatBox
                label="交易总笔数"
                value={String(pnl.total_trades)}
                sub={
                  pnl.total_trades > 0
                    ? `胜率 ${(pnl.win_rate * 100).toFixed(0)}%`
                    : undefined
                }
                color="#c084fc"
              />
              <StatBox
                label="总盈亏"
                value={`¥${pnl.total_pnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}`}
                sub={
                  pnl.total_trades > 0
                    ? `盈亏比 ${pnl.profit_factor.toFixed(2)}`
                    : undefined
                }
                color={pnl.total_pnl >= 0 ? "#22c55e" : "#ef4444"}
              />
            </>
          )}
        </div>
      )}

      {/* Workflow: 建立你的交易系统 */}
      <div
        style={{
          background: "#1a1a2e",
          borderRadius: 8,
          padding: "20px 24px",
          border: "1px solid #2a2a4a",
        }}
      >
        <h3 style={{ color: "#aaa", marginBottom: 8, fontSize: 14 }}>
          建立你的交易系统
        </h3>
        <p style={{ color: "#666", fontSize: 11, marginBottom: 16 }}>
          专业交易员和散户之间最大的区别不是信息差，而是有没有一套可执行、可验证、
          可复盘的交易系统。以下四步帮你从零搭建。
        </p>
        <div style={{ display: "flex", gap: 16, flexWrap: "wrap" }}>
          <WorkflowStep
            num="1"
            title="回测验证"
            desc="把你的交易想法放到历史数据里跑一遍。夏普比率、最大回撤、胜率、盈亏比——用数字代替感觉做决策。"
          />
          <WorkflowStep
            num="2"
            title="扫描选股"
            desc="设定条件（趋势、动量、形态、筹码），全市场批量筛选，找到真正符合你策略的标的，不再靠消息炒股。"
          />
          <WorkflowStep
            num="3"
            title="纪律执行"
            desc="制定买入/持仓/止损规则，图表分析辅助确认入场点。明确盈亏比，赚该赚的钱，亏可控的钱。"
          />
          <WorkflowStep
            num="4"
            title="复盘改进"
            desc="记录每笔交易的情绪和逻辑，定期复盘归因。不是赚了就对、亏了就错——找到真正有效的模式。"
          />
        </div>
      </div>
    </div>
  );
}

function QuickCard({
  icon,
  title,
  desc,
  onClick,
  primary,
}: {
  icon: string;
  title: string;
  desc: string;
  onClick: () => void;
  primary?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        width: 200,
        padding: "16px",
        background: primary ? "#1a1a2e" : "#16213e",
        border: primary ? "1px solid rgba(251,191,36,0.3)" : "1px solid #2a2a4a",
        borderRadius: 8,
        cursor: "pointer",
        textAlign: "left",
        fontFamily: "monospace",
        color: "#ccc",
      }}
    >
      <div style={{ fontSize: 28, marginBottom: 8 }}>{icon}</div>
      <div
        style={{
          fontWeight: 600,
          marginBottom: 4,
          fontSize: 14,
          color: primary ? "#fbbf24" : "#ccc",
        }}
      >
        {title}
      </div>
      <div style={{ color: "#888", fontSize: 12 }}>{desc}</div>
    </button>
  );
}

function WorkflowStep({
  num,
  title,
  desc,
}: {
  num: string;
  title: string;
  desc: string;
}) {
  return (
    <div
      style={{
        flex: "1 1 220px",
        minWidth: 200,
        background: "#16213e",
        borderRadius: 8,
        padding: "16px",
        border: "1px solid #2a2a4a",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          marginBottom: 8,
        }}
      >
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            width: 24,
            height: 24,
            borderRadius: 6,
            background: "rgba(251,191,36,0.15)",
            color: "#fbbf24",
            fontSize: 12,
            fontWeight: 700,
            fontFamily: "monospace",
          }}
        >
          {num}
        </span>
        <span style={{ fontWeight: 600, fontSize: 13, color: "#ddd" }}>
          {title}
        </span>
      </div>
      <p style={{ color: "#888", fontSize: 11, lineHeight: 1.7 }}>{desc}</p>
    </div>
  );
}

function StatBox({
  label,
  value,
  sub,
  color,
}: {
  label: string;
  value: string;
  sub?: string;
  color: string;
}) {
  return (
    <div
      style={{
        flex: "1 1 160px",
        minWidth: 140,
        padding: "14px 16px",
        background: "#16213e",
        borderRadius: 8,
        border: "1px solid #2a2a4a",
      }}
    >
      <div style={{ color: "#888", fontSize: 11, fontFamily: "monospace", marginBottom: 4 }}>
        {label}
      </div>
      <div
        style={{
          color,
          fontSize: 18,
          fontFamily: "monospace",
          fontWeight: 700,
          marginBottom: sub ? 4 : 0,
        }}
      >
        {value}
      </div>
      {sub && (
        <div style={{ color: "#666", fontSize: 11, fontFamily: "monospace" }}>
          {sub}
        </div>
      )}
    </div>
  );
}

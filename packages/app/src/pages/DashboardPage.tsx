import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore, type LicenseStatus } from "../stores/appStore";
import {
  IconBacktest,
  IconScanner,
  IconChart,
  IconReview,
  IconPortfolio,
  IconTrendUp,
  IconDatabase,
  IconShield,
  IconTarget,
  IconDollar,
  IconPercent,
} from "../components/icons";

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
    case "pro":  return "专业版";
    case "trial": return "试用版";
    default:      return "免费版";
  }
}

export default function DashboardPage() {
  const navigateTo = useAppStore((s) => s.navigate);
  const refreshLicense = useAppStore((s) => s.refreshLicense);
  const [license, setLicense] = useState<LicenseStatus | null>(null);
  const [dataSummary, setDataSummary] = useState<DataSummary | null>(null);
  const [pnl, setPnl] = useState<PnLSummary | null>(null);

  const loadLicense = useCallback(async () => {
    try { const s = await refreshLicense(); setLicense(s); } catch {}
  }, [refreshLicense]);

  const loadStats = useCallback(async () => {
    try {
      const [ds, p] = await Promise.all([
        invoke<DataSummary>("get_data_summary"),
        invoke<PnLSummary>("trade_pnl", { stockId: null }),
      ]);
      setDataSummary(ds); setPnl(p);
    } catch {}
  }, []);

  useEffect(() => { loadLicense(); loadStats(); }, [loadLicense, loadStats]);

  return (
    <div className="panel-enter" style={{ padding: "28px 32px", maxWidth: 1100 }}>
      {/* Hero */}
      <section style={{ marginBottom: 28 }}>
        <div
          style={{
            background: "linear-gradient(135deg, rgba(56,189,248,0.04) 0%, rgba(212,160,23,0.03) 50%, rgba(56,189,248,0.02) 100%)",
            border: "1px solid var(--border-subtle)",
            borderRadius: "var(--radius-xl)",
            padding: "28px 32px",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 8 }}>
            <svg width={22} height={22} viewBox="0 0 24 24" fill="none">
              <rect x="3" y="1" width="18" height="22" rx="3" stroke="var(--accent)" strokeWidth={1.8} />
              <path d="M7 8h10M7 12h10M7 16h6" stroke="var(--accent)" strokeWidth={1.8} strokeLinecap="round" />
              <circle cx="17" cy="17" r="2" fill="var(--accent)" />
            </svg>
            <h1 style={{ fontSize: 20, fontWeight: 600, color: "var(--accent)", letterSpacing: "-0.01em" }}>
              QuantVault
            </h1>
            <span className="badge badge-accent" style={{ marginLeft: 4 }}>v0.12.1</span>
          </div>
          <p style={{ color: "var(--text-secondary)", fontSize: 13, marginBottom: 4, lineHeight: 1.6 }}>
            本地离线量化分析工作站 — 回测验证策略，扫描锁定标的，纪律执行交易，复盘持续改进
          </p>
          <p style={{ color: "var(--text-muted)", fontSize: 11, lineHeight: 1.8, maxWidth: 680 }}>
            70% 的散户在亏钱，头号原因不是行情，而是缺乏交易系统。
            一套可执行、可验证、可复盘的系统性方法，比任何"消息"和"感觉"都可靠。
          </p>
        </div>
      </section>

      {/* Quick Actions */}
      <section style={{ marginBottom: 28 }}>
        <h3 style={{ fontSize: 11, fontWeight: 600, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.08em", marginBottom: 12 }}>
          快速入口
        </h3>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))", gap: 10 }}>
          <QuickCard
            Icon={IconBacktest}
            title="策略回测"
            desc="20+ 模板，完整回测报告"
            primary
            onClick={() => navigateTo("/backtest")}
          />
          <QuickCard
            Icon={IconScanner}
            title="条件扫描"
            desc="多条件筛选，技术匹配度排序"
            onClick={() => navigateTo("/scanner")}
          />
          <QuickCard
            Icon={IconChart}
            title="图表分析"
            desc="K 线图，316+ 技术指标"
            onClick={() => navigateTo("/chart")}
          />
          <QuickCard
            Icon={IconReview}
            title="交易复盘"
            desc="情绪标签，复盘模板"
            onClick={() => navigateTo("/review")}
          />
          <QuickCard
            Icon={IconPortfolio}
            title="组合管理"
            desc="持仓跟踪，VaR 风险分析"
            onClick={() => navigateTo("/portfolio")}
          />
        </div>
      </section>

      {/* License + Stats Row */}
      <section style={{ marginBottom: 28, display: "flex", gap: 14, flexWrap: "wrap" }}>
        {license && (
          <div
            className="card"
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              padding: "12px 18px",
              flex: "0 0 auto",
            }}
          >
            {license.tier === "pro" ? (
              <IconShield size={20} />
            ) : (
              <IconTarget size={20} />
            )}
            <div>
              <div style={{ fontWeight: 600, fontSize: 12, color: "var(--text-primary)" }}>
                {tierLabel(license.tier)}
              </div>
              <div style={{ color: "var(--text-muted)", fontSize: 11 }}>
                {license.expiry
                  ? `有效期至 ${license.expiry}`
                  : license.trial_days_left != null
                    ? `试用剩余 ${license.trial_days_left} 天`
                    : "未激活"}
              </div>
            </div>
          </div>
        )}

        {dataSummary && (
          <>
            <StatBox
              icon={<IconDatabase size={16} />}
              label="本地数据"
              value={String(dataSummary.total_stocks)}
              unit="只股票"
              sub={`${(dataSummary.total_rows / 10000).toFixed(1)} 万条日线`}
            />
            <StatBox
              icon={<IconDatabase size={16} />}
              label="数据库"
              value={dataSummary.db_size_mb < 1
                ? `${(dataSummary.db_size_mb * 1024).toFixed(0)}`
                : dataSummary.db_size_mb.toFixed(1)}
              unit={dataSummary.db_size_mb < 1 ? "KB" : "MB"}
            />
            {pnl && pnl.total_trades > 0 && (
              <>
                <StatBox
                  icon={<IconPercent size={16} />}
                  label="胜率"
                  value={`${(pnl.win_rate * 100).toFixed(0)}`}
                  unit="%"
                  sub={`${pnl.total_trades} 笔交易`}
                  accent={pnl.win_rate >= 0.5 ? "positive" : "negative"}
                />
                <StatBox
                  icon={pnl.total_pnl >= 0 ? <IconTrendUp size={16} /> : <span style={{ display: "inline-flex", transform: "rotate(180deg)" }}><IconTrendUp size={16} /></span>}
                  label="总盈亏"
                  value={`¥${Math.abs(pnl.total_pnl).toLocaleString(undefined, { maximumFractionDigits: 0 })}`}
                  unit=""
                  sub={`盈亏比 ${pnl.profit_factor.toFixed(2)}`}
                  accent={pnl.total_pnl >= 0 ? "positive" : "negative"}
                />
              </>
            )}
          </>
        )}
      </section>

      {/* Workflow */}
      <section style={{ marginBottom: 28 }}>
        <h3 style={{ fontSize: 11, fontWeight: 600, color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.08em", marginBottom: 12 }}>
          建立你的交易系统
        </h3>
        <div className="card" style={{ padding: "20px 24px" }}>
          <p style={{ color: "var(--text-muted)", fontSize: 11, marginBottom: 18, lineHeight: 1.7 }}>
            专业交易员和散户最大的区别不是信息差，而是有没有一套可执行、可验证、可复盘的系统。
          </p>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", gap: 12 }}>
            <Step num={1} title="回测验证" desc="把交易想法放进历史数据验证。夏普比率、最大回撤、胜率、盈亏比 — 用数字代替感觉决策。" />
            <Step num={2} title="条件扫描" desc="设定技术条件，全市场筛选。找到符合你交易系统的标的，不再靠消息炒股。" />
            <Step num={3} title="纪律执行" desc="买入/持仓/止损规则明确，图表分析辅助确认入场点。赚该赚的钱，亏可控的钱。" />
            <Step num={4} title="复盘改进" desc="记录每笔交易的情绪和逻辑，定期归因。不是赚了就对、亏了就错 — 找到真正有效的模式。" />
          </div>
        </div>
      </section>

      {/* Risk Disclaimer */}
      <div
        style={{
          padding: "12px 16px",
          background: "rgba(245, 158, 11, 0.04)",
          border: "1px solid rgba(245, 158, 11, 0.12)",
          borderRadius: "var(--radius-md)",
          fontSize: 11,
          color: "var(--text-muted)",
          lineHeight: 1.7,
          display: "flex",
          alignItems: "flex-start",
          gap: 8,
        }}
      >
        <span style={{ flexShrink: 0, marginTop: 1 }}>
          <svg width={14} height={14} viewBox="0 0 24 24" fill="none" stroke="var(--warning)" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 2L2 22h20L12 2z" />
            <line x1="12" y1="10" x2="12" y2="16" />
            <circle cx="12" cy="19" r="0.5" fill="var(--warning)" stroke="none" />
          </svg>
        </span>
        <span>
          <strong style={{ color: "var(--warning)" }}>风险提示：</strong>
          QuantVault 是一款本地离线数据分析工具，不提供任何投资建议。所有功能仅供历史数据分析与自我研究使用。投资有风险，入市需谨慎。过往业绩不代表未来表现。
        </span>
      </div>
    </div>
  );
}

/* ── Sub-components ── */

function QuickCard({
  Icon, title, desc, onClick, primary,
}: {
  Icon: React.ComponentType<{ size?: number }>;
  title: string;
  desc: string;
  onClick: () => void;
  primary?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className="hover-lift"
      style={{
        display: "flex",
        alignItems: "flex-start",
        gap: 12,
        padding: "14px 16px",
        background: primary ? "rgba(212,160,23,0.04)" : "var(--bg-default)",
        border: primary ? "1px solid var(--border-accent)" : "1px solid var(--border-subtle)",
        borderRadius: "var(--radius-lg)",
        cursor: "pointer",
        textAlign: "left",
        color: "var(--text-primary)",
        fontFamily: "var(--font-ui)",
        width: "100%",
      }}
    >
      <span style={{ color: primary ? "var(--accent)" : "var(--text-secondary)", flexShrink: 0, marginTop: 1 }}>
        <Icon size={20} />
      </span>
      <div>
        <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 2, color: primary ? "var(--accent)" : "var(--text-primary)" }}>
          {title}
        </div>
        <div style={{ color: "var(--text-muted)", fontSize: 11, lineHeight: 1.4 }}>{desc}</div>
      </div>
    </button>
  );
}

function Step({ num, title, desc }: { num: number; title: string; desc: string }) {
  return (
    <div style={{ padding: "14px 16px", background: "var(--bg-raised)", borderRadius: "var(--radius-md)", border: "1px solid var(--border-subtle)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 8 }}>
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            width: 22,
            height: 22,
            borderRadius: "var(--radius-sm)",
            background: "rgba(212,160,23,0.1)",
            color: "var(--accent)",
            fontSize: 11,
            fontWeight: 700,
            fontFamily: "var(--font-data)",
          }}
        >
          {num}
        </span>
        <span style={{ fontWeight: 600, fontSize: 12, color: "var(--text-primary)" }}>{title}</span>
      </div>
      <p style={{ color: "var(--text-secondary)", fontSize: 11, lineHeight: 1.7, margin: 0 }}>{desc}</p>
    </div>
  );
}

function StatBox({
  icon, label, value, unit, sub, accent,
}: {
  icon?: React.ReactNode;
  label: string;
  value: string;
  unit: string;
  sub?: string;
  accent?: "positive" | "negative";
}) {
  const accentColor = accent === "positive" ? "var(--positive)" : accent === "negative" ? "var(--negative)" : "var(--accent-secondary)";

  return (
    <div
      className="card hover-lift"
      style={{ flex: "0 0 auto", minWidth: 150, padding: "12px 16px" }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 6, color: "var(--text-muted)" }}>
        {icon}
        <span style={{ fontSize: 10, textTransform: "uppercase", letterSpacing: "0.04em" }}>{label}</span>
      </div>
      <div style={{ display: "flex", alignItems: "baseline", gap: 4 }}>
        <span style={{ fontSize: 22, fontWeight: 700, color: accentColor, fontFamily: "var(--font-data)", fontVariantNumeric: "tabular-nums" }}>{value}</span>
        <span style={{ fontSize: 11, color: "var(--text-muted)", fontFamily: "var(--font-ui)" }}>{unit}</span>
      </div>
      {sub && (
        <div style={{ fontSize: 11, color: "var(--text-muted)", marginTop: 2 }}>{sub}</div>
      )}
    </div>
  );
}

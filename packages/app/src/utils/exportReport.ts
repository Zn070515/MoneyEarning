import type { BacktestConfig, BacktestResult } from "../stores/backtestStore";

function fmtPct(v: number): string {
  return `${(v * 100).toFixed(2)}%`;
}

function fmtNum(v: number, d = 2): string {
  return v.toFixed(d);
}

export function generateBacktestReportMarkdown(
  config: BacktestConfig,
  result: BacktestResult,
  stockCode?: string | null,
): string {
  const now = new Date().toISOString().replace("T", " ").substring(0, 19);
  const lines: string[] = [];

  lines.push(`# 策略回测报告`);
  lines.push("");
  lines.push(`**生成时间**: ${now}`);
  if (stockCode) {
    lines.push(`**标的**: ${stockCode}`);
  }
  lines.push(`**策略模板**: \`${config.template}\``);
  lines.push("");

  // ── 回测配置 ──
  lines.push("## 回测配置");
  lines.push("");
  lines.push("| 参数 | 值 |");
  lines.push("|------|----|");
  lines.push(`| 初始资金 | ¥${config.initialCapital.toLocaleString()} |`);
  lines.push(`| 佣金费率 | ${fmtPct(config.commissionRate)} |`);
  lines.push(`| 印花税率 | ${fmtPct(config.stampTaxRate)} |`);
  lines.push(`| 滑点 | ${fmtPct(config.slippage)} |`);
  lines.push(`| 回测区间 | ${config.startDate} 至 ${config.endDate} |`);
  lines.push("");

  // ── 策略参数 ──
  if (Object.keys(config.params).length > 0) {
    lines.push("## 策略参数");
    lines.push("");
    lines.push("| 参数 | 值 |");
    lines.push("|------|----|");
    for (const [k, v] of Object.entries(config.params)) {
      lines.push(`| \`${k}\` | ${v} |`);
    }
    lines.push("");
  }

  // ── 绩效指标 ──
  lines.push("## 绩效指标");
  lines.push("");
  lines.push("| 指标 | 数值 |");
  lines.push("|------|------|");
  lines.push(`| 总收益率 | ${fmtPct(result.totalReturn)} |`);
  lines.push(`| 年化收益率 | ${fmtPct(result.annualReturn)} |`);
  lines.push(`| 最大回撤 | ${fmtPct(result.maxDrawdown)} |`);
  lines.push(`| 夏普比率 | ${fmtNum(result.sharpeRatio)} |`);
  lines.push(`| 索提诺比率 | ${fmtNum(result.sortinoRatio)} |`);
  lines.push(`| 卡尔玛比率 | ${fmtNum(result.calmarRatio)} |`);
  lines.push(`| 胜率 | ${fmtPct(result.winRate)} |`);
  lines.push(`| 交易次数 | ${result.totalTrades} |`);
  lines.push("");

  // ── 权益曲线摘要 ──
  if (result.equityCurve.length > 0) {
    const first = result.equityCurve[0][1];
    const last = result.equityCurve[result.equityCurve.length - 1][1];
    const peak = Math.max(...result.equityCurve.map(([, v]) => v));
    const trough = Math.min(...result.equityCurve.map(([, v]) => v));

    lines.push("## 权益曲线摘要");
    lines.push("");
    lines.push("| 指标 | 数值 |");
    lines.push("|------|------|");
    lines.push(`| 起始权益 | ¥${first.toFixed(2)} |`);
    lines.push(`| 最终权益 | ¥${last.toFixed(2)} |`);
    lines.push(`| 峰值权益 | ¥${peak.toFixed(2)} |`);
    lines.push(`| 最低权益 | ¥${trough.toFixed(2)} |`);
    lines.push(`| 权益曲线数据点 | ${result.equityCurve.length} |`);
    lines.push("");
  }

  // ── 免责声明 ──
  lines.push("---");
  lines.push("");
  lines.push(
    "*本报告由 QuantVault 本地量化分析工作站自动生成。过往业绩不代表未来表现，投资有风险，入市需谨慎。*",
  );
  lines.push("");

  return lines.join("\n");
}

export function downloadMarkdownReport(
  markdown: string,
  filename?: string,
) {
  const name = filename ?? `backtest-report-${Date.now()}.md`;
  const blob = new Blob([markdown], { type: "text/markdown;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

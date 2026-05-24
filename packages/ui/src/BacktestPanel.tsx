import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { OHLCV } from "@me/chart-engine";

interface BtResult {
  total_return: number;
  annual_return: number;
  max_drawdown: number;
  sharpe_ratio: number;
  sortino_ratio: number;
  calmar_ratio: number;
  win_rate: number;
  profit_loss_ratio: number;
  total_trades: number;
  equity_curve: [string, number][];
  monthly_returns: [string, number][];
}

interface WfWindowResult {
  window_idx: number;
  is_start: number;
  is_end: number;
  oos_start: number;
  oos_end: number;
  best_params: Record<string, number>;
  is_metrics: BtResult;
  oos_metrics: BtResult;
  is_annual_return: number;
  oos_annual_return: number;
  is_sharpe: number;
  oos_sharpe: number;
}

interface WalkForwardResult {
  windows: WfWindowResult[];
  avg_oos_return: number;
  avg_oos_sharpe: number;
  avg_is_return: number;
  avg_is_sharpe: number;
  param_stability_score: number;
  is_oos_correlation: number;
  assessment: string;
}

interface MonteCarloResult {
  terminal_equities: number[];
  max_drawdowns: number[];
  sharpe_ratios: number[];
  annual_returns: number[];
  win_rates: number[];
  ci_lower_return: number;
  ci_upper_return: number;
  median_return: number;
  mean_return: number;
  ci_lower_dd: number;
  ci_upper_dd: number;
  median_dd: number;
  ci_lower_sharpe: number;
  ci_upper_sharpe: number;
  median_sharpe: number;
  var_95: number;
  cvar_95: number;
  prob_profit: number;
  prob_ruin: number;
  num_simulations: number;
  method: string;
}

interface OptimizerResult {
  best_params: Record<string, number>;
  best_score: number;
  best_result: BtResult;
  all_results: [Record<string, number>, number][];
  iterations: number;
  convergence_generation: number;
}

interface StrategyTemplate {
  name: string;
  name_cn: string;
  category: string;
  is_free: boolean;
  params: Record<string, number>;
  description: string;
}

interface BacktestPanelProps {
  data: OHLCV[];
  isPro?: boolean;
}

const CATEGORY_ORDER = ["趋势跟踪", "均值回归", "动量", "突破", "复合"];
type SubMode = "backtest" | "walkforward" | "montecarlo" | "optimization";

export function BacktestPanel({ data, isPro }: BacktestPanelProps) {
  const [mode, setMode] = useState<SubMode>("backtest");
  const [templates, setTemplates] = useState<StrategyTemplate[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState("ma_cross");
  const [params, setParams] = useState<Record<string, number>>({ fast: 5, slow: 20 });
  const [capital, setCapital] = useState(100000);
  const [commission, setCommission] = useState(0.0003);
  const [stampTax, setStampTax] = useState(0.001);
  const [slippage, setSlippage] = useState(0.001);
  const [positionPct, setPositionPct] = useState(1.0);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<BtResult | null>(null);
  const [error, setError] = useState("");

  // WF state
  const [wfInSample, setWfInSample] = useState(252);
  const [wfOutSample, setWfOutSample] = useState(63);
  const [wfStep, setWfStep] = useState(63);
  const [wfAnchor, setWfAnchor] = useState("rolling");
  const [wfResult, setWfResult] = useState<WalkForwardResult | null>(null);

  // MC state
  const [mcTrials, setMcTrials] = useState(1000);
  const [mcMethod, setMcMethod] = useState("trade_shuffle");
  const [mcResult, setMcResult] = useState<MonteCarloResult | null>(null);

  // Opt state
  const [optMethod, setOptMethod] = useState("grid_search");
  const [optMetric, setOptMetric] = useState("sharpe_ratio");
  const [optIterations, setOptIterations] = useState(5000);
  const [optResult, setOptResult] = useState<OptimizerResult | null>(null);

  const loadTemplates = useCallback(async () => {
    try {
      const data = await invoke<StrategyTemplate[]>("list_strategy_templates");
      setTemplates(data);
    } catch (_) {}
  }, []);

  useEffect(() => { loadTemplates(); }, [loadTemplates]);

  const handleTemplate = (name: string) => {
    setSelectedTemplate(name);
    const t = templates.find((s) => s.name === name);
    if (t) setParams({ ...t.params });
  };

  const grouped: Record<string, StrategyTemplate[]> = {};
  templates.forEach((t) => {
    if (!grouped[t.category]) grouped[t.category] = [];
    grouped[t.category].push(t);
  });

  interface IndicatorInput { time: number; open: number; high: number; low: number; close: number; volume: number; amount: number; turnover?: number; }
  const dataForBackend = (): IndicatorInput[] =>
    data.map((d) => ({
      time: d.time, open: d.open, high: d.high, low: d.low, close: d.close,
      volume: d.volume, amount: d.amount ?? 0, turnover: d.turnover,
    }));

  // ── Basic Backtest ──
  const handleRun = async () => {
    if (data.length === 0) return;
    const sel = templates.find((s) => s.name === selectedTemplate);
    if (sel && !sel.is_free && !isPro) {
      setError("此策略为专业版功能，请前往设置页面升级授权后使用");
      return;
    }
    setRunning(true); setError("");
    try {
      const res = await invoke<BtResult>("run_backtest", {
        data: dataForBackend(), template: selectedTemplate, params,
        config: { initial_capital: capital, commission_rate: commission, stamp_tax_rate: stampTax, slippage, position_pct: positionPct },
      });
      setResult(res);
    } catch (e) { setError(String(e)); }
    setRunning(false);
  };

  // ── Walk-Forward ──
  const handleWF = async () => {
    if (data.length === 0) return;
    const sel = templates.find((s) => s.name === selectedTemplate);
    if (sel && !sel.is_free && !isPro) { setError("此策略为专业版功能"); return; }
    setRunning(true); setError("");
    try {
      const res = await invoke<WalkForwardResult>("run_walk_forward", {
        data: dataForBackend(), template: selectedTemplate,
        paramGrid: Object.fromEntries(Object.entries(params).map(([k, v]) => [k, [v * 0.5, v * 2, v * 0.5]])),
        inSample: wfInSample, outSample: wfOutSample, stepSize: wfStep, anchorMode: wfAnchor,
        config: { initial_capital: capital, commission_rate: commission, stamp_tax_rate: stampTax, slippage, position_pct: positionPct },
      });
      setWfResult(res);
    } catch (e) { setError(String(e)); }
    setRunning(false);
  };

  // ── Monte Carlo ──
  const handleMC = async () => {
    if (data.length === 0) return;
    const sel = templates.find((s) => s.name === selectedTemplate);
    if (sel && !sel.is_free && !isPro) { setError("此策略为专业版功能"); return; }
    setRunning(true); setError("");
    try {
      const res = await invoke<MonteCarloResult>("run_monte_carlo", {
        data: dataForBackend(), template: selectedTemplate, params,
        numSimulations: mcTrials, method: mcMethod, confidenceLevel: 0.95,
        config: { initial_capital: capital, commission_rate: commission, stamp_tax_rate: stampTax, slippage, position_pct: positionPct },
      });
      setMcResult(res);
    } catch (e) { setError(String(e)); }
    setRunning(false);
  };

  // ── Optimization ──
  const handleOpt = async () => {
    if (data.length === 0) return;
    const sel = templates.find((s) => s.name === selectedTemplate);
    if (sel && !sel.is_free && !isPro) { setError("此策略为专业版功能"); return; }
    setRunning(true); setError("");
    try {
      const grid: Record<string, number[]> = {};
      for (const [k, v] of Object.entries(params)) {
        grid[k] = [v * 0.3, v * 3.0, Math.max(v * 0.3, 1)];
      }
      const res = await invoke<OptimizerResult>("run_optimization", {
        data: dataForBackend(), template: selectedTemplate, paramGrid: grid,
        method: optMethod, targetMetric: optMetric, maxIterations: optIterations,
        config: { initial_capital: capital, commission_rate: commission, stamp_tax_rate: stampTax, slippage, position_pct: positionPct },
      });
      setOptResult(res);
    } catch (e) { setError(String(e)); }
    setRunning(false);
  };

  const pct = (v: number) => (v * 100).toFixed(2) + "%";
  const fmt = (v: number) => v.toFixed(4);
  const color = (v: number) => (v >= 0 ? "#EF5350" : "#26A69A");

  return (
    <div style={{ background: "#161616", color: "#D4D4D4", fontFamily: "monospace", fontSize: 13, height: "100%", display: "flex", flexDirection: "column", overflow: "hidden" }}>
      <div style={{ padding: "10px 12px", borderBottom: "1px solid #2A2A2A", fontWeight: 600, color: "#fff", fontSize: 14, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span>策略回测</span>
        <span style={{ color: isPro ? "#CCAA00" : "#858585", fontSize: 10, fontWeight: 400 }}>{isPro ? "PRO" : "免费版"}</span>
      </div>

      {/* Sub-mode tabs */}
      <div style={{ display: "flex", borderBottom: "1px solid #2A2A2A", background: "#121212" }}>
        {([
          ["backtest", "回测"], ["walkforward", "WF验证"], ["montecarlo", "蒙特卡洛"], ["optimization", "参数优化"],
        ] as [SubMode, string][]).map(([m, label]) => (
          <button key={m} onClick={() => { setMode(m); setError(""); }}
            style={{
              flex: 1, padding: "8px 6px", border: "none",
              background: mode === m ? "#161616" : "transparent",
              color: mode === m ? "#CCAA00" : "#858585", cursor: "pointer",
              fontSize: 12, fontFamily: "monospace", fontWeight: mode === m ? 600 : 400,
              borderBottom: mode === m ? "2px solid #CCAA00" : "2px solid transparent",
            }}>
            {label}
          </button>
        ))}
      </div>

      <div style={{ flex: 1, overflow: "auto", padding: 12 }}>
        {/* Template + Config (shared across modes) */}
        <div style={{ marginBottom: 12 }}>
          <label style={labelStyle}>策略模板 ({templates.length}个)</label>
          <select value={selectedTemplate} onChange={(e) => handleTemplate(e.target.value)} style={{ ...inputStyle, width: "100%" }}>
            {CATEGORY_ORDER.map((cat) => {
              const group = grouped[cat];
              if (!group) return null;
              return (
                <optgroup key={cat} label={`── ${cat} ──`}>
                  {group.map((s) => (
                    <option key={s.name} value={s.name}>{s.is_free ? "🆓" : "⭐"} {s.name_cn}</option>
                  ))}
                </optgroup>
              );
            })}
          </select>
          {(() => {
            const sel = templates.find((s) => s.name === selectedTemplate);
            if (!sel) return null;
            return (
              <div style={{ marginTop: 6, padding: "6px 8px", background: "#121212", borderRadius: 4, fontSize: 11, color: "#858585", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <span>{sel.description}</span>
                <span style={{ color: sel.is_free ? "#26A69A" : "#CCAA00", fontSize: 10, fontWeight: 600 }}>
                  {sel.is_free ? "免费" : "PRO"}{!sel.is_free && !isPro && <span style={{ color: "#EF5350", marginLeft: 4 }}>（需要升级）</span>}
                </span>
              </div>
            );
          })()}
        </div>

        {/* Params */}
        <div style={{ marginBottom: 12 }}>
          <label style={labelStyle}>参数</label>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
            {Object.entries(params).map(([k, v]) => (
              <div key={k} style={{ display: "flex", alignItems: "center", gap: 6 }}>
                <span style={{ color: "#858585", fontSize: 11, width: 80 }}>{k}</span>
                <input type="number" value={v} step={1}
                  onChange={(e) => setParams({ ...params, [k]: parseFloat(e.target.value) || 0 })}
                  style={{ ...inputStyle, width: 80, textAlign: "center" }} />
              </div>
            ))}
          </div>
        </div>

        {/* Mode-specific content */}
        {mode === "backtest" && (
          <>
            <div style={{ marginBottom: 12 }}>
              <label style={labelStyle}>回测配置</label>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
                <ConfigField label="初始资金" value={capital} onChange={setCapital} />
                <ConfigField label="仓位比例" value={positionPct} onChange={setPositionPct} step={0.1} />
                <ConfigField label="佣金率" value={commission} onChange={setCommission} step={0.0001} />
                <ConfigField label="印花税率" value={stampTax} onChange={setStampTax} step={0.0001} />
                <ConfigField label="滑点" value={slippage} onChange={setSlippage} step={0.0001} />
              </div>
            </div>
            <button onClick={handleRun} disabled={running || data.length === 0}
              style={{ width: "100%", background: running ? "#8a7a3a" : "#CCAA00", color: "#000", border: "none", padding: "8px 16px", borderRadius: 4, cursor: running ? "not-allowed" : "pointer", fontSize: 14, fontWeight: 600, marginBottom: 12 }}>
              {running ? "回测中..." : "开始回测"}
            </button>
            {error && <div style={{ padding: 8, background: "#3a1a2e", borderRadius: 4, color: "#EF5350", fontSize: 12, marginBottom: 12 }}>{error}</div>}
            {result && <BacktestResults result={result} capital={capital} />}
          </>
        )}

        {mode === "walkforward" && (
          <>
            <div style={{ marginBottom: 12 }}>
              <label style={labelStyle}>Walk-Forward 配置</label>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
                <ConfigField label="样本内(K线)" value={wfInSample} onChange={setWfInSample} />
                <ConfigField label="样本外(K线)" value={wfOutSample} onChange={setWfOutSample} />
                <ConfigField label="步长(K线)" value={wfStep} onChange={setWfStep} />
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  <span style={{ color: "#858585", fontSize: 11, width: 70 }}>窗口模式</span>
                  <select value={wfAnchor} onChange={(e) => setWfAnchor(e.target.value)} style={{ ...inputStyle, width: 100, textAlign: "center" }}>
                    <option value="rolling">滑动窗口</option>
                    <option value="anchored">扩展窗口</option>
                  </select>
                </div>
              </div>
            </div>
            <button onClick={handleWF} disabled={running || data.length === 0}
              style={{ width: "100%", background: running ? "#8a7a3a" : "#CCAA00", color: "#000", border: "none", padding: "8px 16px", borderRadius: 4, cursor: running ? "not-allowed" : "pointer", fontSize: 14, fontWeight: 600, marginBottom: 12 }}>
              {running ? "验证中..." : "开始WF验证"}
            </button>
            {error && <div style={{ padding: 8, background: "#3a1a2e", borderRadius: 4, color: "#EF5350", fontSize: 12, marginBottom: 12 }}>{error}</div>}
            {wfResult && <WalkForwardResults result={wfResult} />}
          </>
        )}

        {mode === "montecarlo" && (
          <>
            <div style={{ marginBottom: 12 }}>
              <label style={labelStyle}>Monte Carlo 配置</label>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
                <ConfigField label="模拟次数" value={mcTrials} onChange={setMcTrials} />
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  <span style={{ color: "#858585", fontSize: 11, width: 70 }}>模拟方法</span>
                  <select value={mcMethod} onChange={(e) => setMcMethod(e.target.value)} style={{ ...inputStyle, width: 130, textAlign: "center" }}>
                    <option value="trade_shuffle">交易洗牌</option>
                    <option value="return_bootstrap">收益Bootstrap</option>
                    <option value="parametric">参数化(正态)</option>
                  </select>
                </div>
              </div>
            </div>
            <button onClick={handleMC} disabled={running || data.length === 0}
              style={{ width: "100%", background: running ? "#8a7a3a" : "#CCAA00", color: "#000", border: "none", padding: "8px 16px", borderRadius: 4, cursor: running ? "not-allowed" : "pointer", fontSize: 14, fontWeight: 600, marginBottom: 12 }}>
              {running ? "模拟中..." : "开始Monte Carlo模拟"}
            </button>
            {error && <div style={{ padding: 8, background: "#3a1a2e", borderRadius: 4, color: "#EF5350", fontSize: 12, marginBottom: 12 }}>{error}</div>}
            {mcResult && <MonteCarloResults result={mcResult} capital={capital} />}
          </>
        )}

        {mode === "optimization" && (
          <>
            <div style={{ marginBottom: 12 }}>
              <label style={labelStyle}>参数优化配置</label>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  <span style={{ color: "#858585", fontSize: 11, width: 70 }}>优化方法</span>
                  <select value={optMethod} onChange={(e) => setOptMethod(e.target.value)} style={{ ...inputStyle, width: 130, textAlign: "center" }}>
                    <option value="grid_search">网格搜索</option>
                    <option value="genetic_algorithm">遗传算法</option>
                  </select>
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  <span style={{ color: "#858585", fontSize: 11, width: 70 }}>目标指标</span>
                  <select value={optMetric} onChange={(e) => setOptMetric(e.target.value)} style={{ ...inputStyle, width: 130, textAlign: "center" }}>
                    <option value="sharpe_ratio">夏普比率</option>
                    <option value="total_return">年化收益</option>
                    <option value="calmar_ratio">卡玛比率</option>
                    <option value="sortino_ratio">索提诺比率</option>
                  </select>
                </div>
                <ConfigField label="最大迭代" value={optIterations} onChange={setOptIterations} />
              </div>
            </div>
            <button onClick={handleOpt} disabled={running || data.length === 0}
              style={{ width: "100%", background: running ? "#8a7a3a" : "#CCAA00", color: "#000", border: "none", padding: "8px 16px", borderRadius: 4, cursor: running ? "not-allowed" : "pointer", fontSize: 14, fontWeight: 600, marginBottom: 12 }}>
              {running ? "优化中..." : "开始参数优化"}
            </button>
            {error && <div style={{ padding: 8, background: "#3a1a2e", borderRadius: 4, color: "#EF5350", fontSize: 12, marginBottom: 12 }}>{error}</div>}
            {optResult && <OptimizationResults result={optResult} capital={capital} />}
          </>
        )}

        {/* Running progress indicator */}
        {running && (
          <div style={{ marginBottom: 12 }}>
            <div style={{
              height: 3, background: "#2A2A2A", borderRadius: 2,
              overflow: "hidden", position: "relative",
            }}>
              <div style={{
                position: "absolute", top: 0, left: 0, height: "100%",
                width: "30%",
                background: "linear-gradient(90deg, #CCAA00, #7E57C2)",
                borderRadius: 2,
                animation: "backtest-progress-slide 1.2s ease-in-out infinite",
              }} />
            </div>
            <style>{`
              @keyframes backtest-progress-slide {
                0% { left: -30%; }
                100% { left: 100%; }
              }
            `}</style>
            <div style={{
              color: "#858585", fontSize: 10, fontFamily: "monospace",
              textAlign: "center", marginTop: 4,
            }}>
              正在计算中...
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ── Basic Backtest Results ──
function BacktestResults({ result, capital }: { result: BtResult; capital: number }) {
  const pct = (v: number) => (v * 100).toFixed(2) + "%";
  const fmt = (v: number) => v.toFixed(4);
  const c = (v: number) => (v >= 0 ? "#EF5350" : "#26A69A");
  return (
    <div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6, marginBottom: 12 }}>
        <MetricBox label="总收益率" value={pct(result.total_return)} color={c(result.total_return)} />
        <MetricBox label="年化收益" value={pct(result.annual_return)} color={c(result.annual_return)} />
        <MetricBox label="最大回撤" value={pct(result.max_drawdown)} color="#EF5350" />
        <MetricBox label="夏普比率" value={fmt(result.sharpe_ratio)} />
        <MetricBox label="索提诺比率" value={fmt(result.sortino_ratio)} />
        <MetricBox label="卡玛比率" value={fmt(result.calmar_ratio)} />
        <MetricBox label="胜率" value={pct(result.win_rate)} color="#CCAA00" />
        <MetricBox label="盈亏比" value={fmt(result.profit_loss_ratio)} />
      </div>
      <div style={{ padding: 8, background: "#121212", borderRadius: 4, fontSize: 12, color: "#858585", textAlign: "center" }}>
        总交易：{result.total_trades} 笔{result.equity_curve.length > 0 && <span> · 权益曲线：{result.equity_curve.length}点</span>}
      </div>
      {result.equity_curve.length > 1 && <EquityMiniChart data={result.equity_curve.map(([, v]) => v)} initialCapital={capital} />}
    </div>
  );
}

// ── Walk-Forward Results ──
function WalkForwardResults({ result }: { result: WalkForwardResult }) {
  const pct = (v: number) => (v * 100).toFixed(2) + "%";
  const fmt = (v: number) => v.toFixed(4);
  if (result.windows.length === 0) {
    return <div style={{ padding: 12, color: "#EF5350", fontSize: 12, background: "#121212", borderRadius: 4 }}>{result.assessment}</div>;
  }
  return (
    <div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6, marginBottom: 12 }}>
        <MetricBox label="平均OOS收益" value={pct(result.avg_oos_return)} color={result.avg_oos_return >= 0 ? "#EF5350" : "#26A69A"} />
        <MetricBox label="平均OOS Sharpe" value={fmt(result.avg_oos_sharpe)} />
        <MetricBox label="平均IS收益" value={pct(result.avg_is_return)} color={result.avg_is_return >= 0 ? "#EF5350" : "#26A69A"} />
        <MetricBox label="平均IS Sharpe" value={fmt(result.avg_is_sharpe)} />
        <MetricBox label="参数稳定性" value={fmt(1.0 - result.param_stability_score)} color={result.param_stability_score < 0.3 ? "#26A69A" : "#EF5350"} />
        <MetricBox label="IS/OOS相关性" value={fmt(result.is_oos_correlation)} color={result.is_oos_correlation > 0.5 ? "#26A69A" : "#EF5350"} />
      </div>
      <div style={{ padding: 8, background: "#121212", borderRadius: 4, fontSize: 12, lineHeight: 1.8, marginBottom: 12 }}>
        <div style={{ color: "#CCAA00", fontWeight: 600, marginBottom: 4 }}>综合评估</div>
        <div style={{ color: "#D4D4D4" }}>{result.assessment}</div>
      </div>
      <div style={{ color: "#858585", fontSize: 11, marginBottom: 4 }}>窗口详情 ({result.windows.length}个窗口)</div>
      {result.windows.slice(0, 10).map((w) => (
        <div key={w.window_idx} style={{ padding: "6px 8px", background: "#121212", borderRadius: 4, marginBottom: 4, fontSize: 11, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <span style={{ color: "#858585" }}>#{w.window_idx + 1}</span>
          <span style={{ color: "#666666" }}>IS [{w.is_start}-{w.is_end}] OOS [{w.oos_start}-{w.oos_end}]</span>
          <span style={{ color: w.oos_annual_return >= 0 ? "#EF5350" : "#26A69A" }}>OOS {pct(w.oos_annual_return)}</span>
          <span style={{ color: "#CCAA00" }}>S {fmt(w.oos_sharpe)}</span>
        </div>
      ))}
      {result.windows.length > 10 && <div style={{ color: "#666666", fontSize: 10, textAlign: "center" }}>... 还有 {result.windows.length - 10} 个窗口</div>}
    </div>
  );
}

// ── Monte Carlo Results ──
function MonteCarloResults({ result, capital }: { result: MonteCarloResult; capital: number }) {
  const pct = (v: number) => (v * 100).toFixed(2) + "%";
  const fmt = (v: number) => v.toFixed(4);
  return (
    <div>
      <div style={{ padding: 8, background: "#121212", borderRadius: 4, fontSize: 11, color: "#858585", marginBottom: 12, textAlign: "center" }}>
        方法：{result.method} · 模拟次数：{result.num_simulations}
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6, marginBottom: 12 }}>
        <MetricBox label="中位年化收益" value={pct(result.median_return)} color={result.median_return >= 0 ? "#EF5350" : "#26A69A"} />
        <MetricBox label="平均年化收益" value={pct(result.mean_return)} color={result.mean_return >= 0 ? "#EF5350" : "#26A69A"} />
        <MetricBox label="收益CI下限" value={pct(result.ci_lower_return)} color={result.ci_lower_return >= 0 ? "#EF5350" : "#26A69A"} />
        <MetricBox label="收益CI上限" value={pct(result.ci_upper_return)} color={result.ci_upper_return >= 0 ? "#EF5350" : "#26A69A"} />
        <MetricBox label="中位回撤" value={pct(result.median_dd)} color="#EF5350" />
        <MetricBox label="回撤CI下限" value={pct(result.ci_lower_dd)} color="#EF5350" />
        <MetricBox label="中位Sharpe" value={fmt(result.median_sharpe)} />
        <MetricBox label="Sharpe CI上限" value={fmt(result.ci_upper_sharpe)} />
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: 6, marginBottom: 12 }}>
        <MetricBox label="VaR 95%" value={pct(result.var_95)} color="#EF5350" />
        <MetricBox label="CVaR 95%" value={pct(result.cvar_95)} color="#EF5350" />
        <MetricBox label="盈利概率" value={pct(result.prob_profit)} color={result.prob_profit > 0.5 ? "#26A69A" : "#EF5350"} />
        <MetricBox label="破产概率" value={pct(result.prob_ruin)} color={result.prob_ruin < 0.1 ? "#26A69A" : "#EF5350"} />
      </div>
      {result.annual_returns.length > 1 && <DistHistogram data={result.annual_returns} label="年化收益分布" />}
    </div>
  );
}

// ── Optimization Results ──
function OptimizationResults({ result, capital }: { result: OptimizerResult; capital: number }) {
  const pct = (v: number) => (v * 100).toFixed(2) + "%";
  const fmt = (v: number) => v.toFixed(4);
  return (
    <div>
      <div style={{ padding: 8, background: "#121212", borderRadius: 4, fontSize: 11, color: "#858585", marginBottom: 12, textAlign: "center" }}>
        迭代 {result.iterations} 次 · 收敛于第 {result.convergence_generation} 代
      </div>
      <div style={{ padding: "6px 8px", background: "#121212", borderRadius: 4, marginBottom: 12 }}>
        <div style={{ color: "#CCAA00", fontSize: 11, marginBottom: 4, fontWeight: 600 }}>最优参数</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 4 }}>
          {Object.entries(result.best_params).map(([k, v]) => (
            <div key={k} style={{ fontSize: 11, color: "#D4D4D4" }}><span style={{ color: "#858585" }}>{k}:</span> {typeof v === "number" ? v.toFixed(2) : String(v)}</div>
          ))}
        </div>
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6, marginBottom: 12 }}>
        <MetricBox label="最优得分" value={fmt(result.best_score)} color="#CCAA00" />
        <MetricBox label="年化收益" value={pct(result.best_result.annual_return)} color={result.best_result.annual_return >= 0 ? "#EF5350" : "#26A69A"} />
        <MetricBox label="最大回撤" value={pct(result.best_result.max_drawdown)} color="#EF5350" />
        <MetricBox label="夏普比率" value={fmt(result.best_result.sharpe_ratio)} />
        <MetricBox label="胜率" value={pct(result.best_result.win_rate)} />
        <MetricBox label="总交易" value={String(result.best_result.total_trades)} />
      </div>
      {result.best_result.equity_curve.length > 1 && <EquityMiniChart data={result.best_result.equity_curve.map(([, v]) => v)} initialCapital={capital} />}
    </div>
  );
}

// ── Distribution Histogram (for Monte Carlo) ──
function DistHistogram({ data, label }: { data: number[]; label: string }) {
  const h = 60; const w = 260;
  const min = Math.min(...data);
  const max = Math.max(...data);
  const rng = max - min || 1;
  const bins = 20;
  const binW = rng / bins;
  const counts = new Array(bins).fill(0);
  data.forEach((v) => { const idx = Math.min(Math.floor((v - min) / binW), bins - 1); counts[idx]++; });
  const maxCount = Math.max(...counts, 1);
  const barW = w / bins;

  return (
    <div style={{ marginTop: 8, padding: "6px", background: "#121212", borderRadius: 4 }}>
      <div style={{ color: "#858585", fontSize: 10, marginBottom: 4 }}>{label}</div>
      <svg width="100%" height={h} style={{ display: "block" }}>
        {counts.map((c, i) => (
          <rect key={i} x={i * barW} y={h - (c / maxCount) * h} width={barW - 1} height={(c / maxCount) * h}
            fill={i < bins / 2 ? "#26A69A" : "#EF5350"} opacity={0.7} />
        ))}
        <line x1={0} y1={h} x2={w} y2={h} stroke="#2A2A2A" strokeWidth={1} />
      </svg>
    </div>
  );
}

// ── Shared Components ──

function ConfigField({ label, value, onChange, step = 1 }: { label: string; value: number; onChange: (v: number) => void; step?: number }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
      <span style={{ color: "#858585", fontSize: 11, width: 70 }}>{label}</span>
      <input type="number" value={value} step={step} onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
        style={{ ...inputStyle, width: 100, textAlign: "center" }} />
    </div>
  );
}

function MetricBox({ label, value, color: c }: { label: string; value: string; color?: string }) {
  return (
    <div style={{ padding: "6px 8px", background: "#121212", borderRadius: 4, textAlign: "center" }}>
      <div style={{ color: "#858585", fontSize: 10, marginBottom: 2 }}>{label}</div>
      <div style={{ color: c ?? "#fff", fontSize: 14, fontWeight: 600 }}>{value}</div>
    </div>
  );
}

function EquityMiniChart({ data, initialCapital }: { data: number[]; initialCapital: number }) {
  const h = 80;
  const min = Math.min(...data, initialCapital);
  const max = Math.max(...data, initialCapital);
  const rng = max - min || 1;
  const w = 280;
  const points = data.map((v, i) => { const x = (i / (data.length - 1)) * w; const y = h - ((v - min) / rng) * h; return `${x},${y}`; }).join(" ");
  return (
    <div style={{ marginTop: 8, padding: "6px", background: "#121212", borderRadius: 4 }}>
      <div style={{ color: "#858585", fontSize: 10, marginBottom: 4 }}>权益曲线</div>
      <svg width="100%" height={h} style={{ display: "block" }}>
        <line x1={0} y1={h - ((initialCapital - min) / rng) * h} x2={w} y2={h - ((initialCapital - min) / rng) * h} stroke="#2A2A2A" strokeDasharray="4,4" />
        <polyline points={points} fill="none" stroke={data[data.length - 1] >= initialCapital ? "#EF5350" : "#26A69A"} strokeWidth={1.5} />
      </svg>
    </div>
  );
}

const labelStyle: React.CSSProperties = { fontSize: 11, color: "#858585", marginBottom: 4, display: "block" };

const inputStyle: React.CSSProperties = { background: "#121212", border: "1px solid #2A2A2A", color: "#fff", padding: "4px 8px", borderRadius: 4, fontSize: 12, fontFamily: "monospace", outline: "none" };

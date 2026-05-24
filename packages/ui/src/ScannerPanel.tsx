import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StockInfo {
  id: number;
  code: string;
  name: string;
  exchange: string;
}

interface IndicatorMeta {
  name: string;
  name_cn: string;
  category: string;
  params: ParamDef[];
  is_free: boolean;
}

interface ParamDef {
  name: string;
  default: number;
  min?: number;
  max?: number;
}

interface ScanResultItem {
  stock_id: number;
  score: number;
  signals: string[];
}

interface CapsResult {
  pool: string;
  strategy: string;
  projected_sharpe: number;
  n_assets: number;
}

interface CgpcPool {
  name: string;
  indices: number[];
  avg_quality: number;
  avg_corr: number;
}

interface MarsResult {
  n_regimes: number;
  current_regime: number;
  recommended_strategy: string;
  regime_strategies: Record<number, string>;
  regime_sizes: number[];
}

interface SearchNode {
  alpha_tier: string;
  objective: string;
  ensemble: string;
  factor_subset: string;
  status: string;
  result_sharpe: number | null;
  round_num: number;
}

type ScanOp = "COMPARE" | "CROSS";
type AlgoTab = "condition" | "caps" | "cgpc" | "mars" | "meta";

export function ScannerPanel() {
  const [indicators, setIndicators] = useState<IndicatorMeta[]>([]);
  const [stocks, setStocks] = useState<StockInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState("");

  // Tab state
  const [activeTab, setActiveTab] = useState<AlgoTab>("condition");

  // Condition scan state
  const [scanOp, setScanOp] = useState<ScanOp>("COMPARE");
  const [indicator, setIndicator] = useState("rsi");
  const [compareOp, setCompareOp] = useState("<");
  const [threshold, setThreshold] = useState("30");
  const [crossDir, setCrossDir] = useState("above");
  const [params, setParams] = useState<Record<string, string>>({});
  const [results, setResults] = useState<
    (ScanResultItem & { code?: string; name?: string })[]
  >([]);

  // CAPS state
  const [capsResults, setCapsResults] = useState<CapsResult[]>([]);

  // CGPC state
  const [nPools, setNPools] = useState("3");
  const [poolSize, setPoolSize] = useState("5");
  const [cgpcResults, setCgpcResults] = useState<CgpcPool[]>([]);

  // MARS state
  const [nRegimes, setNRegimes] = useState("3");
  const [marsResult, setMarsResult] = useState<MarsResult | null>(null);

  // MetaSearcher state
  const [metaNode, setMetaNode] = useState<SearchNode | null>(null);
  const [metaBest, setMetaBest] = useState<SearchNode | null>(null);
  const [metaCount, setMetaCount] = useState(0);
  const [metaSharpeInput, setMetaSharpeInput] = useState("1.5");
  const [metaRound, setMetaRound] = useState(0);

  useEffect(() => {
    invoke<IndicatorMeta[]>("list_indicators").then(setIndicators).catch(console.error);
    invoke<StockInfo[]>("query_stock_list").then(setStocks).catch(console.error);
  }, []);

  const selectedMeta = indicators.find((i) => i.name === indicator);

  const handleIndicatorChange = (name: string) => {
    setIndicator(name);
    const meta = indicators.find((i) => i.name === name);
    if (meta) {
      const p: Record<string, string> = {};
      meta.params.forEach((d) => {
        p[d.name] = String(d.default);
      });
      setParams(p);
    }
  };

  // ── Condition scan ──
  const runScan = useCallback(async () => {
    setLoading(true);
    setStatus("扫描中...");
    try {
      const paramMap: Record<string, number> = {};
      Object.entries(params).forEach(([k, v]) => {
        const n = parseFloat(v);
        if (!isNaN(n)) paramMap[k] = n;
      });

      const expr: Record<string, unknown> = {
        op: scanOp,
        children: [],
        indicator,
        params: paramMap,
        compare_op: scanOp === "COMPARE" ? compareOp : crossDir,
        value: scanOp === "COMPARE" ? parseFloat(threshold) || 0 : null,
      };

      const raw = await invoke<ScanResultItem[]>("run_scanner", {
        stockIds: stocks.map((s) => s.id),
        expr,
      });

      const stockMap = new Map(stocks.map((s) => [s.id, s]));
      const mapped = raw.map((r) => ({
        ...r,
        code: stockMap.get(r.stock_id)?.code,
        name: stockMap.get(r.stock_id)?.name,
      }));
      setResults(mapped);
      setStatus(`找到 ${mapped.length} 个匹配结果`);
    } catch (e) {
      console.error("Scanner error:", e);
      setStatus(`扫描失败: ${e}`);
    }
    setLoading(false);
  }, [scanOp, indicator, compareOp, threshold, crossDir, params, stocks]);

  // ── CAPS ──
  const runCaps = useCallback(async () => {
    setLoading(true);
    setStatus("CAPS 运行中...");
    try {
      const raw = await invoke<CapsResult[]>("run_caps_search", {
        stockIds: stocks.map((s) => s.id),
      });
      setCapsResults(raw);
      setStatus(`CAPS: ${raw.length} 个组合-策略对`);
    } catch (e) {
      console.error("CAPS error:", e);
      setStatus(`CAPS 失败: ${e}`);
    }
    setLoading(false);
  }, [stocks]);

  // ── CGPC ──
  const runCgpc = useCallback(async () => {
    setLoading(true);
    setStatus("CGPC 池构建中...");
    try {
      const raw = await invoke<CgpcPool[]>("run_cgpc_search", {
        stockIds: stocks.map((s) => s.id),
        nPools: parseInt(nPools) || 3,
        poolSize: parseInt(poolSize) || 5,
      });
      setCgpcResults(raw);
      setStatus(`CGPC: ${raw.length} 个池`);
    } catch (e) {
      console.error("CGPC error:", e);
      setStatus(`CGPC 失败: ${e}`);
    }
    setLoading(false);
  }, [stocks, nPools, poolSize]);

  // ── MARS ──
  const runMars = useCallback(async () => {
    setLoading(true);
    setStatus("MARS 体制检测中...");
    try {
      const raw = await invoke<MarsResult>("run_mars_search", {
        stockIds: stocks.map((s) => s.id),
        nRegimes: parseInt(nRegimes) || 3,
      });
      setMarsResult(raw);
      setStatus(`MARS: ${raw.n_regimes} 个体制, 当前体制 ${raw.current_regime}`);
    } catch (e) {
      console.error("MARS error:", e);
      setStatus(`MARS 失败: ${e}`);
    }
    setLoading(false);
  }, [stocks, nRegimes]);

  // ── MetaSearcher ──
  const metaSelect = useCallback(async () => {
    try {
      const node = await invoke<SearchNode | null>("run_metasearcher_select");
      setMetaNode(node);
      const best = await invoke<SearchNode | null>("get_metasearcher_best");
      setMetaBest(best);
      const count = await invoke<number>("get_metasearcher_count");
      setMetaCount(count);
      if (node) {
        setStatus(`MetaSearcher: 选中节点 #${node.round_num}`);
      } else {
        setStatus("MetaSearcher: 搜索空间已探索完毕");
      }
    } catch (e) {
      console.error("MetaSearcher error:", e);
      setStatus(`MetaSearcher 失败: ${e}`);
    }
  }, []);

  const metaRecord = useCallback(async () => {
    if (!metaNode) return;
    setLoading(true);
    try {
      const sharpe = parseFloat(metaSharpeInput) || 0;
      const round = metaRound + 1;
      await invoke("run_metasearcher_record", { node: metaNode, sharpe, round });
      setMetaRound(round);
      setMetaNode(null);
      const best = await invoke<SearchNode | null>("get_metasearcher_best");
      setMetaBest(best);
      const count = await invoke<number>("get_metasearcher_count");
      setMetaCount(count);
      setStatus(`MetaSearcher: 记录完成, 已探索 ${count} 个节点`);
    } catch (e) {
      console.error("MetaSearcher record error:", e);
      setStatus(`记录失败: ${e}`);
    }
    setLoading(false);
  }, [metaNode, metaSharpeInput, metaRound]);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: "#121212",
        color: "#D4D4D4",
        fontFamily: "monospace",
        fontSize: 13,
      }}
    >
      {/* Algorithm Tabs */}
      <div
        style={{
          display: "flex",
          borderBottom: "1px solid #2A2A2A",
          background: "#161616",
          flexShrink: 0,
        }}
      >
        {([
          ["condition", "条件扫描"],
          ["caps", "CAPS"],
          ["cgpc", "CGPC"],
          ["mars", "MARS"],
          ["meta", "Meta"],
        ] as [AlgoTab, string][]).map(([key, label]) => (
          <button
            key={key}
            onClick={() => setActiveTab(key)}
            style={{
              flex: 1,
              padding: "8px 6px",
              border: "none",
              background: activeTab === key ? "#121212" : "transparent",
              color: activeTab === key ? "#CCAA00" : "#858585",
              cursor: "pointer",
              fontSize: 12,
              fontFamily: "monospace",
              fontWeight: activeTab === key ? 600 : 400,
              borderBottom: activeTab === key ? "2px solid #CCAA00" : "2px solid transparent",
            }}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div style={{ padding: "12px", borderBottom: "1px solid #2A2A2A", background: "#161616", flexShrink: 0 }}>
        {activeTab === "condition" && (
          <>
            <div style={{ marginBottom: 10 }}>
              <button
                onClick={() => setScanOp("COMPARE")}
                style={{
                  ...opBtn,
                  background: scanOp === "COMPARE" ? "#CCAA00" : "#2A2A2A",
                  color: scanOp === "COMPARE" ? "#000" : "#858585",
                }}
              >
                指标比较
              </button>
              <button
                onClick={() => setScanOp("CROSS")}
                style={{
                  ...opBtn,
                  background: scanOp === "CROSS" ? "#CCAA00" : "#2A2A2A",
                  color: scanOp === "CROSS" ? "#000" : "#858585",
                  marginLeft: 8,
                }}
              >
                交叉信号
              </button>
            </div>
            <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
              <select
                value={indicator}
                onChange={(e) => handleIndicatorChange(e.target.value)}
                style={selectStyle}
              >
                {indicators.map((ind) => (
                  <option key={ind.name} value={ind.name}>
                    {ind.name_cn} ({ind.name.toUpperCase()})
                  </option>
                ))}
              </select>
              {scanOp === "COMPARE" ? (
                <>
                  <select
                    value={compareOp}
                    onChange={(e) => setCompareOp(e.target.value)}
                    style={selectStyle}
                  >
                    <option value=">">{">"} 大于</option>
                    <option value="<">{"<"} 小于</option>
                    <option value=">=">{">="} 大于等于</option>
                    <option value="<=">{"<="} 小于等于</option>
                    <option value="==">== 等于</option>
                    <option value="!=">!= 不等于</option>
                  </select>
                  <input
                    type="number"
                    value={threshold}
                    onChange={(e) => setThreshold(e.target.value)}
                    style={inputStyle}
                    placeholder="阈值"
                    step="any"
                  />
                </>
              ) : (
                <>
                  <select
                    value={crossDir}
                    onChange={(e) => setCrossDir(e.target.value)}
                    style={selectStyle}
                  >
                    <option value="above">上穿 (金叉)</option>
                    <option value="below">下穿 (死叉)</option>
                  </select>
                  <input
                    type="number"
                    value={threshold}
                    onChange={(e) => setThreshold(e.target.value)}
                    style={{ ...inputStyle, width: 80 }}
                    placeholder="阈值(可选)"
                    step="any"
                  />
                </>
              )}
              <button
                onClick={runScan}
                disabled={loading}
                style={{
                  ...opBtn,
                  background: "#CCAA00",
                  color: "#000",
                  fontWeight: 600,
                  opacity: loading ? 0.5 : 1,
                }}
              >
                {loading ? "扫描中..." : "开始扫描"}
              </button>
            </div>
            {selectedMeta && selectedMeta.params.length > 0 && (
              <div style={{ display: "flex", gap: 8, marginTop: 8, flexWrap: "wrap" }}>
                <span style={{ color: "#858585", fontSize: 12 }}>参数:</span>
                {selectedMeta.params.map((p) => (
                  <label key={p.name} style={{ fontSize: 12, color: "#858585" }}>
                    {p.name}
                    <input
                      type="number"
                      value={params[p.name] ?? p.default}
                      onChange={(e) =>
                        setParams((prev) => ({ ...prev, [p.name]: e.target.value }))
                      }
                      style={{ ...inputStyle, width: 56, marginLeft: 4 }}
                      step="any"
                    />
                  </label>
                ))}
              </div>
            )}
          </>
        )}

        {activeTab === "caps" && (
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <span style={{ color: "#858585", fontSize: 12 }}>
              CAPS — 协方差解析投影搜索：对所有策略×组合估计零回测夏普比率
            </span>
            <button
              onClick={runCaps}
              disabled={loading}
              style={{
                ...opBtn,
                background: "#CCAA00",
                color: "#000",
                fontWeight: 600,
                opacity: loading ? 0.5 : 1,
              }}
            >
              {loading ? "运行中..." : "▶ 运行 CAPS"}
            </button>
          </div>
        )}

        {activeTab === "cgpc" && (
          <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
            <label style={{ fontSize: 12, color: "#858585" }}>
              池数量:
              <input
                type="number"
                value={nPools}
                onChange={(e) => setNPools(e.target.value)}
                style={{ ...inputStyle, width: 56, marginLeft: 4 }}
                min="1"
                max="10"
              />
            </label>
            <label style={{ fontSize: 12, color: "#858585" }}>
              池大小:
              <input
                type="number"
                value={poolSize}
                onChange={(e) => setPoolSize(e.target.value)}
                style={{ ...inputStyle, width: 56, marginLeft: 4 }}
                min="2"
                max="20"
              />
            </label>
            <button
              onClick={runCgpc}
              disabled={loading}
              style={{
                ...opBtn,
                background: "#CCAA00",
                color: "#000",
                fontWeight: 600,
                opacity: loading ? 0.5 : 1,
              }}
            >
              {loading ? "运行中..." : "▶ 构建池"}
            </button>
            <span style={{ color: "#666666", fontSize: 11 }}>
              CGPC — 协方差引导池构建 (贪心选股)
            </span>
          </div>
        )}

        {activeTab === "mars" && (
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <label style={{ fontSize: 12, color: "#858585" }}>
              体制数量:
              <input
                type="number"
                value={nRegimes}
                onChange={(e) => setNRegimes(e.target.value)}
                style={{ ...inputStyle, width: 56, marginLeft: 4 }}
                min="2"
                max="6"
              />
            </label>
            <button
              onClick={runMars}
              disabled={loading}
              style={{
                ...opBtn,
                background: "#CCAA00",
                color: "#000",
                fontWeight: 600,
                opacity: loading ? 0.5 : 1,
              }}
            >
              {loading ? "运行中..." : "▶ 运行 MARS"}
            </button>
            <span style={{ color: "#666666", fontSize: 11 }}>
              MARS — 市场自适应体制切换 (K-Means + 策略映射)
            </span>
          </div>
        )}

        {activeTab === "meta" && (
          <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
            <button
              onClick={metaSelect}
              disabled={loading}
              style={{
                ...opBtn,
                background: "#CCAA00",
                color: "#000",
                fontWeight: 600,
                opacity: loading ? 0.5 : 1,
              }}
            >
              {loading ? "探索中..." : "选择节点"}
            </button>
            {metaNode && (
              <>
                <span style={{ color: "#858585", fontSize: 12 }}>反馈夏普:</span>
                <input
                  type="number"
                  value={metaSharpeInput}
                  onChange={(e) => setMetaSharpeInput(e.target.value)}
                  style={{ ...inputStyle, width: 60 }}
                  step="0.1"
                />
                <button
                  onClick={metaRecord}
                  style={{
                    ...opBtn,
                    background: "#26A69A",
                    color: "#000",
                    fontWeight: 600,
                  }}
                >
                  记录结果
                </button>
              </>
            )}
            <span style={{ color: "#666666", fontSize: 11 }}>
              已探索: {metaCount}/{270}
            </span>
          </div>
        )}
      </div>

      {/* Results Area */}
      <div style={{ flex: 1, overflow: "auto", padding: "8px" }}>
        <div
          style={{
            padding: "4px 8px",
            fontSize: 12,
            color: "#858585",
            display: "flex",
            justifyContent: "space-between",
          }}
        >
          <span>{status}</span>
          <span>共 {stocks.length} 只股票</span>
        </div>

        {/* Condition scan results */}
        {activeTab === "condition" && results.length > 0 && (
          <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}>
            <thead>
              <tr style={{ color: "#858585", borderBottom: "1px solid #2A2A2A" }}>
                <th style={thStyle}>排名</th>
                <th style={thStyle}>代码</th>
                <th style={thStyle}>名称</th>
                <th style={{ ...thStyle, textAlign: "right" }}>评分</th>
                <th style={thStyle}>信号</th>
              </tr>
            </thead>
            <tbody>
              {results.map((r, i) => (
                <tr
                  key={i}
                  style={{
                    borderBottom: "1px solid #1f1f3a",
                    background: i < 5 ? "rgba(204,170,0,0.05)" : undefined,
                  }}
                >
                  <td style={{ ...tdStyle, color: i < 3 ? "#CCAA00" : "#858585" }}>{i + 1}</td>
                  <td style={{ ...tdStyle, color: "#858585" }}>{r.code}</td>
                  <td style={tdStyle}>{r.name}</td>
                  <td style={{ ...tdStyle, textAlign: "right", color: "#26A69A" }}>
                    {r.score.toFixed(2)}
                  </td>
                  <td style={{ ...tdStyle, color: "#858585", fontSize: 11 }}>
                    {r.signals.join(", ")}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {/* CAPS results */}
        {activeTab === "caps" && capsResults.length > 0 && (
          <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}>
            <thead>
              <tr style={{ color: "#858585", borderBottom: "1px solid #2A2A2A" }}>
                <th style={thStyle}>排名</th>
                <th style={thStyle}>组合</th>
                <th style={thStyle}>策略</th>
                <th style={{ ...thStyle, textAlign: "right" }}>预估夏普</th>
                <th style={{ ...thStyle, textAlign: "right" }}>资产数</th>
              </tr>
            </thead>
            <tbody>
              {capsResults.map((r, i) => (
                <tr
                  key={i}
                  style={{
                    borderBottom: "1px solid #1f1f3a",
                    background: i < 5 ? "rgba(204,170,0,0.05)" : undefined,
                  }}
                >
                  <td style={{ ...tdStyle, color: i < 3 ? "#CCAA00" : "#858585" }}>{i + 1}</td>
                  <td style={{ ...tdStyle, color: "#858585" }}>{r.pool}</td>
                  <td style={tdStyle}>{strategyLabel(r.strategy)}</td>
                  <td
                    style={{
                      ...tdStyle,
                      textAlign: "right",
                      color: r.projected_sharpe > 1.0 ? "#26A69A" : "#EF5350",
                    }}
                  >
                    {r.projected_sharpe.toFixed(3)}
                  </td>
                  <td style={{ ...tdStyle, textAlign: "right", color: "#858585" }}>
                    {r.n_assets}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {/* CGPC results */}
        {activeTab === "cgpc" && cgpcResults.length > 0 && (
          <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}>
            <thead>
              <tr style={{ color: "#858585", borderBottom: "1px solid #2A2A2A" }}>
                <th style={thStyle}>池名称</th>
                <th style={{ ...thStyle, textAlign: "right" }}>资产数</th>
                <th style={{ ...thStyle, textAlign: "right" }}>平均质量</th>
                <th style={{ ...thStyle, textAlign: "right" }}>平均相关</th>
              </tr>
            </thead>
            <tbody>
              {cgpcResults.map((r, i) => (
                <tr
                  key={i}
                  style={{
                    borderBottom: "1px solid #1f1f3a",
                    background: i < 3 ? "rgba(204,170,0,0.05)" : undefined,
                  }}
                >
                  <td style={{ ...tdStyle, color: "#CCAA00" }}>{r.name}</td>
                  <td style={{ ...tdStyle, textAlign: "right", color: "#858585" }}>
                    {r.indices.length}
                  </td>
                  <td
                    style={{
                      ...tdStyle,
                      textAlign: "right",
                      color: r.avg_quality > 0.5 ? "#26A69A" : "#EF5350",
                    }}
                  >
                    {r.avg_quality.toFixed(3)}
                  </td>
                  <td
                    style={{
                      ...tdStyle,
                      textAlign: "right",
                      color: r.avg_corr < 0.5 ? "#26A69A" : "#CCAA00",
                    }}
                  >
                    {r.avg_corr.toFixed(3)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {/* MARS results */}
        {activeTab === "mars" && marsResult && (
          <div style={{ padding: "12px 0" }}>
            <div
              style={{
                display: "flex",
                gap: 24,
                marginBottom: 16,
                flexWrap: "wrap",
                fontSize: 13,
              }}
            >
              <div>
                <span style={{ color: "#858585" }}>体制数: </span>
                <span style={{ color: "#CCAA00", fontWeight: 600 }}>
                  {marsResult.n_regimes}
                </span>
              </div>
              <div>
                <span style={{ color: "#858585" }}>当前体制: </span>
                <span style={{ color: "#26A69A", fontWeight: 600 }}>
                  {marsResult.current_regime}
                </span>
              </div>
              <div>
                <span style={{ color: "#858585" }}>推荐策略: </span>
                <span style={{ color: "#CCAA00", fontWeight: 600 }}>
                  {strategyLabel(marsResult.recommended_strategy)}
                </span>
              </div>
            </div>
            <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}>
              <thead>
                <tr style={{ color: "#858585", borderBottom: "1px solid #2A2A2A" }}>
                  <th style={thStyle}>体制</th>
                  <th style={thStyle}>推荐策略</th>
                  <th style={{ ...thStyle, textAlign: "right" }}>样本数</th>
                </tr>
              </thead>
              <tbody>
                {Array.from({ length: marsResult.n_regimes }, (_, r) => (
                  <tr
                    key={r}
                    style={{
                      borderBottom: "1px solid #1f1f3a",
                      background:
                        r === marsResult.current_regime
                          ? "rgba(204,170,0,0.08)"
                          : undefined,
                    }}
                  >
                    <td
                      style={{
                        ...tdStyle,
                        color: r === marsResult.current_regime ? "#CCAA00" : "#858585",
                        fontWeight: r === marsResult.current_regime ? 600 : 400,
                      }}
                    >
                      {r === marsResult.current_regime ? `体制 ${r} ◀` : `体制 ${r}`}
                    </td>
                    <td style={{ ...tdStyle, color: "#858585" }}>
                      {strategyLabel(marsResult.regime_strategies[r] ?? "-")}
                    </td>
                    <td style={{ ...tdStyle, textAlign: "right", color: "#858585" }}>
                      {marsResult.regime_sizes[r] ?? 0}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {/* MetaSearcher results */}
        {activeTab === "meta" && (
          <div style={{ padding: "12px 0" }}>
            {metaNode && (
              <div
                style={{
                  padding: "12px",
                  background: "#161616",
                  borderRadius: 6,
                  border: "1px solid #2A2A2A",
                  marginBottom: 12,
                  fontSize: 13,
                }}
              >
                <div style={{ color: "#CCAA00", fontWeight: 600, marginBottom: 8 }}>
                  当前选中节点
                </div>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
                  <NodeField label="Alpha层级" value={metaNode.alpha_tier} />
                  <NodeField label="目标函数" value={metaNode.objective} />
                  <NodeField label="集成方法" value={metaNode.ensemble} />
                  <NodeField label="因子子集" value={metaNode.factor_subset} />
                  <NodeField label="轮次" value={String(metaNode.round_num)} />
                  <NodeField label="状态" value={metaNode.status} />
                </div>
              </div>
            )}
            {metaBest && (
              <div
                style={{
                  padding: "12px",
                  background: "#161616",
                  borderRadius: 6,
                  border: "1px solid #2A2A2A",
                  fontSize: 13,
                }}
              >
                <div style={{ color: "#26A69A", fontWeight: 600, marginBottom: 8 }}>
                  历史最优节点
                </div>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6 }}>
                  <NodeField label="Alpha层级" value={metaBest.alpha_tier} />
                  <NodeField label="目标函数" value={metaBest.objective} />
                  <NodeField label="集成方法" value={metaBest.ensemble} />
                  <NodeField label="因子子集" value={metaBest.factor_subset} />
                  <NodeField
                    label="最佳夏普"
                    value={
                      metaBest.result_sharpe != null
                        ? metaBest.result_sharpe.toFixed(3)
                        : "-"
                    }
                  />
                  <NodeField label="探索轮次" value={String(metaBest.round_num)} />
                </div>
              </div>
            )}
          </div>
        )}

        {/* Empty state */}
        {activeTab === "condition" && results.length === 0 && status && !loading && (
          <div style={{ textAlign: "center", color: "#555", marginTop: 40, fontSize: 14 }}>
            未找到匹配结果
          </div>
        )}
      </div>
    </div>
  );
}

function NodeField({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span style={{ color: "#858585", fontSize: 11 }}>{label}: </span>
      <span style={{ color: "#858585", fontSize: 12 }}>{value}</span>
    </div>
  );
}

function strategyLabel(s: string): string {
  const map: Record<string, string> = {
    risk_parity: "风险平价",
    min_variance: "最小方差",
    hierarchical_rp: "分层风险平价",
    momentum: "动量策略",
    vol_target: "波动率目标",
    etf_rotation: "ETF轮动",
    defensive: "防御轮动",
  };
  return map[s] ?? s;
}

const opBtn: React.CSSProperties = {
  padding: "5px 14px",
  border: "none",
  borderRadius: 4,
  cursor: "pointer",
  fontSize: 13,
  fontFamily: "monospace",
};

const selectStyle: React.CSSProperties = {
  background: "#0C0C0C",
  color: "#D4D4D4",
  border: "1px solid #2A2A2A",
  padding: "4px 8px",
  borderRadius: 4,
  fontSize: 13,
  fontFamily: "monospace",
  maxWidth: 200,
};

const inputStyle: React.CSSProperties = {
  background: "#0C0C0C",
  color: "#D4D4D4",
  border: "1px solid #2A2A2A",
  padding: "4px 8px",
  borderRadius: 4,
  fontSize: 13,
  fontFamily: "monospace",
  width: 90,
};

const thStyle: React.CSSProperties = {
  padding: "6px 8px",
  textAlign: "left",
  fontWeight: 600,
};

const tdStyle: React.CSSProperties = {
  padding: "6px 8px",
};

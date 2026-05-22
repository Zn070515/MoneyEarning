import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PatternDef {
  name: string;
  name_cn: string;
  category: string;
}

interface PatternResult {
  name: string;
  name_cn: string;
  start_idx: number;
  end_idx: number;
  confidence: number;
  direction: string;
  description: string;
}

type DirectionFilter = "all" | "bullish" | "bearish";

const CATEGORY_LABELS: Record<string, string> = {
  single_line: "单根K线",
  double_line: "双根K线",
  triple_line: "三根K线",
  chart_patterns: "图表形态",
};

const CATEGORY_ORDER = ["single_line", "double_line", "triple_line", "chart_patterns"];

export function PatternPanel({ stockId }: { stockId: number | null }) {
  const [patterns, setPatterns] = useState<PatternDef[]>([]);
  const [results, setResults] = useState<PatternResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState("");
  const [viewMode, setViewMode] = useState<"defs" | "results">("results");
  const [dirFilter, setDirFilter] = useState<DirectionFilter>("all");

  useEffect(() => {
    invoke<PatternDef[]>("list_patterns")
      .then(setPatterns)
      .catch(() => {});
  }, []);

  const runScan = useCallback(async () => {
    if (!stockId) return;
    setLoading(true);
    setStatus("扫描中...");
    try {
      const res = await invoke<PatternResult[]>("scan_all_patterns", { stockId });
      setResults(res);
      setStatus(`检测到 ${res.length} 个形态`);
      setViewMode("results");
    } catch (e) {
      setStatus(`扫描失败: ${e}`);
    }
    setLoading(false);
  }, [stockId]);

  // Group pattern definitions by category
  const grouped: Record<string, PatternDef[]> = {};
  patterns.forEach((p) => {
    if (!grouped[p.category]) grouped[p.category] = [];
    grouped[p.category].push(p);
  });

  const filtered =
    dirFilter === "all"
      ? results
      : results.filter((r) => r.direction === dirFilter);

  if (!stockId) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          color: "#555",
          fontFamily: "monospace",
          fontSize: 14,
        }}
      >
        请先选择一只股票
      </div>
    );
  }

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: "#1a1a2e",
        color: "#ccc",
        fontFamily: "monospace",
        fontSize: 13,
      }}
    >
      {/* Header with scan button */}
      <div
        style={{
          padding: "10px 12px",
          borderBottom: "1px solid #2a2a4a",
          background: "#16213e",
          display: "flex",
          alignItems: "center",
          gap: 8,
          flexShrink: 0,
        }}
      >
        <button
          onClick={runScan}
          disabled={loading}
          style={{
            padding: "5px 14px",
            background: loading ? "#8a7a3a" : "#fbbf24",
            color: "#000",
            border: "none",
            borderRadius: 4,
            cursor: loading ? "not-allowed" : "pointer",
            fontFamily: "monospace",
            fontSize: 12,
            fontWeight: 600,
          }}
        >
          {loading ? "扫描中..." : "▶ 扫描形态"}
        </button>
        <span style={{ color: "#888", fontSize: 12 }}>{status}</span>
      </div>

      {/* View toggle + direction filter */}
      <div
        style={{
          padding: "6px 12px",
          borderBottom: "1px solid #2a2a4a",
          background: "#16213e",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexShrink: 0,
        }}
      >
        <div style={{ display: "flex", gap: 4 }}>
          {([
            ["results", "扫描结果"],
            ["defs", "形态库"],
          ] as const).map(([k, label]) => (
            <button
              key={k}
              onClick={() => setViewMode(k)}
              style={{
                padding: "4px 12px",
                background: viewMode === k ? "#fbbf24" : "transparent",
                color: viewMode === k ? "#000" : "#888",
                border: "none",
                borderRadius: 3,
                cursor: "pointer",
                fontFamily: "monospace",
                fontSize: 11,
                fontWeight: viewMode === k ? 600 : 400,
              }}
            >
              {label}
            </button>
          ))}
        </div>
        {viewMode === "results" && results.length > 0 && (
          <div style={{ display: "flex", gap: 4 }}>
            {([
              ["all", "全部"],
              ["bullish", "看涨"],
              ["bearish", "看跌"],
            ] as [DirectionFilter, string][]).map(([k, label]) => (
              <button
                key={k}
                onClick={() => setDirFilter(k)}
                style={{
                  padding: "3px 10px",
                  background: dirFilter === k ? "#3a3a5a" : "transparent",
                  color: dirFilter === k ? "#fff" : "#888",
                  border: "1px solid #3a3a5a",
                  borderRadius: 3,
                  cursor: "pointer",
                  fontFamily: "monospace",
                  fontSize: 10,
                }}
              >
                {label}
                {k !== "all" &&
                  ` (${results.filter((r) => r.direction === k).length})`}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {viewMode === "results" && (
          <div>
            {filtered.length === 0 ? (
              <div
                style={{
                  textAlign: "center",
                  color: "#555",
                  padding: 40,
                  fontSize: 13,
                }}
              >
                {results.length === 0
                  ? "点击「扫描形态」检测K线形态"
                  : "无匹配的形态方向"}
              </div>
            ) : (
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr
                    style={{
                      color: "#888",
                      borderBottom: "1px solid #2a2a4a",
                      position: "sticky",
                      top: 0,
                      background: "#1a1a2e",
                    }}
                  >
                    <th style={{ ...thS, width: 90 }}>名称</th>
                    <th style={{ ...thS, width: 50 }}>方向</th>
                    <th style={{ ...thS, width: 55, textAlign: "right" }}>置信度</th>
                    <th style={{ ...thS, width: 60, textAlign: "right" }}>位置</th>
                    <th style={thS}>说明</th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((r, i) => (
                    <tr
                      key={i}
                      style={{
                        borderBottom: "1px solid #1f1f3a",
                        background:
                          r.confidence > 0.8
                            ? "rgba(251,191,36,0.05)"
                            : undefined,
                      }}
                    >
                      <td
                        style={{
                          ...tdS,
                          color:
                            r.direction === "bullish"
                              ? "#ef4444"
                              : r.direction === "bearish"
                                ? "#22c55e"
                                : "#fbbf24",
                          fontWeight: 600,
                        }}
                      >
                        {r.name_cn}
                      </td>
                      <td style={tdS}>
                        <DirectionBadge dir={r.direction} />
                      </td>
                      <td style={{ ...tdS, textAlign: "right" }}>
                        <ConfidenceBar confidence={r.confidence} />
                      </td>
                      <td style={{ ...tdS, textAlign: "right", color: "#888" }}>
                        {r.start_idx}–{r.end_idx}
                      </td>
                      <td style={{ ...tdS, color: "#888", fontSize: 11 }}>
                        {r.description}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}

        {viewMode === "defs" && (
          <div style={{ padding: "8px 12px" }}>
            {CATEGORY_ORDER.map((cat) => {
              const g = grouped[cat];
              if (!g || g.length === 0) return null;
              return (
                <div key={cat} style={{ marginBottom: 16 }}>
                  <div
                    style={{
                      color: "#fbbf24",
                      fontSize: 12,
                      fontWeight: 600,
                      marginBottom: 8,
                      borderBottom: "1px solid #2a2a4a",
                      paddingBottom: 4,
                    }}
                  >
                    {CATEGORY_LABELS[cat] || cat} ({g.length})
                  </div>
                  <div
                    style={{
                      display: "grid",
                      gridTemplateColumns: "1fr 1fr",
                      gap: 4,
                    }}
                  >
                    {g.map((p) => (
                      <div
                        key={p.name}
                        style={{
                          padding: "4px 8px",
                          background: "#16213e",
                          borderRadius: 4,
                          fontSize: 11,
                          color: "#aaa",
                        }}
                      >
                        {p.name_cn}
                        <span style={{ color: "#555", marginLeft: 6, fontSize: 10 }}>
                          {p.name}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function DirectionBadge({ dir }: { dir: string }) {
  const isBullish = dir === "bullish";
  const isBearish = dir === "bearish";
  return (
    <span
      style={{
        padding: "1px 6px",
        borderRadius: 3,
        fontSize: 10,
        fontWeight: 600,
        background: isBullish
          ? "rgba(239,68,68,0.15)"
          : isBearish
            ? "rgba(34,197,94,0.15)"
            : "rgba(251,191,36,0.15)",
        color: isBullish ? "#ef4444" : isBearish ? "#22c55e" : "#fbbf24",
      }}
    >
      {isBullish ? "看涨" : isBearish ? "看跌" : "中性"}
    </span>
  );
}

function ConfidenceBar({ confidence }: { confidence: number }) {
  const pct = (confidence * 100).toFixed(0);
  const color =
    confidence > 0.8 ? "#22c55e" : confidence > 0.6 ? "#fbbf24" : "#888";
  return (
    <span style={{ color, fontSize: 11 }}>
      {pct}%
    </span>
  );
}

const thS: React.CSSProperties = {
  padding: "6px 8px",
  textAlign: "left",
  fontSize: 11,
  fontWeight: 600,
};

const tdS: React.CSSProperties = {
  padding: "5px 8px",
  fontSize: 12,
};

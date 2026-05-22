import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { OHLCV, IndicatorData } from "@me/chart-engine";

interface ParamDef {
  name: string;
  default: number;
  min: number;
  max: number;
  step: number;
}

interface IndicatorMeta {
  name: string;
  name_cn: string;
  category: string;
  params: ParamDef[];
  is_free: boolean;
  tdx_equivalent: string | null;
  description: string;
}

interface IndicatorOutputRaw {
  name: string;
  values: { F64?: number[]; I32?: number[]; Bool?: boolean[]; String?: string[] };
  style: string | { Band: { upper: { F64: number[] }; lower: { F64: number[] } } };
}

interface IndicatorSelectorProps {
  data: OHLCV[];
  activeIndicators: IndicatorData[];
  onChange: (indicators: IndicatorData[]) => void;
}

const CATEGORIES: Record<string, string> = {
  "均线/叠加类": "均线/叠加",
  "趋势/方向类": "趋势/方向",
  "动量/振荡类": "动量/振荡",
  "波动/通道类": "波动/通道",
  "成交量类": "成交量",
  "统计/分布类": "统计/分布",
  "周期/傅里叶类": "周期/傅里叶",
  "绩效/回撤类": "绩效/回撤",
  "特色工具类": "特色工具",
  "K线形态识别": "K线形态",
};

function toOutputStyle(style: string | object): "line" | "histogram" | "dots" | "band" {
  if (typeof style === "string") {
    const s = style.toLowerCase();
    if (s === "histogram") return "histogram";
    if (s === "dots") return "dots";
    if (s === "band") return "band";
    return "line";
  }
  return "band";
}

const COLORS = [
  "#fbbf24", "#60a5fa", "#34d399", "#f87171", "#a78bfa",
  "#fb923c", "#22d3ee", "#e879f9", "#2dd4bf", "#facc15",
];

export function IndicatorSelector({
  data, activeIndicators, onChange,
}: IndicatorSelectorProps) {
  const [indicators, setIndicators] = useState<IndicatorMeta[]>([]);
  const [selectedCat, setSelectedCat] = useState<string>("均线/叠加类");
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [params, setParams] = useState<Record<string, Record<string, number>>>({});
  const [computing, setComputing] = useState(false);

  useEffect(() => {
    invoke<IndicatorMeta[]>("list_indicators").then(setIndicators).catch(console.error);
  }, []);

  const categories = [...new Set(indicators.map(m => m.category))];

  const handleToggle = async (meta: IndicatorMeta) => {
    const isActive = activeIndicators.some(i => i.name === meta.name);

    if (isActive) {
      onChange(activeIndicators.filter(i => i.name !== meta.name));
      return;
    }

    setComputing(true);
    try {
      const rawParams = params[meta.name] ?? {};
      const result = await invoke<IndicatorOutputRaw[]>("compute_indicator", {
        name: meta.name,
        data,
        params: rawParams,
      });

      const indicatorData: IndicatorData[] = result.map((r, idx) => ({
        name: r.name,
        values: new Float64Array(r.values.F64 ?? r.values.I32?.map(Number) ?? []),
        style: toOutputStyle(r.style),
        color: COLORS[(activeIndicators.length + idx) % COLORS.length],
      }));

      onChange([...activeIndicators, ...indicatorData]);
    } catch (e) {
      console.error("Compute indicator failed:", e);
    }
    setComputing(false);
  };

  const handleParamChange = (indName: string, paramName: string, value: number) => {
    setParams(prev => ({
      ...prev,
      [indName]: { ...(prev[indName] ?? {}), [paramName]: value },
    }));
  };

  const initParams = (meta: IndicatorMeta) => {
    if (!params[meta.name]) {
      const defaults: Record<string, number> = {};
      meta.params.forEach(p => { defaults[p.name] = p.default; });
      setParams(prev => ({ ...prev, [meta.name]: defaults }));
    }
  };

  return (
    <div style={{
      background: "#16213e", color: "#ccc", fontFamily: "monospace",
      fontSize: 12, height: "100%", display: "flex", flexDirection: "column",
      overflow: "hidden",
    }}>
      <div style={{
        padding: "10px 12px", borderBottom: "1px solid #2a2a4a",
        fontWeight: 600, color: "#fff", fontSize: 13,
      }}>
        技术指标 {computing && <span style={{ color: "#fbbf24" }}>计算中...</span>}
        <span style={{ color: "#888", fontSize: 11, marginLeft: 8 }}>
          ({indicators.length})
        </span>
      </div>

      {/* Category tabs */}
      <div style={{
        display: "flex", flexWrap: "wrap", gap: 2,
        padding: "6px 8px", borderBottom: "1px solid #2a2a4a",
      }}>
        {categories.map(cat => (
          <button key={cat} onClick={() => setSelectedCat(cat)} style={{
            background: selectedCat === cat ? "#fbbf24" : "#1a1a2e",
            color: selectedCat === cat ? "#000" : "#888",
            border: "none", padding: "3px 8px", borderRadius: 3,
            cursor: "pointer", fontSize: 11, fontFamily: "monospace",
          }}>
            {CATEGORIES[cat] ?? cat}
          </button>
        ))}
      </div>

      {/* Indicator list */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {indicators
          .filter(m => m.category === selectedCat)
          .map(meta => {
            const isActive = activeIndicators.some(i => i.name === meta.name);
            const isExpanded = expanded[meta.name];

            return (
              <div key={meta.name} style={{ borderBottom: "1px solid #1a1a2e" }}>
                <div
                  onClick={() => handleToggle(meta)}
                  style={{
                    padding: "7px 12px", cursor: "pointer",
                    display: "flex", justifyContent: "space-between",
                    alignItems: "center",
                    background: isActive ? "#2a3a5e" : "transparent",
                  }}
                >
                  <div>
                    <span style={{ color: isActive ? "#fbbf24" : "#ccc" }}>
                      {meta.name_cn}
                    </span>
                    <span style={{ color: "#666", marginLeft: 6, fontSize: 10 }}>
                      {meta.name}
                    </span>
                  </div>
                  <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    {!meta.is_free && (
                      <span style={{ color: "#fbbf24", fontSize: 10 }}>PRO</span>
                    )}
                    <button
                      onClick={e => {
                        e.stopPropagation();
                        initParams(meta);
                        setExpanded(prev => ({ ...prev, [meta.name]: !prev[meta.name] }));
                      }}
                      style={{
                        background: "none", border: "none",
                        color: "#666", cursor: "pointer", fontSize: 12,
                      }}>
                      ⚙
                    </button>
                  </div>
                </div>

                {/* Description */}
                {isExpanded && (
                  <div style={{ padding: "6px 12px", background: "#1a1a2e" }}>
                    <div style={{ color: "#888", fontSize: 11, marginBottom: 6 }}>
                      {meta.description}
                    </div>

                    {/* Params */}
                    {meta.params.map(p => (
                      <div key={p.name} style={{
                        display: "flex", alignItems: "center", gap: 8,
                        marginBottom: 4,
                      }}>
                        <label style={{ color: "#aaa", width: 80, fontSize: 11 }}>
                          {p.name}
                        </label>
                        <input
                          type="number"
                          value={params[meta.name]?.[p.name] ?? p.default}
                          min={p.min}
                          max={p.max}
                          step={p.step}
                          onChange={e => handleParamChange(meta.name, p.name, parseFloat(e.target.value) || p.default)}
                          style={{
                            width: 70, background: "#0f0f23", border: "1px solid #3a3a5a",
                            color: "#fff", padding: "2px 6px", borderRadius: 3,
                            fontSize: 11, fontFamily: "monospace", outline: "none",
                          }}
                        />
                      </div>
                    ))}
                  </div>
                )}
              </div>
            );
          })}
      </div>
    </div>
  );
}

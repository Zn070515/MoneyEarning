import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StockInfo {
  id: number;
  code: string;
  name: string;
}

interface ProfileLevel {
  price: number;
  volume: number;
  is_poc: boolean;
}

interface VolumeProfileResult {
  levels: ProfileLevel[];
  poc: number;
  vah: number;
  val: number;
}

interface DistributionResult {
  price_levels: number[];
  chip_volume: number[];
  avg_cost: number;
  weighted_avg_cost: number;
}

type SubTab = "profile" | "chip" | "sr";

export function DistributionPanel({ stockId }: { stockId: number | null }) {
  const [subTab, setSubTab] = useState<SubTab>("profile");
  const [profile, setProfile] = useState<VolumeProfileResult | null>(null);
  const [chipDist, setChipDist] = useState<DistributionResult | null>(null);
  const [srLevels, setSrLevels] = useState<[number, number, string][]>([]);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState("");

  const loadProfile = useCallback(async (sid: number) => {
    setLoading(true);
    setStatus("计算成交量分布...");
    try {
      const [vp, cd, sr] = await Promise.all([
        invoke<VolumeProfileResult>("compute_volume_profile", { stockId: sid, numBuckets: 100 }),
        invoke<DistributionResult>("compute_chip_distribution", { stockId: sid }),
        invoke<[number, number, string][]>("compute_sr_levels", { stockId: sid, numLevels: 50 }),
      ]);
      setProfile(vp);
      setChipDist(cd);
      setSrLevels(sr);
      setStatus("");
    } catch (e) {
      console.error("Distribution error:", e);
      setStatus(`计算失败: ${e}`);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    if (stockId) {
      loadProfile(stockId);
    } else {
      setProfile(null);
      setChipDist(null);
      setSrLevels([]);
    }
  }, [stockId, loadProfile]);

  if (!stockId) {
    return (
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "center",
        height: "100%", color: "#555", fontFamily: "monospace", fontSize: 14,
      }}>
        请先选择一只股票
      </div>
    );
  }

  return (
    <div style={{
      display: "flex", flexDirection: "column", height: "100%",
      background: "#1a1a2e", color: "#ccc", fontFamily: "monospace", fontSize: 13,
    }}>
      {/* Sub tabs */}
      <div style={{
        display: "flex", borderBottom: "1px solid #2a2a4a",
        background: "#16213e", flexShrink: 0,
      }}>
        {([
          ["profile", "成交量分布"],
          ["chip", "筹码分布"],
          ["sr", "支撑/阻力"],
        ] as [SubTab, string][]).map(([k, label]) => (
          <button key={k} onClick={() => setSubTab(k)} style={{
            flex: 1, padding: "8px 12px", border: "none",
            background: subTab === k ? "#1a1a2e" : "transparent",
            color: subTab === k ? "#fbbf24" : "#888",
            cursor: "pointer", fontSize: 13, fontFamily: "monospace",
            fontWeight: subTab === k ? 600 : 400,
            borderBottom: subTab === k ? "2px solid #fbbf24" : "2px solid transparent",
          }}>
            {label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: "auto", padding: "12px" }}>
        {loading && <div style={{ color: "#fbbf24", textAlign: "center" }}>计算中...</div>}
        {status && !loading && <div style={{ color: "#ef4444", textAlign: "center" }}>{status}</div>}

        {subTab === "profile" && profile && <VolumeProfileChart profile={profile} />}
        {subTab === "chip" && chipDist && <ChipDistChart dist={chipDist} />}
        {subTab === "sr" && <SRLevelsList levels={srLevels} />}
      </div>
    </div>
  );
}

function VolumeProfileChart({ profile }: { profile: VolumeProfileResult }) {
  const { levels, poc, vah, val } = profile;
  if (levels.length === 0) return <div style={{ color: "#555" }}>无数据</div>;

  const maxVol = Math.max(...levels.map(l => l.volume));
  const minP = levels[0].price;
  const maxP = levels[levels.length - 1].price;
  const priceRange = maxP - minP || 1;
  const barWidth = 200;

  // Find POC level for annotation
  const pocLevel = levels.find(l => l.is_poc);

  return (
    <div>
      <div style={{ fontWeight: 600, marginBottom: 8, color: "#fbbf24" }}>成交量分布图</div>

      <div style={{ fontSize: 12, color: "#888", marginBottom: 12 }}>
        <span>POC: <span style={{ color: "#fbbf24" }}>{poc.toFixed(2)}</span></span>
        <span style={{ marginLeft: 16 }}>VAH: <span style={{ color: "#ef4444" }}>{vah.toFixed(2)}</span></span>
        <span style={{ marginLeft: 16 }}>VAL: <span style={{ color: "#22c55e" }}>{val.toFixed(2)}</span></span>
        <span style={{ marginLeft: 16 }}>价值区: {val.toFixed(2)} ~ {vah.toFixed(2)}</span>
      </div>

      <svg width={280} height={Math.min(levels.length * 4, 400)} style={{ background: "#0f0f23" }}>
        {levels.map((l, i) => {
          const w = maxVol > 0 ? (l.volume / maxVol) * barWidth : 0;
          const y = ((l.price - minP) / priceRange) * (levels.length * 4);
          const fill = l.is_poc ? "#fbbf24" : (l.price >= val && l.price <= vah ? "#3b82f6" : "#3a3a5a");
          return (
            <g key={i}>
              <rect x={0} y={y - 1.5} width={w} height={3} fill={fill} rx={1} />
              <text x={w + 4} y={y + 2} fill="#888" fontSize={9} fontFamily="monospace">
                {l.price.toFixed(1)}
              </text>
            </g>
          );
        })}

        {/* VAH/VAL lines */}
        <line x1={0} y1={((vah - minP) / priceRange) * levels.length * 4}
          x2={barWidth} y2={((vah - minP) / priceRange) * levels.length * 4}
          stroke="#ef4444" strokeDasharray="4 2" strokeWidth={1} />
        <line x1={0} y1={((val - minP) / priceRange) * levels.length * 4}
          x2={barWidth} y2={((val - minP) / priceRange) * levels.length * 4}
          stroke="#22c55e" strokeDasharray="4 2" strokeWidth={1} />
      </svg>

      <div style={{ fontSize: 11, color: "#666", marginTop: 8 }}>
        蓝条 = 70%价值区域 · 金条 = POC · 红线 = VAH · 绿线 = VAL
      </div>
    </div>
  );
}

function ChipDistChart({ dist }: { dist: DistributionResult }) {
  const { price_levels, chip_volume, avg_cost, weighted_avg_cost } = dist;
  if (price_levels.length === 0) return <div style={{ color: "#555" }}>无数据</div>;

  const maxChip = Math.max(...chip_volume);
  const h = 200;
  const w = 260;
  const barW = (w / price_levels.length) * 0.9;
  const priceRange = price_levels[price_levels.length - 1] - price_levels[0] || 1;
  const costY = h - ((avg_cost - price_levels[0]) / priceRange) * h + 10;

  return (
    <div>
      <div style={{ fontWeight: 600, marginBottom: 8, color: "#fbbf24" }}>筹码分布</div>

      <div style={{ fontSize: 12, color: "#888", marginBottom: 12 }}>
        <span>平均成本: <span style={{ color: "#fbbf24" }}>{avg_cost.toFixed(2)}</span></span>
        <span style={{ marginLeft: 16 }}>加权均价: <span style={{ color: "#3b82f6" }}>
          {weighted_avg_cost.toFixed(2)}</span>
        </span>
      </div>

      <svg width={w + 40} height={h + 30} style={{ background: "#0f0f23" }}>
        {price_levels.map((p, i) => {
          const bh = maxChip > 0 ? (chip_volume[i] / maxChip) * h : 0;
          const x = (i / price_levels.length) * w + 20;
          const isCost = p < avg_cost * 1.02 && p > avg_cost * 0.98;
          return (
            <rect key={i} x={x - barW / 2} y={h - bh + 10} width={barW} height={bh}
              fill={isCost ? "#fbbf24" : "#3b82f6"} opacity={0.7} rx={1} />
          );
        })}
        <line x1={20} y1={costY} x2={w + 20} y2={costY}
          stroke="#fbbf24" strokeDasharray="4 2" strokeWidth={1} />
      </svg>

      <div style={{ fontSize: 11, color: "#666", marginTop: 8 }}>
        蓝柱 = 筹码量 · 金条 = 平均成本线 · 模拟换手率衰减
      </div>
    </div>
  );
}

function SRLevelsList({ levels }: { levels: [number, number, string][] }) {
  if (levels.length === 0) {
    return <div style={{ color: "#555", textAlign: "center", marginTop: 40 }}>无显著支撑/阻力位</div>;
  }

  const resistance = levels.filter(l => l[2] === "resistance");
  const support = levels.filter(l => l[2] === "support");
  const poc = levels.filter(l => l[2] === "poc");

  return (
    <div>
      <div style={{ fontWeight: 600, marginBottom: 12, color: "#fbbf24" }}>
        支撑/阻力位 (高量节点 {">"} 1.5x 均值)
      </div>

      {resistance.length > 0 && (
        <div style={{ marginBottom: 12 }}>
          <div style={{ color: "#ef4444", fontSize: 12, fontWeight: 600, marginBottom: 6 }}>
            阻力位
          </div>
          {resistance.map((l, i) => (
            <div key={i} style={{
              display: "flex", justifyContent: "space-between",
              padding: "4px 8px", background: "rgba(239,68,68,0.1)",
              borderRadius: 4, marginBottom: 4, fontSize: 12,
            }}>
              <span style={{ color: "#ef4444" }}>{l[0].toFixed(2)}</span>
              <span style={{ color: "#888" }}>量: {l[1].toFixed(0)}</span>
            </div>
          ))}
        </div>
      )}

      {support.length > 0 && (
        <div style={{ marginBottom: 12 }}>
          <div style={{ color: "#22c55e", fontSize: 12, fontWeight: 600, marginBottom: 6 }}>
            支撑位
          </div>
          {support.map((l, i) => (
            <div key={i} style={{
              display: "flex", justifyContent: "space-between",
              padding: "4px 8px", background: "rgba(34,197,94,0.1)",
              borderRadius: 4, marginBottom: 4, fontSize: 12,
            }}>
              <span style={{ color: "#22c55e" }}>{l[0].toFixed(2)}</span>
              <span style={{ color: "#888" }}>量: {l[1].toFixed(0)}</span>
            </div>
          ))}
        </div>
      )}

      {poc.length > 0 && (
        <div>
          <div style={{ color: "#fbbf24", fontSize: 12, fontWeight: 600, marginBottom: 6 }}>
            POC (最大成交量节点)
          </div>
          {poc.map((l, i) => (
            <div key={i} style={{
              display: "flex", justifyContent: "space-between",
              padding: "4px 8px", background: "rgba(251,191,36,0.1)",
              borderRadius: 4, marginBottom: 4, fontSize: 12,
            }}>
              <span style={{ color: "#fbbf24" }}>{l[0].toFixed(2)}</span>
              <span style={{ color: "#888" }}>量: {l[1].toFixed(0)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

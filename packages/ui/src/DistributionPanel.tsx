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

interface ConcentrationOutput {
  cr5: number;
  cr10: number;
  cr20: number;
  trend: number;
  description: string;
}

interface ProfitLossOutput {
  profit_pct: number;
  loss_pct: number;
  avg_cost: number;
  weighted_avg_cost: number;
  last_price: number;
}

interface FrameOutput {
  date: string;
  price_levels: number[];
  chip_volume: number[];
  avg_cost: number;
  profit_pct: number;
  loss_pct: number;
}

type SubTab = "profile" | "chip" | "sr" | "concentration" | "plratio" | "history";

export function DistributionPanel({ stockId }: { stockId: number | null }) {
  const [subTab, setSubTab] = useState<SubTab>("profile");
  const [profile, setProfile] = useState<VolumeProfileResult | null>(null);
  const [chipDist, setChipDist] = useState<DistributionResult | null>(null);
  const [srLevels, setSrLevels] = useState<[number, number, string][]>([]);
  const [concentration, setConcentration] = useState<ConcentrationOutput | null>(null);
  const [plRatio, setPlRatio] = useState<ProfitLossOutput | null>(null);
  const [frames, setFrames] = useState<FrameOutput[]>([]);
  const [frameIdx, setFrameIdx] = useState(0);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState("");

  const loadAll = useCallback(async (sid: number) => {
    setLoading(true);
    setStatus("计算中...");
    try {
      const [vp, cd, sr, conc, pl, hist] = await Promise.all([
        invoke<VolumeProfileResult>("compute_volume_profile", { stockId: sid, numBuckets: 100 }),
        invoke<DistributionResult>("compute_chip_distribution", { stockId: sid }),
        invoke<[number, number, string][]>("compute_sr_levels", { stockId: sid, numLevels: 50 }),
        invoke<ConcentrationOutput>("compute_concentration", { stockId: sid }),
        invoke<ProfitLossOutput>("compute_profit_loss_ratio", { stockId: sid }),
        invoke<FrameOutput[]>("compute_historical_frames", { stockId: sid, frameCount: 50 }),
      ]);
      setProfile(vp);
      setChipDist(cd);
      setSrLevels(sr);
      setConcentration(conc);
      setPlRatio(pl);
      setFrames(hist);
      setFrameIdx(hist.length > 0 ? hist.length - 1 : 0);
      setStatus("");
    } catch (e) {
      console.error("Distribution error:", e);
      setStatus(`计算失败: ${e}`);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    if (stockId) {
      loadAll(stockId);
    } else {
      setProfile(null);
      setChipDist(null);
      setSrLevels([]);
      setConcentration(null);
      setPlRatio(null);
      setFrames([]);
    }
  }, [stockId, loadAll]);

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
      background: "#141b2d", color: "#F1F5F9", fontFamily: "monospace", fontSize: 13,
    }}>
      {/* Sub tabs */}
      <div style={{
        display: "flex", borderBottom: "1px solid #1E293B",
        background: "#111827", flexShrink: 0,
      }}>
        {([
          ["profile", "成交量"],
          ["chip", "筹码"],
          ["sr", "支撑/阻力"],
          ["concentration", "集中度"],
          ["plratio", "盈亏比"],
          ["history", "历史"],
        ] as [SubTab, string][]).map(([k, label]) => (
          <button key={k} onClick={() => setSubTab(k)} style={{
            flex: 1, padding: "8px 12px", border: "none",
            background: subTab === k ? "#141b2d" : "transparent",
            color: subTab === k ? "#00D8FF" : "#94A3B8",
            cursor: "pointer", fontSize: 13, fontFamily: "monospace",
            fontWeight: subTab === k ? 600 : 400,
            borderBottom: subTab === k ? "2px solid #00D8FF" : "2px solid transparent",
          }}>
            {label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: "auto", padding: "12px" }}>
        {loading && <div style={{ color: "#00D8FF", textAlign: "center" }}>计算中...</div>}
        {status && !loading && <div style={{ color: "#FF2A7A", textAlign: "center" }}>{status}</div>}

        {subTab === "profile" && profile && <VolumeProfileChart profile={profile} />}
        {subTab === "chip" && chipDist && <ChipDistChart dist={chipDist} />}
        {subTab === "sr" && <SRLevelsList levels={srLevels} />}
        {subTab === "concentration" && concentration && <ConcentrationView data={concentration} />}
        {subTab === "plratio" && plRatio && <PLRatioView data={plRatio} />}
        {subTab === "history" && frames.length > 0 && (
          <HistoryFramesView frames={frames} frameIdx={frameIdx} onFrameChange={setFrameIdx} />
        )}
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
      <div style={{ fontWeight: 600, marginBottom: 8, color: "#00D8FF" }}>成交量分布图</div>

      <div style={{ fontSize: 12, color: "#94A3B8", marginBottom: 12 }}>
        <span>POC: <span style={{ color: "#00D8FF" }}>{poc.toFixed(2)}</span></span>
        <span style={{ marginLeft: 16 }}>VAH: <span style={{ color: "#FF2A7A" }}>{vah.toFixed(2)}</span></span>
        <span style={{ marginLeft: 16 }}>VAL: <span style={{ color: "#00E676" }}>{val.toFixed(2)}</span></span>
        <span style={{ marginLeft: 16 }}>价值区: {val.toFixed(2)} ~ {vah.toFixed(2)}</span>
      </div>

      <svg width={280} height={Math.min(levels.length * 4, 400)} style={{ background: "#0A0E1A" }}>
        {levels.map((l, i) => {
          const w = maxVol > 0 ? (l.volume / maxVol) * barWidth : 0;
          const y = ((l.price - minP) / priceRange) * (levels.length * 4);
          const fill = l.is_poc ? "#00D8FF" : (l.price >= val && l.price <= vah ? "#3b82f6" : "#1E293B");
          return (
            <g key={i}>
              <rect x={0} y={y - 1.5} width={w} height={3} fill={fill} rx={1} />
              <text x={w + 4} y={y + 2} fill="#94A3B8" fontSize={9} fontFamily="monospace">
                {l.price.toFixed(1)}
              </text>
            </g>
          );
        })}

        {/* VAH/VAL lines */}
        <line x1={0} y1={((vah - minP) / priceRange) * levels.length * 4}
          x2={barWidth} y2={((vah - minP) / priceRange) * levels.length * 4}
          stroke="#FF2A7A" strokeDasharray="4 2" strokeWidth={1} />
        <line x1={0} y1={((val - minP) / priceRange) * levels.length * 4}
          x2={barWidth} y2={((val - minP) / priceRange) * levels.length * 4}
          stroke="#00E676" strokeDasharray="4 2" strokeWidth={1} />
      </svg>

      <div style={{ fontSize: 11, color: "#64748B", marginTop: 8 }}>
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
      <div style={{ fontWeight: 600, marginBottom: 8, color: "#00D8FF" }}>筹码分布</div>

      <div style={{ fontSize: 12, color: "#94A3B8", marginBottom: 12 }}>
        <span>平均成本: <span style={{ color: "#00D8FF" }}>{avg_cost.toFixed(2)}</span></span>
        <span style={{ marginLeft: 16 }}>加权均价: <span style={{ color: "#3b82f6" }}>
          {weighted_avg_cost.toFixed(2)}</span>
        </span>
      </div>

      <svg width={w + 40} height={h + 30} style={{ background: "#0A0E1A" }}>
        {price_levels.map((p, i) => {
          const bh = maxChip > 0 ? (chip_volume[i] / maxChip) * h : 0;
          const x = (i / price_levels.length) * w + 20;
          const isCost = p < avg_cost * 1.02 && p > avg_cost * 0.98;
          return (
            <rect key={i} x={x - barW / 2} y={h - bh + 10} width={barW} height={bh}
              fill={isCost ? "#00D8FF" : "#3b82f6"} opacity={0.7} rx={1} />
          );
        })}
        <line x1={20} y1={costY} x2={w + 20} y2={costY}
          stroke="#00D8FF" strokeDasharray="4 2" strokeWidth={1} />
      </svg>

      <div style={{ fontSize: 11, color: "#64748B", marginTop: 8 }}>
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
      <div style={{ fontWeight: 600, marginBottom: 12, color: "#00D8FF" }}>
        支撑/阻力位 (高量节点 {">"} 1.5x 均值)
      </div>

      {resistance.length > 0 && (
        <div style={{ marginBottom: 12 }}>
          <div style={{ color: "#FF2A7A", fontSize: 12, fontWeight: 600, marginBottom: 6 }}>
            阻力位
          </div>
          {resistance.map((l, i) => (
            <div key={i} style={{
              display: "flex", justifyContent: "space-between",
              padding: "4px 8px", background: "rgba(239,68,68,0.1)",
              borderRadius: 4, marginBottom: 4, fontSize: 12,
            }}>
              <span style={{ color: "#FF2A7A" }}>{l[0].toFixed(2)}</span>
              <span style={{ color: "#94A3B8" }}>量: {l[1].toFixed(0)}</span>
            </div>
          ))}
        </div>
      )}

      {support.length > 0 && (
        <div style={{ marginBottom: 12 }}>
          <div style={{ color: "#00E676", fontSize: 12, fontWeight: 600, marginBottom: 6 }}>
            支撑位
          </div>
          {support.map((l, i) => (
            <div key={i} style={{
              display: "flex", justifyContent: "space-between",
              padding: "4px 8px", background: "rgba(34,197,94,0.1)",
              borderRadius: 4, marginBottom: 4, fontSize: 12,
            }}>
              <span style={{ color: "#00E676" }}>{l[0].toFixed(2)}</span>
              <span style={{ color: "#94A3B8" }}>量: {l[1].toFixed(0)}</span>
            </div>
          ))}
        </div>
      )}

      {poc.length > 0 && (
        <div>
          <div style={{ color: "#00D8FF", fontSize: 12, fontWeight: 600, marginBottom: 6 }}>
            POC (最大成交量节点)
          </div>
          {poc.map((l, i) => (
            <div key={i} style={{
              display: "flex", justifyContent: "space-between",
              padding: "4px 8px", background: "rgba(251,191,36,0.1)",
              borderRadius: 4, marginBottom: 4, fontSize: 12,
            }}>
              <span style={{ color: "#00D8FF" }}>{l[0].toFixed(2)}</span>
              <span style={{ color: "#94A3B8" }}>量: {l[1].toFixed(0)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function ConcentrationView({ data }: { data: ConcentrationOutput }) {
  const barW = 180;
  const maxCR = Math.max(data.cr5, data.cr10, data.cr20, 1);
  const bars: [string, number, string][] = [
    ["CR5 (前5)", data.cr5, "#FF2A7A"],
    ["CR10 (前10)", data.cr10, "#00D8FF"],
    ["CR20 (前20)", data.cr20, "#00E676"],
  ];

  return (
    <div>
      <div style={{ fontWeight: 600, marginBottom: 8, color: "#00D8FF" }}>
        筹码集中度分析
      </div>
      <div style={{ fontSize: 12, color: "#94A3B8", marginBottom: 12 }}>
        趋势: <span style={{
          color: data.trend < 0 ? "#00E676" : "#FF2A7A",
          fontWeight: 600,
        }}>
          {data.trend < 0 ? "集中" : "分散"}{" "}
          ({data.trend > 0 ? "+" : ""}{(data.trend * 100).toFixed(1)}%)
        </span>
        <span style={{ marginLeft: 12 }}>
          说明: {data.description}
        </span>
      </div>
      {bars.map(([label, val, color]) => (
        <div key={label} style={{ marginBottom: 8, display: "flex", alignItems: "center", gap: 8 }}>
          <span style={{ color: "#94A3B8", fontSize: 11, width: 90 }}>{label}</span>
          <div style={{ flex: 1, background: "#0A0E1A", borderRadius: 4, height: 18, overflow: "hidden" }}>
            <div style={{
              width: `${(val / maxCR) * 100}%`, height: "100%",
              background: color, borderRadius: 4, opacity: 0.7,
              transition: "width 0.5s",
            }} />
          </div>
          <span style={{ color, fontSize: 12, fontWeight: 600, width: 45, textAlign: "right" }}>
            {(val * 100).toFixed(1)}%
          </span>
        </div>
      ))}
      <div style={{ fontSize: 11, color: "#64748B", marginTop: 8 }}>
        CR集中度 = 前N大持仓占总市值的比例。集中度高表示筹码集中于少数持有者，筹码锁定性好。
      </div>
    </div>
  );
}

function PLRatioView({ data }: { data: ProfitLossOutput }) {
  const total = data.profit_pct + data.loss_pct || 1;
  const profitW = (data.profit_pct / total) * 200;
  const lossW = (data.loss_pct / total) * 200;

  return (
    <div>
      <div style={{ fontWeight: 600, marginBottom: 8, color: "#00D8FF" }}>
        盈亏分布
      </div>

      {/* Profit/Loss bar */}
      <div style={{ marginBottom: 16 }}>
        <div style={{ fontSize: 12, color: "#94A3B8", marginBottom: 4 }}>
          持仓盈亏比例
        </div>
        <div style={{ display: "flex", height: 24, borderRadius: 4, overflow: "hidden" }}>
          <div style={{
            width: `${profitW}px`, height: "100%",
            background: "#FF2A7A", display: "flex", alignItems: "center",
            justifyContent: "center", fontSize: 11, color: "#fff", fontWeight: 600,
          }}>
            {data.profit_pct > 1 ? `盈利 ${data.profit_pct.toFixed(0)}%` : ""}
          </div>
          <div style={{
            width: `${lossW}px`, height: "100%",
            background: "#00E676", display: "flex", alignItems: "center",
            justifyContent: "center", fontSize: 11, color: "#fff", fontWeight: 600,
          }}>
            {data.loss_pct > 1 ? `亏损 ${data.loss_pct.toFixed(0)}%` : ""}
          </div>
        </div>
        <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11, color: "#94A3B8", marginTop: 4 }}>
          <span>盈利: {data.profit_pct.toFixed(1)}%</span>
          <span>亏损: {data.loss_pct.toFixed(1)}%</span>
        </div>
      </div>

      {/* Cost info */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8, fontSize: 12 }}>
        <CostField label="平均成本" value={data.avg_cost.toFixed(2)} />
        <CostField label="加权均价" value={data.weighted_avg_cost.toFixed(2)} />
        <CostField label="最新价格" value={data.last_price.toFixed(2)} color={
          data.last_price > data.avg_cost ? "#FF2A7A" : "#00E676"
        } />
        <CostField label="盈亏幅度" value={
          ((data.last_price / data.avg_cost - 1) * 100).toFixed(2) + "%"
        } color={
          data.last_price > data.avg_cost ? "#FF2A7A" : "#00E676"
        } />
      </div>
    </div>
  );
}

function CostField({ label, value, color }: { label: string; value: string; color?: string }) {
  return (
    <div style={{ padding: "6px 8px", background: "#111827", borderRadius: 4 }}>
      <div style={{ color: "#94A3B8", fontSize: 10, marginBottom: 2 }}>{label}</div>
      <div style={{ color: color ?? "#F1F5F9", fontSize: 13, fontWeight: 600 }}>{value}</div>
    </div>
  );
}

function HistoryFramesView({
  frames,
  frameIdx,
  onFrameChange,
}: {
  frames: FrameOutput[];
  frameIdx: number;
  onFrameChange: (i: number) => void;
}) {
  const frame = frames[frameIdx];
  if (!frame) return <div style={{ color: "#555" }}>无历史帧数据</div>;

  const maxChip = Math.max(...frame.chip_volume, 1);
  const h = 180;
  const w = 260;
  const priceRange = frame.price_levels[frame.price_levels.length - 1] - frame.price_levels[0] || 1;

  return (
    <div>
      <div style={{ fontWeight: 600, marginBottom: 8, color: "#00D8FF" }}>
        历史筹码分布
      </div>

      {/* Frame slider */}
      <div style={{ marginBottom: 8 }}>
        <input
          type="range"
          min={0}
          max={frames.length - 1}
          value={frameIdx}
          onChange={(e) => onFrameChange(parseInt(e.target.value))}
          style={{ width: "100%", accentColor: "#00D8FF" }}
        />
        <div style={{ display: "flex", justifyContent: "space-between", fontSize: 10, color: "#94A3B8" }}>
          <span>{frames[0]?.date || ""}</span>
          <span style={{ color: "#00D8FF", fontSize: 11 }}>{frame.date}</span>
          <span>{frames[frames.length - 1]?.date || ""}</span>
        </div>
      </div>

      {/* Frame info */}
      <div style={{ fontSize: 12, color: "#94A3B8", marginBottom: 8, display: "flex", gap: 16, flexWrap: "wrap" }}>
        <span>平均成本: <span style={{ color: "#00D8FF" }}>{frame.avg_cost.toFixed(2)}</span></span>
        <span>盈利占比: <span style={{ color: "#FF2A7A" }}>{frame.profit_pct.toFixed(1)}%</span></span>
        <span>亏损占比: <span style={{ color: "#00E676" }}>{frame.loss_pct.toFixed(1)}%</span></span>
      </div>

      {/* Frame distribution chart */}
      <svg width={w + 40} height={h + 20} style={{ background: "#0A0E1A" }}>
        {frame.price_levels.map((p, i) => {
          const bh = maxChip > 0 ? (frame.chip_volume[i] / maxChip) * h : 0;
          const x = (i / frame.price_levels.length) * w + 20;
          const barW = (w / frame.price_levels.length) * 0.9;
          const isCost = Math.abs(p - frame.avg_cost) / frame.avg_cost < 0.02;
          return (
            <rect key={i} x={x - barW / 2} y={h - bh + 10} width={barW} height={bh}
              fill={isCost ? "#00D8FF" : "#3b82f6"} opacity={0.7} rx={1} />
          );
        })}
        {/* Avg cost line */}
        {(() => {
          const costY = h - ((frame.avg_cost - frame.price_levels[0]) / priceRange) * h + 10;
          return (
            <line x1={20} y1={costY} x2={w + 20} y2={costY}
              stroke="#00D8FF" strokeDasharray="4 2" strokeWidth={1} />
          );
        })()}
      </svg>

      <div style={{ fontSize: 11, color: "#64748B", marginTop: 8 }}>
        拖动滑块查看不同时间点的筹码分布变化。盈利占比越高，价格越在平均成本线上方。
      </div>
    </div>
  );
}

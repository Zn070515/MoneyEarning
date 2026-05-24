import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TradeJournalPanel } from "@me/ui";
import { StrategyPanel } from "@me/ui";
import { useAppStore } from "../stores/appStore";

type ReviewTab = "trades" | "strategies" | "review";

export default function ReviewPage() {
  const selectedStockId = useAppStore((s) => s.selectedStockId);
  const selectedStockCode = useAppStore((s) => s.selectedStockCode);
  const [activeTab, setActiveTab] = useState<ReviewTab>("trades");

  return (
    <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
      {/* Header */}
      <div
        style={{
          padding: "12px 20px",
          background: "#161616",
          borderBottom: "1px solid #2A2A2A",
          display: "flex",
          alignItems: "center",
          gap: 16,
          flexShrink: 0,
        }}
      >
        <h2 style={{ color: "#CCAA00", fontSize: 16, fontFamily: "monospace", margin: 0 }}>
          交易复盘
        </h2>
        {selectedStockCode && (
          <span style={{ color: "#858585", fontSize: 12, fontFamily: "monospace" }}>
            当前标的: {selectedStockCode}
          </span>
        )}
      </div>

      {/* Tabs */}
      <div
        style={{
          padding: "4px 20px",
          background: "#121212",
          borderBottom: "1px solid #2A2A2A",
          display: "flex",
          gap: 4,
          flexShrink: 0,
        }}
      >
        {([
          ["trades", "交易记录"],
          ["strategies", "策略管理"],
          ["review", "复盘模板"],
        ] as [ReviewTab, string][]).map(([tab, label]) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              padding: "6px 16px",
              background: activeTab === tab ? "#CCAA00" : "transparent",
              color: activeTab === tab ? "#000" : "#858585",
              border: "none",
              borderRadius: "4px 4px 0 0",
              cursor: "pointer",
              fontFamily: "monospace",
              fontSize: 12,
              fontWeight: activeTab === tab ? 600 : 400,
            }}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        {activeTab === "trades" && (
          <div style={{ height: "100%", overflow: "auto", padding: 16 }}>
            <TradeJournalPanel selectedStockId={selectedStockId} />
          </div>
        )}
        {activeTab === "strategies" && (
          <div style={{ height: "100%", overflow: "auto", padding: 16 }}>
            <StrategyPanel selectedStockId={selectedStockId} />
          </div>
        )}
        {activeTab === "review" && (
          <ReviewTemplatePanel selectedStockId={selectedStockId} selectedStockCode={selectedStockCode} />
        )}
      </div>
    </div>
  );
}

// ── Structured Review Template ──

const REVIEW_QUESTIONS = [
  {
    category: "交易前",
    questions: [
      "买入理由是什么？（技术面/基本面/消息面）",
      "是否符合当前策略规则？",
      "计划持仓周期是多久？",
      "止损位设在哪里？",
      "仓位是否合理（占总资金%）？",
    ],
  },
  {
    category: "持仓中",
    questions: [
      "是否按计划持有？",
      "有没有因为恐慌/贪婪而提前操作？",
      "股价运行是否符合预期？",
      "是否跟踪了相关板块/大盘走势？",
    ],
  },
  {
    category: "卖出后",
    questions: [
      "卖出理由是什么？（止盈/止损/时间到期/策略信号）",
      "实际盈亏是否符合预期？",
      "如果反向操作，结果会怎样？",
      "这次交易的教训是什么？",
      "下一次遇到类似情况会怎么做？",
    ],
  },
];

const EMOTION_TAGS = [
  { value: "理性建仓", label: "理性建仓", color: "#26A69A" },
  { value: "冲动追高", label: "冲动追高", color: "#EF5350" },
  { value: "恐慌割肉", label: "恐慌割肉", color: "#EF5350" },
  { value: "盲目跟风", label: "盲目跟风", color: "#fb923c" },
  { value: "纪律止盈", label: "纪律止盈", color: "#4ade80" },
  { value: "纪律止损", label: "纪律止损", color: "#a78bfa" },
  { value: "犹豫错过", label: "犹豫错过", color: "#94a3b8" },
  { value: "躺平持有", label: "躺平持有", color: "#CCAA00" },
];

interface ReviewRecord {
  date: string;
  emotion: string;
  answers: Record<string, string>;
}

// ── Training Mode ──

interface DailyBar { trade_date: string; open: number; high: number; low: number; close: number }
type TrainState = "idle" | "ready" | "playing" | "done";

function TrainingPanel({ selectedStockId, selectedStockCode }: { selectedStockId: number | null; selectedStockCode: string | null }) {
  const [state, setState] = useState<TrainState>("idle");
  const [bars, setBars] = useState<DailyBar[]>([]);
  const [currentIdx, setCurrentIdx] = useState(0);
  const [decisions, setDecisions] = useState<("bull" | "bear" | null)[]>([]);
  const [score, setScore] = useState<{ correct: number; total: number } | null>(null);

  const loadData = useCallback(async () => {
    if (!selectedStockId) return;
    try {
      const data = await invoke<DailyBar[]>("query_daily_prices", {
        stockId: selectedStockId,
        startDate: "2022-01-01",
        endDate: "2099-12-31",
      });
      if (data.length < 30) {
        alert("该股票数据不足（需至少30条日线），请先导入数据。");
        return;
      }
      setBars(data.slice(-60)); // use last 60 bars for training
      setDecisions(new Array(Math.min(data.length, 60)).fill(null));
      setCurrentIdx(0);
      setScore(null);
      setState("ready");
    } catch (e) {
      alert(`加载数据失败: ${e}`);
    }
  }, [selectedStockId]);

  const startTraining = () => setState("playing");

  const decide = (dir: "bull" | "bear") => {
    const next = [...decisions];
    next[currentIdx] = dir;
    setDecisions(next);
    if (currentIdx + 1 >= bars.length) {
      finishTraining(next);
    } else {
      setCurrentIdx(currentIdx + 1);
    }
  };

  const finishTraining = (final: ("bull" | "bear" | null)[]) => {
    let correct = 0;
    let total = 0;
    for (let i = 0; i < final.length - 1; i++) {
      const d = final[i];
      if (!d) continue;
      total++;
      const actual = bars[i + 1].close >= bars[i].close ? "bull" : "bear";
      if (d === actual) correct++;
    }
    setScore({ correct, total });
    setState("done");
  };

  const reset = () => { setState("idle"); setBars([]); setCurrentIdx(0); setDecisions([]); setScore(null); };

  if (state === "idle") {
    return (
      <div style={{ marginTop: 24, padding: 16, background: "#121212", border: "1px solid #2A2A2A", borderRadius: 8 }}>
        <div style={{ color: "#CCAA00", fontSize: 13, fontFamily: "monospace", marginBottom: 8 }}>K线训练模式</div>
        <div style={{ color: "#666666", fontSize: 12, fontFamily: "monospace", lineHeight: 1.6, marginBottom: 12 }}>
          逐根K线判断涨跌方向，隐藏后市走势，对照实际结果检验判断准确性。帮助训练盘感，改善入场/离场时机把握。
        </div>
        <button
          onClick={loadData}
          disabled={!selectedStockId}
          title={!selectedStockId ? "请先在图表页面选择一只股票" : ""}
          style={{
            padding: "6px 16px", background: selectedStockId ? "#CCAA00" : "#2A2A2A",
            color: selectedStockId ? "#000" : "#666666", border: "none", borderRadius: 4,
            cursor: selectedStockId ? "pointer" : "not-allowed",
            fontFamily: "monospace", fontSize: 12, fontWeight: 600,
          }}
        >
          {selectedStockId ? `开始训练 (${selectedStockCode || ""})` : "请先选择股票"}
        </button>
        {!selectedStockId && (
          <div style={{ color: "#666666", fontSize: 10, fontFamily: "monospace", marginTop: 6 }}>
            在图表页面选择一个已导入数据的股票后，返回此页面开始训练。
          </div>
        )}
      </div>
    );
  }

  if (state === "ready") {
    return (
      <div style={{ marginTop: 24, padding: 16, background: "#121212", border: "1px solid #2A2A2A", borderRadius: 8 }}>
        <div style={{ color: "#26A69A", fontSize: 13, fontFamily: "monospace", marginBottom: 8 }}>数据已就绪</div>
        <div style={{ color: "#858585", fontSize: 12, fontFamily: "monospace", marginBottom: 8 }}>
          已加载 {bars.length} 条日线数据（{bars[0]?.trade_date} ~ {bars[bars.length - 1]?.trade_date}）
        </div>
        <div style={{ color: "#666666", fontSize: 11, fontFamily: "monospace", lineHeight: 1.6, marginBottom: 12 }}>
          规则：逐条显示K线数据（隐藏后续走势），你需要判断下一天是涨还是跌。
          完成全部判断后，系统会与真实走势对比，计算你的准确率。
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button onClick={startTraining} style={{ padding: "6px 20px", background: "#CCAA00", color: "#000", border: "none", borderRadius: 4, cursor: "pointer", fontFamily: "monospace", fontSize: 13, fontWeight: 600 }}>开始挑战</button>
          <button onClick={reset} style={{ padding: "6px 16px", background: "transparent", color: "#858585", border: "1px solid #2A2A2A", borderRadius: 4, cursor: "pointer", fontFamily: "monospace", fontSize: 12 }}>取消</button>
        </div>
      </div>
    );
  }

  if (state === "done" && score) {
    const pct = score.total > 0 ? (score.correct / score.total * 100).toFixed(1) : "0.0";
    const grade = Number(pct) >= 70 ? "优秀" : Number(pct) >= 55 ? "良好" : Number(pct) >= 45 ? "一般" : "需加强";
    const gradeColor = Number(pct) >= 70 ? "#26A69A" : Number(pct) >= 55 ? "#CCAA00" : Number(pct) >= 45 ? "#fb923c" : "#EF5350";
    return (
      <div style={{ marginTop: 24, padding: 16, background: "#121212", border: "1px solid #2A2A2A", borderRadius: 8 }}>
        <div style={{ color: "#CCAA00", fontSize: 14, fontFamily: "monospace", marginBottom: 12 }}>训练结果</div>
        <div style={{ display: "flex", gap: 20, flexWrap: "wrap", marginBottom: 16 }}>
          <div style={{ textAlign: "center" }}>
            <div style={{ color: gradeColor, fontSize: 36, fontFamily: "monospace", fontWeight: 700 }}>{pct}%</div>
            <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace" }}>准确率</div>
          </div>
          <div style={{ textAlign: "center" }}>
            <div style={{ color: "#26A69A", fontSize: 28, fontFamily: "monospace", fontWeight: 700 }}>{score.correct}</div>
            <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace" }}>正确</div>
          </div>
          <div style={{ textAlign: "center" }}>
            <div style={{ color: "#EF5350", fontSize: 28, fontFamily: "monospace", fontWeight: 700 }}>{score.total - score.correct}</div>
            <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace" }}>错误</div>
          </div>
          <div style={{ textAlign: "center" }}>
            <div style={{ color: gradeColor, fontSize: 28, fontFamily: "monospace", fontWeight: 700 }}>{grade}</div>
            <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace" }}>评级</div>
          </div>
        </div>
        <div style={{ color: "#666666", fontSize: 11, fontFamily: "monospace", marginBottom: 12, lineHeight: 1.5 }}>
          {Number(pct) >= 70 ? "盘感很好，继续保持纪律性交易。"
            : Number(pct) >= 55 ? "方向判断基本准确，注意节奏把握。"
            : Number(pct) >= 45 ? "接近随机水平，建议结合指标辅助判断。"
            : "准确率偏低，建议先学习K线基础形态。"}
        </div>
        <button onClick={reset} style={{ padding: "6px 20px", background: "#CCAA00", color: "#000", border: "none", borderRadius: 4, cursor: "pointer", fontFamily: "monospace", fontSize: 13, fontWeight: 600 }}>再次训练</button>
      </div>
    );
  }

  // state === "playing"
  const bar = bars[currentIdx];
  const prevClose = currentIdx > 0 ? bars[currentIdx - 1].close : bar.open;
  const change = ((bar.close - prevClose) / prevClose * 100).toFixed(2);
  const isUp = bar.close >= prevClose;
  const progress = ((currentIdx + 0) / bars.length * 100).toFixed(0);

  return (
    <div style={{ marginTop: 24, padding: 16, background: "#121212", border: "1px solid #2A2A2A", borderRadius: 8 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
        <div style={{ color: "#CCAA00", fontSize: 13, fontFamily: "monospace" }}>
          K线训练 · 第 {currentIdx + 1} / {bars.length} 根
        </div>
        <div style={{ color: "#666666", fontSize: 11, fontFamily: "monospace" }}>
          进度 {progress}%
        </div>
      </div>

      {/* Progress bar */}
      <div style={{ background: "#0C0C0C", borderRadius: 4, height: 4, marginBottom: 16, overflow: "hidden" }}>
        <div style={{ width: `${progress}%`, height: "100%", background: "linear-gradient(90deg, #CCAA00, #7E57C2)", borderRadius: 4, transition: "width 0.2s" }} />
      </div>

      {/* Current bar display */}
      <div style={{ padding: 12, background: "#161616", borderRadius: 6, border: "1px solid #2A2A2A", marginBottom: 16 }}>
        <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace", marginBottom: 8 }}>
          日期: {bar.trade_date}
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: 12 }}>
          <div>
            <div style={{ color: "#666666", fontSize: 10, fontFamily: "monospace" }}>开盘</div>
            <div style={{ color: "#D4D4D4", fontSize: 16, fontFamily: "monospace", fontWeight: 600 }}>{bar.open.toFixed(2)}</div>
          </div>
          <div>
            <div style={{ color: "#666666", fontSize: 10, fontFamily: "monospace" }}>最高</div>
            <div style={{ color: "#26A69A", fontSize: 16, fontFamily: "monospace", fontWeight: 600 }}>{bar.high.toFixed(2)}</div>
          </div>
          <div>
            <div style={{ color: "#666666", fontSize: 10, fontFamily: "monospace" }}>最低</div>
            <div style={{ color: "#EF5350", fontSize: 16, fontFamily: "monospace", fontWeight: 600 }}>{bar.low.toFixed(2)}</div>
          </div>
          <div>
            <div style={{ color: "#666666", fontSize: 10, fontFamily: "monospace" }}>收盘</div>
            <div style={{ color: isUp ? "#26A69A" : "#EF5350", fontSize: 16, fontFamily: "monospace", fontWeight: 600 }}>{bar.close.toFixed(2)}</div>
          </div>
        </div>
        <div style={{ marginTop: 8, color: change.startsWith("-") ? "#EF5350" : "#26A69A", fontSize: 11, fontFamily: "monospace" }}>
          较前收盘: {change}%
        </div>
        <div style={{ marginTop: 4, color: "#666666", fontSize: 10, fontFamily: "monospace" }}>
          振幅: {((bar.high - bar.low) / bar.low * 100).toFixed(2)}%
        </div>
      </div>

      {/* Previous decisions summary */}
      <div style={{ marginBottom: 16, display: "flex", gap: 4, flexWrap: "wrap", maxHeight: 80, overflow: "auto" }}>
        {decisions.slice(0, currentIdx).map((d, i) => (
          <span key={i} style={{
            padding: "2px 6px", borderRadius: 2, fontSize: 9, fontFamily: "monospace",
            background: d === "bull" ? "rgba(38,166,154,0.15)" : "rgba(239,83,80,0.15)",
            color: d === "bull" ? "#26A69A" : "#EF5350",
          }}>
            {i + 1}:{d === "bull" ? "涨" : "跌"}
          </span>
        ))}
      </div>

      {/* Decision buttons */}
      <div style={{ display: "flex", gap: 10 }}>
        <button onClick={() => decide("bull")} style={{
          padding: "8px 28px", background: "#26A69A", color: "#000", border: "none", borderRadius: 6,
          cursor: "pointer", fontFamily: "monospace", fontSize: 14, fontWeight: 700,
        }}>
          看涨 ↑
        </button>
        <button onClick={() => decide("bear")} style={{
          padding: "8px 28px", background: "#EF5350", color: "#fff", border: "none", borderRadius: 6,
          cursor: "pointer", fontFamily: "monospace", fontSize: 14, fontWeight: 700,
        }}>
          看跌 ↓
        </button>
        <button onClick={reset} style={{
          padding: "8px 16px", background: "transparent", color: "#858585",
          border: "1px solid #2A2A2A", borderRadius: 6, cursor: "pointer",
          fontFamily: "monospace", fontSize: 12,
        }}>
          退出
        </button>
      </div>
    </div>
  );
}

function ReviewTemplatePanel({ selectedStockId, selectedStockCode }: { selectedStockId: number | null; selectedStockCode: string | null }) {
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [selectedEmotion, setSelectedEmotion] = useState("");
  const [saved, setSaved] = useState(false);
  const [history, setHistory] = useState<ReviewRecord[]>([]);
  const [showHistory, setShowHistory] = useState(false);

  useEffect(() => {
    try {
      const data = JSON.parse(localStorage.getItem("me-reviews") || "[]");
      setHistory(data);
    } catch {}
  }, [saved]);

  const handleSave = () => {
    const reviewData: ReviewRecord = {
      date: new Date().toISOString(),
      emotion: selectedEmotion,
      answers,
    };
    const existing: ReviewRecord[] = JSON.parse(localStorage.getItem("me-reviews") || "[]");
    existing.push(reviewData);
    localStorage.setItem("me-reviews", JSON.stringify(existing));
    setSaved(true);
    setAnswers({});
    setSelectedEmotion("");
    setTimeout(() => setSaved(false), 2000);
  };

  // Emotion distribution from history
  const emotionDist: Record<string, number> = {};
  history.forEach((r) => {
    if (r.emotion) emotionDist[r.emotion] = (emotionDist[r.emotion] || 0) + 1;
  });
  const maxCount = Math.max(1, ...Object.values(emotionDist));

  return (
    <div style={{ padding: 16, overflow: "auto", height: "100%", maxWidth: 800 }}>
      {/* Emotion analytics from history */}
      {history.length > 0 && (
        <div style={{
          marginBottom: 20, padding: 14,
          background: "#121212", borderRadius: 8, border: "1px solid #2A2A2A",
        }}>
          <div style={{ color: "#CCAA00", fontSize: 13, fontFamily: "monospace", marginBottom: 10 }}>
            复盘情绪分布 ({history.length} 条记录)
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {EMOTION_TAGS.map((tag) => {
              const count = emotionDist[tag.value] || 0;
              if (!count) return null;
              const pct = (count / history.length * 100).toFixed(0);
              return (
                <div key={tag.value} style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <span style={{ color: tag.color, fontSize: 11, fontFamily: "monospace", width: 72, textAlign: "right" }}>
                    {tag.label}
                  </span>
                  <div style={{ flex: 1, background: "#0C0C0C", borderRadius: 4, height: 14, overflow: "hidden" }}>
                    <div style={{
                      width: `${(count / maxCount) * 100}%`, height: "100%",
                      background: tag.color, borderRadius: 4, opacity: 0.7,
                      transition: "width 0.5s",
                    }} />
                  </div>
                  <span style={{ color: "#858585", fontSize: 10, fontFamily: "monospace", width: 40 }}>
                    {count} ({pct}%)
                  </span>
                </div>
              );
            })}
          </div>
          <div style={{ color: "#666666", fontSize: 10, fontFamily: "monospace", marginTop: 8 }}>
            提示：观察情绪分布，找出导致亏损的主要情绪模式。纪律性交易（理性建仓 + 纪律止盈/止损）占比越高，长期盈利概率越大。
          </div>
        </div>
      )}

      {/* New review form */}
      <div style={{
        marginBottom: 16, padding: 14,
        background: "#161616", borderRadius: 8, border: "1px solid #2A2A2A",
      }}>
        <div style={{ color: "#CCAA00", fontSize: 13, fontFamily: "monospace", marginBottom: 10 }}>
          新建复盘记录
        </div>

        {/* Emotion tag selector */}
        <div style={{ marginBottom: 16 }}>
          <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace", marginBottom: 6 }}>
            本次交易情绪标签
          </div>
          <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
            {EMOTION_TAGS.map((tag) => (
              <button
                key={tag.value}
                onClick={() => setSelectedEmotion(tag.value)}
                style={{
                  padding: "4px 10px",
                  background: selectedEmotion === tag.value ? tag.color : "#121212",
                  color: selectedEmotion === tag.value ? "#000" : tag.color,
                  border: `1px solid ${tag.color}`,
                  borderRadius: 12,
                  cursor: "pointer",
                  fontFamily: "monospace",
                  fontSize: 11,
                }}
              >
                {tag.label}
              </button>
            ))}
          </div>
        </div>

        {/* Review questions */}
        {REVIEW_QUESTIONS.map((section) => (
          <div key={section.category} style={{ marginBottom: 16 }}>
            <div style={{
              color: "#CCAA00", fontSize: 12, fontFamily: "monospace",
              marginBottom: 6, borderBottom: "1px solid #2A2A2A", paddingBottom: 4,
            }}>
              {section.category}
            </div>
            {section.questions.map((q) => (
              <div key={q} style={{ marginBottom: 8 }}>
                <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace", marginBottom: 3 }}>
                  {q}
                </div>
                <textarea
                  value={answers[q] || ""}
                  onChange={(e) => setAnswers({ ...answers, [q]: e.target.value })}
                  rows={2}
                  style={{
                    width: "100%", maxWidth: 600, padding: "6px 8px",
                    background: "#0C0C0C", color: "#D4D4D4", border: "1px solid #2A2A2A",
                    borderRadius: 4, fontFamily: "monospace", fontSize: 12, resize: "vertical",
                    boxSizing: "border-box",
                  }}
                />
              </div>
            ))}
          </div>
        ))}

        <button onClick={handleSave} style={{
          padding: "8px 24px", background: saved ? "#26A69A" : "#CCAA00",
          color: "#000", border: "none", borderRadius: 4, cursor: "pointer",
          fontFamily: "monospace", fontSize: 13, fontWeight: 600,
        }}>
          {saved ? "已保存" : "保存复盘记录"}
        </button>
      </div>

      {/* Review history toggle */}
      {history.length > 0 && (
        <div style={{ marginBottom: 16 }}>
          <button onClick={() => setShowHistory(!showHistory)} style={{
            background: "transparent", color: "#858585", border: "1px solid #2A2A2A",
            padding: "6px 16px", borderRadius: 4, cursor: "pointer",
            fontFamily: "monospace", fontSize: 12,
          }}>
            {showHistory ? "收起历史记录" : `查看历史记录 (${history.length})`}
          </button>
        </div>
      )}

      {showHistory && history.slice().reverse().map((r, i) => (
        <div key={i} style={{
          marginBottom: 8, padding: "10px 14px",
          background: "#161616", borderRadius: 6, border: "1px solid #2A2A2A",
        }}>
          <div style={{ display: "flex", gap: 10, alignItems: "center", marginBottom: 6 }}>
            <span style={{ color: "#858585", fontSize: 11, fontFamily: "monospace" }}>
              {new Date(r.date).toLocaleDateString("zh-CN")}
            </span>
            {r.emotion && (
              <span style={{
                fontSize: 10, fontFamily: "monospace",
                color: EMOTION_TAGS.find(e => e.value === r.emotion)?.color || "#858585",
                padding: "1px 8px", borderRadius: 8,
                border: `1px solid ${EMOTION_TAGS.find(e => e.value === r.emotion)?.color || "#858585"}`,
              }}>
                {r.emotion}
              </span>
            )}
          </div>
          {Object.entries(r.answers).filter(([, v]) => v).slice(0, 3).map(([q, a]) => (
            <div key={q} style={{ marginBottom: 3, fontSize: 11, fontFamily: "monospace" }}>
              <span style={{ color: "#666666" }}>{q.slice(0, 20)}...: </span>
              <span style={{ color: "#858585" }}>{a.slice(0, 80)}{a.length > 80 ? "..." : ""}</span>
            </div>
          ))}
        </div>
      ))}

      {/* Training mode */}
      <TrainingPanel selectedStockId={selectedStockId} selectedStockCode={selectedStockCode} />
    </div>
  );
}

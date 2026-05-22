import { useState, useEffect } from "react";
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
          background: "#16213e",
          borderBottom: "1px solid #2a2a4a",
          display: "flex",
          alignItems: "center",
          gap: 16,
          flexShrink: 0,
        }}
      >
        <h2 style={{ color: "#fbbf24", fontSize: 16, fontFamily: "monospace", margin: 0 }}>
          交易复盘
        </h2>
        {selectedStockCode && (
          <span style={{ color: "#888", fontSize: 12, fontFamily: "monospace" }}>
            当前标的: {selectedStockCode}
          </span>
        )}
      </div>

      {/* Tabs */}
      <div
        style={{
          padding: "4px 20px",
          background: "#1a1a2e",
          borderBottom: "1px solid #2a2a4a",
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
              background: activeTab === tab ? "#fbbf24" : "transparent",
              color: activeTab === tab ? "#000" : "#888",
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
          <ReviewTemplatePanel />
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
  { value: "理性建仓", label: "理性建仓", color: "#22c55e" },
  { value: "冲动追高", label: "冲动追高", color: "#ef4444" },
  { value: "恐慌割肉", label: "恐慌割肉", color: "#f87171" },
  { value: "盲目跟风", label: "盲目跟风", color: "#fb923c" },
  { value: "纪律止盈", label: "纪律止盈", color: "#4ade80" },
  { value: "纪律止损", label: "纪律止损", color: "#a78bfa" },
  { value: "犹豫错过", label: "犹豫错过", color: "#94a3b8" },
  { value: "躺平持有", label: "躺平持有", color: "#60a5fa" },
];

interface ReviewRecord {
  date: string;
  emotion: string;
  answers: Record<string, string>;
}

function ReviewTemplatePanel() {
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
          background: "#1a1a2e", borderRadius: 8, border: "1px solid #2a2a4a",
        }}>
          <div style={{ color: "#fbbf24", fontSize: 13, fontFamily: "monospace", marginBottom: 10 }}>
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
                  <div style={{ flex: 1, background: "#0f0f23", borderRadius: 4, height: 14, overflow: "hidden" }}>
                    <div style={{
                      width: `${(count / maxCount) * 100}%`, height: "100%",
                      background: tag.color, borderRadius: 4, opacity: 0.7,
                      transition: "width 0.5s",
                    }} />
                  </div>
                  <span style={{ color: "#888", fontSize: 10, fontFamily: "monospace", width: 40 }}>
                    {count} ({pct}%)
                  </span>
                </div>
              );
            })}
          </div>
          <div style={{ color: "#666", fontSize: 10, fontFamily: "monospace", marginTop: 8 }}>
            提示：观察情绪分布，找出导致亏损的主要情绪模式。纪律性交易（理性建仓 + 纪律止盈/止损）占比越高，长期盈利概率越大。
          </div>
        </div>
      )}

      {/* New review form */}
      <div style={{
        marginBottom: 16, padding: 14,
        background: "#16213e", borderRadius: 8, border: "1px solid #2a2a4a",
      }}>
        <div style={{ color: "#fbbf24", fontSize: 13, fontFamily: "monospace", marginBottom: 10 }}>
          新建复盘记录
        </div>

        {/* Emotion tag selector */}
        <div style={{ marginBottom: 16 }}>
          <div style={{ color: "#888", fontSize: 11, fontFamily: "monospace", marginBottom: 6 }}>
            本次交易情绪标签
          </div>
          <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
            {EMOTION_TAGS.map((tag) => (
              <button
                key={tag.value}
                onClick={() => setSelectedEmotion(tag.value)}
                style={{
                  padding: "4px 10px",
                  background: selectedEmotion === tag.value ? tag.color : "#1a1a2e",
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
              color: "#fbbf24", fontSize: 12, fontFamily: "monospace",
              marginBottom: 6, borderBottom: "1px solid #2a2a4a", paddingBottom: 4,
            }}>
              {section.category}
            </div>
            {section.questions.map((q) => (
              <div key={q} style={{ marginBottom: 8 }}>
                <div style={{ color: "#aaa", fontSize: 11, fontFamily: "monospace", marginBottom: 3 }}>
                  {q}
                </div>
                <textarea
                  value={answers[q] || ""}
                  onChange={(e) => setAnswers({ ...answers, [q]: e.target.value })}
                  rows={2}
                  style={{
                    width: "100%", maxWidth: 600, padding: "6px 8px",
                    background: "#0f0f23", color: "#ccc", border: "1px solid #2a2a4a",
                    borderRadius: 4, fontFamily: "monospace", fontSize: 12, resize: "vertical",
                    boxSizing: "border-box",
                  }}
                />
              </div>
            ))}
          </div>
        ))}

        <button onClick={handleSave} style={{
          padding: "8px 24px", background: saved ? "#22c55e" : "#fbbf24",
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
            background: "transparent", color: "#888", border: "1px solid #2a2a4a",
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
          background: "#16213e", borderRadius: 6, border: "1px solid #2a2a4a",
        }}>
          <div style={{ display: "flex", gap: 10, alignItems: "center", marginBottom: 6 }}>
            <span style={{ color: "#888", fontSize: 11, fontFamily: "monospace" }}>
              {new Date(r.date).toLocaleDateString("zh-CN")}
            </span>
            {r.emotion && (
              <span style={{
                fontSize: 10, fontFamily: "monospace",
                color: EMOTION_TAGS.find(e => e.value === r.emotion)?.color || "#888",
                padding: "1px 8px", borderRadius: 8,
                border: `1px solid ${EMOTION_TAGS.find(e => e.value === r.emotion)?.color || "#888"}`,
              }}>
                {r.emotion}
              </span>
            )}
          </div>
          {Object.entries(r.answers).filter(([, v]) => v).slice(0, 3).map(([q, a]) => (
            <div key={q} style={{ marginBottom: 3, fontSize: 11, fontFamily: "monospace" }}>
              <span style={{ color: "#666" }}>{q.slice(0, 20)}...: </span>
              <span style={{ color: "#aaa" }}>{a.slice(0, 80)}{a.length > 80 ? "..." : ""}</span>
            </div>
          ))}
        </div>
      ))}

      {/* Training mode placeholder */}
      <div style={{
        marginTop: 24, padding: 16, background: "#1a1a2e",
        border: "1px solid #2a2a4a", borderRadius: 8,
      }}>
        <div style={{ color: "#fbbf24", fontSize: 13, fontFamily: "monospace", marginBottom: 8 }}>
          训练模式（付费功能）
        </div>
        <div style={{ color: "#666", fontSize: 12, fontFamily: "monospace", lineHeight: 1.6 }}>
          使用历史数据逐根K线判断买卖点，隐藏后续走势，完成后对照实际走势检验判断准确性。
          帮助训练盘感，改善入场/离场时机把握。
        </div>
        <button disabled style={{
          marginTop: 12, padding: "4px 16px", background: "#3a3a5a",
          color: "#888", border: "none", borderRadius: 4,
          cursor: "not-allowed", fontFamily: "monospace", fontSize: 12,
        }}>
          升级专业版解锁
        </button>
      </div>
    </div>
  );
}

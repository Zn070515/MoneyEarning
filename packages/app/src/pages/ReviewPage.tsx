import { useState } from "react";
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

function ReviewTemplatePanel() {
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [selectedEmotion, setSelectedEmotion] = useState("");
  const [saved, setSaved] = useState(false);

  const handleSave = () => {
    const reviewData = {
      date: new Date().toISOString(),
      emotion: selectedEmotion,
      answers,
    };
    const existing = JSON.parse(localStorage.getItem("me-reviews") || "[]");
    existing.push(reviewData);
    localStorage.setItem("me-reviews", JSON.stringify(existing));
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div style={{ padding: 16, overflow: "auto", height: "100%" }}>
      {/* Emotion tag selector */}
      <div style={{ marginBottom: 20 }}>
        <div style={{ color: "#fbbf24", fontSize: 13, fontFamily: "monospace", marginBottom: 8 }}>
          本次交易情绪标签
        </div>
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
          {EMOTION_TAGS.map((tag) => (
            <button
              key={tag.value}
              onClick={() => setSelectedEmotion(tag.value)}
              style={{
                padding: "4px 12px",
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
        <div key={section.category} style={{ marginBottom: 20 }}>
          <div
            style={{
              color: "#fbbf24",
              fontSize: 13,
              fontFamily: "monospace",
              marginBottom: 8,
              borderBottom: "1px solid #2a2a4a",
              paddingBottom: 4,
            }}
          >
            {section.category}
          </div>
          {section.questions.map((q) => (
            <div key={q} style={{ marginBottom: 10 }}>
              <div style={{ color: "#aaa", fontSize: 12, fontFamily: "monospace", marginBottom: 4 }}>
                {q}
              </div>
              <textarea
                value={answers[q] || ""}
                onChange={(e) => setAnswers({ ...answers, [q]: e.target.value })}
                rows={2}
                style={{
                  width: "100%",
                  maxWidth: 600,
                  padding: "6px 8px",
                  background: "#0f0f23",
                  color: "#ccc",
                  border: "1px solid #2a2a4a",
                  borderRadius: 4,
                  fontFamily: "monospace",
                  fontSize: 12,
                  resize: "vertical",
                }}
              />
            </div>
          ))}
        </div>
      ))}

      <button
        onClick={handleSave}
        style={{
          padding: "8px 24px",
          background: saved ? "#22c55e" : "#fbbf24",
          color: "#000",
          border: "none",
          borderRadius: 4,
          cursor: "pointer",
          fontFamily: "monospace",
          fontSize: 13,
          fontWeight: 600,
        }}
      >
        {saved ? "已保存 ✓" : "保存复盘记录"}
      </button>

      {/* Training mode placeholder */}
      <div
        style={{
          marginTop: 32,
          padding: 16,
          background: "#1a1a2e",
          border: "1px solid #2a2a4a",
          borderRadius: 8,
        }}
      >
        <div style={{ color: "#fbbf24", fontSize: 13, fontFamily: "monospace", marginBottom: 8 }}>
          训练模式（付费功能）
        </div>
        <div style={{ color: "#666", fontSize: 12, fontFamily: "monospace", lineHeight: 1.6 }}>
          使用历史数据逐根K线判断买卖点，隐藏后续走势，完成后对照实际走势检验判断准确性。
          帮助训练盘感，改善入场/离场时机把握。
        </div>
        <button
          disabled
          style={{
            marginTop: 12,
            padding: "4px 16px",
            background: "#3a3a5a",
            color: "#888",
            border: "none",
            borderRadius: 4,
            cursor: "not-allowed",
            fontFamily: "monospace",
            fontSize: 12,
          }}
        >
          升级专业版解锁
        </button>
      </div>
    </div>
  );
}

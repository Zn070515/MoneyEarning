import { useState } from "react";

export interface OnboardingWizardProps {
  onComplete: () => void;
  onLoadDemo: () => Promise<void>;
  onImportTdx?: () => void;
  demoLoading?: boolean;
}

export function OnboardingWizard({
  onComplete,
  onLoadDemo,
  onImportTdx,
  demoLoading,
}: OnboardingWizardProps) {
  const [step, setStep] = useState(0);
  const [riskAcknowledged, setRiskAcknowledged] = useState(false);

  const steps = [
    { title: "加载演示数据", desc: "一键生成演示股票，立刻看到 K 线图" },
    { title: "导入数据（可选）", desc: "导入你的通达信数据或跳过此步" },
    { title: "风险须知", desc: "了解软件定位与投资风险" },
  ];

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 9999,
        background: "rgba(0,0,0,0.85)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontFamily: "monospace",
      }}
    >
      <div
        style={{
          background: "#161616",
          border: "1px solid #2A2A2A",
          borderRadius: 12,
          width: 520,
          maxWidth: "90vw",
          padding: 32,
          color: "#D4D4D4",
        }}
      >
        {/* Progress dots */}
        <div style={{ display: "flex", justifyContent: "center", gap: 8, marginBottom: 24 }}>
          {steps.map((_, i) => (
            <div
              key={i}
              style={{
                width: 8, height: 8, borderRadius: "50%",
                background: i <= step ? "#CCAA00" : "#2A2A2A",
                transition: "background 0.3s",
              }}
            />
          ))}
        </div>

        <h2 style={{ color: "#CCAA00", fontSize: 18, marginBottom: 8, textAlign: "center" }}>
          {steps[step].title}
        </h2>
        <p style={{ color: "#858585", fontSize: 12, textAlign: "center", marginBottom: 24 }}>
          {steps[step].desc}
        </p>

        {/* Step 0: Load demo data */}
        {step === 0 && (
          <div style={{ textAlign: "center" }}>
            <p style={{ fontSize: 13, color: "#858585", marginBottom: 24, lineHeight: 1.8 }}>
              没有数据什么都看不到。
              <br />
              点击下方按钮，立即生成 3 只演示股票（贵州茅台、宁德时代、招商银行），
              <br />
              马上看到第一张 K 线图。
            </p>
            <button
              disabled={demoLoading}
              onClick={async () => {
                try {
                  await onLoadDemo();
                  setStep(1);
                } catch {
                  // keep on step 0, user can retry
                }
              }}
              style={{
                padding: "12px 32px",
                background: "#CCAA00",
                color: "#000",
                border: "none",
                borderRadius: 6,
                cursor: demoLoading ? "wait" : "pointer",
                fontSize: 14,
                fontWeight: 600,
                fontFamily: "monospace",
                opacity: demoLoading ? 0.6 : 1,
              }}
            >
              {demoLoading ? "生成中..." : "⚡ 加载演示数据（3秒搞定）"}
            </button>
          </div>
        )}

        {/* Step 1: Import TDX or skip */}
        {step === 1 && (
          <div style={{ textAlign: "center" }}>
            <p style={{ fontSize: 13, color: "#858585", marginBottom: 16, lineHeight: 1.8 }}>
              如果你电脑里有通达信软件，
              <br />
              可以导入其中的 .day 数据文件——你积累的历史数据立即可用。
              <br />
              此步骤可跳过，之后随时导入。
            </p>
            <div style={{ display: "flex", gap: 12, justifyContent: "center" }}>
              {onImportTdx && (
                <button
                  onClick={onImportTdx}
                  style={{
                    padding: "10px 24px",
                    background: "#2A2A2A",
                    color: "#D4D4D4",
                    border: "1px solid #444444",
                    borderRadius: 6,
                    cursor: "pointer",
                    fontSize: 13,
                    fontFamily: "monospace",
                  }}
                >
                  📥 导入通达信数据
                </button>
              )}
              <button
                onClick={() => setStep(2)}
                style={{
                  padding: "10px 24px",
                  background: "transparent",
                  color: "#858585",
                  border: "1px solid #333333",
                  borderRadius: 6,
                  cursor: "pointer",
                  fontSize: 13,
                  fontFamily: "monospace",
                }}
              >
                跳过 →
              </button>
            </div>
          </div>
        )}

        {/* Step 2: Risk acknowledgement */}
        {step === 2 && (
          <div>
            <div
              style={{
                background: "#0C0C0C",
                border: "1px solid #2A2A2A",
                borderRadius: 8,
                padding: 16,
                marginBottom: 20,
                fontSize: 12,
                lineHeight: 1.8,
                maxHeight: 200,
                overflow: "auto",
              }}
            >
              <p style={{ color: "#CCAA00", fontWeight: 600, marginBottom: 8 }}>
                QuantVault 使用须知
              </p>
              <ul style={{ color: "#858585", paddingLeft: 18, margin: 0 }}>
                <li>本软件是<strong style={{ color: "#D4D4D4" }}>本地数据分析工具</strong>，不提供任何证券投资咨询服务</li>
                <li>不构成任何<strong style={{ color: "#D4D4D4" }}>买入、卖出或持有建议</strong></li>
                <li>所有指标、回测、扫描、预警结果仅用于<strong style={{ color: "#D4D4D4" }}>历史数据分析和用户自我研究</strong></li>
                <li><strong style={{ color: "#EF5350" }}>历史回测表现不代表未来收益</strong></li>
                <li>投资有风险，入市需谨慎。用户需<strong style={{ color: "#D4D4D4" }}>独立判断</strong>并自行承担投资风险</li>
                <li>本软件不推荐股票、不预测价格、不保证收益</li>
                <li>数据完全存储在本地，不上传、不联网（手动下载行情除外）</li>
              </ul>
            </div>
            <label
              style={{
                display: "flex",
                alignItems: "center",
                gap: 10,
                cursor: "pointer",
                fontSize: 13,
                marginBottom: 24,
              }}
            >
              <input
                type="checkbox"
                checked={riskAcknowledged}
                onChange={(e) => setRiskAcknowledged(e.target.checked)}
                style={{ accentColor: "#CCAA00", width: 16, height: 16 }}
              />
              <span style={{ color: "#D4D4D4" }}>
                我已阅读并理解以上声明
              </span>
            </label>
            <div style={{ textAlign: "center" }}>
              <button
                disabled={!riskAcknowledged}
                onClick={onComplete}
                style={{
                  padding: "12px 32px",
                  background: riskAcknowledged ? "#CCAA00" : "#2A2A2A",
                  color: riskAcknowledged ? "#000" : "#666666",
                  border: "none",
                  borderRadius: 6,
                  cursor: riskAcknowledged ? "pointer" : "not-allowed",
                  fontSize: 14,
                  fontWeight: 600,
                  fontFamily: "monospace",
                }}
              >
                开始使用 QuantVault
              </button>
            </div>
          </div>
        )}

        {/* Back button */}
        {step > 0 && (
          <div style={{ textAlign: "center", marginTop: 16 }}>
            <button
              onClick={() => setStep(step - 1)}
              style={{
                background: "none",
                border: "none",
                color: "#858585",
                cursor: "pointer",
                fontSize: 12,
                fontFamily: "monospace",
                textDecoration: "underline",
              }}
            >
              ← 返回上一步
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

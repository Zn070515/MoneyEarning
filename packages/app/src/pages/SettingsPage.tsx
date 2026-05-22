import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/appStore";
import { LicensePanel } from "@me/ui";

interface LicenseStatus {
  valid: boolean;
  tier: string;
  expiry: string | null;
  trial_days_left: number | null;
}

function tierLabel(t: string) {
  switch (t) {
    case "pro":
      return "专业版";
    case "trial":
      return "试用版";
    default:
      return "免费版";
  }
}

export default function SettingsPage() {
  const largeFont = useAppStore((s) => s.largeFont);
  const highContrast = useAppStore((s) => s.highContrast);
  const toggleLargeFont = useAppStore((s) => s.toggleLargeFont);
  const toggleHighContrast = useAppStore((s) => s.toggleHighContrast);
  const [license, setLicense] = useState<LicenseStatus | null>(null);

  const loadLicense = useCallback(async () => {
    try {
      const s = await invoke<LicenseStatus>("check_license");
      setLicense(s);
    } catch {
      // unavailable
    }
  }, []);

  useEffect(() => {
    loadLicense();
  }, [loadLicense]);

  return (
    <div
      style={{
        flex: 1,
        padding: 24,
        overflow: "auto",
        fontFamily: "monospace",
        color: "#ccc",
      }}
    >
      <h2 style={{ color: "#fbbf24", fontSize: 16, marginBottom: 24 }}>
        设置
      </h2>

      {/* Accessibility */}
      <section style={{ marginBottom: 32 }}>
        <h3
          style={{
            color: "#aaa",
            fontSize: 13,
            marginBottom: 12,
            borderBottom: "1px solid #2a2a4a",
            paddingBottom: 8,
          }}
        >
          辅助功能
        </h3>
        <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
          <label
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              cursor: "pointer",
              fontSize: 13,
            }}
          >
            <input
              type="checkbox"
              checked={largeFont}
              onChange={toggleLargeFont}
              style={{ accentColor: "#fbbf24" }}
            />
            <div>
              <div>大字体模式</div>
              <div style={{ color: "#888", fontSize: 11 }}>
                增大所有文字尺寸，适合视力不佳的用户
              </div>
            </div>
          </label>
          <label
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              cursor: "pointer",
              fontSize: 13,
            }}
          >
            <input
              type="checkbox"
              checked={highContrast}
              onChange={toggleHighContrast}
              style={{ accentColor: "#fbbf24" }}
            />
            <div>
              <div>高对比度模式</div>
              <div style={{ color: "#888", fontSize: 11 }}>
                增强颜色对比度，提升可读性
              </div>
            </div>
          </label>
        </div>
      </section>

      {/* License */}
      <section style={{ marginBottom: 32 }}>
        <h3
          style={{
            color: "#aaa",
            fontSize: 13,
            marginBottom: 12,
            borderBottom: "1px solid #2a2a4a",
            paddingBottom: 8,
          }}
        >
          授权管理
        </h3>
        {license && (
          <div
            style={{
              marginBottom: 16,
              padding: "10px 14px",
              background: "#16213e",
              borderRadius: 6,
              border: "1px solid #2a2a4a",
              fontSize: 13,
            }}
          >
            <span style={{ color: "#888" }}>当前状态: </span>
            <span style={{ color: "#fbbf24", fontWeight: 600 }}>
              {tierLabel(license.tier)}
            </span>
            {license.expiry && (
              <span style={{ color: "#888", marginLeft: 12, fontSize: 11 }}>
                有效期至 {license.expiry}
              </span>
            )}
            {license.trial_days_left != null && (
              <span style={{ color: "#888", marginLeft: 12, fontSize: 11 }}>
                试用剩余 {license.trial_days_left} 天
              </span>
            )}
          </div>
        )}
        <LicensePanel onActivated={loadLicense} />
      </section>

      {/* About */}
      <section>
        <h3
          style={{
            color: "#aaa",
            fontSize: 13,
            marginBottom: 12,
            borderBottom: "1px solid #2a2a4a",
            paddingBottom: 8,
          }}
        >
          关于
        </h3>
        <div style={{ fontSize: 12, color: "#888", lineHeight: 1.8 }}>
          <div>MoneyEarning v0.6.0</div>
          <div>本地量化分析工作站</div>
          <div>Tauri 2.x + Rust + React/TypeScript</div>
        </div>
      </section>
    </div>
  );
}

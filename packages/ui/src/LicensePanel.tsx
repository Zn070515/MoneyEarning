import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LicenseInfo {
  tier: string;
  expiry: string | null;
  features: string[];
  valid: boolean;
}

interface LicenseStatus {
  valid: boolean;
  tier: string;
  expiry: string | null;
  trial_days_left: number | null;
}

interface LicensePanelProps {
  onActivated?: () => void;
}

export function LicensePanel({ onActivated }: LicensePanelProps) {
  const [status, setStatus] = useState<LicenseStatus | null>(null);
  const [fingerprint, setFingerprint] = useState("");
  const [licenseKey, setLicenseKey] = useState("");
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");
  const [messageType, setMessageType] = useState<"success" | "error" | "">("");

  const loadStatus = useCallback(async () => {
    try {
      const s = await invoke<LicenseStatus>("check_license");
      setStatus(s);
    } catch (e) {
      console.error("License check failed:", e);
    }
  }, []);

  const loadFingerprint = useCallback(async () => {
    try {
      const fp = await invoke<string>("get_machine_fingerprint");
      setFingerprint(fp);
    } catch (e) {
      console.error("Fingerprint failed:", e);
    }
  }, []);

  useEffect(() => {
    loadStatus();
    loadFingerprint();
  }, [loadStatus, loadFingerprint]);

  const handleActivate = async () => {
    if (!licenseKey.trim()) return;
    setLoading(true);
    setMessage("");
    try {
      const info = await invoke<LicenseInfo>("activate_license", {
        licenseKey: licenseKey.trim(),
        fingerprint,
      });
      if (info.valid) {
        setMessage("激活成功！");
        setMessageType("success");
        onActivated?.();
        loadStatus();
      } else {
        setMessage("激活失败：无效的授权码");
        setMessageType("error");
      }
    } catch (e) {
      setMessage(`激活失败：${e}`);
      setMessageType("error");
    }
    setLoading(false);
  };

  const tierLabel = (t: string) => {
    switch (t) {
      case "pro": return "专业版";
      case "free": return "免费版";
      case "trial": return "试用版";
      default: return t;
    }
  };

  const tierColor = (t: string) => {
    switch (t) {
      case "pro": return "#CCAA00";
      case "free": return "#26A69A";
      case "trial": return "#60a5fa";
      default: return "#858585";
    }
  };

  return (
    <div style={{
      background: "#161616", color: "#D4D4D4", fontFamily: "monospace",
      fontSize: 13, height: "100%", display: "flex", flexDirection: "column",
      padding: 16,
    }}>
      <h3 style={{ margin: "0 0 16px", color: "#fff", fontSize: 15 }}>
        授权管理
      </h3>

      {/* Status */}
      {status && (
        <div style={{
          padding: 12, background: "#121212", borderRadius: 6,
          border: `1px solid ${tierColor(status.tier)}`,
          marginBottom: 16,
        }}>
          <div style={{
            fontSize: 18, fontWeight: 700, color: tierColor(status.tier),
            marginBottom: 6,
          }}>
            {tierLabel(status.tier)}
          </div>
          {status.expiry && (
            <div style={{ fontSize: 12, color: "#858585", marginBottom: 4 }}>
              有效期至：{status.expiry}
            </div>
          )}
          {status.trial_days_left != null && status.trial_days_left > 0 && (
            <div style={{ fontSize: 12, color: "#60a5fa" }}>
              试用剩余：{status.trial_days_left} 天
            </div>
          )}
          {status.trial_days_left != null && status.trial_days_left <= 0 && (
            <div style={{ fontSize: 12, color: "#EF5350" }}>
              试用已过期
            </div>
          )}
        </div>
      )}

      {/* Trial expiry upgrade prompt */}
      {status && status.trial_days_left != null && status.trial_days_left <= 3 && (
        <div style={{
          padding: 12, marginBottom: 16, borderRadius: 6,
          background: status.trial_days_left <= 0
            ? "rgba(239,83,80,0.08)" : "rgba(255,167,38,0.08)",
          border: status.trial_days_left <= 0
            ? "1px solid rgba(239,83,80,0.3)" : "1px solid rgba(255,167,38,0.3)",
        }}>
          <div style={{
            color: status.trial_days_left <= 0 ? "#EF5350" : "#FFA726",
            fontSize: 13, fontWeight: 600, fontFamily: "monospace", marginBottom: 6,
          }}>
            {status.trial_days_left <= 0
              ? "🔒 试用已到期"
              : `⏰ 试用还剩 ${status.trial_days_left} 天`}
          </div>
          <div style={{ color: "#858585", fontSize: 11, fontFamily: "monospace", lineHeight: 1.5, marginBottom: 8 }}>
            升级到专业版以解锁全部功能：
            <ul style={{ margin: "4px 0", paddingLeft: 18 }}>
              <li>全部27个PRO交易策略模板</li>
              <li>参数优化 & 遗传算法搜索</li>
              <li>Walk-Forward 分析 & Monte Carlo 模拟</li>
              <li>组合扫描（CAPS / CGPC / MARS）</li>
              <li>筹码分布 & 历史帧分析</li>
            </ul>
          </div>
          <a href="https://zn070515.github.io/MoneyEarning/#pricing"
            target="_blank" rel="noopener noreferrer"
            style={{
              display: "inline-block", padding: "5px 14px",
              background: "#CCAA00", color: "#000",
              borderRadius: 4, textDecoration: "none",
              fontSize: 12, fontWeight: 600, fontFamily: "monospace",
            }}>
            获取专业版 →
          </a>
        </div>
      )}

      {/* Fingerprint */}
      <div style={{ marginBottom: 16 }}>
        <label style={{ fontSize: 12, color: "#858585", marginBottom: 4, display: "block" }}>
          机器指纹
        </label>
        <div style={{
          padding: 8, background: "#121212", borderRadius: 4,
          fontSize: 11, color: "#666666", wordBreak: "break-all",
          fontFamily: "monospace",
        }}>
          {fingerprint || "获取中..."}
        </div>
      </div>

      {/* Activation */}
      <div style={{ marginBottom: 12 }}>
        <label style={{ fontSize: 12, color: "#858585", marginBottom: 4, display: "block" }}>
          授权码
        </label>
        <textarea
          value={licenseKey}
          onChange={e => setLicenseKey(e.target.value)}
          onPaste={e => {
            const pasted = e.clipboardData.getData("text");
            // Auto-clean: remove whitespace, normalize dashes, remove surrounding quotes
            const cleaned = pasted
              .replace(/^["']|["']$/g, "")
              .replace(/\s+/g, "")
              .replace(/—/g, "-")
              .replace(/–/g, "-");
            e.preventDefault();
            setLicenseKey(cleaned);
            setMessage("已粘贴并自动格式化授权码");
            setMessageType("success");
          }}
          placeholder="粘贴授权码（自动格式化）..."
          rows={4}
          style={{
            width: "100%", background: "#121212", border: "1px solid #2A2A2A",
            color: "#fff", padding: "8px", borderRadius: 4, fontSize: 12,
            fontFamily: "monospace", outline: "none", resize: "vertical",
            boxSizing: "border-box",
          }}
        />
      </div>

      {/* Message */}
      {message && (
        <div style={{
          padding: 8, borderRadius: 4, fontSize: 12, marginBottom: 12,
          background: messageType === "success" ? "#1a3a2e" : "#3a1a2e",
          color: messageType === "success" ? "#26A69A" : "#EF5350",
        }}>
          {message}
        </div>
      )}

      <button onClick={handleActivate} disabled={loading || !licenseKey.trim()}
        style={{
          background: loading ? "#2A2210" : "#CCAA00",
          color: "#000", border: "none",
          padding: "8px 16px", borderRadius: 4,
          cursor: loading ? "not-allowed" : "pointer",
          fontSize: 13, fontWeight: 600,
        }}>
        {loading ? "激活中..." : "激活授权"}
      </button>
    </div>
  );
}

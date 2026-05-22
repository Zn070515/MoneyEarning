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
      case "pro": return "#fbbf24";
      case "free": return "#22c55e";
      case "trial": return "#60a5fa";
      default: return "#888";
    }
  };

  return (
    <div style={{
      background: "#16213e", color: "#ccc", fontFamily: "monospace",
      fontSize: 13, height: "100%", display: "flex", flexDirection: "column",
      padding: 16,
    }}>
      <h3 style={{ margin: "0 0 16px", color: "#fff", fontSize: 15 }}>
        授权管理
      </h3>

      {/* Status */}
      {status && (
        <div style={{
          padding: 12, background: "#1a1a2e", borderRadius: 6,
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
            <div style={{ fontSize: 12, color: "#888", marginBottom: 4 }}>
              有效期至：{status.expiry}
            </div>
          )}
          {status.trial_days_left != null && status.trial_days_left > 0 && (
            <div style={{ fontSize: 12, color: "#60a5fa" }}>
              试用剩余：{status.trial_days_left} 天
            </div>
          )}
          {status.trial_days_left != null && status.trial_days_left <= 0 && (
            <div style={{ fontSize: 12, color: "#ef4444" }}>
              试用已过期
            </div>
          )}
        </div>
      )}

      {/* Fingerprint */}
      <div style={{ marginBottom: 16 }}>
        <label style={{ fontSize: 12, color: "#888", marginBottom: 4, display: "block" }}>
          机器指纹
        </label>
        <div style={{
          padding: 8, background: "#1a1a2e", borderRadius: 4,
          fontSize: 11, color: "#666", wordBreak: "break-all",
          fontFamily: "monospace",
        }}>
          {fingerprint || "获取中..."}
        </div>
      </div>

      {/* Activation */}
      <div style={{ marginBottom: 12 }}>
        <label style={{ fontSize: 12, color: "#888", marginBottom: 4, display: "block" }}>
          授权码
        </label>
        <textarea
          value={licenseKey}
          onChange={e => setLicenseKey(e.target.value)}
          placeholder="粘贴授权码..."
          rows={4}
          style={{
            width: "100%", background: "#1a1a2e", border: "1px solid #3a3a5a",
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
          color: messageType === "success" ? "#22c55e" : "#ef4444",
        }}>
          {message}
        </div>
      )}

      <button onClick={handleActivate} disabled={loading || !licenseKey.trim()}
        style={{
          background: loading ? "#8a7a3a" : "#fbbf24",
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

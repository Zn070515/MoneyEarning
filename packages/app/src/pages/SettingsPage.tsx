import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore, type LicenseStatus } from "../stores/appStore";
import { LicensePanel, DataPanel } from "@me/ui";

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
  const refreshLicense = useAppStore((s) => s.refreshLicense);
  const toggleLargeFont = useAppStore((s) => s.toggleLargeFont);
  const toggleHighContrast = useAppStore((s) => s.toggleHighContrast);
  const [license, setLicense] = useState<LicenseStatus | null>(null);

  const loadLicense = useCallback(async () => {
    try {
      const s = await refreshLicense();
      setLicense(s);
    } catch {
      // unavailable
    }
  }, [refreshLicense]);

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
        color: "#D4D4D4",
      }}
    >
      <h2 style={{ color: "#CCAA00", fontSize: 16, marginBottom: 24 }}>
        设置
      </h2>

      {/* Accessibility */}
      <section style={{ marginBottom: 32 }}>
        <h3
          style={{
            color: "#858585",
            fontSize: 13,
            marginBottom: 12,
            borderBottom: "1px solid #2A2A2A",
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
              style={{ accentColor: "#CCAA00" }}
            />
            <div>
              <div>大字体模式</div>
              <div style={{ color: "#858585", fontSize: 11 }}>
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
              style={{ accentColor: "#CCAA00" }}
            />
            <div>
              <div>高对比度模式</div>
              <div style={{ color: "#858585", fontSize: 11 }}>
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
            color: "#858585",
            fontSize: 13,
            marginBottom: 12,
            borderBottom: "1px solid #2A2A2A",
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
              background: "#161616",
              borderRadius: 6,
              border: "1px solid #2A2A2A",
              fontSize: 13,
            }}
          >
            <span style={{ color: "#858585" }}>当前状态: </span>
            <span style={{ color: "#CCAA00", fontWeight: 600 }}>
              {tierLabel(license.tier)}
            </span>
            {license.expiry && (
              <span style={{ color: "#858585", marginLeft: 12, fontSize: 11 }}>
                有效期至 {license.expiry}
              </span>
            )}
            {license.trial_days_left != null && (
              <span style={{ color: "#858585", marginLeft: 12, fontSize: 11 }}>
                试用剩余 {license.trial_days_left} 天
              </span>
            )}
          </div>
        )}
        <LicensePanel onActivated={loadLicense} />
      </section>

      {/* Data Management */}
      <section style={{ marginBottom: 32 }}>
        <h3
          style={{
            color: "#858585",
            fontSize: 13,
            marginBottom: 12,
            borderBottom: "1px solid #2A2A2A",
            paddingBottom: 8,
          }}
        >
          数据管理
        </h3>
        <div style={{ height: 400, border: "1px solid #2A2A2A", borderRadius: 8, overflow: "hidden", marginBottom: 16 }}>
          <DataPanel />
        </div>
        {/* Backup / Restore */}
        <div style={{ display: "flex", gap: 12, marginTop: 8 }}>
          <button
            onClick={async () => {
              try {
                const { save } = await import("@tauri-apps/plugin-dialog");
                const path = await save({
                  defaultPath: `quantvault_backup_${new Date().toISOString().slice(0, 10).replace(/-/g, "")}.db`,
                  filters: [{ name: "SQLite Database", extensions: ["db"] }],
                });
                if (path) {
                  const result = await invoke<string>("backup_database", { destPath: path });
                  alert(result);
                }
              } catch (e: any) {
                alert("备份失败: " + (e?.toString?.() ?? String(e)));
              }
            }}
            style={{
              padding: "8px 16px",
              background: "#2A2A2A",
              border: "1px solid #444444",
              color: "#D4D4D4",
              borderRadius: 4,
              cursor: "pointer",
              fontFamily: "monospace",
              fontSize: 12,
            }}
          >
            📥 备份数据
          </button>
          <button
            onClick={async () => {
              try {
                const { open } = await import("@tauri-apps/plugin-dialog");
                const path = await open({
                  filters: [{ name: "SQLite Database", extensions: ["db"] }],
                  multiple: false,
                });
                if (path) {
                  const confirmed = window.confirm("恢复数据将替换当前所有数据。\n\n恢复前会自动备份当前数据，是否继续？");
                  if (confirmed) {
                    const result = await invoke<string>("restore_database", { backupPath: path });
                    alert(result + "\n\n请重新启动应用以加载恢复的数据。");
                  }
                }
              } catch (e: any) {
                alert("恢复失败: " + (e?.toString?.() ?? String(e)));
              }
            }}
            style={{
              padding: "8px 16px",
              background: "#2A2A2A",
              border: "1px solid #444444",
              color: "#D4D4D4",
              borderRadius: 4,
              cursor: "pointer",
              fontFamily: "monospace",
              fontSize: 12,
            }}
          >
            📤 恢复数据
          </button>
        </div>
        <div style={{ fontSize: 11, color: "#666666", marginTop: 8 }}>
          备份文件包含你的行情数据、交易记录和预警规则。建议定期备份。
        </div>
      </section>

      {/* About */}
      <section>
        <h3
          style={{
            color: "#858585",
            fontSize: 13,
            marginBottom: 12,
            borderBottom: "1px solid #2A2A2A",
            paddingBottom: 8,
          }}
        >
          关于
        </h3>
        <div style={{ fontSize: 12, color: "#858585", lineHeight: 1.8 }}>
          <div>QuantVault v0.10.0</div>
          <div>本地量化分析工作站</div>
          <div>Tauri 2.x + Rust + React/TypeScript</div>
        </div>

        {/* Share button */}
        <button
          onClick={() => {
            const link = "https://zn070515.github.io/MoneyEarning/";
            navigator.clipboard.writeText(link).then(() => {
              alert("推荐链接已复制到剪贴板！\n\n分享链接: " + link);
            }).catch(() => {
              prompt("复制此链接分享给朋友：", link);
            });
          }}
          style={{
            marginTop: 12,
            padding: "6px 14px",
            background: "transparent",
            border: "1px solid #CCAA00",
            color: "#CCAA00",
            borderRadius: 4,
            cursor: "pointer",
            fontFamily: "monospace",
            fontSize: 12,
          }}
        >
          🔗 推荐给朋友
        </button>
      </section>
    </div>
  );
}

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface ImportResult {
  stock_count: number;
  row_count: number;
  skipped: number;
  date_range: [string, string] | null;
}

interface ImportDialogProps {
  visible: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export function ImportDialog({ visible, onClose, onSuccess }: ImportDialogProps) {
  const [filePath, setFilePath] = useState("");
  const [stockCode, setStockCode] = useState("");
  const [exchange, setExchange] = useState("SZ");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState("");

  if (!visible) return null;

  const handleImport = async () => {
    if (!filePath.trim() || !stockCode.trim()) return;
    setLoading(true);
    setError("");
    try {
      const res = await invoke<ImportResult>("import_csv", {
        filePath: filePath.trim(),
        stockCode: stockCode.trim(),
        exchange,
      });
      setResult(res);
      onSuccess();
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  };

  return (
    <div style={{
      position: "fixed", inset: 0, background: "rgba(0,0,0,0.6)",
      display: "flex", alignItems: "center", justifyContent: "center",
      zIndex: 1000,
    }} onClick={onClose}>
      <div onClick={e => e.stopPropagation()} style={{
        background: "#111827", border: "1px solid #1E293B",
        borderRadius: 8, padding: 24, width: 480, maxWidth: "90vw",
        fontFamily: "monospace", color: "#F1F5F9", fontSize: 13,
      }}>
        <h2 style={{ margin: "0 0 20px", color: "#00D8FF", fontSize: 16 }}>
          导入CSV数据
        </h2>

        {/* File path */}
        <div style={{ marginBottom: 14 }}>
          <label style={{ fontSize: 12, color: "#94A3B8", marginBottom: 4, display: "block" }}>
            CSV文件路径
          </label>
          <div style={{ display: "flex", gap: 8 }}>
            <input value={filePath}
              onChange={e => setFilePath(e.target.value)}
              placeholder='选择或输入 .csv 文件路径'
              style={{ ...inputStyle, flex: 1 }} />
            <button onClick={async () => {
              const selected = await open({
                multiple: false,
                filters: [{ name: "CSV文件", extensions: ["csv"] }],
              });
              if (selected && typeof selected === "string") {
                setFilePath(selected);
              }
            }} style={{
              background: "#141b2d", border: "1px solid #1E293B",
              color: "#00D8FF", padding: "6px 14px", borderRadius: 4,
              cursor: "pointer", fontSize: 12, fontFamily: "monospace",
              whiteSpace: "nowrap",
            }}>
              浏览...
            </button>
          </div>
        </div>

        {/* Code + Exchange */}
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 14 }}>
          <div>
            <label style={{ fontSize: 12, color: "#94A3B8", marginBottom: 4, display: "block" }}>
              股票代码
            </label>
            <input value={stockCode}
              onChange={e => setStockCode(e.target.value)}
              placeholder='如 "000001"'
              style={inputStyle} />
          </div>
          <div>
            <label style={{ fontSize: 12, color: "#94A3B8", marginBottom: 4, display: "block" }}>
              交易所
            </label>
            <select value={exchange}
              onChange={e => setExchange(e.target.value)}
              style={inputStyle}>
              <option value="SZ">深圳 (SZ)</option>
              <option value="SH">上海 (SH)</option>
              <option value="BJ">北京 (BJ)</option>
              <option value="HK">香港 (HK)</option>
              <option value="US">美股 (US)</option>
            </select>
          </div>
        </div>

        {/* Format hint */}
        <div style={{
          padding: 8, background: "#141b2d", borderRadius: 4,
          fontSize: 11, color: "#94A3B8", marginBottom: 14,
        }}>
          支持格式：trade_date, open, high, low, close, volume[, amount, turnover]
          <br />
          日期格式：YYYY-MM-DD，编码：UTF-8 或 GBK
        </div>

        {/* Result */}
        {result && (
          <div style={{
            padding: 10, background: "#1a3a2e", borderRadius: 4,
            fontSize: 12, marginBottom: 14,
          }}>
            <div style={{ color: "#00E676", fontWeight: 600, marginBottom: 4 }}>
              导入成功
            </div>
            <div>股票：{result.stock_count}只，数据行：{result.row_count}条</div>
            {result.skipped > 0 && (
              <div style={{ color: "#00D8FF" }}>跳过重复行：{result.skipped}条</div>
            )}
            {result.date_range && (
              <div style={{ color: "#94A3B8" }}>
                日期范围：{result.date_range[0]} ~ {result.date_range[1]}
              </div>
            )}
          </div>
        )}

        {/* Error */}
        {error && (
          <div style={{
            padding: 10, background: "#3a1a2e", borderRadius: 4,
            color: "#FF2A7A", fontSize: 12, marginBottom: 14,
          }}>
            {error}
          </div>
        )}

        {/* Actions */}
        <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
          <button onClick={onClose} style={{
            background: "transparent", border: "1px solid #1E293B",
            color: "#F1F5F9", padding: "6px 16px", borderRadius: 4,
            cursor: "pointer", fontSize: 12,
          }}>
            关闭
          </button>
          <button onClick={handleImport} disabled={loading || !filePath.trim() || !stockCode.trim()}
            style={{
              background: loading ? "#8a7a3a" : "#00D8FF",
              color: "#000", border: "none",
              padding: "6px 16px", borderRadius: 4,
              cursor: loading ? "not-allowed" : "pointer",
              fontSize: 12, fontWeight: 600,
            }}>
            {loading ? "导入中..." : "开始导入"}
          </button>
        </div>
      </div>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  width: "100%", background: "#141b2d", border: "1px solid #1E293B",
  color: "#fff", padding: "6px 8px", borderRadius: 4, fontSize: 12,
  fontFamily: "monospace", outline: "none", boxSizing: "border-box",
};

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

type ImportMode = "csv" | "tdx";

export function ImportDialog({ visible, onClose, onSuccess }: ImportDialogProps) {
  const [mode, setMode] = useState<ImportMode>("csv");
  const [filePath, setFilePath] = useState("");
  const [dirPath, setDirPath] = useState("");
  const [stockCode, setStockCode] = useState("");
  const [exchange, setExchange] = useState("SZ");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [results, setResults] = useState<ImportResult[] | null>(null);
  const [error, setError] = useState("");

  if (!visible) return null;

  const reset = () => {
    setResult(null);
    setResults(null);
    setError("");
  };

  const handleCsvImport = async () => {
    if (!filePath.trim() || !stockCode.trim()) return;
    setLoading(true);
    setError("");
    reset();
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

  const handleTdxFileImport = async () => {
    if (!filePath.trim()) return;
    setLoading(true);
    setError("");
    reset();
    try {
      const res = await invoke<ImportResult>("import_tdx_day", {
        filePath: filePath.trim(),
        stockCode: stockCode.trim() || null,
        exchange: stockCode.trim() ? exchange : null,
      });
      setResult(res);
      onSuccess();
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  };

  const handleTdxDirImport = async () => {
    if (!dirPath.trim()) return;
    setLoading(true);
    setError("");
    reset();
    try {
      const res = await invoke<ImportResult[]>("import_tdx_directory", {
        dirPath: dirPath.trim(),
      });
      setResults(res);
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
        background: "#161616", border: "1px solid #2A2A2A",
        borderRadius: 8, padding: 24, width: 520, maxWidth: "90vw",
        maxHeight: "90vh", overflow: "auto",
        fontFamily: "monospace", color: "#D4D4D4", fontSize: 13,
      }}>
        <h2 style={{ margin: "0 0 16px", color: "#CCAA00", fontSize: 16 }}>
          导入数据
        </h2>

        {/* Mode tabs */}
        <div style={{ display: "flex", gap: 4, marginBottom: 20 }}>
          {(["csv", "tdx"] as ImportMode[]).map(key => (
            <button key={key} onClick={() => { setMode(key); reset(); }}
              style={{
                padding: "6px 16px",
                background: mode === key ? "#CCAA00" : "#121212",
                color: mode === key ? "#000" : "#858585",
                border: "none", borderRadius: 4, cursor: "pointer",
                fontFamily: "monospace", fontSize: 12, fontWeight: 600,
              }}>
              {key === "csv" ? "CSV 文件" : "通达信 .day"}
            </button>
          ))}
        </div>

        {/* ── CSV Mode ── */}
        {mode === "csv" && (
          <>
            <div style={{ marginBottom: 14 }}>
              <label style={labelStyle}>CSV文件路径</label>
              <div style={{ display: "flex", gap: 8 }}>
                <input value={filePath} onChange={e => setFilePath(e.target.value)}
                  placeholder="选择或输入 .csv 文件路径"
                  style={{ ...inputStyle, flex: 1 }} />
                <button onClick={async () => {
                  const selected = await open({
                    multiple: false,
                    filters: [{ name: "CSV文件", extensions: ["csv"] }],
                  });
                  if (selected && typeof selected === "string") setFilePath(selected);
                }} style={browseBtnStyle}>浏览...</button>
              </div>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 14 }}>
              <div>
                <label style={labelStyle}>股票代码</label>
                <input value={stockCode} onChange={e => setStockCode(e.target.value)}
                  placeholder='如 "000001"' style={inputStyle} />
              </div>
              <div>
                <label style={labelStyle}>交易所</label>
                <select value={exchange} onChange={e => setExchange(e.target.value)} style={inputStyle}>
                  <option value="SZ">深圳 (SZ)</option>
                  <option value="SH">上海 (SH)</option>
                  <option value="BJ">北京 (BJ)</option>
                  <option value="HK">香港 (HK)</option>
                  <option value="US">美股 (US)</option>
                </select>
              </div>
            </div>
            <FormatHint text="支持格式：trade_date, open, high, low, close, volume[, amount, turnover]&#10;日期格式：YYYY-MM-DD，编码：UTF-8 或 GBK" />
            <ResultBlock result={result} error={error} />
            <Actions onClose={onClose} onAction={handleCsvImport}
              loading={loading} disabled={!filePath.trim() || !stockCode.trim()}
              btnLabel="开始导入" />
          </>
        )}

        {/* ── TDX Mode ── */}
        {mode === "tdx" && (
          <>
            {/* Single file import */}
            <div style={{
              padding: 12, background: "#121212", borderRadius: 6,
              border: "1px solid #2A2A2A", marginBottom: 16,
            }}>
              <div style={{ color: "#CCAA00", fontSize: 12, fontWeight: 600, marginBottom: 10 }}>
                导入单个 .day 文件
              </div>
              <div style={{ marginBottom: 10 }}>
                <label style={labelStyle}>.day 文件路径</label>
                <div style={{ display: "flex", gap: 8 }}>
                  <input value={filePath} onChange={e => setFilePath(e.target.value)}
                    placeholder="如 sh600519.day"
                    style={{ ...inputStyle, flex: 1 }} />
                  <button onClick={async () => {
                    const selected = await open({
                      multiple: false,
                      filters: [{ name: "通达信日线", extensions: ["day"] }],
                    });
                    if (selected && typeof selected === "string") {
                      setFilePath(selected);
                      // Extract code from filename
                      const stem = selected.replace(/\\/g, "/").split("/").pop()?.replace(".day", "") || "";
                      if (stem.length >= 3) {
                        const code = stem.startsWith("sh") || stem.startsWith("sz") || stem.startsWith("bj")
                          ? stem.slice(2) : stem;
                        if (code.length >= 6) setStockCode(code.slice(0, 6));
                      }
                    }
                  }} style={browseBtnStyle}>浏览...</button>
                </div>
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10, marginBottom: 10 }}>
                <div>
                  <label style={labelStyle}>股票代码（可留空，从文件名识别）</label>
                  <input value={stockCode} onChange={e => setStockCode(e.target.value)}
                    placeholder="自动识别" style={inputStyle} />
                </div>
                <div>
                  <label style={labelStyle}>交易所（可留空）</label>
                  <select value={exchange} onChange={e => setExchange(e.target.value)} style={inputStyle}>
                    <option value="SZ">深圳 (SZ)</option>
                    <option value="SH">上海 (SH)</option>
                    <option value="BJ">北京 (BJ)</option>
                  </select>
                </div>
              </div>
              <FormatHint text="通达信 .day 二进制日线格式（每条 32 字节）&#10;文件名自动识别代码+交易所，如 sh600519.day → SH/600519" />
              <ResultBlock result={result} error={error} />
              <div style={{ display: "flex", justifyContent: "flex-end", marginTop: 8 }}>
                <button onClick={handleTdxFileImport} disabled={loading || !filePath.trim()}
                  style={actionBtnStyle(loading)}>
                  {loading ? "导入中..." : "导入此文件"}
                </button>
              </div>
            </div>

            {/* Directory batch import */}
            <div style={{
              padding: 12, background: "#121212", borderRadius: 6,
              border: "1px solid #2A2A2A",
            }}>
              <div style={{ color: "#CCAA00", fontSize: 12, fontWeight: 600, marginBottom: 10 }}>
                批量导入目录
              </div>
              <div style={{ marginBottom: 10 }}>
                <label style={labelStyle}>通达信数据目录（如 vipdoc/ 或 T0002/export/）</label>
                <div style={{ display: "flex", gap: 8 }}>
                  <input value={dirPath} onChange={e => setDirPath(e.target.value)}
                    placeholder="选择通达信数据根目录..."
                    style={{ ...inputStyle, flex: 1 }} />
                  <button onClick={async () => {
                    const selected = await open({ directory: true, multiple: false });
                    if (selected && typeof selected === "string") setDirPath(selected);
                  }} style={browseBtnStyle}>浏览...</button>
                </div>
              </div>
              <FormatHint text="递归扫描目录下所有 .day 文件（最大深度 4 层）&#10;支持通达信标准目录结构 vipdoc/sh/lday/*.day" />
              {results && results.length > 0 && (
                <div style={{
                  padding: 10, background: "#1a3a2e", borderRadius: 4,
                  fontSize: 11, marginBottom: 10, maxHeight: 120, overflow: "auto",
                }}>
                  <div style={{ color: "#26A69A", fontWeight: 600, marginBottom: 4 }}>
                    批量导入完成（{results.length} 只股票）
                  </div>
                  {results.map((r, i) => (
                    <div key={i} style={{ color: "#858585" }}>
                      #{i + 1}: {r.row_count}条
                      {r.date_range ? ` · ${r.date_range[0]}~${r.date_range[1]}` : ""}
                      {r.skipped > 0 ? ` (跳过${r.skipped})` : ""}
                    </div>
                  ))}
                </div>
              )}
              {error && (
                <div style={{
                  padding: 10, background: "#3a1a2e", borderRadius: 4,
                  color: "#EF5350", fontSize: 12, marginBottom: 10,
                }}>
                  {error}
                </div>
              )}
              <div style={{ display: "flex", justifyContent: "flex-end" }}>
                <button onClick={handleTdxDirImport} disabled={loading || !dirPath.trim()}
                  style={actionBtnStyle(loading)}>
                  {loading ? "扫描导入中..." : "扫描并导入"}
                </button>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function FormatHint({ text }: { text: string }) {
  return (
    <div style={{
      padding: 8, background: "#0C0C0C", borderRadius: 4,
      fontSize: 10, color: "#858585", lineHeight: 1.6,
      whiteSpace: "pre-line", marginBottom: 10,
    }}>
      {text}
    </div>
  );
}

function ResultBlock({ result, error }: { result: ImportResult | null; error: string }) {
  return (
    <>
      {result && (
        <div style={{
          padding: 10, background: "#1a3a2e", borderRadius: 4,
          fontSize: 12, marginBottom: 10,
        }}>
          <div style={{ color: "#26A69A", fontWeight: 600, marginBottom: 4 }}>导入成功</div>
          <div style={{ color: "#D4D4D4" }}>数据行：{result.row_count}条</div>
          {result.skipped > 0 && (
            <div style={{ color: "#CCAA00" }}>跳过重复行：{result.skipped}条</div>
          )}
          {result.date_range && (
            <div style={{ color: "#858585" }}>
              日期范围：{result.date_range[0]} ~ {result.date_range[1]}
            </div>
          )}
        </div>
      )}
      {error && (
        <div style={{
          padding: 10, background: "#3a1a2e", borderRadius: 4,
          color: "#EF5350", fontSize: 12, marginBottom: 10,
        }}>
          {error}
        </div>
      )}
    </>
  );
}

function Actions({ onClose, onAction, loading, disabled, btnLabel }: {
  onClose: () => void;
  onAction: () => void;
  loading: boolean;
  disabled: boolean;
  btnLabel: string;
}) {
  return (
    <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
      <button onClick={onClose} style={{
        background: "transparent", border: "1px solid #2A2A2A",
        color: "#D4D4D4", padding: "6px 16px", borderRadius: 4,
        cursor: "pointer", fontSize: 12,
      }}>
        关闭
      </button>
      <button onClick={onAction} disabled={loading || disabled}
        style={actionBtnStyle(loading)}>
        {loading ? "导入中..." : btnLabel}
      </button>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  width: "100%", background: "#121212", border: "1px solid #2A2A2A",
  color: "#fff", padding: "6px 8px", borderRadius: 4, fontSize: 12,
  fontFamily: "monospace", outline: "none", boxSizing: "border-box",
};

const labelStyle: React.CSSProperties = {
  fontSize: 11, color: "#858585", marginBottom: 4, display: "block",
};

const browseBtnStyle: React.CSSProperties = {
  background: "#121212", border: "1px solid #2A2A2A",
  color: "#CCAA00", padding: "6px 14px", borderRadius: 4,
  cursor: "pointer", fontSize: 12, fontFamily: "monospace",
  whiteSpace: "nowrap",
};

function actionBtnStyle(loading: boolean): React.CSSProperties {
  return {
    background: loading ? "#8a7a3a" : "#CCAA00",
    color: "#000", border: "none",
    padding: "6px 16px", borderRadius: 4,
    cursor: loading ? "not-allowed" : "pointer",
    fontSize: 12, fontWeight: 600, fontFamily: "monospace",
  };
}

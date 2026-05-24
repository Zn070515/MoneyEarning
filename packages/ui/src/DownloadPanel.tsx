import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StockListItem {
  code: string;
  name: string;
  price: number;
  change_pct: number;
}

interface DownloadSummary {
  code: string;
  name: string;
  rows_inserted: number;
  date_range: [string, string] | null;
}

interface MinuteImportSummary {
  code: string;
  klt: number;
  klt_label: string;
  rows_inserted: number;
  time_range: [string, string] | null;
}

export function DownloadPanel() {
  const [stockList, setStockList] = useState<StockListItem[]>([]);
  const [stockListLoading, setStockListLoading] = useState(false);
  const [stockListStatus, setStockListStatus] = useState("");

  const [dlCode, setDlCode] = useState("");
  const [dlName, setDlName] = useState("");
  const [dlLoading, setDlLoading] = useState(false);
  const [dlResult, setDlResult] = useState<DownloadSummary | null>(null);
  const [dlError, setDlError] = useState("");

  // Filter/search for stock list
  const [search, setSearch] = useState("");

  // Minute data state
  const [minCode, setMinCode] = useState("");
  const [minKlt, setMinKlt] = useState(5);
  const [minLoading, setMinLoading] = useState(false);
  const [minResult, setMinResult] = useState<MinuteImportSummary | null>(null);
  const [minError, setMinError] = useState("");

  const downloadMinuteData = useCallback(async () => {
    if (!minCode.trim()) return;
    setMinLoading(true);
    setMinError("");
    setMinResult(null);
    try {
      const summary = await invoke<MinuteImportSummary>("download_minute_data", {
        code: minCode.trim(),
        klt: minKlt,
      });
      setMinResult(summary);
    } catch (e) {
      setMinError(String(e));
    }
    setMinLoading(false);
  }, [minCode, minKlt]);

  const loadStockList = useCallback(async () => {
    setStockListLoading(true);
    setStockListStatus("下载中...");
    try {
      const list = await invoke<StockListItem[]>("download_stock_list");
      setStockList(list);
      setStockListStatus(`共 ${list.length} 只股票`);
    } catch (e) {
      setStockListStatus(`下载失败: ${e}`);
    }
    setStockListLoading(false);
  }, []);

  const downloadStockData = useCallback(async () => {
    if (!dlCode.trim()) return;
    setDlLoading(true);
    setDlError("");
    setDlResult(null);
    try {
      const summary = await invoke<DownloadSummary>("download_stock_data", {
        code: dlCode.trim(),
        name: dlName.trim() || null,
      });
      setDlResult(summary);
    } catch (e) {
      setDlError(String(e));
    }
    setDlLoading(false);
  }, [dlCode, dlName]);

  const handleImportOne = async (item: StockListItem) => {
    setDlCode(item.code);
    setDlName(item.name);
    setDlLoading(true);
    setDlError("");
    setDlResult(null);
    try {
      const summary = await invoke<DownloadSummary>("download_stock_data", {
        code: item.code,
        name: item.name,
      });
      setDlResult(summary);
    } catch (e) {
      setDlError(String(e));
    }
    setDlLoading(false);
  };

  const filtered =
    search.trim() === ""
      ? stockList
      : stockList.filter(
          (s) =>
            s.code.includes(search.trim()) || s.name.includes(search.trim()),
        );

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        background: "#141b2d",
        color: "#F1F5F9",
        fontFamily: "monospace",
        fontSize: 13,
      }}
    >
      <div
        style={{
          padding: 10,
          borderBottom: "1px solid #1E293B",
          background: "#111827",
          fontWeight: 600,
          color: "#fff",
          fontSize: 14,
          flexShrink: 0,
        }}
      >
        数据下载
      </div>

      {/* Stock list download */}
      <div
        style={{
          padding: "10px 12px",
          borderBottom: "1px solid #1E293B",
          background: "#111827",
          flexShrink: 0,
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            marginBottom: 8,
          }}
        >
          <button
            onClick={loadStockList}
            disabled={stockListLoading}
            style={{
              padding: "5px 14px",
              background: stockListLoading ? "#8a7a3a" : "#00D8FF",
              color: "#000",
              border: "none",
              borderRadius: 4,
              cursor: stockListLoading ? "not-allowed" : "pointer",
              fontFamily: "monospace",
              fontSize: 12,
              fontWeight: 600,
            }}
          >
            {stockListLoading ? "下载中..." : "获取A股列表"}
          </button>
          <span style={{ color: "#94A3B8", fontSize: 11 }}>{stockListStatus}</span>
        </div>
        {stockList.length > 0 && (
          <div>
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="搜索代码或名称..."
              style={{
                background: "#0A0E1A",
                border: "1px solid #1E293B",
                color: "#fff",
                padding: "4px 8px",
                borderRadius: 4,
                fontSize: 12,
                fontFamily: "monospace",
                outline: "none",
                width: 200,
              }}
            />
            <span style={{ color: "#94A3B8", fontSize: 11, marginLeft: 8 }}>
              显示 {filtered.length}/{stockList.length} 只
            </span>
          </div>
        )}
      </div>

      {/* Manual download */}
      <div
        style={{
          padding: "10px 12px",
          borderBottom: "1px solid #1E293B",
          background: "#111827",
          flexShrink: 0,
        }}
      >
        <div style={{ color: "#00D8FF", fontSize: 12, marginBottom: 8 }}>
          单只股票下载
        </div>
        <div style={{ display: "flex", gap: 8, marginBottom: 8 }}>
          <input
            type="text"
            value={dlCode}
            onChange={(e) => setDlCode(e.target.value)}
            placeholder="股票代码 如 000001"
            style={inputS}
          />
          <input
            type="text"
            value={dlName}
            onChange={(e) => setDlName(e.target.value)}
            placeholder="名称（可选）"
            style={{ ...inputS, width: 120 }}
          />
          <button
            onClick={downloadStockData}
            disabled={dlLoading || !dlCode.trim()}
            style={{
              padding: "5px 14px",
              background:
                dlLoading || !dlCode.trim() ? "#1E293B" : "#00E676",
              color:
                dlLoading || !dlCode.trim() ? "#94A3B8" : "#000",
              border: "none",
              borderRadius: 4,
              cursor:
                dlLoading || !dlCode.trim() ? "not-allowed" : "pointer",
              fontFamily: "monospace",
              fontSize: 12,
              fontWeight: 600,
            }}
          >
            {dlLoading ? "下载中..." : "下载"}
          </button>
        </div>
        {dlResult && (
          <div
            style={{
              padding: 8,
              background: "#1a3a2e",
              borderRadius: 4,
              fontSize: 12,
            }}
          >
            <span style={{ color: "#00E676", fontWeight: 600 }}>
              下载成功
            </span>{" "}
            <span style={{ color: "#94A3B8" }}>
              {dlResult.code} {dlResult.name} · 导入 {dlResult.rows_inserted} 条
              {dlResult.date_range &&
                ` · ${dlResult.date_range[0]}~${dlResult.date_range[1]}`}
            </span>
          </div>
        )}
        {dlError && (
          <div
            style={{
              padding: 8,
              background: "#3a1a2e",
              borderRadius: 4,
              color: "#FF2A7A",
              fontSize: 12,
            }}
          >
            {dlError}
          </div>
        )}
      </div>

      {/* Minute data download */}
      <div
        style={{
          padding: "10px 12px",
          borderBottom: "1px solid #1E293B",
          background: "#111827",
          flexShrink: 0,
        }}
      >
        <div style={{ color: "#7C3CFF", fontSize: 12, marginBottom: 8 }}>
          分钟数据下载 <span style={{ color: "#64748B", fontSize: 10 }}>(Pro功能)</span>
        </div>
        <div style={{ display: "flex", gap: 8, marginBottom: 8, alignItems: "center" }}>
          <input
            type="text"
            value={minCode}
            onChange={(e) => setMinCode(e.target.value)}
            placeholder="股票代码 如 000001"
            style={inputS}
          />
          <select
            value={minKlt}
            onChange={(e) => setMinKlt(Number(e.target.value))}
            style={{
              background: "#0A0E1A", border: "1px solid #1E293B", color: "#00D8FF",
              padding: "4px 8px", borderRadius: 4, fontSize: 12, fontFamily: "monospace",
              outline: "none", width: 100,
            }}
          >
            <option value={1}>1分钟</option>
            <option value={5}>5分钟</option>
            <option value={15}>15分钟</option>
            <option value={30}>30分钟</option>
            <option value={60}>60分钟</option>
          </select>
          <button
            onClick={downloadMinuteData}
            disabled={minLoading || !minCode.trim()}
            style={{
              padding: "5px 14px",
              background: minLoading || !minCode.trim() ? "#1E293B" : "#7C3CFF",
              color: minLoading || !minCode.trim() ? "#94A3B8" : "#000",
              border: "none", borderRadius: 4,
              cursor: minLoading || !minCode.trim() ? "not-allowed" : "pointer",
              fontFamily: "monospace", fontSize: 12, fontWeight: 600,
            }}
          >
            {minLoading ? "下载中..." : "下载"}
          </button>
        </div>
        {minResult && (
          <div style={{ padding: 8, background: "#1a2a3e", borderRadius: 4, fontSize: 12 }}>
            <span style={{ color: "#7C3CFF", fontWeight: 600 }}>下载成功</span>{" "}
            <span style={{ color: "#94A3B8" }}>
              {minResult.code} · {minResult.klt_label} · 导入 {minResult.rows_inserted} 条
              {minResult.time_range &&
                ` · ${minResult.time_range[0]}~${minResult.time_range[1]}`}
            </span>
          </div>
        )}
        {minError && (
          <div style={{ padding: 8, background: "#3a1a2e", borderRadius: 4, color: "#FF2A7A", fontSize: 12 }}>
            {minError}
          </div>
        )}
      </div>

      {/* Stock list table */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {stockList.length > 0 && (
          <table
            style={{ width: "100%", borderCollapse: "collapse", fontSize: 12 }}
          >
            <thead>
              <tr
                style={{
                  color: "#94A3B8",
                  borderBottom: "1px solid #1E293B",
                  position: "sticky",
                  top: 0,
                  background: "#141b2d",
                }}
              >
                <th style={thS}>代码</th>
                <th style={thS}>名称</th>
                <th style={{ ...thS, textAlign: "right" }}>最新价</th>
                <th style={{ ...thS, textAlign: "right" }}>涨跌幅</th>
                <th style={thS}>操作</th>
              </tr>
            </thead>
            <tbody>
              {filtered.slice(0, 200).map((s, i) => (
                <tr
                  key={i}
                  style={{
                    borderBottom: "1px solid #1f1f3a",
                    background: i % 2 === 0 ? "transparent" : "rgba(255,255,255,0.02)",
                  }}
                >
                  <td style={{ ...tdS, color: "#00D8FF" }}>{s.code}</td>
                  <td style={tdS}>{s.name}</td>
                  <td style={{ ...tdS, textAlign: "right", color: "#94A3B8" }}>
                    {s.price?.toFixed(2) || "-"}
                  </td>
                  <td
                    style={{
                      ...tdS,
                      textAlign: "right",
                      color: s.change_pct > 0 ? "#FF2A7A" : s.change_pct < 0 ? "#00E676" : "#94A3B8",
                    }}
                  >
                    {s.change_pct > 0 ? "+" : ""}
                    {s.change_pct?.toFixed(2) || "0.00"}%
                  </td>
                  <td style={tdS}>
                    <button
                      onClick={() => handleImportOne(s)}
                      style={{
                        padding: "2px 8px",
                        background: "#1E293B",
                        color: "#F1F5F9",
                        border: "none",
                        borderRadius: 3,
                        cursor: "pointer",
                        fontFamily: "monospace",
                        fontSize: 10,
                      }}
                      title="下载历史数据"
                    >
                      导入
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}

const inputS: React.CSSProperties = {
  width: 110,
  background: "#0A0E1A",
  border: "1px solid #1E293B",
  color: "#fff",
  padding: "4px 8px",
  borderRadius: 4,
  fontSize: 12,
  fontFamily: "monospace",
  outline: "none",
};

const thS: React.CSSProperties = {
  padding: "5px 8px",
  textAlign: "left",
  fontSize: 11,
  fontWeight: 600,
};

const tdS: React.CSSProperties = {
  padding: "4px 8px",
};

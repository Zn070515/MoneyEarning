import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface DataSummary {
  total_stocks: number;
  total_rows: number;
  db_size_mb: number;
}

interface StockInfo {
  id: number;
  code: string;
  name: string;
  exchange: string;
  total_rows: number;
  first_date: string | null;
  last_date: string | null;
}

export function DataPanel() {
  const [summary, setSummary] = useState<DataSummary | null>(null);
  const [stocks, setStocks] = useState<StockInfo[]>([]);
  const [message, setMessage] = useState("");

  const load = useCallback(async () => {
    try {
      const [s, st] = await Promise.all([
        invoke<DataSummary>("get_data_summary"),
        invoke<StockInfo[]>("query_stock_list"),
      ]);
      setSummary(s);
      setStocks(st);
    } catch (e) {
      console.error("DataPanel load:", e);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleDeleteStock = async (id: number, code: string) => {
    if (!confirm(`确认删除 ${code} 及其所有交易数据？此操作不可撤销。`)) return;
    try {
      await invoke("delete_stock", { id });
      setMessage(`已删除 ${code}`);
      load();
      setTimeout(() => setMessage(""), 3000);
    } catch (e) {
      setMessage(`删除失败：${e}`);
    }
  };

  const formatMB = (mb: number) => {
    if (mb < 0.1) return `${(mb * 1024).toFixed(1)} KB`;
    if (mb < 1000) return `${mb.toFixed(1)} MB`;
    return `${(mb / 1024).toFixed(2)} GB`;
  };

  return (
    <div style={{
      background: "#111827", color: "#F1F5F9", fontFamily: "monospace",
      fontSize: 13, height: "100%", display: "flex", flexDirection: "column",
      overflow: "hidden",
    }}>
      <div style={{
        padding: "10px 12px", borderBottom: "1px solid #1E293B",
        fontWeight: 600, color: "#fff", fontSize: 14,
      }}>
        数据管理
      </div>

      {/* Summary */}
      {summary && (
        <div style={{
          padding: 12, borderBottom: "1px solid #141b2d",
          display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 8,
          fontSize: 12,
        }}>
          <div style={{ textAlign: "center", padding: "8px 4px", background: "#141b2d", borderRadius: 4 }}>
            <div style={{ color: "#94A3B8", marginBottom: 2 }}>股票数</div>
            <div style={{ color: "#00D8FF", fontSize: 18, fontWeight: 600 }}>
              {summary.total_stocks}
            </div>
          </div>
          <div style={{ textAlign: "center", padding: "8px 4px", background: "#141b2d", borderRadius: 4 }}>
            <div style={{ color: "#94A3B8", marginBottom: 2 }}>总行数</div>
            <div style={{ color: "#00D8FF", fontSize: 18, fontWeight: 600 }}>
              {summary.total_rows.toLocaleString()}
            </div>
          </div>
          <div style={{ textAlign: "center", padding: "8px 4px", background: "#141b2d", borderRadius: 4 }}>
            <div style={{ color: "#94A3B8", marginBottom: 2 }}>数据库</div>
            <div style={{ color: "#34d399", fontSize: 18, fontWeight: 600 }}>
              {formatMB(summary.db_size_mb)}
            </div>
          </div>
        </div>
      )}

      {message && (
        <div style={{
          padding: "6px 12px", background: "#1a3a2e", color: "#00E676",
          fontSize: 12, textAlign: "center",
        }}>
          {message}
        </div>
      )}

      {/* Stock table */}
      <div style={{ flex: 1, overflow: "auto" }}>
        <div style={{
          display: "grid", gridTemplateColumns: "1fr 1.5fr 0.8fr 1fr 0.8fr",
          padding: "6px 12px", background: "#141b2d", fontSize: 11,
          color: "#94A3B8", borderBottom: "1px solid #1E293B",
        }}>
          <span>代码</span>
          <span>名称</span>
          <span>交易所</span>
          <span>数据量</span>
          <span>操作</span>
        </div>
        {stocks.map(s => (
          <div key={s.id} style={{
            display: "grid", gridTemplateColumns: "1fr 1.5fr 0.8fr 1fr 0.8fr",
            padding: "6px 12px", borderBottom: "1px solid #141b2d",
            fontSize: 12, alignItems: "center",
          }}>
            <span style={{ color: "#00D8FF" }}>{s.code}</span>
            <span style={{ color: "#F1F5F9" }}>{s.name || "-"}</span>
            <span style={{ color: "#94A3B8" }}>{s.exchange}</span>
            <span style={{ color: "#94A3B8", fontSize: 11 }}>
              {s.total_rows}条{s.first_date ? ` · ${s.first_date}~${s.last_date}` : ""}
            </span>
            <button onClick={() => handleDeleteStock(s.id, s.code)} style={{
              background: "none", border: "1px solid #5a3a3a", color: "#FF2A7A",
              cursor: "pointer", fontSize: 11, padding: "2px 6px", borderRadius: 3,
            }}>
              删除
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

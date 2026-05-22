import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface StockInfo {
  id: number;
  code: string;
  name: string;
  exchange: string;
  ipo_date: string | null;
  total_rows: number;
  first_date: string | null;
  last_date: string | null;
}

interface StockListPanelProps {
  onSelectStock?: (stock: StockInfo) => void;
  selectedWatchlistId?: number | null;
  onAddToWatchlist?: (stockId: number) => void;
}

export function StockListPanel({ onSelectStock, selectedWatchlistId, onAddToWatchlist }: StockListPanelProps) {
  const [stocks, setStocks] = useState<StockInfo[]>([]);
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState<number | null>(null);

  const loadStocks = useCallback(async () => {
    try {
      const data = await invoke<StockInfo[]>("query_stock_list");
      setStocks(data);
    } catch (e) {
      console.error("Failed to load stocks:", e);
    }
  }, []);

  useEffect(() => { loadStocks(); }, [loadStocks]);

  const filtered = search.trim()
    ? stocks.filter(s =>
        s.code.includes(search) || s.name.includes(search))
    : stocks;

  const handleSelect = (s: StockInfo) => {
    setSelectedId(s.id);
    onSelectStock?.(s);
  };

  return (
    <div style={{
      background: "#16213e", color: "#ccc", fontFamily: "monospace",
      fontSize: 13, height: "100%", display: "flex", flexDirection: "column",
    }}>
      <div style={{
        padding: "10px 12px", borderBottom: "1px solid #2a2a4a",
        fontWeight: 600, color: "#fff", fontSize: 14,
      }}>
        股票列表
        <span style={{ color: "#888", fontSize: 11, marginLeft: 8 }}>
          ({stocks.length})
        </span>
      </div>

      <div style={{ padding: "8px 12px" }}>
        <input
          value={search}
          onChange={e => setSearch(e.target.value)}
          placeholder="搜索代码或名称..."
          style={{
            width: "100%", background: "#1a1a2e", border: "1px solid #3a3a5a",
            color: "#fff", padding: "6px 8px", borderRadius: 4, fontSize: 12,
            fontFamily: "monospace", outline: "none",
            boxSizing: "border-box",
          }}
        />
      </div>

      <div style={{ flex: 1, overflow: "auto" }}>
        {filtered.map(s => (
          <div key={s.id}
            onClick={() => handleSelect(s)}
            style={{
              padding: "7px 12px", cursor: "pointer",
              background: selectedId === s.id ? "#2a3a5e" : "transparent",
              borderBottom: "1px solid #1a1a2e",
              display: "flex", justifyContent: "space-between", alignItems: "center",
            }}
          >
            <span>
              <span style={{ color: selectedId === s.id ? "#fbbf24" : "#ccc" }}>
                {s.code}
              </span>
              <span style={{ color: "#aaa", marginLeft: 8, fontSize: 12 }}>
                {s.name}
              </span>
            </span>
            <span style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <span style={{ color: "#888", fontSize: 10 }}>{s.exchange}</span>
              {selectedWatchlistId && onAddToWatchlist && (
                <button
                  onClick={e => { e.stopPropagation(); onAddToWatchlist(s.id); }}
                  style={{
                    background: "#3a3a5a", border: "none", color: "#ccc",
                    cursor: "pointer", fontSize: 11, padding: "2px 6px",
                    borderRadius: 3,
                  }}
                  title="添加到自选股"
                >
                  +自选
                </button>
              )}
            </span>
          </div>
        ))}
        {filtered.length === 0 && (
          <div style={{ padding: 12, color: "#666", fontSize: 11, textAlign: "center" }}>
            {search ? "无匹配结果" : "暂无数据，请先导入CSV"}
          </div>
        )}
      </div>
    </div>
  );
}

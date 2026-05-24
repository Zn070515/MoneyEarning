import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Watchlist {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
  item_count: number;
}

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

interface WatchlistPanelProps {
  onSelectStock?: (stock: StockInfo) => void;
  onSelectWatchlist?: (id: number) => void;
}

export function WatchlistPanel({ onSelectStock, onSelectWatchlist }: WatchlistPanelProps) {
  const [lists, setLists] = useState<Watchlist[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [items, setItems] = useState<StockInfo[]>([]);
  const [newName, setNewName] = useState("");
  const [loading, setLoading] = useState(false);

  const loadLists = useCallback(async () => {
    try {
      const data = await invoke<Watchlist[]>("watchlist_list");
      setLists(data);
    } catch (e) {
      console.error("Failed to load watchlists:", e);
    }
  }, []);

  useEffect(() => { loadLists(); }, [loadLists]);

  const loadItems = useCallback(async (wlId: number) => {
    try {
      const data = await invoke<StockInfo[]>("watchlist_items", { watchlistId: wlId });
      setItems(data);
    } catch (e) {
      console.error("Failed to load items:", e);
    }
  }, []);

  const handleSelect = (wl: Watchlist) => {
    setSelectedId(wl.id);
    loadItems(wl.id);
    onSelectWatchlist?.(wl.id);
  };

  const handleCreate = async () => {
    if (!newName.trim()) return;
    setLoading(true);
    try {
      await invoke<number>("watchlist_create", { name: newName, description: "" });
      setNewName("");
      await loadLists();
    } catch (e) {
      console.error("Failed to create watchlist:", e);
    }
    setLoading(false);
  };

  const handleDelete = async (id: number) => {
    setLoading(true);
    try {
      await invoke("watchlist_delete", { id });
      if (selectedId === id) { setSelectedId(null); setItems([]); }
      await loadLists();
    } catch (e) {
      console.error("Failed to delete watchlist:", e);
    }
    setLoading(false);
  };

  const handleRemoveItem = async (stockId: number) => {
    if (!selectedId) return;
    try {
      await invoke("watchlist_remove_item", { watchlistId: selectedId, stockId });
      await loadItems(selectedId);
      await loadLists();
    } catch (e) {
      console.error("Failed to remove item:", e);
    }
  };

  return (
    <div style={{
      background: "#161616", color: "#D4D4D4", fontFamily: "monospace",
      fontSize: 13, height: "100%", display: "flex", flexDirection: "column",
    }}>
      {/* Header */}
      <div style={{
        padding: "10px 12px", borderBottom: "1px solid #2A2A2A",
        fontWeight: 600, color: "#fff", fontSize: 14,
      }}>
        自选股列表
      </div>

      {/* Create */}
      <div style={{ padding: "8px 12px", display: "flex", gap: 6 }}>
        <input
          value={newName}
          onChange={e => setNewName(e.target.value)}
          onKeyDown={e => e.key === "Enter" && handleCreate()}
          placeholder="新建列表名称..."
          style={{
            flex: 1, background: "#121212", border: "1px solid #2A2A2A",
            color: "#fff", padding: "4px 8px", borderRadius: 4, fontSize: 12,
            fontFamily: "monospace", outline: "none",
          }}
        />
        <button onClick={handleCreate} disabled={loading} style={{
          background: "#CCAA00", color: "#000", border: "none",
          padding: "4px 10px", borderRadius: 4, cursor: "pointer",
          fontSize: 12, fontWeight: 600,
        }}>
          +
        </button>
      </div>

      {/* List */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {lists.map(wl => (
          <div key={wl.id}>
            <div
              onClick={() => handleSelect(wl)}
              style={{
                padding: "8px 12px", cursor: "pointer",
                background: selectedId === wl.id ? "#2a3a5e" : "transparent",
                borderBottom: "1px solid #121212",
                display: "flex", justifyContent: "space-between", alignItems: "center",
              }}
            >
              <span style={{ color: selectedId === wl.id ? "#CCAA00" : "#D4D4D4" }}>
                {wl.name}
                <span style={{ color: "#858585", marginLeft: 8, fontSize: 11 }}>
                  ({wl.item_count})
                </span>
              </span>
              <button onClick={e => { e.stopPropagation(); handleDelete(wl.id); }}
                style={{
                  background: "none", border: "none", color: "#666666",
                  cursor: "pointer", fontSize: 14, padding: "0 4px",
                }}
                title="删除列表"
              >
                ×
              </button>
            </div>

            {/* Items */}
            {selectedId === wl.id && (
              <div style={{ paddingLeft: 12 }}>
                {items.length === 0 && (
                  <div style={{ padding: 8, color: "#666666", fontSize: 11 }}>
                    暂无股票，从股票列表中拖入或搜索添加
                  </div>
                )}
                {items.map(s => (
                  <div key={s.id}
                    onClick={() => onSelectStock?.(s)}
                    style={{
                      padding: "6px 12px", cursor: "pointer",
                      borderBottom: "1px solid #121212",
                      display: "flex", justifyContent: "space-between",
                      fontSize: 12,
                    }}
                  >
                    <span>
                      <span style={{ color: "#CCAA00" }}>{s.code}</span>
                      <span style={{ color: "#858585", marginLeft: 8 }}>{s.name}</span>
                    </span>
                    <span style={{ display: "flex", gap: 12, alignItems: "center" }}>
                      <span style={{ color: "#858585", fontSize: 11 }}>
                        {s.exchange} · {s.total_rows}条
                      </span>
                      <button onClick={e => { e.stopPropagation(); handleRemoveItem(s.id); }}
                        style={{
                          background: "none", border: "none", color: "#666666",
                          cursor: "pointer", fontSize: 12,
                        }}
                      >
                        ×
                      </button>
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

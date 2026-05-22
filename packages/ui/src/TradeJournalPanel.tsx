import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Trade {
  id: number;
  stock_id: number;
  stock_code?: string;
  stock_name?: string;
  trade_date: string;
  direction: "buy" | "sell";
  price: number;
  quantity: number;
  commission: number;
  stamp_tax: number;
  strategy_name: string | null;
  notes: string | null;
  created_at: string;
}

interface PnLSummary {
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
  win_rate: number;
  total_pnl: number;
  avg_win: number;
  avg_loss: number;
  max_win: number;
  max_loss: number;
  profit_factor: number;
}

const EMPTY_PNL: PnLSummary = {
  total_trades: 0, winning_trades: 0, losing_trades: 0,
  win_rate: 0, total_pnl: 0, avg_win: 0, avg_loss: 0,
  max_win: 0, max_loss: 0, profit_factor: 0,
};

interface TradeJournalPanelProps {
  selectedStockId?: number | null;
  compact?: boolean;
}

export function TradeJournalPanel({ selectedStockId, compact }: TradeJournalPanelProps) {
  const [trades, setTrades] = useState<Trade[]>([]);
  const [pnl, setPnl] = useState<PnLSummary>(EMPTY_PNL);
  const [showForm, setShowForm] = useState(false);

  // Form state
  const [formDate, setFormDate] = useState(new Date().toISOString().slice(0, 10));
  const [formDirection, setFormDirection] = useState<"buy" | "sell">("buy");
  const [formPrice, setFormPrice] = useState("");
  const [formQty, setFormQty] = useState("");
  const [formCommission, setFormCommission] = useState("0");
  const [formStampTax, setFormStampTax] = useState("0");
  const [formStrategy, setFormStrategy] = useState("");
  const [formNotes, setFormNotes] = useState("");

  const loadTrades = useCallback(async () => {
    try {
      // TODO: Replace with proper backend command after adding trade_query
      const data = await invoke<Trade[]>("trade_list", {
        stockId: selectedStockId ?? 0,
      });
      setTrades(data);
    } catch (e) {
      // Backend not ready yet, use empty list
      setTrades([]);
    }
  }, [selectedStockId]);

  const loadPnL = useCallback(async () => {
    try {
      const data = await invoke<PnLSummary>("trade_pnl", {
        stockId: selectedStockId ?? 0,
      });
      setPnl(data);
    } catch (e) {
      setPnl(EMPTY_PNL);
    }
  }, [selectedStockId]);

  useEffect(() => { loadTrades(); loadPnL(); }, [loadTrades, loadPnL]);

  const handleSubmit = async () => {
    if (!formPrice || !formQty) return;
    try {
      await invoke("trade_create", {
        stockId: selectedStockId ?? 0,
        tradeDate: formDate,
        direction: formDirection,
        price: parseFloat(formPrice),
        quantity: parseFloat(formQty),
        commission: parseFloat(formCommission) || 0,
        stampTax: parseFloat(formStampTax) || 0,
        strategyName: formStrategy || null,
        notes: formNotes || null,
      });
      setShowForm(false);
      resetForm();
      loadTrades();
      loadPnL();
    } catch (e) {
      console.error("Failed to create trade:", e);
    }
  };

  const resetForm = () => {
    setFormDate(new Date().toISOString().slice(0, 10));
    setFormDirection("buy");
    setFormPrice("");
    setFormQty("");
    setFormCommission("0");
    setFormStampTax("0");
    setFormStrategy("");
    setFormNotes("");
  };

  const colorPnl = (v: number) => v > 0 ? "#ef4444" : v < 0 ? "#22c55e" : "#888";

  return (
    <div style={{
      background: "#16213e", color: "#ccc", fontFamily: "monospace",
      fontSize: 13, height: "100%", display: "flex", flexDirection: "column",
      overflow: "hidden",
    }}>
      <div style={{
        padding: "10px 12px", borderBottom: "1px solid #2a2a4a",
        fontWeight: 600, color: "#fff", fontSize: 14,
        display: "flex", justifyContent: "space-between", alignItems: "center",
      }}>
        <span>交易日志</span>
        <button onClick={() => setShowForm(!showForm)} style={{
          background: "#fbbf24", color: "#000", border: "none",
          padding: "3px 10px", borderRadius: 4, cursor: "pointer",
          fontSize: 12, fontWeight: 600,
        }}>
          {showForm ? "取消" : "+ 记录"}
        </button>
      </div>

      {/* PnL Summary */}
      <div style={{
        padding: "8px 12px", borderBottom: "1px solid #1a1a2e",
        display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 6,
        fontSize: 11,
      }}>
        <StatBox label="总笔数" value={pnl.total_trades.toString()} />
        <StatBox label="胜率" value={`${pnl.win_rate.toFixed(1)}%`} color="#fbbf24" />
        <StatBox label="总盈亏" value={pnl.total_pnl.toFixed(2)} color={colorPnl(pnl.total_pnl)} />
        <StatBox label="盈亏比" value={pnl.profit_factor.toFixed(2)} />
      </div>

      {/* Trade Form */}
      {showForm && (
        <div style={{
          padding: "10px 12px", borderBottom: "1px solid #2a2a4a",
          background: "#1a1a2e",
        }}>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8, marginBottom: 8 }}>
            <div>
              <label style={{ fontSize: 11, color: "#888" }}>日期</label>
              <input type="date" value={formDate} onChange={e => setFormDate(e.target.value)}
                style={inputStyle} />
            </div>
            <div>
              <label style={{ fontSize: 11, color: "#888" }}>方向</label>
              <select value={formDirection} onChange={e => setFormDirection(e.target.value as "buy" | "sell")}
                style={inputStyle}>
                <option value="buy">买入</option>
                <option value="sell">卖出</option>
              </select>
            </div>
            <div>
              <label style={{ fontSize: 11, color: "#888" }}>价格</label>
              <input type="number" step="0.01" value={formPrice}
                onChange={e => setFormPrice(e.target.value)} style={inputStyle} />
            </div>
            <div>
              <label style={{ fontSize: 11, color: "#888" }}>数量(股)</label>
              <input type="number" step="100" value={formQty}
                onChange={e => setFormQty(e.target.value)} style={inputStyle} />
            </div>
            <div>
              <label style={{ fontSize: 11, color: "#888" }}>佣金</label>
              <input type="number" step="0.01" value={formCommission}
                onChange={e => setFormCommission(e.target.value)} style={inputStyle} />
            </div>
            <div>
              <label style={{ fontSize: 11, color: "#888" }}>印花税</label>
              <input type="number" step="0.01" value={formStampTax}
                onChange={e => setFormStampTax(e.target.value)} style={inputStyle} />
            </div>
          </div>
          <div style={{ marginBottom: 8 }}>
            <label style={{ fontSize: 11, color: "#888" }}>策略</label>
            <input value={formStrategy} onChange={e => setFormStrategy(e.target.value)}
              placeholder="使用的策略名称..." style={{ ...inputStyle, width: "100%", boxSizing: "border-box" }} />
          </div>
          <div style={{ marginBottom: 8 }}>
            <label style={{ fontSize: 11, color: "#888" }}>备注</label>
            <input value={formNotes} onChange={e => setFormNotes(e.target.value)}
              placeholder="交易理由、心得..." style={{ ...inputStyle, width: "100%", boxSizing: "border-box" }} />
          </div>
          <button onClick={handleSubmit} style={{
            width: "100%", background: "#fbbf24", color: "#000",
            border: "none", padding: "6px 12px", borderRadius: 4,
            cursor: "pointer", fontSize: 13, fontWeight: 600,
          }}>
            记录交易
          </button>
        </div>
      )}

      {/* Trade List */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {trades.length === 0 ? (
          <div style={{ padding: 16, color: "#666", fontSize: 12, textAlign: "center" }}>
            暂无交易记录
          </div>
        ) : (
          trades.map(t => (
            <div key={t.id} style={{
              padding: "6px 12px", borderBottom: "1px solid #1a1a2e",
              display: "flex", justifyContent: "space-between", alignItems: "center",
              fontSize: 12,
            }}>
              <span style={{ display: "flex", gap: 8, alignItems: "center" }}>
                <span style={{ color: t.direction === "buy" ? "#ef4444" : "#22c55e", fontWeight: 600 }}>
                  {t.direction === "buy" ? "买" : "卖"}
                </span>
                <span style={{ color: "#888" }}>{t.trade_date}</span>
                {t.stock_code && <span style={{ color: "#fbbf24" }}>{t.stock_code}</span>}
              </span>
              <span style={{ display: "flex", gap: 12, alignItems: "center" }}>
                <span>{t.price.toFixed(2)}</span>
                <span style={{ color: "#888" }}>×{t.quantity}</span>
                <span style={{ color: "#aaa" }}>
                  ¥{(t.price * t.quantity).toLocaleString()}
                </span>
              </span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function StatBox({ label, value, color }: { label: string; value: string; color?: string }) {
  return (
    <div style={{ textAlign: "center" }}>
      <div style={{ color: "#888", marginBottom: 2 }}>{label}</div>
      <div style={{ color: color || "#fff", fontWeight: 600 }}>{value}</div>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  background: "#0f0f23", border: "1px solid #3a3a5a",
  color: "#fff", padding: "4px 8px", borderRadius: 4, fontSize: 12,
  fontFamily: "monospace", outline: "none", width: "100%", boxSizing: "border-box",
};

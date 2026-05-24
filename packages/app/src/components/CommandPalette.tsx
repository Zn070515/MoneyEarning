import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useAppStore } from "../stores/appStore";
import {
  IconDashboard,
  IconChart,
  IconBacktest,
  IconScanner,
  IconPortfolio,
  IconReview,
  IconEditor,
  IconSettings,
  IconDownload,
  IconUpload,
  IconDatabase,
  IconSearch,
  IconCommand,
} from "./icons";

interface Command {
  id: string;
  label: string;
  category: string;
  keywords: string[];
  action: () => void;
  Icon?: React.ComponentType<{ size?: number }>;
}

const RECENTS_KEY = "qv_command_recents";
const MAX_RECENTS = 5;

function loadRecents(): string[] {
  try {
    const raw = localStorage.getItem(RECENTS_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveRecents(id: string) {
  const recents = loadRecents().filter((r) => r !== id);
  recents.unshift(id);
  localStorage.setItem(RECENTS_KEY, JSON.stringify(recents.slice(0, MAX_RECENTS)));
}

export default function CommandPalette() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  const commands = useMemo((): Command[] => {
    const nav = useAppStore.getState().navigate;
    return [
      { id: "nav-dashboard", label: "概览面板", category: "导航", keywords: ["dashboard", "home", "首页"], action: () => navigate("/"), Icon: IconDashboard },
      { id: "nav-chart", label: "图表分析", category: "导航", keywords: ["chart", "kline", "k线", "图表"], action: () => navigate("/chart"), Icon: IconChart },
      { id: "nav-backtest", label: "策略回测", category: "导航", keywords: ["backtest", "回测", "策略"], action: () => navigate("/backtest"), Icon: IconBacktest },
      { id: "nav-scanner", label: "条件扫描", category: "导航", keywords: ["scanner", "扫描", "选股", "筛选"], action: () => navigate("/scanner"), Icon: IconScanner },
      { id: "nav-portfolio", label: "组合管理", category: "导航", keywords: ["portfolio", "组合", "持仓"], action: () => navigate("/portfolio"), Icon: IconPortfolio },
      { id: "nav-review", label: "交易复盘", category: "导航", keywords: ["review", "复盘", "日志"], action: () => navigate("/review"), Icon: IconReview },
      { id: "nav-editor", label: "脚本编辑器", category: "导航", keywords: ["editor", "脚本", "mescript", "公式"], action: () => navigate("/editor"), Icon: IconEditor },
      { id: "nav-settings", label: "系统设置", category: "导航", keywords: ["settings", "设置", "配置"], action: () => navigate("/settings"), Icon: IconSettings },
      { id: "import-csv", label: "导入 CSV 数据", category: "数据", keywords: ["import", "导入", "csv", "通达信"], action: () => { navigate("/chart"); setTimeout(() => window.dispatchEvent(new CustomEvent("open-import-dialog")), 300); }, Icon: IconUpload },
      { id: "download-data", label: "下载股票数据", category: "数据", keywords: ["download", "下载", "更新"], action: () => { navigate("/chart"); }, Icon: IconDownload },
      { id: "backup-db", label: "备份数据库", category: "数据", keywords: ["backup", "备份", "导出"], action: () => { navigate("/settings"); }, Icon: IconDatabase },
      { id: "search-stock", label: "搜索股票", category: "数据", keywords: ["search", "搜索", "股票", "代码"], action: () => { navigate("/chart"); }, Icon: IconSearch },
    ];
  }, [navigate]);

  // Filter + sort
  const results = useMemo(() => {
    const q = query.toLowerCase().trim();
    if (!q) {
      const recents = loadRecents();
      const recentCmds = recents
        .map((id) => commands.find((c) => c.id === id))
        .filter(Boolean) as Command[];
      const rest = commands.filter((c) => !recents.includes(c.id));
      return [...recentCmds, ...rest];
    }
    const scored = commands
      .map((c) => {
        let score = 0;
        const l = c.label.toLowerCase();
        if (l === q) score += 50;
        if (l.startsWith(q)) score += 30;
        if (l.includes(q)) score += 15;
        for (const kw of c.keywords) {
          if (kw === q) score += 25;
          if (kw.startsWith(q)) score += 10;
          if (kw.includes(q)) score += 5;
        }
        return { cmd: c, score };
      })
      .filter(({ score }) => score > 0)
      .sort((a, b) => b.score - a.score)
      .map(({ cmd }) => cmd);
    return scored;
  }, [query, commands]);

  // Reset selection on query change
  useEffect(() => { setSelectedIdx(0); }, [query]);

  // Toggle with Ctrl+K
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setOpen((prev) => !prev);
        setQuery("");
      }
      if (e.key === "Escape" && open) {
        e.preventDefault();
        setOpen(false);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open]);

  // Focus input on open
  useEffect(() => {
    if (open) {
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  const execute = useCallback(
    (cmd: Command) => {
      saveRecents(cmd.id);
      setOpen(false);
      setQuery("");
      cmd.action();
    },
    []
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIdx((i) => Math.min(i + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIdx((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (results[selectedIdx]) execute(results[selectedIdx]);
    }
  };

  if (!open) return null;

  // Group results by category
  const grouped: Record<string, Command[]> = {};
  for (const c of results) {
    (grouped[c.category] ??= []).push(c);
  }

  return (
    <div
      onClick={() => setOpen(false)}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 9999,
        display: "flex",
        justifyContent: "center",
        paddingTop: "14vh",
        background: "rgba(0, 0, 0, 0.5)",
        backdropFilter: "blur(4px)",
        animation: "fade-in 120ms ease-out forwards",
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="glass panel-enter"
        style={{
          width: 520,
          maxHeight: "60vh",
          borderRadius: "var(--radius-xl)",
          border: "1px solid var(--border-active)",
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
          boxShadow: "var(--shadow-modal)",
        }}
      >
        {/* Search input */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 10,
            padding: "14px 16px",
            borderBottom: "1px solid var(--border-subtle)",
          }}
        >
          <span style={{ color: "var(--text-muted)", flexShrink: 0 }}>
            <IconSearch size={16} />
          </span>
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="输入命令搜索..."
            style={{
              flex: 1,
              background: "transparent",
              border: "none",
              color: "var(--text-primary)",
              fontFamily: "var(--font-ui)",
              fontSize: 14,
              outline: "none",
            }}
          />
          <span
            style={{
              fontSize: 10,
              color: "var(--text-muted)",
              fontFamily: "var(--font-data)",
              padding: "2px 6px",
              background: "var(--bg-raised)",
              borderRadius: "var(--radius-sm)",
              border: "1px solid var(--border-subtle)",
            }}
          >
            ESC
          </span>
        </div>

        {/* Results */}
        <div style={{ overflow: "auto", flex: 1, padding: "6px 0" }}>
          {results.length === 0 && (
            <div style={{ padding: "32px 16px", textAlign: "center", color: "var(--text-muted)", fontSize: 13 }}>
              没有找到匹配的命令
            </div>
          )}
          {Object.entries(grouped).map(([category, cmds]) => (
            <div key={category}>
              <div
                style={{
                  padding: "6px 16px 2px",
                  fontSize: 10,
                  fontWeight: 600,
                  color: "var(--text-muted)",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                }}
              >
                {category}
              </div>
              {cmds.map((cmd) => {
                const globalIdx = results.indexOf(cmd);
                const isSelected = globalIdx === selectedIdx;
                return (
                  <button
                    key={cmd.id}
                    onClick={() => execute(cmd)}
                    onMouseEnter={() => setSelectedIdx(globalIdx)}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 10,
                      width: "100%",
                      padding: "8px 16px",
                      border: "none",
                      background: isSelected ? "var(--bg-raised)" : "transparent",
                      color: isSelected ? "var(--accent)" : "var(--text-primary)",
                      cursor: "pointer",
                      fontFamily: "var(--font-ui)",
                      fontSize: 13,
                      textAlign: "left",
                      transition: "background 80ms ease",
                    }}
                  >
                    {cmd.Icon && <cmd.Icon size={16} />}
                    <span>{cmd.label}</span>
                    {loadRecents().includes(cmd.id) && (
                      <span style={{ marginLeft: "auto", fontSize: 9, color: "var(--text-muted)" }}>最近</span>
                    )}
                  </button>
                );
              })}
            </div>
          ))}
        </div>

        {/* Footer */}
        <div
          style={{
            display: "flex",
            gap: 14,
            padding: "8px 16px",
            borderTop: "1px solid var(--border-subtle)",
            fontSize: 10,
            color: "var(--text-muted)",
          }}
        >
          <span><kbd style={{ fontFamily: "var(--font-data)", padding: "1px 4px", background: "var(--bg-raised)", borderRadius: 3, border: "1px solid var(--border-subtle)" }}>↑↓</kbd> 导航</span>
          <span><kbd style={{ fontFamily: "var(--font-data)", padding: "1px 4px", background: "var(--bg-raised)", borderRadius: 3, border: "1px solid var(--border-subtle)" }}>Enter</kbd> 选择</span>
          <span><kbd style={{ fontFamily: "var(--font-data)", padding: "1px 4px", background: "var(--bg-raised)", borderRadius: 3, border: "1px solid var(--border-subtle)" }}>Esc</kbd> 关闭</span>
        </div>
      </div>
    </div>
  );
}

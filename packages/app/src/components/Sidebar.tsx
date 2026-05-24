import { useNavigate, useLocation } from "react-router-dom";
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
} from "./icons";

interface NavItem {
  path: string;
  label: string;
  Icon: React.ComponentType<{ size?: number; className?: string }>;
}

const navItems: NavItem[] = [
  { path: "/",         label: "概览", Icon: IconDashboard },
  { path: "/chart",    label: "图表", Icon: IconChart },
  { path: "/backtest", label: "回测", Icon: IconBacktest },
  { path: "/scanner",  label: "扫描", Icon: IconScanner },
  { path: "/portfolio",label: "组合", Icon: IconPortfolio },
  { path: "/review",   label: "复盘", Icon: IconReview },
  { path: "/editor",   label: "脚本", Icon: IconEditor },
  { path: "/settings", label: "设置", Icon: IconSettings },
];

export default function Sidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const selectedStockCode = useAppStore((s) => s.selectedStockCode);
  const currentPath = location.pathname;

  return (
    <nav
      style={{
        width: 56,
        background: "var(--bg-deepest)",
        borderRight: "1px solid var(--border-subtle)",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        paddingTop: 6,
        flexShrink: 0,
        gap: 0,
      }}
    >
      {navItems.map((item) => {
        const active =
          item.path === "/" ? currentPath === "/" : currentPath.startsWith(item.path);
        return (
          <button
            key={item.path}
            onClick={() => {
              navigate(item.path);
              useAppStore.getState().navigate(item.path);
            }}
            title={item.label}
            style={{
              width: 44,
              height: 44,
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              border: "none",
              borderRadius: 6,
              background: active ? "var(--bg-raised)" : "transparent",
              color: active ? "var(--accent)" : "var(--text-muted)",
              cursor: "pointer",
              position: "relative",
              transition: "color 150ms ease, background 150ms ease",
              marginBottom: 1,
            }}
          >
            <item.Icon size={18} />
            <span style={{ fontSize: 8, marginTop: 1, fontWeight: 500, lineHeight: 1 }}>
              {item.label}
            </span>
            {active && (
              <span
                style={{
                  position: "absolute",
                  left: 0,
                  top: 8,
                  bottom: 8,
                  width: 2.5,
                  background: "var(--accent)",
                  borderRadius: "0 2px 2px 0",
                }}
              />
            )}
          </button>
        );
      })}

      {/* Stock indicator at bottom */}
      {selectedStockCode && (
        <div
          title={selectedStockCode}
          style={{
            marginTop: "auto",
            marginBottom: 10,
            padding: "3px 6px",
            background: "var(--bg-raised)",
            border: "1px solid var(--border-accent)",
            borderRadius: 4,
            fontSize: 10,
            color: "var(--accent)",
            fontFamily: "var(--font-data)",
            textAlign: "center",
            maxWidth: 48,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {selectedStockCode}
        </div>
      )}
    </nav>
  );
}

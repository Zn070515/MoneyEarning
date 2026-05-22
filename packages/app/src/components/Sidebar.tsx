import { useNavigate, useLocation } from "react-router-dom";
import { useAppStore } from "../stores/appStore";

interface NavItem {
  path: string;
  label: string;
  icon: string;
}

const navItems: NavItem[] = [
  { path: "/", label: "概览", icon: "□" },
  { path: "/chart", label: "图表", icon: "📈" },
  { path: "/backtest", label: "回测", icon: "⚡" },
  { path: "/scanner", label: "扫描", icon: "🔍" },
  { path: "/portfolio", label: "组合", icon: "📊" },
  { path: "/review", label: "复盘", icon: "📝" },
  { path: "/settings", label: "设置", icon: "⚙" },
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
        background: "#16213e",
        borderRight: "1px solid #2a2a4a",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        paddingTop: 8,
        flexShrink: 0,
        gap: 2,
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
              background: active ? "#1a1a2e" : "transparent",
              color: active ? "#fbbf24" : "#888",
              cursor: "pointer",
              fontSize: 18,
              fontFamily: "monospace",
              position: "relative",
            }}
          >
            <span style={{ fontSize: 16, lineHeight: 1 }}>{item.icon}</span>
            <span style={{ fontSize: 9, marginTop: 1, lineHeight: 1 }}>
              {item.label}
            </span>
            {active && (
              <span
                style={{
                  position: "absolute",
                  left: 0,
                  top: "25%",
                  bottom: "25%",
                  width: 3,
                  background: "#fbbf24",
                  borderRadius: "0 2px 2px 0",
                }}
              />
            )}
          </button>
        );
      })}

      {/* Stock indicator */}
      {selectedStockCode && (
        <div
          style={{
            marginTop: "auto",
            marginBottom: 12,
            padding: "4px 6px",
            background: "#1a1a2e",
            borderRadius: 4,
            fontSize: 10,
            color: "#fbbf24",
            fontFamily: "monospace",
            textAlign: "center",
            maxWidth: 48,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
          title={selectedStockCode}
        >
          {selectedStockCode}
        </div>
      )}
    </nav>
  );
}

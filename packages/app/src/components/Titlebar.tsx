import { useState, useEffect } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  IconMinimize,
  IconMaximize,
  IconRestore,
  IconClose,
} from "./icons";

export default function Titlebar() {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const win = getCurrentWebviewWindow();

    // Check initial state
    win.isMaximized().then((m) => {
      if (!cancelled) setMaximized(m);
    });

    // Listen for resize to detect maximize/restore
    const checkMaximized = async () => {
      try {
        const m = await win.isMaximized();
        if (!cancelled) setMaximized(m);
      } catch {}
    };

    // Poll for changes since Tauri 2 doesn't have a maximize event
    const interval = setInterval(checkMaximized, 500);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  const handleMinimize = () => getCurrentWebviewWindow().minimize();
  const handleToggleMaximize = () => getCurrentWebviewWindow().toggleMaximize();
  const handleClose = () => getCurrentWebviewWindow().close();

  return (
    <div
      data-tauri-drag-region
      style={{
        height: 36,
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        background: "var(--bg-deepest)",
        borderBottom: "1px solid var(--border-subtle)",
        flexShrink: 0,
        userSelect: "none",
      }}
    >
      {/* Left: App icon + title */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          paddingLeft: 14,
          height: "100%",
        }}
      >
        <svg width={16} height={16} viewBox="0 0 24 24" fill="none">
          <rect x="3" y="1" width="18" height="22" rx="3" stroke="var(--accent)" strokeWidth={1.8} />
          <path d="M7 8h10M7 12h10M7 16h6" stroke="var(--accent)" strokeWidth={1.8} strokeLinecap="round" />
          <circle cx="17" cy="17" r="2" fill="var(--accent)" />
        </svg>
        <span
          style={{
            fontSize: 11,
            fontWeight: 500,
            color: "var(--text-secondary)",
            fontFamily: "var(--font-ui)",
            letterSpacing: "0.02em",
          }}
        >
          QuantVault
        </span>
      </div>

      {/* Right: Window controls */}
      <div style={{ display: "flex", height: "100%" }}>
        <WindowBtn onClick={handleMinimize} title="最小化">
          <IconMinimize size={14} />
        </WindowBtn>
        <WindowBtn onClick={handleToggleMaximize} title={maximized ? "还原" : "最大化"}>
          {maximized ? <IconRestore size={13} /> : <IconMaximize size={13} />}
        </WindowBtn>
        <WindowBtn onClick={handleClose} title="关闭" isClose>
          <IconClose size={14} />
        </WindowBtn>
      </div>
    </div>
  );
}

function WindowBtn({
  children,
  onClick,
  title,
  isClose,
}: {
  children: React.ReactNode;
  onClick: () => void;
  title: string;
  isClose?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      style={{
        width: 44,
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        border: "none",
        background: "transparent",
        color: "var(--text-secondary)",
        cursor: "pointer",
        transition: "background 120ms ease, color 120ms ease",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.background = isClose
          ? "var(--negative)"
          : "var(--bg-raised)";
        e.currentTarget.style.color = isClose ? "#fff" : "var(--text-primary)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.background = "transparent";
        e.currentTarget.style.color = "var(--text-secondary)";
      }}
    >
      {children}
    </button>
  );
}

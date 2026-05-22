import { ScannerPanel } from "@me/ui";

export default function ScannerPage() {
  return (
    <div
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          padding: "12px 20px",
          background: "#16213e",
          borderBottom: "1px solid #2a2a4a",
          flexShrink: 0,
        }}
      >
        <h2
          style={{
            color: "#fbbf24",
            fontSize: 16,
            fontFamily: "monospace",
            margin: 0,
          }}
        >
          股票扫描
        </h2>
        <p
          style={{
            color: "#888",
            fontSize: 11,
            fontFamily: "monospace",
            margin: "4px 0 0",
          }}
        >
          CAPS / CGPC / MARS / MetaSearcher 智能选股算法
        </p>
      </div>
      <div style={{ flex: 1, overflow: "hidden" }}>
        <ScannerPanel />
      </div>
    </div>
  );
}

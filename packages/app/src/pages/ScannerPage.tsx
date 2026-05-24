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
          background: "#161616",
          borderBottom: "1px solid #2A2A2A",
          flexShrink: 0,
        }}
      >
        <h2
          style={{
            color: "#CCAA00",
            fontSize: 16,
            fontFamily: "monospace",
            margin: 0,
          }}
        >
          股票扫描
        </h2>
        <p
          style={{
            color: "#858585",
            fontSize: 11,
            fontFamily: "monospace",
            margin: "4px 0 0",
          }}
        >
          CAPS / CGPC / MARS / MetaSearcher 条件扫描算法
        </p>
      </div>
      <div style={{ flex: 1, overflow: "hidden" }}>
        <ScannerPanel />
      </div>
    </div>
  );
}

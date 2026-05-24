import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: string | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error: error.message ?? String(error) };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            height: "100vh",
            background: "#0C0C0C",
            color: "#D4D4D4",
            fontFamily: "monospace",
          }}
        >
          <div style={{ textAlign: "center", maxWidth: 480 }}>
            <h3 style={{ color: "#EF5350", fontSize: 16, marginBottom: 12 }}>
              出错了
            </h3>
            <p style={{ color: "#858585", fontSize: 13, marginBottom: 8 }}>
              {this.state.error || "未知错误"}
            </p>
            <p style={{ color: "#666666", fontSize: 11, marginBottom: 20 }}>
              请尝试重新启动应用。如果问题持续，请反馈给我们。
            </p>
            <button
              onClick={() => {
                this.setState({ hasError: false, error: null });
                window.location.reload();
              }}
              style={{
                padding: "8px 24px",
                background: "#CCAA00",
                color: "#000",
                border: "none",
                borderRadius: 4,
                cursor: "pointer",
                fontFamily: "monospace",
                fontSize: 13,
                fontWeight: 600,
              }}
            >
              重新加载
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

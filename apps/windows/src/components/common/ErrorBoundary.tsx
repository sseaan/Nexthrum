import { Component, type ErrorInfo, type ReactNode } from "react";

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  error: Error | null;
}

/**
 * Last-resort render guard. A crash inside the tree would otherwise leave an
 * empty window with no clue on screen; this shows the message and stack so a
 * tester can screenshot it.
 *
 * Styles are inline on purpose: the boundary must stay readable even if the
 * stylesheet failed to load.
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Nexthrum render error:", error, info.componentStack);
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;

    return (
      <div
        style={{
          height: "100%",
          overflow: "auto",
          padding: "24px",
          background: "#0f1724",
          color: "#a4b3c8",
          font: "13px/1.6 system-ui, 'Segoe UI', 'Microsoft YaHei UI', sans-serif",
        }}
      >
        <div
          style={{
            maxWidth: "48rem",
            margin: "0 auto",
            border: "1px solid rgba(248,113,113,0.35)",
            borderRadius: "12px",
            background: "rgba(248,113,113,0.08)",
            padding: "16px",
          }}
        >
          <h1 style={{ margin: 0, color: "#f87171", fontSize: "15px", fontWeight: 600 }}>
            界面加载失败
          </h1>
          <p style={{ margin: "6px 0 12px", fontSize: "12px" }}>
            请把下面的错误信息（含 Console 堆栈）截图反馈，修复后再重新测试。
          </p>
          <pre
            style={{
              margin: 0,
              whiteSpace: "pre-wrap",
              wordBreak: "break-word",
              font: "12px/1.5 ui-monospace, 'Cascadia Mono', Consolas, monospace",
              color: "#e8f0fb",
            }}
          >
            {error.message}
            {error.stack ? `\n\n${error.stack}` : ""}
          </pre>
        </div>
      </div>
    );
  }
}

import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ErrorBoundary } from "./components/common/ErrorBoundary";
import { initTheme } from "./theme/useTheme";
import { enableDevPreviewIfRequested } from "./devPreview";
import "./styles/index.css";

const container = document.getElementById("root") as HTMLElement | null;

/**
 * Replace the bootstrap fallback with the real error. Only used while the app
 * has not rendered yet, so ordinary runtime errors are left to the boundary.
 */
function renderFatalError(detail: unknown): void {
  const root = document.getElementById("root");
  if (!root) return;
  if (root.querySelector("[data-app-mounted]")) return;
  const text = detail instanceof Error ? (detail.stack ?? detail.message) : String(detail);
  root.innerHTML = "";

  const box = document.createElement("div");
  box.style.cssText =
    "height:100%;overflow:auto;padding:24px;background:#0f1724;color:#a4b3c8;" +
    "font:13px/1.6 system-ui,'Segoe UI','Microsoft YaHei UI',sans-serif";

  const inner = document.createElement("div");
  inner.style.cssText =
    "max-width:48rem;margin:0 auto;border:1px solid rgba(248,113,113,.35);" +
    "border-radius:12px;background:rgba(248,113,113,.08);padding:16px";

  const title = document.createElement("h1");
  title.textContent = "界面未能启动";
  title.style.cssText = "margin:0;color:#f87171;font-size:15px;font-weight:600";

  const hint = document.createElement("p");
  hint.textContent =
    "前端脚本在挂载前抛出错误。请截图本页与 DevTools Console 反馈。";
  hint.style.cssText = "margin:6px 0 12px;font-size:12px";

  const pre = document.createElement("pre");
  pre.textContent = text;
  pre.style.cssText =
    "margin:0;white-space:pre-wrap;word-break:break-word;color:#e8f0fb;" +
    "font:12px/1.5 ui-monospace,'Cascadia Mono',Consolas,monospace";

  inner.append(title, hint, pre);
  box.append(inner);
  root.append(box);
}

window.addEventListener("error", (event) => {
  if (!event.error && !event.message) return;
  renderFatalError(event.error ?? event.message);
});
window.addEventListener("unhandledrejection", (event) => {
  renderFatalError(event.reason);
});

// Paint the resolved theme before the first render so the window never flashes
// the wrong palette.
initTheme();

// `?preview=demo` seeds fake messages so the list can be reviewed in a browser
// preview, where the Rust host is absent. Dev-only.
if (import.meta.env.DEV) {
  enableDevPreviewIfRequested();
}

if (container) {
  ReactDOM.createRoot(container).render(
    <React.StrictMode>
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </React.StrictMode>,
  );
} else {
  renderFatalError("#root 容器缺失，index.html 可能没有正确加载。");
}

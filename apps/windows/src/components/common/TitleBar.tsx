import { useEffect, useMemo, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "../../i18n";
import { isTauriRuntime } from "../../utils/tauriEnv";
import { cx } from "../../utils/cx";
import {
  MaximizeGlyph,
  MinimizeGlyph,
  RestoreGlyph,
  WindowCloseGlyph,
} from "../ui/icons";

/**
 * Windows caption bar.
 *
 * The window is created with `decorations: false`, so the app owns the logo,
 * product name and the three caption buttons. The drag region and the buttons
 * are separate elements: pressing a button must never start a window drag.
 *
 * Outside Tauri (plain browser) the buttons are disabled: `getCurrentWindow()`
 * would throw because the IPC internals are not injected.
 */
export function TitleBar() {
  const { t } = useI18n();
  const [maximized, setMaximized] = useState(false);

  // `getCurrentWindow()` throws without the Tauri IPC bridge, so resolve it once
  // and only when the runtime is available.
  const appWindow = useMemo(() => (isTauriRuntime() ? getCurrentWindow() : null), []);

  useEffect(() => {
    if (!appWindow) return;
    let disposed = false;

    const sync = async () => {
      try {
        const value = await appWindow.isMaximized();
        if (!disposed) setMaximized(value);
      } catch {
        // A failed probe keeps the last known state instead of flickering.
      }
    };

    void sync();
    const unlisten = appWindow.onResized(() => {
      void sync();
    });

    return () => {
      disposed = true;
      void unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [appWindow]);

  const withWindow = (action: (window: NonNullable<typeof appWindow>) => Promise<void>) => () => {
    if (!appWindow) return;
    void action(appWindow).catch(() => {});
  };

  const startDragging = withWindow((window) => window.startDragging());
  const handleToggleMaximize = withWindow((window) => window.toggleMaximize());
  const handleMinimize = withWindow((window) => window.minimize());
  const handleClose = withWindow((window) => window.close());

  const captionButton = (
    label: string,
    onClick: () => void,
    glyph: ReactNode,
    danger = false,
  ) => (
    <button
      type="button"
      title={label}
      aria-label={label}
      onClick={onClick}
      disabled={!appWindow}
      className={cx(
        "inline-flex w-11 items-center justify-center transition-colors",
        "disabled:cursor-not-allowed disabled:opacity-40",
        danger
          ? "text-[var(--color-text-secondary)] hover:bg-[var(--color-danger)] hover:text-white"
          : "text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
      )}
    >
      {glyph}
    </button>
  );

  return (
    <div className="flex h-[34px] flex-shrink-0 select-none items-stretch border-b border-[var(--color-border)] bg-[var(--color-surface)]">
      <div
        className="flex min-w-0 flex-1 cursor-default items-center gap-1.5 px-3"
        onMouseDown={(event) => {
          if (event.button !== 0) return;
          startDragging();
        }}
        onDoubleClick={handleToggleMaximize}
      >
        <img
          src="/favicon.png"
          alt=""
          width={14}
          height={14}
          className="h-3.5 w-3.5 flex-shrink-0"
          draggable={false}
        />
        <span className="truncate text-[11px] font-medium tracking-[0.01em] text-[var(--color-text-muted)]">
          {t.app.title}
        </span>
        {!appWindow && (
          <span
            className="flex-shrink-0 rounded-full bg-[var(--color-surface-hover)] px-1.5 text-[10px] text-[var(--color-text-muted)]"
            title="没有检测到 Tauri 运行时：当前是浏览器预览，消息与窗口操作不可用"
          >
            browser preview
          </span>
        )}
      </div>

      <div
        className="flex flex-shrink-0 items-stretch"
        onMouseDown={(event) => event.stopPropagation()}
        onDoubleClick={(event) => event.stopPropagation()}
      >
        {captionButton(t.window.minimize, handleMinimize, <MinimizeGlyph />)}
        {captionButton(
          maximized ? t.window.restore : t.window.maximize,
          handleToggleMaximize,
          maximized ? <RestoreGlyph /> : <MaximizeGlyph />,
        )}
        {captionButton(t.window.close, handleClose, <WindowCloseGlyph />, true)}
      </div>
    </div>
  );
}

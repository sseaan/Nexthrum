import { useEffect, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "../../i18n";
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
 */
export function TitleBar() {
  const { t } = useI18n();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let disposed = false;
    const appWindow = getCurrentWindow();

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
  }, []);

  const startDragging = () => {
    void getCurrentWindow().startDragging().catch(() => {});
  };

  const handleToggleMaximize = () => {
    void getCurrentWindow().toggleMaximize().catch(() => {});
  };

  const handleMinimize = () => {
    void getCurrentWindow().minimize().catch(() => {});
  };

  const handleClose = () => {
    void getCurrentWindow().close().catch(() => {});
  };

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
      className={cx(
        "inline-flex w-11 items-center justify-center transition-colors",
        danger
          ? "text-[var(--color-text-secondary)] hover:bg-[var(--color-danger)] hover:text-white"
          : "text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
      )}
    >
      {glyph}
    </button>
  );

  return (
    <div className="flex h-9 flex-shrink-0 select-none items-stretch border-b border-[var(--color-border)] bg-[var(--color-surface)]">
      <div
        className="flex min-w-0 flex-1 cursor-default items-center gap-2 px-3"
        onMouseDown={(event) => {
          if (event.button !== 0) return;
          startDragging();
        }}
        onDoubleClick={handleToggleMaximize}
      >
        <img
          src="/favicon.png"
          alt=""
          width={16}
          height={16}
          className="h-4 w-4 flex-shrink-0"
          draggable={false}
        />
        <span className="truncate text-xs font-semibold tracking-wide text-[var(--color-text-secondary)]">
          {t.app.title}
        </span>
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

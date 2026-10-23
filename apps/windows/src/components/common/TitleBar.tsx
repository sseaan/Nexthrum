import { getCurrentWindow } from "@tauri-apps/api/window";

export function TitleBar() {
  const appWindow = getCurrentWindow();

  const handleMinimize = async () => {
    await appWindow.minimize();
  };

  const handleClose = async () => {
    await appWindow.close();
  };

  return (
    <div
      onMouseDown={() => appWindow.startDragging()}
      className="flex items-center justify-between h-9 px-2 select-none bg-[var(--color-surface)] border-b border-[var(--color-border)] cursor-grab active:cursor-grabbing"
    >
      <div className="flex-1" />

      <div
        className="flex items-center -mr-1"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <button
          onClick={handleMinimize}
          className="w-8 h-7 inline-flex items-center justify-center rounded-md text-[var(--color-text-secondary)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)] transition-colors"
          title="最小化"
        >
          <svg width="10" height="1" viewBox="0 0 10 1">
            <rect width="10" height="1" fill="currentColor" />
          </svg>
        </button>
        <button
          onClick={handleClose}
          className="w-8 h-7 inline-flex items-center justify-center rounded-md text-[var(--color-text-secondary)] hover:bg-[var(--color-danger)] hover:text-white transition-colors"
          title="关闭"
        >
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path
              d="M1 1l8 8M9 1l-8 8"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}

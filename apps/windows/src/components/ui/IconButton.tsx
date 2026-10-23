import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cx } from "../../utils/cx";

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Accessible name; also used as the native tooltip. */
  label: string;
  children: ReactNode;
  tone?: "default" | "danger";
}

/** Square icon-only button. Its accessible name is never optional. */
export function IconButton({
  label,
  children,
  tone = "default",
  className,
  type = "button",
  ...rest
}: IconButtonProps) {
  return (
    <button
      type={type}
      title={label}
      aria-label={label}
      className={cx(
        "inline-flex h-8 w-8 items-center justify-center rounded-lg border border-transparent transition-colors",
        "disabled:cursor-not-allowed disabled:opacity-40",
        tone === "danger"
          ? "text-[var(--color-text-secondary)] hover:bg-[var(--color-danger-soft)] hover:text-[var(--color-danger)]"
          : "text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
        className,
      )}
      {...rest}
    >
      {children}
    </button>
  );
}

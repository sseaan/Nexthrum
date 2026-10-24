import type { ReactNode } from "react";
import { cx } from "../../utils/cx";

interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  description?: ReactNode;
  id?: string;
}

/** Compact macOS-style switch; replaces the raw checkbox. */
export function Switch({ checked, onChange, label, description, id }: SwitchProps) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="min-w-0 space-y-0.5">
        <label
          htmlFor={id}
          className="block cursor-pointer text-[13px] text-[var(--color-text)]"
        >
          {label}
        </label>
        {description && (
          <p className="text-xs leading-relaxed text-[var(--color-text-muted)]">
            {description}
          </p>
        )}
      </div>
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        onClick={() => onChange(!checked)}
        className={cx(
          "relative inline-flex h-[22px] w-[38px] flex-shrink-0 items-center rounded-full transition-colors",
          checked
            ? "bg-[var(--color-primary)]"
            : "bg-[var(--color-border-strong)]/70",
        )}
      >
        <span
          aria-hidden="true"
          className={cx(
            "h-[18px] w-[18px] rounded-full bg-white shadow-sm transition-transform",
            checked ? "translate-x-[18px]" : "translate-x-[2px]",
          )}
        />
      </button>
    </div>
  );
}

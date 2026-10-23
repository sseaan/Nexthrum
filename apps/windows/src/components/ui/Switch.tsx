import type { ReactNode } from "react";
import { cx } from "../../utils/cx";

interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  description?: ReactNode;
  id?: string;
}

/** Accessible on/off switch row; replaces the raw checkbox. */
export function Switch({ checked, onChange, label, description, id }: SwitchProps) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div className="min-w-0 space-y-0.5">
        <label
          htmlFor={id}
          className="block cursor-pointer text-sm font-medium text-[var(--color-text)]"
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
          "relative mt-0.5 inline-flex h-5 w-9 flex-shrink-0 items-center rounded-full border transition-colors",
          checked
            ? "border-[var(--color-primary)] bg-[var(--color-primary)]"
            : "border-[var(--color-border-strong)] bg-[var(--color-surface-hover)]",
        )}
      >
        <span
          aria-hidden="true"
          className={cx(
            "ml-0.5 h-4 w-4 rounded-full bg-white shadow-sm transition-transform",
            checked ? "translate-x-4" : "translate-x-0",
          )}
        />
      </button>
    </div>
  );
}

import type { InputHTMLAttributes, ReactNode } from "react";
import { cx } from "../../utils/cx";

interface FieldProps {
  label: string;
  hint?: ReactNode;
  htmlFor?: string;
  children: ReactNode;
  className?: string;
}

/** Label + control + optional hint, used by every settings-style form. */
export function Field({ label, hint, htmlFor, children, className }: FieldProps) {
  return (
    <div className={cx("space-y-1.5", className)}>
      <label
        htmlFor={htmlFor}
        className="block text-xs font-medium text-[var(--color-text-secondary)]"
      >
        {label}
      </label>
      {children}
      {hint && (
        <p className="text-xs leading-relaxed text-[var(--color-text-muted)]">{hint}</p>
      )}
    </div>
  );
}

type InputSize = "sm" | "md";

interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "size"> {
  size?: InputSize;
  /** Reserve room for a leading icon slot. */
  leading?: boolean;
  /** Reserve room for a trailing button slot. */
  trailing?: boolean;
  invalid?: boolean;
}

const HEIGHTS: Record<InputSize, string> = {
  sm: "h-7 text-xs",
  md: "h-8 text-[13px]",
};

/**
 * Reserves icon space with explicit padding classes instead of letting callers
 * override base utilities: Tailwind emits same-property utilities in source
 * order, so a `className="pl-7"` would not reliably beat the base `px-2.5`.
 */
function padding(size: InputSize, leading?: boolean, trailing?: boolean): string {
  const left = leading ? (size === "sm" ? "pl-6" : "pl-7") : "pl-2.5";
  const right = trailing ? (size === "sm" ? "pr-6" : "pr-7") : "pr-2.5";
  return `${left} ${right}`;
}

/**
 * Filled field instead of an outlined box: the macOS text field reads as a
 * slightly darker well on the surface, with the focus ring as the only chrome.
 */
export function Input({
  className,
  size = "md",
  leading,
  trailing,
  invalid,
  ...rest
}: InputProps) {
  return (
    <input
      className={cx(
        "w-full rounded-lg border border-transparent",
        "bg-[var(--color-surface-hover)] text-[var(--color-text)]",
        "placeholder:text-[var(--color-text-muted)]",
        "transition-colors focus:outline-none focus:ring-2",
        invalid
          ? "ring-[var(--color-danger)]/35 focus:bg-[var(--color-surface)]"
          : "focus:bg-[var(--color-surface)] focus:ring-[var(--color-primary)]/30 hover:bg-[var(--color-border)]/60",
        HEIGHTS[size],
        padding(size, leading, trailing),
        className,
      )}
      {...rest}
    />
  );
}

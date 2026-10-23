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

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  invalid?: boolean;
}

/** Text input with a visible focus ring that works in both themes. */
export function Input({ className, invalid, ...rest }: InputProps) {
  return (
    <input
      className={cx(
        "h-9 w-full rounded-lg border bg-[var(--color-surface)] px-3 text-sm",
        "text-[var(--color-text)] placeholder:text-[var(--color-text-muted)]",
        "transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]/35",
        invalid
          ? "border-[var(--color-danger)] focus:border-[var(--color-danger)]"
          : "border-[var(--color-border)] hover:border-[var(--color-border-strong)] focus:border-[var(--color-primary)]",
        className,
      )}
      {...rest}
    />
  );
}

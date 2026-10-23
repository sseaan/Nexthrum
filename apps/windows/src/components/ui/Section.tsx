import type { ReactNode } from "react";
import { cx } from "../../utils/cx";

interface SectionProps {
  title: string;
  description?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
}

/** Grouped settings card: heading outside, fields on a raised surface. */
export function Section({
  title,
  description,
  actions,
  children,
  className,
}: SectionProps) {
  return (
    <section className={cx("space-y-2", className)}>
      <div className="flex items-baseline justify-between gap-3 px-1">
        <h2 className="text-sm font-semibold text-[var(--color-text)]">{title}</h2>
        {actions}
      </div>
      {description && (
        <p className="px-1 text-xs leading-relaxed text-[var(--color-text-muted)]">
          {description}
        </p>
      )}
      <div className="space-y-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-card)]">
        {children}
      </div>
    </section>
  );
}

import type { ReactNode } from "react";
import { cx } from "../../utils/cx";

interface SectionProps {
  title: string;
  description?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
}

/**
 * Grouped settings block, macOS System Settings style: a small caption above a
 * flat surface, rows separated by hairlines instead of boxes inside boxes.
 */
export function Section({
  title,
  description,
  actions,
  children,
  className,
}: SectionProps) {
  return (
    <section className={cx("space-y-1.5", className)}>
      <div className="flex items-baseline justify-between gap-3 px-1">
        <h2 className="text-[11px] font-semibold uppercase tracking-[0.06em] text-[var(--color-text-muted)]">
          {title}
        </h2>
        {actions}
      </div>
      {description && (
        <p className="px-1 text-xs leading-relaxed text-[var(--color-text-muted)]">
          {description}
        </p>
      )}
      <div
        className={cx(
          "divide-y divide-[var(--color-border)] overflow-hidden rounded-xl bg-[var(--color-surface)]",
          "[&>*]:px-3.5 [&>*]:py-3",
        )}
      >
        {children}
      </div>
    </section>
  );
}

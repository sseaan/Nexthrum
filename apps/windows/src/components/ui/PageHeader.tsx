import type { ReactNode } from "react";

interface PageHeaderProps {
  title: string;
  /** Optional leading control, e.g. a back button. */
  leading?: ReactNode;
  /** Optional badge rendered inline after the title (e.g. unread count). */
  badge?: ReactNode;
  /** Right-aligned actions. */
  actions?: ReactNode;
  /** Rendered under the title row, still inside the header surface. */
  children?: ReactNode;
}

/**
 * Compact toolbar above the content column: one row of title/actions, an
 * optional second row for search or status. Flat surface plus a hairline, no
 * shadow or box.
 */
export function PageHeader({ title, leading, badge, actions, children }: PageHeaderProps) {
  return (
    <header className="flex-shrink-0 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
      <div className="mx-auto w-full max-w-2xl px-3 pb-1.5 pt-2">
        <div className="flex min-h-8 items-center gap-1.5">
          {leading}
          <h1 className="truncate text-[15px] font-semibold tracking-[-0.01em] text-[var(--color-text)]">
            {title}
          </h1>
          {badge}
          <div className="flex-1" />
          {actions && <div className="flex items-center gap-0.5">{actions}</div>}
        </div>
        {children}
      </div>
    </header>
  );
}

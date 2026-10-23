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
 * Sticky page header used by messages and settings: one row of title/actions,
 * optional second row for search or status. Keeps both pages aligned.
 */
export function PageHeader({ title, leading, badge, actions, children }: PageHeaderProps) {
  return (
    <header className="flex-shrink-0 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
      <div className="mx-auto w-full max-w-3xl px-4 py-2.5">
        <div className="flex min-h-8 items-center gap-2">
          {leading}
          <h1 className="truncate text-base font-semibold text-[var(--color-text)]">
            {title}
          </h1>
          {badge}
          <div className="flex-1" />
          {actions && <div className="flex items-center gap-1.5">{actions}</div>}
        </div>
        {children}
      </div>
    </header>
  );
}

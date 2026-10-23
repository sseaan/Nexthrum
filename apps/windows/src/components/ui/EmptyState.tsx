import type { ReactNode } from "react";

interface EmptyStateProps {
  icon: ReactNode;
  title: string;
  hint?: string;
  /** Optional inline action, e.g. "open settings". */
  action?: ReactNode;
}

/** Shared placeholder for "no messages" and "no search matches". */
export function EmptyState({ icon, title, hint, action }: EmptyStateProps) {
  return (
    <div className="flex h-full items-center justify-center px-6 py-10">
      <div className="flex max-w-sm flex-col items-center text-center">
        <div
          className="mb-3 flex h-11 w-11 items-center justify-center rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text-muted)]"
          aria-hidden="true"
        >
          {icon}
        </div>
        <p className="text-sm font-medium text-[var(--color-text)]">{title}</p>
        {hint && (
          <p className="mt-1 text-xs leading-relaxed text-[var(--color-text-muted)]">
            {hint}
          </p>
        )}
        {action && <div className="mt-3">{action}</div>}
      </div>
    </div>
  );
}

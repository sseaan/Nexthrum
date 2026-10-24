import type { ReactNode } from "react";

interface EmptyStateProps {
  icon: ReactNode;
  title: string;
  hint?: string;
  /** Optional inline action, e.g. "open settings". */
  action?: ReactNode;
}

/** Quiet placeholder used by "no messages" and "no search matches". */
export function EmptyState({ icon, title, hint, action }: EmptyStateProps) {
  return (
    <div className="flex h-full items-center justify-center px-6 py-10">
      <div className="flex max-w-[16rem] flex-col items-center text-center">
        <span className="mb-3 text-[var(--color-text-muted)]/70" aria-hidden="true">
          {icon}
        </span>
        <p className="text-[13px] font-medium text-[var(--color-text-secondary)]">{title}</p>
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

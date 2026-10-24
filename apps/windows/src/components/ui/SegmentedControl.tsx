import { cx } from "../../utils/cx";

interface SegmentedOption<T extends string> {
  value: T;
  label: string;
}

interface SegmentedControlProps<T extends string> {
  options: SegmentedOption<T>[];
  value: T;
  onChange: (value: T) => void;
  label: string;
  className?: string;
}

/** Inline radio group; macOS segmented control with a raised selected pill. */
export function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
  label,
  className,
}: SegmentedControlProps<T>) {
  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={cx(
        "inline-flex flex-wrap gap-0.5 rounded-lg bg-[var(--color-surface-hover)] p-0.5",
        className,
      )}
    >
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={selected}
            onClick={() => onChange(option.value)}
            className={cx(
              "rounded-[6px] px-2.5 py-1 text-xs font-medium transition-colors",
              selected
                ? "bg-[var(--color-surface)] text-[var(--color-text)] shadow-[0_1px_2px_rgba(0,0,0,0.12)]"
                : "text-[var(--color-text-secondary)] hover:text-[var(--color-text)]",
            )}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}

/**
 * Inline icon set. All icons are 16×16 viewBoxes drawn with `currentColor` so
 * they inherit the surrounding text colour in both themes.
 */
interface IconProps {
  size?: number;
  className?: string;
}

function base(size: number) {
  return {
    width: size,
    height: size,
    viewBox: "0 0 16 16",
    fill: "none",
    "aria-hidden": true as const,
    focusable: "false" as const,
  };
}

export function RefreshIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path
        d="M13.2 8a5.2 5.2 0 1 1-1.7-3.86M13.2 2.2v3.4h-3.4"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function SettingsIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <circle cx="8" cy="8" r="2.1" stroke="currentColor" strokeWidth="1.4" />
      <path
        d="M8 1.8v1.5M8 12.7v1.5M1.8 8h1.5M12.7 8h1.5M3.6 3.6l1.05 1.05M11.35 11.35l1.05 1.05M12.4 3.6l-1.05 1.05M4.65 11.35l-1.05 1.05"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function SearchIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <circle cx="7.1" cy="7.1" r="4.3" stroke="currentColor" strokeWidth="1.4" />
      <path
        d="M10.4 10.4l3 3"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function ChevronLeftIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path
        d="M9.8 3.6L5.4 8l4.4 4.4"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function CloseIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path
        d="M4 4l8 8M12 4l-8 8"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function CheckIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path
        d="M3.2 8.4l3.1 3.1 6.5-6.9"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function AlertIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path
        d="M8 2.6l6 10.8H2L8 2.6z"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinejoin="round"
      />
      <path
        d="M8 6.6v3.2"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
      />
      <circle cx="8" cy="11.6" r="0.75" fill="currentColor" />
    </svg>
  );
}

export function BellIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path
        d="M4.2 6.8a3.8 3.8 0 0 1 7.6 0c0 2 .5 3 .9 3.5H3.3c.4-.5.9-1.5.9-3.5z"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinejoin="round"
      />
      <path
        d="M6.6 12.2a1.5 1.5 0 0 0 2.8 0"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
      />
    </svg>
  );
}

/* ---- Windows window controls (10×10 glyphs inside a 16×16 box) ---- */

export function MinimizeGlyph() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
      <path d="M2.5 6h7" stroke="currentColor" strokeWidth="1" strokeLinecap="round" />
    </svg>
  );
}

export function MaximizeGlyph() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
      <rect x="2.5" y="2.5" width="7" height="7" rx="1" stroke="currentColor" strokeWidth="1" />
    </svg>
  );
}

export function RestoreGlyph() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
      <rect x="2.2" y="3.6" width="6.2" height="6.2" rx="1" stroke="currentColor" strokeWidth="1" />
      <path d="M4.4 3.4V2.6a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v4a1 1 0 0 1-1 1h-.8" stroke="currentColor" strokeWidth="1" strokeLinecap="round" />
    </svg>
  );
}

export function WindowCloseGlyph() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
      <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
    </svg>
  );
}

export function InboxIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path
        d="M2.2 9.2L4 3.4h8l1.8 5.8v3.2a1.2 1.2 0 0 1-1.2 1.2H3.4a1.2 1.2 0 0 1-1.2-1.2V9.2z"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinejoin="round"
      />
      <path
        d="M2.2 9.2h3l.8 1.6h3.9l.8-1.6h3.1"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export function DownloadIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path
        d="M8 2.6v8.2M4.9 7.8L8 10.9l3.1-3.1"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M3.2 12.6h9.6"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function SunIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <circle cx="8" cy="8" r="3.1" stroke="currentColor" strokeWidth="1.4" />
      <path
        d="M8 1.4v1.7M8 12.9v1.7M1.4 8h1.7M12.9 8h1.7M3.3 3.3l1.2 1.2M11.5 11.5l1.2 1.2M12.7 3.3l-1.2 1.2M4.5 11.5l-1.2 1.2"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function MoonIcon({ size = 16, className }: IconProps) {
  return (
    <svg {...base(size)} className={className}>
      <path
        d="M13.2 9.6A5.6 5.6 0 0 1 6.4 2.8a5.6 5.6 0 1 0 6.8 6.8z"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinejoin="round"
      />
    </svg>
  );
}

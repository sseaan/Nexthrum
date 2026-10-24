# Images

Assets referenced by `README.md` / `README.zh.md`.

| File | Purpose |
|---|---|
| `logo.png` | README header logo (256×256, from `apps/windows/src-tauri/icons/128x128@2x.png`) |
| `macos-messages.png` | macOS message list |
| `macos-settings.png` | macOS settings |
| `windows-messages.png` | Windows message list |
| `windows-settings.png` | Windows settings |

## Replacing the screenshots

The four `*-messages.png` / `*-settings.png` files are placeholders. Replace them in place with the same file names so the README keeps working, then commit.

- Export at 2× (Retina) resolution, then downscale to **1400 px wide** — `sips -Z 1400 shot.png`
- Crop to the window, no desktop background, no cursor
- macOS: capture with `⌘⇧4` then `Space`, or `screencapture -o -l <windowid> shot.png`
- Windows: `Alt + PrtScn` (current window) at 100 % scaling, then downscale
- Prefer one light-theme and one dark-theme shot per platform when both are available; keep the file names above for the primary (light) shot

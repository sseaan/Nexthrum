import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    // Must stay identical to src-tauri/tauri.conf.json "devUrl".
    // 1420 is commonly inside a Hyper-V/WinNAT excluded range on Windows, which
    // fails the whole launch with strictPort; 1430 is less frequently reserved.
    // See docs/windows-testing.md 8.2 if this port is taken as well.
    port: 1430,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1431,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));

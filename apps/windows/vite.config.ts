import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

/**
 * Dev server port.
 *
 * Must stay identical to `build.devUrl` in src-tauri/tauri.conf.json — Tauri
 * loads that URL verbatim, so a mismatched port shows a blank window.
 *
 * Windows reserves port ranges for Hyper-V/WinNAT (`netsh int ipv4 show
 * excludedportrange protocol=tcp`) and 1420/1430 are frequently inside one.
 * Run `scripts/dev-port.ps1` to pick a free port and rewrite both files.
 */
const DEV_PORT = 1520;
const DEV_HMR_PORT = DEV_PORT + 1;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: DEV_PORT,
    // Failing loudly beats vite silently choosing another port: Tauri would
    // keep loading DEV_PORT and render an empty page.
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: DEV_HMR_PORT,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));

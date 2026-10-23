import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // The store and utilities are plain TypeScript; no DOM is required.
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});

import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "path";

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      "$lib/components/settings/doomscrolling-browser-connection": path.resolve(
        "./src/lib/components/settings/DoomscrollingBrowserConnectionStatus.svelte",
      ),
      "$lib/components/settings/doomscrolling-desktop-selector": path.resolve(
        "./src/lib/components/settings/DoomscrollingAppSelector.svelte",
      ),
      "$lib/stores/doomscrolling-usage.svelte": path.resolve(
        "./src/lib/stores/doomscrolling-usage.svelte.ts",
      ),
      $lib: path.resolve("./src/lib"),
    },
    conditions: ["browser"],
  },
  test: {
    include: ["src/**/*.test.ts"],
    setupFiles: ["./test-setup.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "html"],
      include: ["src/lib/**/*.ts"],
      exclude: ["src/lib/components/ui/**", "**/*.test.ts", "**/*.d.ts"],
    },
  },
});

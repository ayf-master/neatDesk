import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],

  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },

  // prevent vite from obscuring rust errors
  clearScreen: false,

  // to access the Tauri environment variables set by the CLI with information about the current target
  envPrefix: ["VITE_", "TAURI_ENV_*"],

  build: {
    // Tauri uses WebView2 (Chromium) on Windows — modern targets
    target: ["es2021", "chrome100"],
    // don't minify for debug builds
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});

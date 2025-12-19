import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

// eslint-disable-next-line node/prefer-global/process
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    host: host || false,
    port: 1420,
    strictPort: true,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
  },
});

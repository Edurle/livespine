import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  // 明确指定前端 root，避免 Vite 监视到 src-tauri/target 触发 EBUSY
  root: "src",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
  },
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // 不监视 Rust 后端的编译产物（避免 EBUSY）
      ignored: ["**/src-tauri/**"],
    },
  },
});

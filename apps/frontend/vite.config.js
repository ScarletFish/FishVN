import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// 后端地址与常量保持一致：apps/server/constants.ts 的 PORT / DEFAULT_HOST
const API_TARGET = 'http://127.0.0.1:4545';

export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: '../../dist/web',
    emptyOutDir: true,
  },
  server: {
    // 与全项目统一：只绑 IPv4 回环，不用 localhost（本机解析先 ::1，
    // 会让 127.0.0.1 访问被拒 / IPv6-first 的客户端多吃回退延迟）
    host: '127.0.0.1',
    proxy: {
      '/api': API_TARGET,
    },
  },
});
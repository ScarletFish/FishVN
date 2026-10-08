import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    // 与全项目统一：只绑 IPv4 回环，不用 localhost（本机解析先 ::1，
    // 会让 127.0.0.1 访问被拒 / IPv6-first 的客户端多吃回退延迟）
    host: '127.0.0.1',
  },
});
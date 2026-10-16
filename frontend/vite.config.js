import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const isTest = process.env.VITEST === 'true';

export default defineConfig({
  plugins: [svelte()],
  // 测试时 svelte 要解析到浏览器构建：SSR 构建里 mount() 直接抛错。
  // 不能写成 conditions: [] —— 空数组同样会顶掉 vite 的默认条件，产物里会混进 svelte 的服务端模块
  ...(isTest ? { resolve: { conditions: ['browser'] } } : {}),
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    // host / port 要与 tauri.conf.json 的 devUrl 同协议族、同端口：
    // localhost 会让 vite 只绑 ::1，devUrl 的 IPv4 字面量就够不着；
    // strictPort 让端口占用直接失败，不自动往上换
    host: '127.0.0.1',
    port: 4545,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**'] },
    proxy: {
      // 开发通道：只在这里放开跨源转发
      '/dev-http': {
        target: 'http://127.0.0.1:4546',
        rewrite: (p) => p.replace(/^\/dev-http/, ''),
        changeOrigin: false,
      },
    },
  },
  test: {
    include: ['src/**/*.test.js'],
  },
});
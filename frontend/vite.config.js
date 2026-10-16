import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const isTest = process.env.VITEST === 'true';

export default defineConfig({
  plugins: [svelte()],
  ...(isTest ? { resolve: { conditions: ['browser'] } } : {}),
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    host: '127.0.0.1',
    port: 4545,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**'] },
    proxy: {
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
import net from 'node:net';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const isTest = process.env.VITEST === 'true';
const BACKEND_PORT = 4546;

function probeBackend() {
  return new Promise((resolve) => {
    const socket = net.connect({ host: '127.0.0.1', port: BACKEND_PORT });
    let settled = false;
    const done = (value) => {
      if (settled) return;
      settled = true;
      socket.destroy();
      resolve(value);
    };
    socket.setTimeout(200);
    socket.once('connect', () => done(true));
    socket.once('timeout', () => done(false));
    socket.once('error', () => done(false));
  });
}

function backendReadyProbe() {
  return {
    name: 'fishvn-backend-ready',
    configureServer(server) {
      server.middlewares.use('/__dev_backend_ready', async (_req, res) => {
        const ready = await probeBackend();
        res.statusCode = ready ? 204 : 503;
        res.setHeader('cache-control', 'no-store');
        res.end();
      });
    },
  };
}

export default defineConfig({
  plugins: [svelte(), backendReadyProbe()],
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
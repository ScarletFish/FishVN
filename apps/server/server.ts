import { serve } from '@hono/node-server';
import { serveStatic } from '@hono/node-server/serve-static';
import { readFile } from 'node:fs/promises';
import type { AddressInfo } from 'node:net';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { Hono } from 'hono';
import { API_PREFIX, DEFAULT_HOST, PORT } from './constants.js';
import { getDb } from './db.js';
import { WEB_DIR } from './lib/paths.js';
import { healthRoutes } from './routes/health.js';
import { gameRoutes } from './routes/games.js';
import { installationRoutes } from './routes/installations.js';

const app = new Hono();

app.route(API_PREFIX, healthRoutes);
app.route(API_PREFIX, gameRoutes);
app.route(API_PREFIX, installationRoutes);

// 静态托管放在 API 之后：/api/* 直接跳过，否则 SPA 兜底会把 API 路径也吃掉
const serveWeb = serveStatic({ root: WEB_DIR });
app.use('*', (c, next) =>
  c.req.path.startsWith(`${API_PREFIX}/`) ? next() : serveWeb(c, next)
);

// 静态未命中的非 API 路径一律回 index.html，交给前端路由
app.get('*', async (c) => {
  if (c.req.path.startsWith(`${API_PREFIX}/`)) return c.notFound();
  return c.html(await readFile(join(WEB_DIR, 'index.html'), 'utf8'));
});

app.notFound((c) => c.json({ error: 'not found' }, 404));

export interface RunningServer {
  port: number;
  close: () => Promise<void>;
}

/** 必须显式给 hostname：不传时 Node 会绑双栈通配地址 */
export function startServer(
  port: number = PORT,
  host: string = process.env.HOST ?? DEFAULT_HOST
): Promise<RunningServer> {
  getDb();
  return new Promise<RunningServer>((done, fail) => {
    const server = serve({ fetch: app.fetch, port, hostname: host }, (info: AddressInfo) => {
      done({
        port: info.port,
        close: () =>
          new Promise<void>((closed, broken) => {
            server.close((err) => (err ? broken(err) : closed()));
          }),
      });
    });
    server.once('error', fail);
  });
}

/** 被 import 时不启动，只有直接执行本文件才起服务 */
const entry = process.argv[1];
if (entry !== undefined && pathToFileURL(resolve(entry)).href === import.meta.url) {
  const { port } = await startServer();
  console.log(`[server] http://${process.env.HOST ?? DEFAULT_HOST}:${port}`);
}
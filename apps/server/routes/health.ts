import { Hono } from 'hono';

/** 挂在 API_PREFIX 下，对外路径为 /api/health */
export const healthRoutes = new Hono().get('/health', (c) => c.json({ ok: true }));
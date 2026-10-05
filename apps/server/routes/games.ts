import { randomUUID } from 'node:crypto';
import { Hono } from 'hono';
import type { Context } from 'hono';
import { getDb } from '../db.js';
import type { Game } from '../types.js';

/** VNDB 编号形如 v14，v 前缀 + 数字 */
const VNDB_ID = /^v\d+$/;

type Fields = { name?: string; vndb_id?: string | null };
type Parse = { ok: true; fields: Fields } | { ok: false; error: string };

/** vndb_id 显式给 null 是解绑，不给才是「不动」 */
async function parseBody(c: Context): Promise<Parse> {
  let raw: unknown;
  try {
    raw = await c.req.json();
  } catch {
    return { ok: false, error: 'body 不是合法 JSON' };
  }
  if (typeof raw !== 'object' || raw === null) return { ok: false, error: 'body 必须是 JSON 对象' };

  const body = raw as Record<string, unknown>;
  const fields: Fields = {};

  if (body.name !== undefined) {
    if (typeof body.name !== 'string') return { ok: false, error: 'name 必须是字符串' };
    const name = body.name.trim();
    if (!name) return { ok: false, error: 'name 不能为空' };
    fields.name = name;
  }

  if (body.vndb_id !== undefined) {
    if (body.vndb_id === null) {
      fields.vndb_id = null;
    } else if (typeof body.vndb_id === 'string' && VNDB_ID.test(body.vndb_id.trim())) {
      fields.vndb_id = body.vndb_id.trim();
    } else {
      return { ok: false, error: 'vndb_id 必须是 v+数字，或 null 解绑' };
    }
  }

  return { ok: true, fields };
}

const now = (): number => Math.floor(Date.now() / 1000);

/** UNIQUE 冲突即「vndb_id 已归属另一条作品」，是业务状态，回 409 */
function isUniqueViolation(err: unknown): boolean {
  return err instanceof Error && (err as { code?: string }).code === 'SQLITE_CONSTRAINT_UNIQUE';
}

const COLUMNS = 'id, name, vndb_id, created_at';

export const gameRoutes = new Hono()
  .get('/games', (c) => {
    const games = getDb()
      .prepare(`SELECT ${COLUMNS} FROM games ORDER BY created_at DESC, id`)
      .all() as Game[];
    return c.json({ games });
  })

  .post('/games', async (c) => {
    const parsed = await parseBody(c);
    if (!parsed.ok) return c.json({ error: parsed.error }, 400);
    if (parsed.fields.name === undefined) return c.json({ error: 'name 必填' }, 400);

    const game: Game = {
      id: randomUUID(),
      name: parsed.fields.name,
      vndb_id: parsed.fields.vndb_id ?? null,
      created_at: now(),
    };
    try {
      getDb()
        .prepare('INSERT INTO games (id, name, vndb_id, created_at) VALUES (?, ?, ?, ?)')
        .run(game.id, game.name, game.vndb_id, game.created_at);
    } catch (err) {
      if (isUniqueViolation(err)) return c.json({ error: 'vndb_id 已被其他条目占用' }, 409);
      throw err;
    }
    return c.json({ game }, 201);
  })

  .patch('/games/:id', async (c) => {
    const parsed = await parseBody(c);
    if (!parsed.ok) return c.json({ error: parsed.error }, 400);

    const { name, vndb_id } = parsed.fields;
    if (name === undefined && vndb_id === undefined) {
      return c.json({ error: '没有可更新的字段' }, 400);
    }

    const sets: string[] = [];
    const params: (string | number | null)[] = [];
    if (name !== undefined) {
      sets.push('name = ?');
      params.push(name);
    }
    if (vndb_id !== undefined) {
      sets.push('vndb_id = ?');
      params.push(vndb_id);
    }
    const db = getDb();
    const id = c.req.param('id');
    params.push(id);

    let changes: number;
    try {
      changes = db
        .prepare(`UPDATE games SET ${sets.join(', ')} WHERE id = ?`)
        .run(...params).changes;
    } catch (err) {
      if (isUniqueViolation(err)) return c.json({ error: 'vndb_id 已被其他条目占用' }, 409);
      throw err;
    }
    if (changes === 0) return c.json({ error: '条目不存在' }, 404);

    const game = db.prepare(`SELECT ${COLUMNS} FROM games WHERE id = ?`).get(id) as Game | undefined;
    return c.json({ game });
  })

  .delete('/games/:id', (c) => {
    const changes = getDb().prepare('DELETE FROM games WHERE id = ?').run(c.req.param('id')).changes;
    if (changes === 0) return c.json({ error: '条目不存在' }, 404);
    return c.body(null, 204);
  });
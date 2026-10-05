import { randomUUID } from 'node:crypto';
import { Hono } from 'hono';
import type { Context } from 'hono';
import { getDb, type Db } from '../db.js';
import { STATUSES, type Game, type Status, type UserGameState } from '../types.js';

/** VNDB 编号形如 v14，v 前缀 + 数字 */
const VNDB_ID = /^v\d+$/;

const DEFAULT_STATUS: Status = 'wishlist';

const GAME_COLUMNS = 'id, name, vndb_id, created_at';
const STATE_COLUMNS = 'game_id, status, user_rating, notes, last_played_at, play_count';

type Fields = { name?: string; vndb_id?: string | null };
type StateFields = { status?: Status; user_rating?: number | null; notes?: string | null };
type Reject = { ok: false; error: string };
type Update = { sets: string[]; params: unknown[] };

/** 读 body 并确认是 JSON 对象 */
async function readObject(c: Context): Promise<{ ok: true; body: Record<string, unknown> } | Reject> {
  let raw: unknown;
  try {
    raw = await c.req.json();
  } catch {
    return { ok: false, error: 'body 不是合法 JSON' };
  }
  if (typeof raw !== 'object' || raw === null) return { ok: false, error: 'body 必须是 JSON 对象' };
  return { ok: true, body: raw as Record<string, unknown> };
}

async function parseGameBody(c: Context): Promise<{ ok: true; fields: Fields } | Reject> {
  const read = await readObject(c);
  if (!read.ok) return read;
  const body = read.body;

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

async function parseStateBody(c: Context): Promise<{ ok: true; fields: StateFields } | Reject> {
  const read = await readObject(c);
  if (!read.ok) return read;
  const body = read.body;

  const fields: StateFields = {};

  if (body.status !== undefined) {
    if (typeof body.status !== 'string' || !STATUSES.includes(body.status as Status)) {
      return { ok: false, error: `status 必须是 ${STATUSES.join(' / ')} 之一` };
    }
    fields.status = body.status as Status;
  }

  if (body.user_rating !== undefined) {
    if (body.user_rating !== null && typeof body.user_rating !== 'number') {
      return { ok: false, error: 'user_rating 必须是数字或 null' };
    }
    fields.user_rating = body.user_rating;
  }

  if (body.notes !== undefined) {
    if (body.notes !== null && typeof body.notes !== 'string') {
      return { ok: false, error: 'notes 必须是字符串或 null' };
    }
    fields.notes = body.notes;
  }

  return { ok: true, fields };
}

const now = (): number => Math.floor(Date.now() / 1000);

/** 键名来自调用处的字面量，不是用户输入 */
function compileUpdate(fields: Record<string, unknown>): Update | null {
  const sets: string[] = [];
  const params: unknown[] = [];
  for (const [column, value] of Object.entries(fields)) {
    if (value === undefined) continue;
    sets.push(`${column} = ?`);
    params.push(value);
  }
  return sets.length > 0 ? { sets, params } : null;
}

/** 建作品与它的初始状态，同一事务 */
function insertGameWithState(db: Db, game: Game): void {
  db.transaction(() => {
    db.prepare(`INSERT INTO games (${GAME_COLUMNS}) VALUES (?, ?, ?, ?)`).run(
      game.id,
      game.name,
      game.vndb_id,
      game.created_at
    );
    db.prepare('INSERT INTO user_game_state (game_id, status) VALUES (?, ?)').run(
      game.id,
      DEFAULT_STATUS
    );
  })();
}

/** UNIQUE 冲突即「vndb_id 已归属另一条作品」，是业务状态，回 409 */
function isUniqueViolation(err: unknown): boolean {
  return err instanceof Error && (err as { code?: string }).code === 'SQLITE_CONSTRAINT_UNIQUE';
}

export const gameRoutes = new Hono()
  .get('/games', (c) => {
    const games = getDb()
      .prepare(`SELECT ${GAME_COLUMNS} FROM games ORDER BY created_at DESC, id`)
      .all() as Game[];
    return c.json({ games });
  })

  .post('/games', async (c) => {
    const parsed = await parseGameBody(c);
    if (!parsed.ok) return c.json({ error: parsed.error }, 400);
    if (parsed.fields.name === undefined) return c.json({ error: 'name 必填' }, 400);

    const game: Game = {
      id: randomUUID(),
      name: parsed.fields.name,
      vndb_id: parsed.fields.vndb_id ?? null,
      created_at: now(),
    };
    try {
      insertGameWithState(getDb(), game);
    } catch (err) {
      if (isUniqueViolation(err)) return c.json({ error: 'vndb_id 已被其他条目占用' }, 409);
      throw err;
    }
    return c.json({ game }, 201);
  })

  .patch('/games/:id', async (c) => {
    const parsed = await parseGameBody(c);
    if (!parsed.ok) return c.json({ error: parsed.error }, 400);

    // vndb_id 显式给 null 是解绑，不给才是「不动」
    const id = c.req.param('id');
    const update = compileUpdate(parsed.fields);
    if (!update) return c.json({ error: '没有可更新的字段' }, 400);
    update.params.push(id);

    const db = getDb();
    let changes: number;
    try {
      changes = db
        .prepare(`UPDATE games SET ${update.sets.join(', ')} WHERE id = ?`)
        .run(...update.params).changes;
    } catch (err) {
      if (isUniqueViolation(err)) return c.json({ error: 'vndb_id 已被其他条目占用' }, 409);
      throw err;
    }
    if (changes === 0) return c.json({ error: '条目不存在' }, 404);

    const game = db.prepare(`SELECT ${GAME_COLUMNS} FROM games WHERE id = ?`).get(id) as Game;
    return c.json({ game });
  })

  .delete('/games/:id', (c) => {
    const changes = getDb().prepare('DELETE FROM games WHERE id = ?').run(c.req.param('id')).changes;
    if (changes === 0) return c.json({ error: '条目不存在' }, 404);
    return c.body(null, 204);
  })

  .get('/games/:id/state', (c) => {
    const state = getDb()
      .prepare(`SELECT ${STATE_COLUMNS} FROM user_game_state WHERE game_id = ?`)
      .get(c.req.param('id')) as UserGameState | undefined;
    if (!state) return c.json({ error: '条目不存在' }, 404);
    return c.json({ state });
  })

  .patch('/games/:id/state', async (c) => {
    const parsed = await parseStateBody(c);
    if (!parsed.ok) return c.json({ error: parsed.error }, 400);

    const id = c.req.param('id');
    const update = compileUpdate(parsed.fields);
    if (!update) return c.json({ error: '没有可更新的字段' }, 400);
    update.params.push(id);

    const db = getDb();
    const changes = db
      .prepare(`UPDATE user_game_state SET ${update.sets.join(', ')} WHERE game_id = ?`)
      .run(...update.params).changes;
    if (changes === 0) return c.json({ error: '条目不存在' }, 404);

    const state = db
      .prepare(`SELECT ${STATE_COLUMNS} FROM user_game_state WHERE game_id = ?`)
      .get(id) as UserGameState;
    return c.json({ state });
  });
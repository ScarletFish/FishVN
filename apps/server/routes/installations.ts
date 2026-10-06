import { randomUUID } from 'node:crypto';
import { resolve } from 'node:path';
import { Hono } from 'hono';
import type { Context } from 'hono';
import { getDb } from '../db.js';
import { inspectDir } from '../lib/fsinfo.js';
import { compileUpdate, isUniqueViolation, readObject } from '../lib/utils.js';
import type { Reject } from '../lib/utils.js';
import type { Installation } from '../types.js';

const COLUMNS = 'id, game_id, dir_path, entry_path, exes, size_bytes';

/** 库里 exes 是 JSON 文本，对外给数组 */
type Row = Omit<Installation, 'exes'> & { exes: string | null };

type CreateFields = { game_id: string; dir_path: string };
type PatchFields = { game_id?: string | null; entry_path?: string | null };

function toApi(row: Row): Installation {
  const parsed: unknown = row.exes ? JSON.parse(row.exes) : null;
  return { ...row, exes: Array.isArray(parsed) ? (parsed as string[]) : null };
}

async function parseCreateBody(c: Context): Promise<{ ok: true; fields: CreateFields } | Reject> {
  const read = await readObject(c);
  if (!read.ok) return read;

  if (typeof read.body.game_id !== 'string' || !read.body.game_id) {
    return { ok: false, error: 'game_id 必填' };
  }
  if (typeof read.body.dir_path !== 'string' || !read.body.dir_path.trim()) {
    return { ok: false, error: 'dir_path 必填' };
  }

  return {
    ok: true,
    fields: {
      game_id: read.body.game_id,
      dir_path: read.body.dir_path.trim(),
    },
  };
}

async function parsePatchBody(c: Context): Promise<{ ok: true; fields: PatchFields } | Reject> {
  const read = await readObject(c);
  if (!read.ok) return read;

  const fields: PatchFields = {};

  if (read.body.game_id !== undefined) {
    if (read.body.game_id !== null && typeof read.body.game_id !== 'string') {
      return { ok: false, error: 'game_id 必须是字符串或 null' };
    }
    fields.game_id = read.body.game_id;
  }

  if (read.body.entry_path !== undefined) {
    if (read.body.entry_path !== null && (typeof read.body.entry_path !== 'string' || !read.body.entry_path)) {
      return { ok: false, error: 'entry_path 是非空字符串或 null' };
    }
    fields.entry_path = read.body.entry_path;
  }

  return { ok: true, fields };
}

export const installationRoutes = new Hono()
  .get('/installations', (c) => {
    const db = getDb();
    const gameId = c.req.query('game_id');
    const stmt = gameId
      ? db.prepare(`SELECT ${COLUMNS} FROM installations WHERE game_id = ? ORDER BY dir_path`)
      : db.prepare(`SELECT ${COLUMNS} FROM installations ORDER BY dir_path`);
    const rows = (gameId ? stmt.all(gameId) : stmt.all()) as Row[];
    return c.json({ installations: rows.map(toApi) });
  })

  .post('/installations', async (c) => {
    const parsed = await parseCreateBody(c);
    if (!parsed.ok) return c.json({ error: parsed.error }, 400);

    const db = getDb();
    const game = db.prepare('SELECT id FROM games WHERE id = ?').get(parsed.fields.game_id);
    if (!game) return c.json({ error: '作品不存在' }, 404);

    const dirPath = resolve(parsed.fields.dir_path);
    // 先查重复再遍历目录：拖同一个目录两次不该付出读整个目录的代价。
    // UNIQUE 约束仍是最终保证，这里只是提前失败。
    const dup = db.prepare('SELECT id FROM installations WHERE dir_path = ?').get(dirPath);
    if (dup) return c.json({ error: '该目录已有安装记录' }, 409);

    const inspected = inspectDir(dirPath);
    if ('error' in inspected) return c.json({ error: inspected.error }, 400);

    const row: Row = {
      id: randomUUID(),
      game_id: parsed.fields.game_id,
      dir_path: inspected.dirPath,
      // 候选唯一时没有选择余地，直接填上
      entry_path: inspected.info.exes.length === 1 ? inspected.info.exes[0] : null,
      exes: JSON.stringify(inspected.info.exes),
      size_bytes: inspected.info.sizeBytes,
    };
    try {
      db.prepare(`INSERT INTO installations (${COLUMNS}) VALUES (?, ?, ?, ?, ?, ?)`).run(
        row.id,
        row.game_id,
        row.dir_path,
        row.entry_path,
        row.exes,
        row.size_bytes
      );
    } catch (err) {
      if (isUniqueViolation(err)) return c.json({ error: '该目录已有安装记录' }, 409);
      throw err;
    }
    return c.json({ installation: toApi(row) }, 201);
  })

  .patch('/installations/:id', async (c) => {
    const parsed = await parsePatchBody(c);
    if (!parsed.ok) return c.json({ error: parsed.error }, 400);

    const { game_id } = parsed.fields;
    if (game_id !== undefined && game_id !== null) {
      const game = getDb().prepare('SELECT id FROM games WHERE id = ?').get(game_id);
      if (!game) return c.json({ error: '作品不存在' }, 404);
    }

    const id = c.req.param('id');
    const update = compileUpdate(parsed.fields);
    if (!update) return c.json({ error: '没有可更新的字段' }, 400);
    update.params.push(id);

    const db = getDb();
    const changes = db
      .prepare(`UPDATE installations SET ${update.sets.join(', ')} WHERE id = ?`)
      .run(...update.params).changes;
    if (changes === 0) return c.json({ error: '安装记录不存在' }, 404);

    const row = db.prepare(`SELECT ${COLUMNS} FROM installations WHERE id = ?`).get(id) as Row;
    return c.json({ installation: toApi(row) });
  })

  .delete('/installations/:id', (c) => {
    const changes = getDb().prepare('DELETE FROM installations WHERE id = ?').run(c.req.param('id')).changes;
    if (changes === 0) return c.json({ error: '安装记录不存在' }, 404);
    return c.body(null, 204);
  });
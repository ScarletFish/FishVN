import type { Context } from 'hono';

export type Reject = { ok: false; error: string };

/** 读 body 并确认是 JSON 对象 */
export async function readObject(c: Context): Promise<{ ok: true; body: Record<string, unknown> } | Reject> {
  let raw: unknown;
  try {
    raw = await c.req.json();
  } catch {
    return { ok: false, error: 'body 不是合法 JSON' };
  }
  if (typeof raw !== 'object' || raw === null) return { ok: false, error: 'body 必须是 JSON 对象' };
  return { ok: true, body: raw as Record<string, unknown> };
}

export type Update = { sets: string[]; params: unknown[] };

/** 键名来自调用处的字面量，不是用户输入 */
export function compileUpdate(fields: Record<string, unknown>): Update | null {
  const sets: string[] = [];
  const params: unknown[] = [];
  for (const [column, value] of Object.entries(fields)) {
    if (value === undefined) continue;
    sets.push(`${column} = ?`);
    params.push(value);
  }
  return sets.length > 0 ? { sets, params } : null;
}

/** UNIQUE 冲突回 409 */
export function isUniqueViolation(err: unknown): boolean {
  return err instanceof Error && (err as { code?: string }).code === 'SQLITE_CONSTRAINT_UNIQUE';
}
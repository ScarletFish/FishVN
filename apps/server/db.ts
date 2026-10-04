import Database from 'better-sqlite3';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { DB_FILE } from './constants.js';
import { DATA_DIR } from './lib/paths.js';

export type Db = Database.Database;

let instance: Db | null = null;

/** 幂等：表结构演进都写在这里 */
export function ensureSchema(db: Db): void {
  db.pragma('journal_mode = WAL');
  // 首个表出现时在此生长
}

/** 惰性打开库：进程内单例 */
export function getDb(): Db {
  if (instance) return instance;
  mkdirSync(DATA_DIR, { recursive: true });
  instance = new Database(join(DATA_DIR, DB_FILE));
  ensureSchema(instance);
  return instance;
}
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
  // 建表大量依赖 ON DELETE CASCADE，SQLite 自身默认关闭外键，显式打开
  db.pragma('foreign_keys = ON');
  db.exec(`
    -- 作品身份：显示名 + VNDB 绑定。路径、元数据、用户状态各在其表。
    -- vndb_id 可空且 UNIQUE：NULL 之间不算重复，一部 VN 只对应一条作品。
    CREATE TABLE IF NOT EXISTS games (
      id         TEXT PRIMARY KEY,
      name       TEXT NOT NULL,
      vndb_id    TEXT UNIQUE,
      created_at INTEGER NOT NULL
    );

    -- 用户状态：每部作品一行，随作品一同建立、随作品删除而清除。
    CREATE TABLE IF NOT EXISTS user_game_state (
      game_id        TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
      status         TEXT NOT NULL DEFAULT 'wishlist',
      user_rating    REAL,
      notes          TEXT,
      last_played_at INTEGER,
      play_count     INTEGER NOT NULL DEFAULT 0
    );
  `);
}

/** 惰性打开库：进程内单例 */
export function getDb(): Db {
  if (instance) return instance;
  mkdirSync(DATA_DIR, { recursive: true });
  instance = new Database(join(DATA_DIR, DB_FILE));
  ensureSchema(instance);
  return instance;
}

/** 关闭并置空，下次 getDb() 重新打开 */
export function closeDb(): void {
  instance?.close();
  instance = null;
}
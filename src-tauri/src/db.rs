use std::fs;
use std::path::Path;

use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;

use crate::error::{AppError, Result};

pub const DB_FILE: &str = "fishvn.db";

pub fn open(dir: &Path) -> Result<Connection> {
    fs::create_dir_all(dir)?;
    let conn = Connection::open(dir.join(DB_FILE))?;
    ensure_schema(&conn)?;
    Ok(conn)
}

pub fn lock<'a>(shared: &'a Arc<Mutex<Connection>>) -> Result<MutexGuard<'a, Connection>> {
    shared
        .lock()
        .map_err(|_| AppError::Io(std::io::Error::other("数据库锁已毒化，之前的命令 panic 了")))
}

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS games (
          id         TEXT PRIMARY KEY,
          name       TEXT NOT NULL,
          vndb_id    TEXT UNIQUE,
          created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS installations (
          id         TEXT PRIMARY KEY,
          game_id    TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
          dir_path   TEXT NOT NULL,
          entry_path TEXT,
          exes       TEXT,
          size_bytes INTEGER,
          sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE UNIQUE INDEX IF NOT EXISTS installations_path_key
          ON installations(dir_path, IFNULL(entry_path, ''));

        CREATE TABLE IF NOT EXISTS user_game_state (
          game_id                 TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
          status                  TEXT NOT NULL DEFAULT 'wishlist',
          user_rating             REAL,
          notes                   TEXT,
          default_installation_id TEXT REFERENCES installations(id) ON DELETE SET NULL,
          play_seconds            INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS play_sessions (
          id                 TEXT PRIMARY KEY,
          game_id            TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
          installation_id    TEXT REFERENCES installations(id) ON DELETE SET NULL,
          started_at         INTEGER NOT NULL,
          last_seen_at       INTEGER NOT NULL,
          ended_at           INTEGER,
          foreground_seconds INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS play_sessions_game ON play_sessions(game_id);

        CREATE TABLE IF NOT EXISTS game_metadata (
          game_id     TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
          alias       TEXT,
          titles      TEXT,
          rating      REAL,
          votecount   INTEGER,
          length_minutes INTEGER,
          released    TEXT,
          description TEXT,
          image_id    TEXT,
          image_path  TEXT,
          tags        TEXT,
          developers  TEXT,
          staff       TEXT
        );

        CREATE TABLE IF NOT EXISTS characters (
          game_id     TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
          id          TEXT NOT NULL,
          name        TEXT,
          original    TEXT,
          aliases     TEXT,
          description TEXT,
          gender      TEXT,
          age         INTEGER,
          birthday    TEXT,
          blood_type  TEXT,
          height      INTEGER,
          weight      INTEGER,
          bust        INTEGER,
          waist       INTEGER,
          hips        INTEGER,
          cup         TEXT,
          role        TEXT,
          image_id    TEXT,
          image_path  TEXT,
          sort_order  INTEGER NOT NULL DEFAULT 0,
          traits      TEXT,
          PRIMARY KEY (game_id, id)
        );

        CREATE TABLE IF NOT EXISTS staffs (
          id       TEXT PRIMARY KEY,
          name     TEXT,
          original TEXT
        );

        CREATE TABLE IF NOT EXISTS character_va (
          game_id      TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
          character_id TEXT NOT NULL,
          staff_id     TEXT NOT NULL REFERENCES staffs(id) ON DELETE CASCADE,
          note         TEXT NOT NULL DEFAULT '',
          PRIMARY KEY (game_id, character_id, staff_id, note),
          FOREIGN KEY (game_id, character_id) REFERENCES characters(game_id, id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS character_va_staff ON character_va(staff_id);

        CREATE TRIGGER IF NOT EXISTS character_va_prune_staff
        AFTER DELETE ON character_va
        BEGIN
          DELETE FROM staffs WHERE id = OLD.staff_id
            AND NOT EXISTS (
              SELECT 1 FROM character_va WHERE staff_id = OLD.staff_id
            );
        END;
        ",
    )?;

    Ok(())
}

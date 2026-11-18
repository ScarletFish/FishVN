use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use crate::error::{AppError, Result};

pub const DB_FILE: &str = "fishvn.db";

pub fn open(dir: &Path) -> Result<Connection> {
    fs::create_dir_all(dir)?;
    let existing = dir.join(DB_FILE).exists();
    let mut conn = Connection::open(dir.join(DB_FILE))?;
    prepare(&mut conn, dir, existing)?;
    Ok(conn)
}

pub fn prepare(conn: &mut Connection, dir: &Path, existing: bool) -> Result<()> {
    ensure_schema(conn)?;
    let pending = if existing {
        crate::migrations::MIGRATIONS.pending_migrations(conn)?
    } else {
        0
    };
    if pending != 0 {
        backup_before_migration(conn, dir)?;
    }
    ensure_columns(conn)?;
    if pending != 0 {
        crate::migrations::MIGRATIONS.to_latest(conn)?;
        ensure_columns(conn)?;
    } else if !existing {
        conn.pragma_update(
            None,
            "user_version",
            crate::migrations::LATEST_VERSION as i32,
        )?;
    }
    Ok(())
}

fn backup_before_migration(conn: &Connection, dir: &Path) -> Result<()> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let id = format!("{:010}{:09}", duration.as_secs(), duration.subsec_nanos());
    let dest = dir.join(crate::backup::BACKUP_DIR).join(id);
    fs::create_dir_all(&dest)?;
    let target = dest.join(DB_FILE);
    let escaped = target.to_string_lossy().replace('\'', "''");
    conn.execute_batch(&format!("VACUUM INTO '{escaped}'"))?;
    Ok(())
}

pub fn ensure_columns(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    add_missing_columns(
        &tx,
        "game_metadata",
        crate::generated::metadata::METADATA_ADD_COLUMNS,
    )?;
    add_missing_columns(
        &tx,
        "characters",
        crate::generated::characters::CHARACTER_ADD_COLUMNS,
    )?;
    tx.commit()?;
    Ok(())
}

fn add_missing_columns(conn: &Connection, table: &str, columns: &[(&str, &str)]) -> Result<()> {
    let names = {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let found = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<HashSet<String>>>()?;
        found
    };
    for (name, definition) in columns {
        if names.contains(*name) {
            continue;
        }
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {name} {definition}"),
            [],
        )?;
    }
    Ok(())
}

pub fn lock<'a>(shared: &'a Arc<Mutex<Connection>>) -> Result<MutexGuard<'a, Connection>> {
    shared
        .lock()
        .map_err(|_| AppError::Io(std::io::Error::other("数据库锁已毒化，之前的命令 panic 了")))
}

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(&format!(
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
          sort_order INTEGER NOT NULL DEFAULT 0,
          le_launch   INTEGER NOT NULL DEFAULT 0,
          magpie      INTEGER NOT NULL DEFAULT 0,  -- 0 = 关，1 = 全屏缩放，2 = 窗口化缩放
          launch_args TEXT
        );

        CREATE UNIQUE INDEX IF NOT EXISTS installations_path_key
          ON installations(dir_path, IFNULL(entry_path, ''));

        CREATE TABLE IF NOT EXISTS user_game_state (
          game_id                 TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
          status                  TEXT NOT NULL DEFAULT 'wishlist',
          user_rating             REAL,
          notes                   TEXT,
          default_installation_id TEXT REFERENCES installations(id) ON DELETE SET NULL,
          play_seconds            INTEGER NOT NULL DEFAULT 0,
          synced_status           TEXT,
          synced_rating           REAL,
          state_modified_at       INTEGER
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
{}
        );

        CREATE TABLE IF NOT EXISTS tags (
          id     TEXT PRIMARY KEY,
          name   TEXT NOT NULL,
          source TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS game_tag (
          game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
          tag_id  TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
          rating  REAL,
          spoiler INTEGER,
          lie     INTEGER,
          PRIMARY KEY (game_id, tag_id)
        );

        CREATE INDEX IF NOT EXISTS game_tag_tag ON game_tag(tag_id);

        CREATE TABLE IF NOT EXISTS traits (
          id     TEXT PRIMARY KEY,
          name   TEXT NOT NULL,
          grp    TEXT,
          source TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS characters (
{}
        );

        CREATE TABLE IF NOT EXISTS staffs (
          id       TEXT PRIMARY KEY,
          name     TEXT,
          original TEXT
        );

        CREATE TABLE IF NOT EXISTS character_va (
          game_id       TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
          character_id  TEXT NOT NULL,
          staff_id      TEXT NOT NULL REFERENCES staffs(id) ON DELETE CASCADE,
          note          TEXT NOT NULL DEFAULT '',
          staff_name    TEXT,
          staff_original TEXT,
          PRIMARY KEY (game_id, character_id, staff_id, note),
          FOREIGN KEY (game_id, character_id) REFERENCES characters(game_id, id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS character_va_staff ON character_va(staff_id);

        CREATE TABLE IF NOT EXISTS character_trait (
          game_id      TEXT NOT NULL,
          character_id TEXT NOT NULL,
          trait_id     TEXT NOT NULL REFERENCES traits(id) ON DELETE CASCADE,
          spoiler      INTEGER,
          lie          INTEGER,
          PRIMARY KEY (game_id, character_id, trait_id),
          FOREIGN KEY (game_id, character_id) REFERENCES characters(game_id, id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS character_trait_trait ON character_trait(trait_id);

        CREATE TRIGGER IF NOT EXISTS character_va_prune_staff
        AFTER DELETE ON character_va
        BEGIN
          DELETE FROM staffs WHERE id = OLD.staff_id
            AND NOT EXISTS (
              SELECT 1 FROM character_va WHERE staff_id = OLD.staff_id
            );
        END;

        CREATE TABLE IF NOT EXISTS backfill_run (
          id         INTEGER PRIMARY KEY AUTOINCREMENT,
          started_at INTEGER NOT NULL,
          ended_at   INTEGER NOT NULL,
          summary    TEXT NOT NULL,
          events     TEXT NOT NULL
        );
        ",
        crate::generated::metadata::DDL_COLUMNS,
        crate::generated::characters::CHARACTER_DDL_COLUMNS,
    ))?;

    Ok(())
}

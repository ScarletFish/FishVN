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
          cover_focal_x INTEGER NOT NULL DEFAULT 50,
          cover_focal_y INTEGER NOT NULL DEFAULT 50,
          ambience_path TEXT,
          ambience_mode TEXT NOT NULL DEFAULT 'soft',
          ambience_plate INTEGER NOT NULL DEFAULT 0,
          ambience_focal_x INTEGER NOT NULL DEFAULT 50,
          ambience_focal_y INTEGER NOT NULL DEFAULT 22,
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

    dedupe_metadata_tags(conn)?;

    Ok(())
}

fn dedupe_metadata_tags(conn: &Connection) -> Result<()> {
    let rows: Vec<(String, String)> = {
        let mut stmt =
            conn.prepare("SELECT game_id, tags FROM game_metadata WHERE tags IS NOT NULL")?;
        let mapped = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        mapped.collect::<std::result::Result<Vec<_>, _>>()?
    };
    for (game_id, json) in rows {
        let Ok(tags) = serde_json::from_str::<Vec<crate::types::MetadataTag>>(&json) else {
            continue;
        };
        let before = tags.len();
        let mut kept: Vec<crate::types::MetadataTag> = Vec::with_capacity(before);
        let mut seen: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut changed = false;
        for tag in tags {
            match seen.get(&tag.name) {
                Some(&idx) => {
                    if kept[idx].id.is_none() && tag.id.is_some() {
                        kept[idx] = tag;
                        changed = true;
                    }
                }
                None => {
                    seen.insert(tag.name.clone(), kept.len());
                    kept.push(tag);
                }
            }
        }
        if !changed && kept.len() == before {
            continue;
        }
        conn.execute(
            "UPDATE game_metadata SET tags = ?1 WHERE game_id = ?2",
            rusqlite::params![serde_json::to_string(&kept)?, game_id],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MetadataTag;

    fn tag(id: Option<&str>, name: &str) -> MetadataTag {
        MetadataTag {
            id: id.map(String::from),
            name: String::from(name),
            ..Default::default()
        }
    }

    fn meta_with_tags(conn: &Connection, game_id: &str, tags: &[MetadataTag]) {
        conn.execute(
            "INSERT INTO games (id, name, created_at) VALUES (?1, ?2, 0)",
            rusqlite::params![game_id, game_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO game_metadata (game_id, tags) VALUES (?1, ?2)",
            rusqlite::params![game_id, serde_json::to_string(tags).unwrap()],
        )
        .unwrap();
    }

    fn tags_of(conn: &Connection, game_id: &str) -> Vec<MetadataTag> {
        let json: String = conn
            .query_row(
                "SELECT tags FROM game_metadata WHERE game_id = ?1",
                [game_id],
                |row| row.get(0),
            )
            .unwrap();
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn 升级建表时标签去重规则() {
        for (label, tags, expected_len, expected_id) in [
            (
                "同名时优先保留带 id 的条目",
                vec![tag(None, "百合"), tag(Some("g1890"), "百合")],
                1,
                Some("g1890"),
            ),
            (
                "都没 id 时只留第一条",
                vec![tag(None, "自定义"), tag(None, "自定义")],
                1,
                None,
            ),
            (
                "不同标签不动",
                vec![tag(Some("g1a"), "A"), tag(None, "B")],
                2,
                None,
            ),
        ] {
            let conn = Connection::open_in_memory().unwrap();
            ensure_schema(&conn).unwrap();
            meta_with_tags(&conn, "g1", &tags);
            ensure_schema(&conn).unwrap();
            let got = tags_of(&conn, "g1");
            assert_eq!(got.len(), expected_len, "{label}");
            if let Some(id) = expected_id {
                assert_eq!(got[0].id.as_deref(), Some(id), "{label}");
            }
        }
    }
}

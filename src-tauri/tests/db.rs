use std::collections::HashSet;

use rusqlite::Connection;
use tempfile::tempdir;

use fishvn::db;

fn column_names(conn: &Connection, table: &str) -> HashSet<String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<rusqlite::Result<HashSet<_>>>()
        .unwrap()
}

#[test]
fn 建表跑两遍仍保住原有作品() {
    let dir = tempdir().unwrap();
    let conn = Connection::open(dir.path().join(db::DB_FILE)).unwrap();
    conn.execute_batch(
        "CREATE TABLE games (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            vndb_id TEXT UNIQUE,
            created_at INTEGER NOT NULL
         );
         INSERT INTO games (id, name, created_at) VALUES ('g1', '已存在', 1);",
    )
    .unwrap();

    db::ensure_schema(&conn).expect("第二遍 ensure_schema 不该报错");
    db::ensure_columns(&conn).expect("补列不该报错");

    let name: String = conn
        .query_row("SELECT name FROM games WHERE id = 'g1'", [], |row| row.get(0))
        .unwrap();
    assert_eq!(name, "已存在", "重复建表后作品身份保持");
}

#[test]
fn 新库跳过历史迁移并落当前版本() {
    let dir = tempdir().unwrap();
    let conn = db::open(dir.path()).expect("新库开不了");
    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 3);
    assert!(!dir.path().join("backups").exists(), "新库不该备份");
}

#[test]
fn 旧库缺列时补齐列并保住数据() {
    let dir = tempdir().unwrap();
    let mut conn = Connection::open(dir.path().join(db::DB_FILE)).unwrap();
    conn.execute_batch(
        "CREATE TABLE games (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            vndb_id TEXT UNIQUE,
            created_at INTEGER NOT NULL
         );
         CREATE TABLE game_metadata (
            game_id TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
            image_path TEXT,
            ambience_path TEXT,
            ambience_mode TEXT NOT NULL DEFAULT 'soft',
            ambience_focal_x INTEGER NOT NULL DEFAULT 50,
            ambience_focal_y INTEGER NOT NULL DEFAULT 22
         );
         CREATE TABLE characters (
            game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            id TEXT NOT NULL,
            PRIMARY KEY (game_id, id)
         );
         INSERT INTO games (id, name, created_at) VALUES ('g1', '旧作品', 1);
         INSERT INTO game_metadata (game_id, image_path, ambience_mode, ambience_focal_x, ambience_focal_y)
            VALUES ('g1', 'old.jpg', 'clear', 80, 10);
         INSERT INTO characters (game_id, id) VALUES ('g1', 'c1');",
    )
    .unwrap();

    db::prepare(&mut conn, dir.path(), true).expect("旧库升级不该失败");

    let meta_cols = column_names(&conn, "game_metadata");
    assert!(meta_cols.contains("showcase_focal_x"));
    assert!(meta_cols.contains("showcase_focal_y"));
    assert!(meta_cols.contains("ambience_plate"));
    assert!(column_names(&conn, "characters").contains("age"));

    let mode: String = conn
        .query_row(
            "SELECT ambience_mode FROM game_metadata WHERE game_id = 'g1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(mode, "clear");
    let focal: (i64, i64) = conn
        .query_row(
            "SELECT showcase_focal_x, showcase_focal_y FROM game_metadata WHERE game_id = 'g1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(focal, (80, 10));
    let showcase_mode: String = conn
        .query_row(
            "SELECT showcase_mode FROM game_metadata WHERE game_id = 'g1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(showcase_mode, "clear");

    let name: String = conn
        .query_row("SELECT name FROM games WHERE id = 'g1'", [], |row| row.get(0))
        .unwrap();
    assert_eq!(name, "旧作品");

    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 3);

    let backups = std::fs::read_dir(dir.path().join("backups")).unwrap();
    assert!(backups.count() > 0, "迁移前应有备份");
}

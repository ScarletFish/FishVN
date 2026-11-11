use fishvn::db;
use fishvn::generated::characters::{
    field, insert_values, CharacterFieldType, CHARACTER_COLUMNS, CHARACTER_DDL_COLUMNS,
    CHARACTER_FIELDS, CHARACTER_MERGE_KEYS,
};
use fishvn::types::GameCharacter;
use rusqlite::types::Value;
use rusqlite::Connection;
use serde_json::json;

#[test]
fn ddl列定义与规格逐行对应() {
    let lines = CHARACTER_DDL_COLUMNS.lines().collect::<Vec<_>>();
    let declared = CHARACTER_FIELDS
        .iter()
        .filter(|f| f.in_ddl)
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), declared.len() + 1);
    for (line, field) in lines.iter().zip(declared.iter()) {
        assert!(line.trim_start().starts_with(field.sql), "{line}");
    }
    assert_eq!(lines.last().unwrap().trim(), "PRIMARY KEY (game_id, id)");
}

const FROZEN_CHARACTERS_DDL: &str = r#"          game_id     TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
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
          PRIMARY KEY (game_id, id)"#;

#[test]
fn 生成ddl与冻结的建表定义逐字一致() {
    assert_eq!(CHARACTER_DDL_COLUMNS, FROZEN_CHARACTERS_DDL);
}

#[test]
fn 按key查得到字段规格() {
    assert_eq!(field("age").unwrap().ty, CharacterFieldType::Integer);
    assert!(field("traits").unwrap().joined);
    assert!(field("不存在").is_none());
}

fn full_character() -> GameCharacter {
    serde_json::from_value(json!({
        "id": "c17",
        "name": "Saber",
        "original": "セイバー",
        "aliases": ["阿尔托莉雅", "呆毛王"],
        "description": "描述",
        "gender": ["f"],
        "age": 17,
        "birthday": "12-15",
        "blood_type": "a",
        "height": 154,
        "weight": 42,
        "bust": 73,
        "waist": 53,
        "hips": 76,
        "cup": "B",
        "role": "main",
        "image_id": "ch17",
        "image_path": "data/ch17.jpg",
        "sort_order": 9,
        "traits": [{ "id": "i1", "name": "Long Hair", "group_name": "Hair" }],
        "va": [{ "staff_id": "s1", "name": "Kawasumi Ayako", "note": "" }]
    }))
    .unwrap()
}

#[test]
fn 插入值按列顺序产出且json列序列化() {
    let values = insert_values("g1", &full_character(), 3).unwrap();
    let columns = CHARACTER_COLUMNS.split(", ").collect::<Vec<_>>();
    assert_eq!(values.len(), columns.len() + 1);

    assert_eq!(values[0], Value::Text("g1".into()));
    assert_eq!(values[1], Value::Text("c17".into()));
    assert_eq!(values[2], Value::Text("Saber".into()));
    assert_eq!(values[4], Value::Text("[\"阿尔托莉雅\",\"呆毛王\"]".into()));
    assert_eq!(values[6], Value::Text("[\"f\"]".into()));
    assert_eq!(values[7], Value::Integer(17));
    assert_eq!(values[values.len() - 1], Value::Integer(3));
}

#[test]
fn 空值字段插入null而不是空串() {
    let values = insert_values("g1", &GameCharacter::default(), 0).unwrap();
    assert_eq!(values[0], Value::Text("g1".into()));
    assert_eq!(values[2], Value::Null);
    assert_eq!(values[5], Value::Null);
    assert_eq!(values[8], Value::Null);
}

#[test]
fn 实际建表的角色表列与规格一致() {
    let conn = Connection::open_in_memory().unwrap();
    db::ensure_schema(&conn).unwrap();
    let mut stmt = conn.prepare("PRAGMA table_info(characters)").unwrap();
    let names = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();

    let expected = CHARACTER_FIELDS
        .iter()
        .filter(|f| f.in_ddl)
        .map(|f| f.sql.to_owned())
        .collect::<Vec<_>>();
    assert_eq!(names, expected);
}

#[test]
fn 键集合无重复且身份字段不进合并() {
    let mut all = CHARACTER_MERGE_KEYS.to_vec();
    all.sort_unstable();
    let mut deduped = all.clone();
    deduped.dedup();
    assert_eq!(all, deduped);
    for key in ["game_id", "id", "sort_order"] {
        assert!(!CHARACTER_MERGE_KEYS.contains(&key), "{key}");
    }
}

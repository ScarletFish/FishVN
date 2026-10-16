//! 角色与声优：一部作品一套角色，按作品整包替换。
//!
//! 与 metadata 那张表的差别是这里是集合：同一部作品有多行，一次保存要能
//! 同时处理新增、修改与删除，所以写路径不做逐字段 patch。

use std::collections::{HashMap, HashSet};

use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{AppError, Result};
use crate::types::{CharacterTrait, CharacterVa, GameCharacter};

const CHARACTER_COLUMNS: &str = "id, name, original, aliases, description, gender, age, birthday, \
     blood_type, height, weight, bust, waist, hips, cup, role, image_id, image_path, sort_order, traits";

struct CharacterRow {
    id: String,
    name: Option<String>,
    original: Option<String>,
    aliases: Option<String>,
    description: Option<String>,
    gender: Option<String>,
    age: Option<i64>,
    birthday: Option<String>,
    blood_type: Option<String>,
    height: Option<i64>,
    weight: Option<i64>,
    bust: Option<i64>,
    waist: Option<i64>,
    hips: Option<i64>,
    cup: Option<String>,
    role: Option<String>,
    image_id: Option<String>,
    image_path: Option<String>,
    sort_order: i64,
    traits: Option<String>,
}

pub fn list(conn: &Connection, game_id: &str) -> Result<Vec<GameCharacter>> {
    if !game_exists(conn, game_id)? {
        return Err(AppError::NotFound("作品不存在".into()));
    }
    let mut stmt = conn.prepare(&format!(
        "SELECT {CHARACTER_COLUMNS} FROM characters WHERE game_id = ? ORDER BY sort_order, id"
    ))?;
    let rows = stmt
        .query_map(params![game_id], read_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.iter().map(|row| to_api(conn, game_id, row)).collect()
}

/// 整包替换：**先删该作品的全部角色行与配音关系，再按数组顺序写回。**
///
/// 排序不看入参里的 sort_order，取数组下标：客户端重排时逐个重编号既容易撞值，
/// 也让顺序有两个真相来源。
pub fn replace_all(
    conn: &Connection,
    game_id: &str,
    characters: &[GameCharacter],
) -> Result<Vec<GameCharacter>> {
    if !game_exists(conn, game_id)? {
        return Err(AppError::NotFound("作品不存在".into()));
    }
    let mut seen = HashSet::with_capacity(characters.len());
    for entry in characters {
        if entry.id.trim().is_empty() {
            return Err(AppError::BadRequest("角色缺少 id".into()));
        }
        if !seen.insert(entry.id.as_str()) {
            return Err(AppError::BadRequest("同一部作品里出现重复的角色 id".into()));
        }
        for credit in entry.va.iter().flatten() {
            if credit.staff_id.trim().is_empty() {
                return Err(AppError::BadRequest("配音记录缺少声优 id".into()));
            }
        }
    }

    let tx = conn.unchecked_transaction()?;
    let kept_staff = staff_snapshot(&tx, game_id)?;
    conn.execute("DELETE FROM characters WHERE game_id = ?", params![game_id])?;
    for (index, entry) in characters.iter().enumerate() {
        insert_character(&tx, game_id, entry, index as i64, &kept_staff)?;
    }
    tx.commit()?;

    list(conn, game_id)
}

fn staff_snapshot(conn: &Connection, game_id: &str) -> Result<HashMap<String, StaffName>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT s.id, s.name, s.original
         FROM character_va v JOIN staffs s ON s.id = v.staff_id
         WHERE v.game_id = ?",
    )?;
    let rows = stmt
        .query_map(params![game_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                StaffName {
                    name: row.get::<_, Option<String>>(1)?,
                    original: row.get::<_, Option<String>>(2)?,
                },
            ))
        })?
        .collect::<rusqlite::Result<HashMap<_, _>>>()?;
    Ok(rows)
}

#[derive(Clone)]
struct StaffName {
    name: Option<String>,
    original: Option<String>,
}

impl StaffName {
    fn keep(
        &self,
        incoming_name: &Option<String>,
        incoming_original: &Option<String>,
    ) -> (String, String) {
        let pick = |kept: &Option<String>, incoming: &Option<String>| {
            let kept = kept.as_deref().unwrap_or("").trim();
            if kept.is_empty() {
                incoming.as_deref().unwrap_or("").to_owned()
            } else {
                kept.to_owned()
            }
        };
        (
            pick(&self.name, incoming_name),
            pick(&self.original, incoming_original),
        )
    }
}

fn insert_character(
    conn: &Connection,
    game_id: &str,
    entry: &GameCharacter,
    sort_order: i64,
    kept_staff: &HashMap<String, StaffName>,
) -> Result<()> {
    let values: Vec<Value> = vec![
        Value::Text(game_id.to_owned()),
        Value::Text(entry.id.clone()),
        text(&entry.name),
        text(&entry.original),
        json(&entry.aliases)?,
        text(&entry.description),
        json(&entry.gender)?,
        entry.age.map(Value::Integer).unwrap_or(Value::Null),
        text(&entry.birthday),
        text(&entry.blood_type),
        entry.height.map(Value::Integer).unwrap_or(Value::Null),
        entry.weight.map(Value::Integer).unwrap_or(Value::Null),
        entry.bust.map(Value::Integer).unwrap_or(Value::Null),
        entry.waist.map(Value::Integer).unwrap_or(Value::Null),
        entry.hips.map(Value::Integer).unwrap_or(Value::Null),
        text(&entry.cup),
        text(&entry.role),
        text(&entry.image_id),
        text(&entry.image_path),
        Value::Integer(sort_order),
        json(&entry.traits)?,
    ];
    let placeholders = vec!["?"; values.len()].join(", ");
    conn.execute(
        &format!("INSERT INTO characters (game_id, {CHARACTER_COLUMNS}) VALUES ({placeholders})"),
        params_from_iter(values.iter()),
    )?;

    if let Some(va) = &entry.va {
        for credit in va {
            insert_va(conn, game_id, &entry.id, credit, kept_staff)?;
        }
    }
    Ok(())
}

fn insert_va(
    conn: &Connection,
    game_id: &str,
    character_id: &str,
    entry: &CharacterVa,
    kept_staff: &HashMap<String, StaffName>,
) -> Result<()> {
    let (name, original) = kept_staff
        .get(&entry.staff_id)
        .map(|kept| kept.keep(&entry.name, &entry.original))
        .unwrap_or_else(|| {
            (
                entry.name.as_deref().unwrap_or("").to_owned(),
                entry.original.as_deref().unwrap_or("").to_owned(),
            )
        });
    conn.execute(
        "INSERT INTO staffs (id, name, original) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET
           name     = COALESCE(NULLIF(staffs.name, ''), excluded.name),
           original = COALESCE(NULLIF(staffs.original, ''), excluded.original)",
        params![entry.staff_id, name, original],
    )?;
    conn.execute(
        "INSERT INTO character_va (game_id, character_id, staff_id, note)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(game_id, character_id, staff_id, note) DO NOTHING",
        params![
            game_id,
            character_id,
            entry.staff_id,
            entry.note.as_deref().unwrap_or_default()
        ],
    )?;
    Ok(())
}

fn game_exists(conn: &Connection, id: &str) -> Result<bool> {
    let found = conn
        .query_row("SELECT id FROM games WHERE id = ?", params![id], |row| {
            row.get::<_, String>(0)
        })
        .optional()?;
    Ok(found.is_some())
}

fn read_row(row: &Row<'_>) -> rusqlite::Result<CharacterRow> {
    Ok(CharacterRow {
        id: row.get("id")?,
        name: row.get("name")?,
        original: row.get("original")?,
        aliases: row.get("aliases")?,
        description: row.get("description")?,
        gender: row.get("gender")?,
        age: row.get("age")?,
        birthday: row.get("birthday")?,
        blood_type: row.get("blood_type")?,
        height: row.get("height")?,
        weight: row.get("weight")?,
        bust: row.get("bust")?,
        waist: row.get("waist")?,
        hips: row.get("hips")?,
        cup: row.get("cup")?,
        role: row.get("role")?,
        image_id: row.get("image_id")?,
        image_path: row.get("image_path")?,
        sort_order: row.get("sort_order")?,
        traits: row.get("traits")?,
    })
}

fn to_api(conn: &Connection, game_id: &str, row: &CharacterRow) -> Result<GameCharacter> {
    Ok(GameCharacter {
        id: row.id.clone(),
        name: row.name.clone(),
        original: row.original.clone(),
        aliases: from_json(&row.aliases)?,
        description: row.description.clone(),
        gender: from_json(&row.gender)?,
        age: row.age,
        birthday: row.birthday.clone(),
        blood_type: row.blood_type.clone(),
        height: row.height,
        weight: row.weight,
        bust: row.bust,
        waist: row.waist,
        hips: row.hips,
        cup: row.cup.clone(),
        role: row.role.clone(),
        image_id: row.image_id.clone(),
        image_path: row.image_path.clone(),
        sort_order: row.sort_order,
        traits: from_json::<Vec<CharacterTrait>>(&row.traits)?,
        va: load_va(conn, game_id, row.id.as_str())?,
    })
}

fn load_va(
    conn: &Connection,
    game_id: &str,
    character_id: &str,
) -> Result<Option<Vec<CharacterVa>>> {
    let mut stmt = conn.prepare(
        "SELECT v.staff_id, s.name, s.original, v.note
         FROM character_va v
         JOIN staffs s ON s.id = v.staff_id
         WHERE v.game_id = ? AND v.character_id = ?
         ORDER BY v.note, v.staff_id",
    )?;
    let rows = stmt
        .query_map(params![game_id, character_id], |row| {
            Ok(CharacterVa {
                staff_id: row.get("staff_id")?,
                name: row.get("name")?,
                original: row.get("original")?,
                note: row.get("note")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(Some(rows))
}

fn text(value: &Option<String>) -> Value {
    match value {
        Some(v) => Value::Text(v.clone()),
        None => Value::Null,
    }
}

fn json<T: Serialize>(value: &Option<T>) -> Result<Value> {
    match value {
        Some(v) => Ok(Value::Text(serde_json::to_string(v)?)),
        None => Ok(Value::Null),
    }
}

fn from_json<T: DeserializeOwned>(raw: &Option<String>) -> Result<Option<T>> {
    match raw {
        Some(text) => Ok(Some(serde_json::from_str(text)?)),
        None => Ok(None),
    }
}

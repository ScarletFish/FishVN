use std::collections::{HashMap, HashSet};

use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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
        return Err(AppError::NotFound("游戏不存在".into()));
    }
    let mut stmt = conn.prepare(&format!(
        "SELECT {CHARACTER_COLUMNS} FROM characters WHERE game_id = ? ORDER BY sort_order, id"
    ))?;
    let rows = stmt
        .query_map(params![game_id], read_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.iter().map(|row| to_api(conn, game_id, row)).collect()
}

const STAFF_OPTION_SQL: &str = "WITH freq AS (
   SELECT staff_id, staff_name, staff_original,
          ROW_NUMBER() OVER (
            PARTITION BY staff_id
            ORDER BY COUNT(*) DESC, MAX(game_id) DESC
          ) AS rn
   FROM character_va
   WHERE COALESCE(TRIM(staff_name), '') != ''
   GROUP BY staff_id, staff_name, staff_original
 )
 SELECT s.id, s.name, s.original,
        COALESCE(NULLIF(s.name, ''), f.staff_name),
        COALESCE(NULLIF(s.original, ''), f.staff_original),
        COUNT(cv.character_id) AS roles
 FROM staffs s
 LEFT JOIN freq f ON f.staff_id = s.id AND f.rn = 1
 LEFT JOIN character_va cv ON cv.staff_id = s.id
 {FILTER}
 GROUP BY s.id
 ORDER BY 4, s.id";

pub fn list_all_staff(conn: &Connection) -> Result<Vec<StaffOption>> {
    let sql = STAFF_OPTION_SQL.replace("{FILTER}", "");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([], read_staff_option)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn read_staff_option(row: &Row<'_>) -> rusqlite::Result<StaffOption> {
    Ok(StaffOption {
        id: row.get(0)?,
        name: row.get(1)?,
        original: row.get(2)?,
        display_name: row.get(3)?,
        display_original: row.get(4)?,
        roles: row.get(5)?,
    })
}

fn staff_option(conn: &Connection, id: &str) -> Result<StaffOption> {
    let sql = STAFF_OPTION_SQL.replace("{FILTER}", "WHERE s.id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    stmt.query_row(params![id], read_staff_option)
        .optional()?
        .ok_or_else(|| AppError::NotFound("声优不存在".into()))
}

#[derive(Debug, Deserialize)]
pub struct StaffUpdate {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub original: String,
}

#[derive(Debug, Deserialize)]
pub struct StaffMerge {
    pub target_id: String,
    pub source_id: String,
}

pub fn update_staff(conn: &Connection, patch: &StaffUpdate) -> Result<StaffOption> {
    let name = patch.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("声优主名不能为空".into()));
    }
    let updated = conn.execute(
        "UPDATE staffs SET name = ?2, original = ?3 WHERE id = ?1",
        params![patch.id, name, patch.original.trim()],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound("声优不存在".into()));
    }
    staff_option(conn, &patch.id)
}

pub fn merge_staff(conn: &Connection, merge: &StaffMerge) -> Result<StaffOption> {
    if merge.target_id == merge.source_id {
        return Err(AppError::BadRequest("不能把声优并入它自己".into()));
    }
    let tx = conn.unchecked_transaction()?;
    for id in [&merge.target_id, &merge.source_id] {
        let found = tx
            .query_row("SELECT 1 FROM staffs WHERE id = ?", params![id], |_| Ok(()))
            .optional()?;
        if found.is_none() {
            return Err(AppError::NotFound("声优不存在".into()));
        }
    }
    tx.execute(
        "UPDATE OR IGNORE character_va SET staff_id = ?1 WHERE staff_id = ?2",
        params![merge.target_id, merge.source_id],
    )?;
    tx.execute(
        "DELETE FROM character_va WHERE staff_id = ?1",
        params![merge.source_id],
    )?;
    tx.execute(
        "UPDATE staffs SET
           name     = COALESCE(NULLIF(name, ''), (SELECT name FROM staffs WHERE id = ?2)),
           original = COALESCE(NULLIF(original, ''), (SELECT original FROM staffs WHERE id = ?2))
         WHERE id = ?1",
        params![merge.target_id, merge.source_id],
    )?;
    tx.execute(
        "DELETE FROM staffs WHERE id = ?1",
        params![merge.source_id],
    )?;
    tx.commit()?;
    staff_option(conn, &merge.target_id)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffOption {
    pub id: String,
    pub name: Option<String>,
    pub original: Option<String>,
    pub display_name: Option<String>,
    pub display_original: Option<String>,
    pub roles: i64,
}

pub fn replace_all(
    conn: &Connection,
    game_id: &str,
    characters: &[GameCharacter],
) -> Result<Vec<GameCharacter>> {
    if !game_exists(conn, game_id)? {
        return Err(AppError::NotFound("游戏不存在".into()));
    }
    let mut seen = HashSet::with_capacity(characters.len());
    for entry in characters {
        if entry.id.trim().is_empty() {
            return Err(AppError::BadRequest("角色缺少 id".into()));
        }
        if !seen.insert(entry.id.as_str()) {
            return Err(AppError::BadRequest("同一个游戏里出现重复的角色 id".into()));
        }
        for credit in entry.va.iter().flatten() {
            if credit.staff_id.trim().is_empty() {
                return Err(AppError::BadRequest("配音记录缺少声优 id".into()));
            }
        }
    }

    let tx = conn.unchecked_transaction()?;
    let kept_staff = staff_snapshot(&tx, game_id)?;
    tx.execute("DELETE FROM characters WHERE game_id = ?", params![game_id])?;
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
        "INSERT INTO character_va (game_id, character_id, staff_id, note, staff_name, staff_original)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(game_id, character_id, staff_id, note) DO NOTHING",
        params![
            game_id,
            character_id,
            entry.staff_id,
            entry.note.as_deref().unwrap_or_default(),
            entry.name,
            entry.original
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

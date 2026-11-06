use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde::de::DeserializeOwned;

use crate::error::{AppError, Result};
use crate::generated::metadata::COLUMNS;
use crate::types::{GameMetadata, MetadataPatch, MetadataTag};

struct MetaRow {
    game_id: String,
    alias: Option<String>,
    titles: Option<String>,
    rating: Option<f64>,
    votecount: Option<i64>,
    length_minutes: Option<i64>,
    released: Option<String>,
    description: Option<String>,
    image_id: Option<String>,
    image_path: Option<String>,
    cover_focal_x: Option<i64>,
    cover_focal_y: Option<i64>,
    ambience_path: Option<String>,
    ambience_mode: String,
    ambience_plate: bool,
    ambience_focal_x: i64,
    ambience_focal_y: i64,
    tags: Option<String>,
    developers: Option<String>,
    staff: Option<String>,
}

pub fn get(conn: &Connection, game_id: &str) -> Result<Option<GameMetadata>> {
    if !game_exists(conn, game_id)? {
        return Err(AppError::NotFound("游戏不存在".into()));
    }
    let row = conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM game_metadata WHERE game_id = ?"),
            params![game_id],
            read_row,
        )
        .optional()?;
    match row {
        Some(row) => Ok(Some(to_api(&row)?)),
        None => Ok(None),
    }
}

pub fn upsert(conn: &Connection, game_id: &str, patch: &MetadataPatch) -> Result<GameMetadata> {
    if !game_exists(conn, game_id)? {
        return Err(AppError::NotFound("游戏不存在".into()));
    }

    let mut patch = patch.clone();
    if cover_swapped(&get(conn, game_id)?, &patch) {
        patch.cover_focal_x = Some(Some(50));
        patch.cover_focal_y = Some(Some(50));
    }

    let (columns, mut values) = crate::generated::metadata::patch_columns(&patch)?;
    if columns.is_empty() {
        return Err(AppError::BadRequest("没有可更新的字段".into()));
    }

    let assignments = columns
        .iter()
        .map(|c| format!("{c} = ?"))
        .collect::<Vec<_>>()
        .join(", ");
    let changed = conn.execute(
        &format!("UPDATE game_metadata SET {assignments} WHERE game_id = ?"),
        params_from_iter(
            values
                .iter()
                .chain(std::iter::once(&Value::Text(game_id.to_owned()))),
        ),
    )?;

    if changed == 0 {
        values.insert(0, Value::Text(game_id.to_owned()));
        let placeholders = vec!["?"; columns.len() + 1].join(", ");
        conn.execute(
            &format!(
                "INSERT INTO game_metadata (game_id, {}) VALUES ({placeholders})",
                columns.join(", ")
            ),
            params_from_iter(values.iter()),
        )?;
    }

    get(conn, game_id)?.ok_or_else(|| AppError::NotFound("游戏不存在".into()))
}

fn game_exists(conn: &Connection, id: &str) -> Result<bool> {
    let found = conn
        .query_row("SELECT id FROM games WHERE id = ?", params![id], |row| {
            row.get::<_, String>(0)
        })
        .optional()?;
    Ok(found.is_some())
}

fn cover_swapped(current: &Option<GameMetadata>, patch: &MetadataPatch) -> bool {
    let Some(next) = patch.image_path.as_ref() else {
        return false;
    };
    let cur = current.as_ref().and_then(|m| m.image_path.clone());
    cur != next.clone()
}

fn read_row(row: &Row<'_>) -> rusqlite::Result<MetaRow> {
    Ok(MetaRow {
        game_id: row.get("game_id")?,
        alias: row.get("alias")?,
        titles: row.get("titles")?,
        rating: row.get("rating")?,
        votecount: row.get("votecount")?,
        length_minutes: row.get("length_minutes")?,
        released: row.get("released")?,
        description: row.get("description")?,
        image_id: row.get("image_id")?,
        image_path: row.get("image_path")?,
        cover_focal_x: row.get("cover_focal_x")?,
        cover_focal_y: row.get("cover_focal_y")?,
        ambience_path: row.get("ambience_path")?,
        ambience_mode: row.get("ambience_mode")?,
        ambience_plate: row.get("ambience_plate")?,
        ambience_focal_x: row.get("ambience_focal_x")?,
        ambience_focal_y: row.get("ambience_focal_y")?,
        tags: row.get("tags")?,
        developers: row.get("developers")?,
        staff: row.get("staff")?,
    })
}

fn to_api(row: &MetaRow) -> Result<GameMetadata> {
    Ok(GameMetadata {
        game_id: row.game_id.clone(),
        alias: row.alias.clone(),
        titles: from_json(&row.titles)?,
        rating: row.rating,
        votecount: row.votecount,
        length_minutes: row.length_minutes,
        released: row.released.clone(),
        description: row.description.clone(),
        image_id: row.image_id.clone(),
        image_path: row.image_path.clone(),
        cover_focal_x: row.cover_focal_x,
        cover_focal_y: row.cover_focal_y,
        ambience_path: row.ambience_path.clone(),
        ambience_mode: row.ambience_mode.clone(),
        ambience_plate: row.ambience_plate,
        ambience_focal_x: row.ambience_focal_x,
        ambience_focal_y: row.ambience_focal_y,
        tags: from_json::<Vec<MetadataTag>>(&row.tags)?,
        developers: from_json(&row.developers)?,
        staff: from_json(&row.staff)?,
    })
}

fn from_json<T: DeserializeOwned>(raw: &Option<String>) -> Result<Option<T>> {
    match raw {
        Some(text) => Ok(Some(serde_json::from_str(text)?)),
        None => Ok(None),
    }
}

use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};

use crate::error::{AppError, Result};
use crate::generated::metadata::{patch_columns, read_row, to_api, COLUMNS};
use crate::types::{GameMetadata, MetadataPatch};

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

    let (columns, mut values) = patch_columns(&patch)?;
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

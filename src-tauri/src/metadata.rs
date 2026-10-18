use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{AppError, Result};
use crate::types::{GameMetadata, MetadataPatch, MetadataTag};

const COLUMNS: &str =
    "game_id, alias, titles, rating, votecount, length_minutes, released, description, image_id, image_path, tags, developers";

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
    tags: Option<String>,
    developers: Option<String>,
}

pub fn get(conn: &Connection, game_id: &str) -> Result<Option<GameMetadata>> {
    if !game_exists(conn, game_id)? {
        return Err(AppError::NotFound("作品不存在".into()));
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
        return Err(AppError::NotFound("作品不存在".into()));
    }

    let (columns, mut values) = build_columns(patch)?;
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

    get(conn, game_id)?.ok_or_else(|| AppError::NotFound("作品不存在".into()))
}

fn build_columns(patch: &MetadataPatch) -> Result<(Vec<&'static str>, Vec<Value>)> {
    let mut columns: Vec<&'static str> = Vec::new();
    let mut values: Vec<Value> = Vec::new();

    macro_rules! nullable {
        ($field:expr, $column:literal) => {
            if let Some(raw) = $field {
                columns.push($column);
                values.push(match raw {
                    Some(v) => Value::Text(v.clone()),
                    None => Value::Null,
                });
            }
        };
    }
    nullable!(&patch.alias, "alias");
    nullable!(&patch.released, "released");
    nullable!(&patch.description, "description");
    nullable!(&patch.image_path, "image_path");

    if let Some(raw) = &patch.rating {
        columns.push("rating");
        values.push(match raw {
            Some(v) => Value::Real(*v),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.votecount {
        columns.push("votecount");
        values.push(match raw {
            Some(v) => Value::Integer(*v),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.length_minutes {
        columns.push("length_minutes");
        values.push(match raw {
            Some(v) => Value::Integer(*v),
            None => Value::Null,
        });
    }

    if let Some(raw) = &patch.titles {
        columns.push("titles");
        values.push(match raw {
            Some(v) => Value::Text(to_json(v)?),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.tags {
        columns.push("tags");
        values.push(match raw {
            Some(v) => Value::Text(to_json(v)?),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.developers {
        columns.push("developers");
        values.push(match raw {
            Some(v) => Value::Text(to_json(v)?),
            None => Value::Null,
        });
    }

    Ok((columns, values))
}

fn to_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

fn game_exists(conn: &Connection, id: &str) -> Result<bool> {
    let found = conn
        .query_row("SELECT id FROM games WHERE id = ?", params![id], |row| {
            row.get::<_, String>(0)
        })
        .optional()?;
    Ok(found.is_some())
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
        tags: row.get("tags")?,
        developers: row.get("developers")?,
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
        tags: from_json::<Vec<MetadataTag>>(&row.tags)?,
        developers: from_json(&row.developers)?,
    })
}

fn from_json<T: DeserializeOwned>(raw: &Option<String>) -> Result<Option<T>> {
    match raw {
        Some(text) => Ok(Some(serde_json::from_str(text)?)),
        None => Ok(None),
    }
}

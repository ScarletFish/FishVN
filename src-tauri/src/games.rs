use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};

use crate::error::{write_conflict, AppError, Result};
use crate::metadata;
use crate::types::{Game, GameListItem, GamePatch, NewGame, StatePatch, Status, UserGameState};
use crate::{new_id, now};

const DEFAULT_STATUS: Status = Status::Wishlist;

pub fn list_games(conn: &Connection) -> Result<Vec<GameListItem>> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.vndb_id, g.created_at,
                s.status, s.user_rating,
                COUNT(i.id) AS installation_count,
                MAX(CASE WHEN i.entry_path IS NOT NULL THEN 1 ELSE 0 END) AS can_launch,
                m.image_path AS cover_path, m.alias AS alias
         FROM games g
         JOIN user_game_state s ON s.game_id = g.id
         LEFT JOIN installations i ON i.game_id = g.id
         LEFT JOIN game_metadata m ON m.game_id = g.id
         GROUP BY g.id, m.image_path, m.alias
         ORDER BY g.created_at DESC, g.id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(GameListItem {
            id: row.get("id")?,
            name: row.get("name")?,
            vndb_id: row.get("vndb_id")?,
            alias: row.get("alias")?,
            created_at: row.get("created_at")?,
            status: row.get("status")?,
            user_rating: row.get("user_rating")?,
            installation_count: row.get("installation_count")?,
            can_launch: row.get::<_, i64>("can_launch")? == 1,
            cover_path: row.get("cover_path")?,
        })
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn get_game(conn: &Connection, id: &str) -> Result<Game> {
    conn.query_row(
        "SELECT id, name, vndb_id, created_at FROM games WHERE id = ?",
        params![id],
        |row| {
            Ok(Game {
                id: row.get("id")?,
                name: row.get("name")?,
                vndb_id: row.get("vndb_id")?,
                created_at: row.get("created_at")?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound("游戏不存在".into()))
}

pub fn create_game(conn: &Connection, input: &NewGame) -> Result<Game> {
    let name = validate_name(&input.name)?;
    let vndb_id = match &input.vndb_id {
        Some(raw) => Some(validate_vndb_id(raw)?),
        None => None,
    };
    let game = Game {
        id: new_id(),
        name,
        vndb_id,
        created_at: now(),
    };

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO games (id, name, vndb_id, created_at) VALUES (?, ?, ?, ?)",
        params![&game.id, &game.name, &game.vndb_id, game.created_at],
    )
    .map_err(|e| write_conflict(e, "vndb_id 已被其他游戏占用"))?;
    tx.execute(
        "INSERT INTO user_game_state (game_id, status) VALUES (?, ?)",
        params![&game.id, DEFAULT_STATUS],
    )?;
    if let Some(patch) = &input.metadata {
        metadata::upsert(&tx, &game.id, patch)?;
    }
    tx.commit()?;
    Ok(game)
}

pub fn update_game(conn: &Connection, id: &str, patch: &GamePatch) -> Result<Game> {
    let mut sets: Vec<&'static str> = Vec::new();
    let mut values: Vec<Value> = Vec::new();

    if let Some(raw) = &patch.name {
        sets.push("name = ?");
        values.push(Value::Text(validate_name(raw)?));
    }
    if let Some(raw) = &patch.vndb_id {
        sets.push("vndb_id = ?");
        values.push(match raw {
            Some(v) => Value::Text(validate_vndb_id(v)?),
            None => Value::Null,
        });
    }
    if sets.is_empty() {
        return Err(AppError::BadRequest("没有可更新的字段".into()));
    }
    values.push(Value::Text(id.to_owned()));

    let changes = conn
        .execute(
            &format!("UPDATE games SET {} WHERE id = ?", sets.join(", ")),
            params_from_iter(values.iter()),
        )
        .map_err(|e| write_conflict(e, "vndb_id 已被其他游戏占用"))?;
    if changes == 0 {
        return Err(AppError::NotFound("游戏不存在".into()));
    }
    get_game(conn, id)
}

pub fn delete_game(conn: &Connection, id: &str) -> Result<()> {
    let changes = conn.execute("DELETE FROM games WHERE id = ?", params![id])?;
    if changes == 0 {
        return Err(AppError::NotFound("游戏不存在".into()));
    }
    Ok(())
}

pub fn get_game_state(conn: &Connection, id: &str) -> Result<UserGameState> {
    conn.query_row(
        "SELECT s.game_id, s.status, s.user_rating, s.notes, s.default_installation_id,
                s.play_seconds,
                COUNT(p.id) AS play_count,
                MAX(p.started_at) AS last_played_at,
                COALESCE(SUM(p.foreground_seconds), 0) AS foreground_seconds,
                COALESCE(SUM(COALESCE(p.ended_at, p.last_seen_at) - p.started_at), 0) AS alive_seconds
         FROM user_game_state s
         LEFT JOIN play_sessions p ON p.game_id = s.game_id
         WHERE s.game_id = ?
         GROUP BY s.game_id",
        params![id],
        read_state_row,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound("游戏不存在".into()))
}

pub fn update_game_state(conn: &Connection, id: &str, patch: &StatePatch) -> Result<UserGameState> {
    if let Some(Some(wanted)) = &patch.default_installation_id {
        if wanted.is_empty() {
            return Err(AppError::BadRequest(
                "default_installation_id 是非空字符串或 null".into(),
            ));
        }
        let owner: Option<Option<String>> = conn
            .query_row(
                "SELECT game_id FROM installations WHERE id = ?",
                params![wanted],
                |row| row.get(0),
            )
            .optional()?;
        match owner {
            None => return Err(AppError::NotFound("安装记录不存在".into())),
            Some(Some(owner_id)) if owner_id != id => {
                return Err(AppError::BadRequest("该安装不属于这个游戏".into()))
            }
            _ => {}
        }
    }

    let mut sets: Vec<&'static str> = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    if let Some(status) = patch.status {
        sets.push("status = ?");
        values.push(Value::Text(status.as_str().to_owned()));
    }
    if let Some(raw) = &patch.user_rating {
        sets.push("user_rating = ?");
        values.push(match raw {
            Some(v) => Value::Real(*v),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.notes {
        sets.push("notes = ?");
        values.push(match raw {
            Some(s) => Value::Text(s.clone()),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.default_installation_id {
        sets.push("default_installation_id = ?");
        values.push(match raw {
            Some(s) => Value::Text(s.clone()),
            None => Value::Null,
        });
    }
    if let Some(seconds) = patch.play_seconds {
        if seconds < 0 {
            return Err(AppError::BadRequest("总时长不能是负数".into()));
        }
        sets.push("play_seconds = ?");
        values.push(Value::Integer(seconds));
    }
    if sets.is_empty() {
        return Err(AppError::BadRequest("没有可更新的字段".into()));
    }
    values.push(Value::Text(id.to_owned()));

    let changes = conn.execute(
        &format!(
            "UPDATE user_game_state SET {} WHERE game_id = ?",
            sets.join(", ")
        ),
        params_from_iter(values.iter()),
    )?;
    if changes == 0 {
        return Err(AppError::NotFound("游戏不存在".into()));
    }
    get_game_state(conn, id)
}

fn read_state_row(row: &Row<'_>) -> rusqlite::Result<UserGameState> {
    Ok(UserGameState {
        game_id: row.get("game_id")?,
        status: row.get("status")?,
        user_rating: row.get("user_rating")?,
        notes: row.get("notes")?,
        default_installation_id: row.get("default_installation_id")?,
        play_seconds: row.get("play_seconds")?,
        play_count: row.get("play_count")?,
        last_played_at: row.get("last_played_at")?,
        foreground_seconds: row.get("foreground_seconds")?,
        alive_seconds: row.get("alive_seconds")?,
    })
}

fn validate_name(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("name 不能为空".into()));
    }
    Ok(trimmed.to_owned())
}

fn validate_vndb_id(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    let digits = trimmed.strip_prefix('v').unwrap_or_default();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "vndb_id 必须是 v+数字".into(),
        ));
    }
    Ok(trimmed.to_owned())
}

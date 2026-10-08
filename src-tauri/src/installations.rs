use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde_json::Value as Json;

use crate::error::{write_conflict, AppError, Result};
use crate::fsinfo::{ensure_dir, read_dir_info, resolve_path};
use crate::new_id;
use crate::types::{Installation, InstallationPatch, NewInstallation};

const COLUMNS: &str = "id, game_id, dir_path, entry_path, exes, size_bytes";

/// 库里 exes 是 JSON 文本，对外给数组
struct InstallRow {
    id: String,
    game_id: Option<String>,
    dir_path: String,
    entry_path: Option<String>,
    exes: Option<String>,
    size_bytes: Option<i64>,
}

pub fn list_installations(conn: &Connection, game_id: Option<&str>) -> Result<Vec<Installation>> {
    let (sql, values): (String, Vec<Value>) = match game_id {
        Some(gid) => (
            format!("SELECT {COLUMNS} FROM installations WHERE game_id = ? ORDER BY dir_path"),
            vec![Value::Text(gid.to_owned())],
        ),
        None => (
            format!("SELECT {COLUMNS} FROM installations ORDER BY dir_path"),
            vec![],
        ),
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(values.iter()), read_row)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(to_api(&row?)?);
    }
    Ok(out)
}

pub fn get_installation(conn: &Connection, id: &str) -> Result<Installation> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM installations WHERE id = ?"))?;
    let row = stmt
        .query_row(params![id], read_row)
        .optional()?
        .ok_or_else(|| AppError::NotFound("安装记录不存在".into()))?;
    to_api(&row)
}

pub fn create_installation(conn: &Connection, input: &NewInstallation) -> Result<Installation> {
    if input.game_id.is_empty() {
        return Err(AppError::BadRequest("game_id 必填".into()));
    }
    let requested = input.dir_path.trim();
    if requested.is_empty() {
        return Err(AppError::BadRequest("dir_path 必填".into()));
    }

    if !game_exists(conn, &input.game_id)? {
        return Err(AppError::NotFound("作品不存在".into()));
    }

    let dir = resolve_path(requested);
    let dir_path = dir.to_string_lossy().into_owned();

    // 先查重复再遍历目录：拖同一个目录两次不该付出读整个目录的代价。
    // UNIQUE 约束仍是最终保证，这里只是提前失败。
    let dup = conn
        .query_row(
            "SELECT id FROM installations WHERE dir_path = ?",
            params![&dir_path],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    if dup.is_some() {
        return Err(AppError::Conflict("该目录已有安装记录".into()));
    }

    ensure_dir(&dir)?;
    let info = read_dir_info(&dir);

    // 候选唯一时没有选择余地，直接填上
    let entry_path = match info.exes.as_slice() {
        [only] => Some(only.clone()),
        _ => None,
    };
    let row = InstallRow {
        id: new_id(),
        game_id: Some(input.game_id.clone()),
        dir_path,
        entry_path,
        exes: Some(serde_json::to_string(&info.exes)?),
        size_bytes: Some(info.size_bytes),
    };

    conn.execute(
        &format!("INSERT INTO installations ({COLUMNS}) VALUES (?, ?, ?, ?, ?, ?)"),
        params![
            &row.id,
            &row.game_id,
            &row.dir_path,
            &row.entry_path,
            &row.exes,
            row.size_bytes,
        ],
    )
    .map_err(|e| write_conflict(e, "该目录已有安装记录"))?;

    to_api(&row)
}

pub fn update_installation(
    conn: &Connection,
    id: &str,
    patch: &InstallationPatch,
) -> Result<Installation> {
    // entry_path 的合法性先判：归属查不到是 404，入口不合法是 400，
    // 两个同时非法时要报 400 那个
    if let Some(Some(entry)) = &patch.entry_path {
        if entry.is_empty() {
            return Err(AppError::BadRequest(
                "entry_path 是非空字符串或 null".into(),
            ));
        }
    }
    if let Some(Some(gid)) = &patch.game_id {
        if !game_exists(conn, gid)? {
            return Err(AppError::NotFound("作品不存在".into()));
        }
    }

    let mut sets: Vec<&'static str> = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    if let Some(raw) = &patch.game_id {
        sets.push("game_id = ?");
        values.push(match raw {
            Some(g) => Value::Text(g.clone()),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.entry_path {
        sets.push("entry_path = ?");
        values.push(match raw {
            Some(e) => Value::Text(e.clone()),
            None => Value::Null,
        });
    }
    if sets.is_empty() {
        return Err(AppError::BadRequest("没有可更新的字段".into()));
    }
    values.push(Value::Text(id.to_owned()));

    let changes = conn.execute(
        &format!("UPDATE installations SET {} WHERE id = ?", sets.join(", ")),
        params_from_iter(values.iter()),
    )?;
    if changes == 0 {
        return Err(AppError::NotFound("安装记录不存在".into()));
    }
    get_installation(conn, id)
}

pub fn delete_installation(conn: &Connection, id: &str) -> Result<()> {
    let changes = conn.execute("DELETE FROM installations WHERE id = ?", params![id])?;
    if changes == 0 {
        return Err(AppError::NotFound("安装记录不存在".into()));
    }
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

fn read_row(row: &Row<'_>) -> rusqlite::Result<InstallRow> {
    Ok(InstallRow {
        id: row.get("id")?,
        game_id: row.get("game_id")?,
        dir_path: row.get("dir_path")?,
        entry_path: row.get("entry_path")?,
        exes: row.get("exes")?,
        size_bytes: row.get("size_bytes")?,
    })
}

fn to_api(row: &InstallRow) -> Result<Installation> {
    let exes = match &row.exes {
        Some(raw) => match serde_json::from_str::<Json>(raw)? {
            Json::Array(items) => Some(
                items
                    .into_iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect(),
            ),
            _ => None,
        },
        None => None,
    };
    Ok(Installation {
        id: row.id.clone(),
        game_id: row.game_id.clone(),
        dir_path: row.dir_path.clone(),
        entry_path: row.entry_path.clone(),
        exes,
        size_bytes: row.size_bytes,
    })
}

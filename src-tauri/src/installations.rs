use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Component, Path};

use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde_json::Value as Json;

use crate::error::{write_conflict, AppError, Result};
use crate::fsinfo::{ensure_dir, read_dir_info, read_exes, resolve_path};
use crate::new_id;
use crate::types::{Installation, InstallationInput, InstallationPatch, NewInstallation};

const COLUMNS: &str = "id, game_id, dir_path, entry_path, exes, size_bytes";

const ORDER_BY: &str = "sort_order, dir_path, IFNULL(entry_path, '')";

const DIR_TAKEN: &str = "该目录已有安装记录";

const TAKEN: &str = "该目录的这条启动入口已有安装记录";

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
            format!(
                "SELECT {COLUMNS} FROM installations WHERE game_id = ? ORDER BY {ORDER_BY}"
            ),
            vec![Value::Text(gid.to_owned())],
        ),
        None => (
            format!("SELECT {COLUMNS} FROM installations ORDER BY {ORDER_BY}"),
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

    if dir_holder(conn, &dir_path)?.is_some() {
        return Err(AppError::Conflict(DIR_TAKEN.into()));
    }

    ensure_dir(&dir)?;
    let info = read_dir_info(&dir);

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
    .map_err(|e| write_conflict(e, DIR_TAKEN))?;

    to_api(&row)
}

pub fn update_installation(
    conn: &Connection,
    id: &str,
    patch: &InstallationPatch,
) -> Result<Installation> {
    if let Some(Some(entry)) = &patch.entry_path {
        validate_entry_path(entry)?;
        let current: Option<(String, Option<String>)> = conn
            .query_row(
                "SELECT dir_path, entry_path FROM installations WHERE id = ?",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if let Some((dir, before)) = current {
            if before.as_deref() != Some(entry.as_str())
                && target_holder(conn, &dir, Some(entry), id)?.is_some()
            {
                return Err(AppError::Conflict(TAKEN.into()));
            }
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

    let changes = conn
        .execute(
            &format!("UPDATE installations SET {} WHERE id = ?", sets.join(", ")),
            params_from_iter(values.iter()),
        )
        .map_err(|e| write_conflict(e, TAKEN))?;
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

struct Existing {
    id: String,
    game_id: Option<String>,
    dir_path: String,
    entry_path: Option<String>,
}

struct Draft {
    id: Option<String>,
    dir_path: String,
    entry_path: Option<String>,
}

pub fn save_installations(
    conn: &Connection,
    game_id: &str,
    entries: &[InstallationInput],
) -> Result<Vec<Installation>> {
    if !game_exists(conn, game_id)? {
        return Err(AppError::NotFound("作品不存在".into()));
    }
    let drafts = validate_drafts(entries)?;

    let tx = conn.unchecked_transaction()?;
    let existing = existing_rows(&tx)?;
    let by_id: HashMap<&str, &Existing> = existing.iter().map(|r| (r.id.as_str(), r)).collect();
    let by_key: HashMap<(&str, &str), &Existing> = existing
        .iter()
        .map(|r| ((r.dir_path.as_str(), r.entry_path.as_deref().unwrap_or("")), r))
        .collect();
    let dir_owner: HashMap<&str, Option<&str>> = existing
        .iter()
        .map(|r| (r.dir_path.as_str(), r.game_id.as_deref()))
        .collect();
    let mut orphans: HashMap<&str, VecDeque<&Existing>> = HashMap::new();
    for row in existing.iter().filter(|row| row.game_id.is_none()) {
        orphans
            .entry(row.dir_path.as_str())
            .or_default()
            .push_back(row);
    }

    let taken_by_other = |dir_path: &str| {
        dir_owner
            .get(dir_path)
            .copied()
            .flatten()
            .is_some_and(|owner| owner != game_id)
    };

    let mut kept: HashSet<String> = HashSet::with_capacity(drafts.len());
    for (index, draft) in drafts.iter().enumerate() {
        let order = index as i64;
        let key = (
            draft.dir_path.as_str(),
            draft.entry_path.as_deref().unwrap_or(""),
        );
        let target = match &draft.id {
            Some(id) => {
                let row = by_id
                    .get(id.as_str())
                    .ok_or_else(|| AppError::NotFound("安装记录不存在".into()))?;
                if row.game_id.as_deref() != Some(game_id) {
                    return Err(AppError::Conflict("安装记录不属于这部作品".into()));
                }
                if taken_by_other(draft.dir_path.as_str()) {
                    return Err(AppError::Conflict(DIR_TAKEN.into()));
                }
                if by_key.get(&key).is_some_and(|holder| holder.id != *id) {
                    return Err(AppError::Conflict(TAKEN.into()));
                }
                Some(*row)
            }
            None => {
                if taken_by_other(draft.dir_path.as_str()) {
                    return Err(AppError::Conflict(DIR_TAKEN.into()));
                }
                match by_key.get(&key) {
                    Some(&holder) => {
                        if holder
                            .game_id
                            .as_deref()
                            .is_some_and(|owner| owner != game_id)
                        {
                            return Err(AppError::Conflict(TAKEN.into()));
                        }
                        Some(holder)
                    }
                    None => orphans
                        .get_mut(draft.dir_path.as_str())
                        .and_then(|queue| queue.pop_front()),
                }
            }
        };

        let id = match target {
            Some(row) => {
                tx.execute(
                    "UPDATE installations SET game_id = ?, dir_path = ?, entry_path = ?, sort_order = ?
                     WHERE id = ?",
                    params![game_id, &draft.dir_path, &draft.entry_path, order, &row.id],
                )?;
                if row.dir_path != draft.dir_path {
                    let dir = Path::new(&draft.dir_path);
                    ensure_dir(dir)?;
                    tx.execute(
                        "UPDATE installations SET exes = ?, size_bytes = NULL WHERE id = ?",
                        params![serde_json::to_string(&read_exes(dir))?, &row.id],
                    )?;
                }
                row.id.clone()
            }
            None => {
                let dir = Path::new(&draft.dir_path);
                ensure_dir(dir)?;
                let id = new_id();
                tx.execute(
                    "INSERT INTO installations
                     (id, game_id, dir_path, entry_path, exes, size_bytes, sort_order)
                     VALUES (?, ?, ?, ?, ?, NULL, ?)",
                    params![
                        &id,
                        game_id,
                        &draft.dir_path,
                        &draft.entry_path,
                        serde_json::to_string(&read_exes(dir))?,
                        order,
                    ],
                )
                .map_err(|e| write_conflict(e, TAKEN))?;
                id
            }
        };
        kept.insert(id);
    }

    let touched: HashSet<&str> = drafts.iter().map(|draft| draft.dir_path.as_str()).collect();
    for row in &existing {
        if kept.contains(&row.id) {
            continue;
        }
        let dropped = row.game_id.as_deref() == Some(game_id)
            || (row.game_id.is_none() && touched.contains(row.dir_path.as_str()));
        if dropped {
            tx.execute("DELETE FROM installations WHERE id = ?", params![&row.id])?;
        }
    }
    tx.commit()?;

    list_installations(conn, Some(game_id))
}

fn validate_drafts(entries: &[InstallationInput]) -> Result<Vec<Draft>> {
    let mut out = Vec::with_capacity(entries.len());
    let mut targets: HashSet<(String, String)> = HashSet::with_capacity(entries.len());
    let mut ids: HashSet<String> = HashSet::with_capacity(entries.len());
    for entry in entries {
        let requested = entry.dir_path.trim();
        if requested.is_empty() {
            return Err(AppError::BadRequest("dir_path 必填".into()));
        }
        let dir_path = resolve_path(requested).to_string_lossy().into_owned();
        let target = (dir_path.clone(), entry.entry_path.clone().unwrap_or_default());
        if !targets.insert(target) {
            return Err(AppError::Conflict("同一目录的同一入口重复添加".into()));
        }
        let id = match &entry.id {
            Some(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return Err(AppError::BadRequest("安装 id 不能是空串".into()));
                }
                if !ids.insert(trimmed.to_owned()) {
                    return Err(AppError::BadRequest("同一安装项重复出现".into()));
                }
                Some(trimmed.to_owned())
            }
            None => None,
        };
        if let Some(path) = &entry.entry_path {
            validate_entry_path(path)?;
        }
        out.push(Draft {
            id,
            dir_path,
            entry_path: entry.entry_path.clone(),
        });
    }
    Ok(out)
}

fn validate_entry_path(entry: &str) -> Result<()> {
    if entry.is_empty() {
        return Err(AppError::BadRequest(
            "entry_path 是非空字符串或 null".into(),
        ));
    }
    let path = Path::new(entry);
    let rooted = path.is_absolute() || entry.starts_with('/') || entry.starts_with('\\');
    let escapes = path
        .components()
        .any(|part| matches!(part, Component::ParentDir));
    if rooted || escapes {
        return Err(AppError::BadRequest(
            "entry_path 用相对该目录的路径".into(),
        ));
    }
    Ok(())
}

fn existing_rows(conn: &Connection) -> Result<Vec<Existing>> {
    let mut stmt =
        conn.prepare("SELECT id, game_id, dir_path, entry_path FROM installations ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Existing {
            id: row.get(0)?,
            game_id: row.get(1)?,
            dir_path: row.get(2)?,
            entry_path: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn dir_holder(conn: &Connection, dir_path: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT id FROM installations WHERE dir_path = ? LIMIT 1",
            params![dir_path],
            |row| row.get::<_, String>(0),
        )
        .optional()?)
}

fn target_holder(
    conn: &Connection,
    dir_path: &str,
    entry_path: Option<&str>,
    skip: &str,
) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT id FROM installations
             WHERE dir_path = ? AND IFNULL(entry_path, '') = IFNULL(?, '') AND id <> ?",
            params![dir_path, entry_path, skip],
            |row| row.get::<_, String>(0),
        )
        .optional()?)
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

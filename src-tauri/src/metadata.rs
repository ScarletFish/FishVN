use std::path::{Path, PathBuf};

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
        Some(row) => {
            let mut meta = to_api(&row)?;
            meta.tags = Some(crate::tagstore::read_tags(conn, game_id)?);
            Ok(Some(meta))
        }
        None => Ok(None),
    }
}

pub fn ensure_row(conn: &Connection, game_id: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO game_metadata (game_id) VALUES (?1)",
        params![game_id],
    )?;
    Ok(())
}

pub fn upsert(
    conn: &Connection,
    game_id: &str,
    patch: &MetadataPatch,
    data_dir: Option<&Path>,
) -> Result<GameMetadata> {
    if !game_exists(conn, game_id)? {
        return Err(AppError::NotFound("游戏不存在".into()));
    }

    let mut patch = patch.clone();
    let mut copied = Vec::new();
    let written = (|| -> Result<()> {
        if let Some(dir) = data_dir {
            import_images(&mut patch, game_id, dir, &mut copied)?;
        }
        let current = get(conn, game_id)?;
        if cover_swapped(&current, &patch) {
            patch.cover_focal_x = Some(Some(50));
            patch.cover_focal_y = Some(Some(50));
        }
        if ambience_swapped(&current, &patch) {
            patch.ambience_focal_x = Some(Some(50));
            patch.ambience_focal_y = Some(Some(22));
            patch.showcase_focal_x = Some(Some(50));
            patch.showcase_focal_y = Some(Some(22));
        }

        let (columns, mut values) = patch_columns(&patch)?;
        if columns.is_empty() && patch.tags.is_none() {
            return Err(AppError::BadRequest("没有可更新的字段".into()));
        }

        let tx = if conn.is_autocommit() {
            Some(conn.unchecked_transaction()?)
        } else {
            None
        };
        let w = tx.as_deref().unwrap_or(conn);

        if columns.is_empty() {
            w.execute(
                "INSERT OR IGNORE INTO game_metadata (game_id) VALUES (?1)",
                params![game_id],
            )?;
        }

        let assignments = columns
            .iter()
            .map(|c| format!("{c} = ?"))
            .collect::<Vec<_>>()
            .join(", ");
        let changed = if columns.is_empty() {
            1
        } else {
            w.execute(
                &format!("UPDATE game_metadata SET {assignments} WHERE game_id = ?"),
                params_from_iter(
                    values
                        .iter()
                        .chain(std::iter::once(&Value::Text(game_id.to_owned()))),
                ),
            )?
        };

        if changed == 0 {
            values.insert(0, Value::Text(game_id.to_owned()));
            let placeholders = vec!["?"; columns.len() + 1].join(", ");
            w.execute(
                &format!(
                    "INSERT INTO game_metadata (game_id, {}) VALUES ({placeholders})",
                    columns.join(", ")
                ),
                params_from_iter(values.iter()),
            )?;
        }

        if let Some(tags) = &patch.tags {
            crate::tagstore::replace_tags(w, game_id, tags.as_deref().unwrap_or_default())?;
        }

        if let Some(tx) = tx {
            tx.commit()?;
        }
        Ok(())
    })();

    if let Err(e) = written {
        for path in &copied {
            let _ = std::fs::remove_file(path);
        }
        return Err(e);
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

fn import_images(
    patch: &mut MetadataPatch,
    game_id: &str,
    data_dir: &Path,
    copied: &mut Vec<PathBuf>,
) -> Result<()> {
    let dir = data_dir.join("covers").join(game_id).join("source");
    for field in [&mut patch.image_path, &mut patch.ambience_path] {
        let Some(raw) = field.as_ref().and_then(|v| v.as_ref()) else {
            continue;
        };
        let path = Path::new(raw.trim());
        if path.as_os_str().is_empty() || path.starts_with(&dir) {
            continue;
        }
        if !path.is_file() {
            return Err(AppError::BadRequest(format!("图片文件不存在：{}", path.display())));
        }
        std::fs::create_dir_all(&dir)?;
        let name = path.file_name().ok_or_else(|| AppError::BadRequest("图片路径无效".into()))?;
        let mut target = dir.join(name);
        if target != path && target.exists() {
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
            let mut n = 2;
            loop {
                let candidate = if ext.is_empty() {
                    dir.join(format!("{stem}-{n}"))
                } else {
                    dir.join(format!("{stem}-{n}.{ext}"))
                };
                if !candidate.exists() {
                    target = candidate;
                    break;
                }
                n += 1;
            }
        }
        std::fs::copy(path, &target)?;
        copied.push(target.clone());
        *field = Some(Some(target.to_string_lossy().into_owned()));
    }
    Ok(())
}

fn cover_swapped(current: &Option<GameMetadata>, patch: &MetadataPatch) -> bool {
    let Some(next) = patch.image_path.as_ref() else {
        return false;
    };
    let cur = current.as_ref().and_then(|m| m.image_path.clone());
    cur != next.clone()
}

fn ambience_swapped(current: &Option<GameMetadata>, patch: &MetadataPatch) -> bool {
    let Some(next) = patch.ambience_path.as_ref() else {
        return false;
    };
    let cur = current.as_ref().and_then(|m| m.ambience_path.clone());
    cur != next.clone()
}

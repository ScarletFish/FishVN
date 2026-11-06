use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::error::Result;
use crate::settings;

pub const BACKUP_DIR: &str = "backups";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMeta {
    pub id: String,
    pub created_at: i64,
    pub size_bytes: u64,
    pub cover_count: u64,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn backups_dir(dir: &Path) -> PathBuf {
    dir.join(BACKUP_DIR)
}

fn id_valid(id: &str) -> bool {
    if id.is_empty() || id == "." || id == ".." {
        return false;
    }
    match Path::new(id).parent() {
        None => true,
        Some(p) => p.as_os_str().is_empty(),
    }
}

fn id_created_at(id: &str) -> i64 {
    id.get(..10)
        .and_then(|head| head.parse::<i64>().ok())
        .unwrap_or(0)
}

fn vacuum_target(path: &Path) -> String {
    let raw = path.to_string_lossy().into_owned().replace('\'', "''");
    format!("'{raw}'")
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ftype = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if ftype.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if ftype.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if ft.is_dir() {
                total += dir_size(&entry.path());
            } else if ft.is_file() {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

fn count_files(path: &Path) -> u64 {
    let mut n = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    n += 1;
                }
            }
        }
    }
    n
}

fn meta_from_dir(path: &Path, id: &str) -> BackupMeta {
    BackupMeta {
        id: id.to_owned(),
        created_at: id_created_at(id),
        size_bytes: dir_size(path),
        cover_count: count_files(&path.join("covers")),
    }
}

pub fn create_backup(conn: &Arc<Mutex<Connection>>, dir: &Path) -> Result<BackupMeta> {
    let backups_dir = backups_dir(dir);
    fs::create_dir_all(&backups_dir)?;
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let id = format!("{:010}{:09}", dur.as_secs(), dur.subsec_nanos());
    let dest = backups_dir.join(&id);
    fs::create_dir_all(&dest)?;

    {
        let guard = db::lock(conn)?;
        let dest_db = dest.join(db::DB_FILE);
        guard.execute_batch(&format!("VACUUM INTO {}", vacuum_target(&dest_db)))?;
    }

    let live_covers = dir.join("covers");
    if live_covers.is_dir() {
        copy_tree(&live_covers, &dest.join("covers"))?;
    }
    let live_cfg = dir.join("config.json");
    if live_cfg.is_file() {
        fs::copy(&live_cfg, dest.join("config.json"))?;
    }

    let meta = meta_from_dir(&dest, &id);
    apply_retention(dir, settings::current().backup_retention_days)?;
    Ok(meta)
}

pub fn list_backups(dir: &Path) -> Result<Vec<BackupMeta>> {
    let backups_dir = backups_dir(dir);
    let mut out = Vec::new();
    if !backups_dir.is_dir() {
        return Ok(out);
    }
    for entry in fs::read_dir(&backups_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        if !id_valid(&id) {
            continue;
        }
        out.push(meta_from_dir(&entry.path(), &id));
    }
    out.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(out)
}

struct Reopen<'a> {
    conn: &'a mut Connection,
    dir: &'a Path,
    armed: bool,
}

impl Reopen<'_> {
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for Reopen<'_> {
    fn drop(&mut self) {
        if self.armed {
            if let Ok(conn) = db::open(self.dir) {
                *self.conn = conn;
            }
        }
    }
}

pub fn restore_backup(conn: &Arc<Mutex<Connection>>, dir: &Path, id: &str) -> Result<()> {
    if !id_valid(id) {
        return Err(crate::error::AppError::BadRequest("备份标识非法".into()));
    }
    let src = backups_dir(dir).join(id);
    if !src.is_dir() {
        return Err(crate::error::AppError::NotFound("备份不存在".into()));
    }

    let mut guard = db::lock(conn)?;
    let mut held = Reopen {
        conn: &mut guard,
        dir,
        armed: true,
    };
    *held.conn = Connection::open_in_memory()?;

    restore_files(dir, &src)?;

    *held.conn = db::open(dir)?;
    held.disarm();
    Ok(())
}

fn restore_files(dir: &Path, src: &Path) -> Result<()> {
    let tmp = dir.join(format!("{}.restore-tmp", db::DB_FILE));
    let _ = fs::remove_file(&tmp);
    fs::copy(src.join(db::DB_FILE), &tmp)?;

    let live_db = dir.join(db::DB_FILE);
    let _ = fs::remove_file(dir.join(format!("{}-wal", db::DB_FILE)));
    let _ = fs::remove_file(dir.join(format!("{}-shm", db::DB_FILE)));
    fs::rename(&tmp, &live_db)?;

    let backup_covers = src.join("covers");
    if backup_covers.is_dir() {
        let live_covers = dir.join("covers");
        if live_covers.exists() {
            fs::remove_dir_all(&live_covers)?;
        }
        copy_tree(&backup_covers, &live_covers)?;
    }

    let backup_cfg = src.join("config.json");
    if backup_cfg.is_file() {
        fs::copy(&backup_cfg, dir.join("config.json"))?;
        settings::init(dir);
    }
    Ok(())
}

pub fn delete_backup(dir: &Path, id: &str) -> Result<()> {
    if !id_valid(id) {
        return Err(crate::error::AppError::BadRequest("备份标识非法".into()));
    }
    let src = backups_dir(dir).join(id);
    if !src.is_dir() {
        return Err(crate::error::AppError::NotFound("备份不存在".into()));
    }
    fs::remove_dir_all(&src)?;
    Ok(())
}

fn apply_retention(dir: &Path, days: Option<u64>) -> Result<()> {
    let Some(days) = days else {
        return Ok(());
    };
    if days == 0 {
        return Ok(());
    }
    let cutoff = now_secs() - days as i64 * 86400;
    let backups_dir = backups_dir(dir);
    if !backups_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&backups_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        if !id_valid(&id) || id_created_at(&id) >= cutoff {
            continue;
        }
        let _ = fs::remove_dir_all(entry.path());
    }
    Ok(())
}

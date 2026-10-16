use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

const EXECUTABLE_EXT: [&str; 3] = ["exe", "bat", "cmd"];

const MAX_EXE_DEPTH: usize = 3;

const MAX_EXES: usize = 30;

pub struct DirInfo {
    pub size_bytes: i64,
    pub exes: Vec<String>,
}

pub fn resolve_path(input: &str) -> PathBuf {
    std::path::absolute(input).unwrap_or_else(|_| PathBuf::from(input))
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    let meta = fs::metadata(path).map_err(|_| AppError::BadRequest("目录不存在".into()))?;
    if !meta.is_dir() {
        return Err(AppError::BadRequest("所选路径不是目录".into()));
    }
    Ok(())
}

pub fn read_dir_info(dir: &Path) -> DirInfo {
    let mut size_bytes = 0i64;
    let mut found: Vec<(usize, String)> = Vec::new();
    walk(dir, "", 0, &mut size_bytes, &mut found);
    found.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    DirInfo {
        size_bytes,
        exes: found.into_iter().map(|(_, rel)| rel).collect(),
    }
}

fn walk(
    current: &Path,
    rel: &str,
    depth: usize,
    size_bytes: &mut i64,
    found: &mut Vec<(usize, String)>,
) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let child_rel = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        let Ok(meta) = entry.path().symlink_metadata() else {
            continue;
        };
        let file_type = meta.file_type();
        if file_type.is_dir() {
            walk(&entry.path(), &child_rel, depth + 1, size_bytes, found);
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        *size_bytes += i64::try_from(meta.len()).unwrap_or(i64::MAX);
        if depth < MAX_EXE_DEPTH && is_executable(&name) && found.len() < MAX_EXES {
            found.push((depth, child_rel));
        }
    }
}

fn is_executable(name: &str) -> bool {
    let Some(dot) = name.rfind('.') else {
        return false;
    };
    if dot == 0 || dot + 1 == name.len() {
        return false;
    }
    let ext = name[dot + 1..].to_ascii_lowercase();
    EXECUTABLE_EXT.contains(&ext.as_str())
}

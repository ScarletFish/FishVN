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
    walk(dir, "", 0, Some(&mut size_bytes), &mut found);
    DirInfo {
        size_bytes,
        exes: sorted(found),
    }
}

pub fn read_exes(dir: &Path) -> Vec<String> {
    let mut found: Vec<(usize, String)> = Vec::new();
    walk(dir, "", 0, None, &mut found);
    sorted(found)
}

fn sorted(mut found: Vec<(usize, String)>) -> Vec<String> {
    found.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    found.into_iter().map(|(_, rel)| rel).collect()
}

fn walk(
    current: &Path,
    rel: &str,
    depth: usize,
    mut size_bytes: Option<&mut i64>,
    found: &mut Vec<(usize, String)>,
) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    let descend = size_bytes.is_some() || depth < MAX_EXE_DEPTH;
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
            if descend {
                walk(
                    &entry.path(),
                    &child_rel,
                    depth + 1,
                    size_bytes.as_deref_mut(),
                    found,
                );
            }
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        if let Some(acc) = size_bytes.as_deref_mut() {
            *acc += i64::try_from(meta.len()).unwrap_or(i64::MAX);
        }
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

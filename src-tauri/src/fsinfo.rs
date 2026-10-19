use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

const EXECUTABLE_EXT: [&str; 3] = ["exe", "bat", "cmd"];

const MAX_EXE_DEPTH: usize = 3;

const MAX_EXES: usize = 30;

const NON_LAUNCHER_STEMS: [&str; 10] = [
    "unins",
    "setup",
    "update",
    "redist",
    "vcredist",
    "dxsetup",
    "dxwebsetup",
    "dotnetfx",
    "crashreport",
    "langsetup",
];

const NON_LAUNCHER_DIRS: [&str; 3] = ["redist", "_redist", "uninstall"];

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
    size_bytes: Option<&mut i64>,
    found: &mut Vec<(usize, String)>,
) {
    walk_inner(current, rel, depth, size_bytes, found, true)
}

fn walk_inner(
    current: &Path,
    rel: &str,
    depth: usize,
    mut size_bytes: Option<&mut i64>,
    found: &mut Vec<(usize, String)>,
    collect: bool,
) {
    if size_bytes.is_none() && found.len() >= MAX_EXES {
        return;
    }
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
                let child_collect =
                    collect && !NON_LAUNCHER_DIRS.contains(&name.to_ascii_lowercase().as_str());
                walk_inner(
                    &entry.path(),
                    &child_rel,
                    depth + 1,
                    size_bytes.as_deref_mut(),
                    found,
                    child_collect,
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
        if collect && depth < MAX_EXE_DEPTH && is_executable(&name) && found.len() < MAX_EXES {
            found.push((depth, child_rel));
        }
    }
}

pub(crate) fn is_executable(name: &str) -> bool {
    let Some(dot) = name.rfind('.') else {
        return false;
    };
    if dot == 0 || dot + 1 == name.len() {
        return false;
    }
    let ext = name[dot + 1..].to_ascii_lowercase();
    EXECUTABLE_EXT.contains(&ext.as_str()) && !is_non_launcher(name[..dot].to_ascii_lowercase())
}

fn is_non_launcher(stem: String) -> bool {
    NON_LAUNCHER_STEMS.iter().any(|prefix| stem.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 非启动文件不进候选_目录大小不受影响() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("redist")).expect("建 redist");
        fs::create_dir_all(dir.path().join("bin")).expect("建 bin");
        for rel in ["unins000.exe", "setup.exe", "game.exe", "redist/vcredist.exe", "bin/launcher.exe"] {
            fs::write(dir.path().join(rel), b"x").expect("写文件");
        }

        let exes = read_exes(dir.path());
        assert_eq!(exes, vec!["game.exe", "bin/launcher.exe"]);

        let info = read_dir_info(dir.path());
        let names: Vec<&str> = info.exes.iter().map(String::as_str).collect();
        assert_eq!(names, vec!["game.exe", "bin/launcher.exe"]);
        let expected: i64 = 5;
        assert_eq!(info.size_bytes, expected);
    }
}

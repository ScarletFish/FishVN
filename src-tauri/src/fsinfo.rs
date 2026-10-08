use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

/// 双击即可运行的扩展名
const EXECUTABLE_EXT: [&str; 3] = ["exe", "bat", "cmd"];

/// 顶层记第 0 层，候选入口最深收到第 2 层，即两层子目录
const MAX_EXE_DEPTH: usize = 3;

/// 候选入口最多留几个，超出部分丢弃
const MAX_EXES: usize = 30;

pub struct DirInfo {
    pub size_bytes: i64,
    /// 候选入口，相对目录，层数浅的在前
    pub exes: Vec<String>,
}

/// 相对路径接上工作目录，`.` 与 `..` 一并归一化，与 Node 的 path.resolve 同义。
/// 不 stat 目标：路径是否真的存在留给 ensure_dir 判，好让调用方先查重再读盘。
pub fn resolve_path(input: &str) -> PathBuf {
    std::path::absolute(input).unwrap_or_else(|_| PathBuf::from(input))
}

/// 目录必须存在且是目录
pub fn ensure_dir(path: &Path) -> Result<()> {
    let meta = fs::metadata(path).map_err(|_| AppError::BadRequest("目录不存在".into()))?;
    if !meta.is_dir() {
        return Err(AppError::BadRequest("所选路径不是目录".into()));
    }
    Ok(())
}

/// 一次遍历同时算出目录大小与候选入口。
/// 大小累加全部深度；候选入口只收 MAX_EXE_DEPTH 层以内。
/// 符号链接既不递归也不计大小：lstat 看来它既不是目录也不是普通文件。
///
/// 候选上限在遍历途中生效，即按遍历顺序截断，之后才排序。
///
/// 大小是全深度遍历，单个目录（几个 GB）可接受。批量遍历要另走异步并带进度。
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
    // 与 path.extname 一致：`.exe` 和 `foo.` 都取不到扩展名
    if dot == 0 || dot + 1 == name.len() {
        return false;
    }
    let ext = name[dot + 1..].to_ascii_lowercase();
    EXECUTABLE_EXT.contains(&ext.as_str())
}

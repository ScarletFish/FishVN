use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{AppError, Result};
use crate::fsinfo::{is_executable, read_exes, resolve_path};

#[derive(Debug, Serialize)]
pub struct PathInspect {
    pub name: String,
    pub dir_path: String,
    pub entry_path: Option<String>,
    pub exes: Vec<String>,
}

pub fn inspect(raw: &str) -> Result<PathInspect> {
    let requested = raw.trim();
    if requested.is_empty() {
        return Err(AppError::BadRequest("路径必填".into()));
    }
    let target = resolve_path(requested);
    let meta = std::fs::metadata(&target).map_err(|_| AppError::NotFound("路径不存在".into()))?;
    if meta.is_file() {
        return inspect_file(&target);
    }
    if meta.is_dir() {
        return inspect_dir(&target);
    }
    Err(AppError::BadRequest("所选路径既不是文件也不是目录".into()))
}

#[derive(Debug, Serialize)]
pub struct ScanCandidate {
    pub name: String,
    pub dir_path: String,
    pub entry_path: Option<String>,
    pub exes: Vec<String>,
    pub registered: bool,
}

pub fn scan_root(
    root_raw: &str,
    registered: &std::collections::HashSet<String>,
    progress: &dyn crate::tasks::Progress,
) -> Result<Vec<ScanCandidate>> {
    if progress.cancelled() {
        return Err(AppError::Cancelled);
    }
    let requested = root_raw.trim();
    if requested.is_empty() {
        return Err(AppError::BadRequest("路径必填".into()));
    }
    let root = resolve_path(requested);
    let meta = std::fs::metadata(&root).map_err(|_| AppError::NotFound("路径不存在".into()))?;
    if !meta.is_dir() {
        return Err(AppError::BadRequest("所选路径不是目录".into()));
    }
    let mut subdirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .map_err(AppError::Io)?
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path())
        .collect();
    subdirs.sort();

    let mut targets: Vec<PathBuf> = Vec::new();
    if !top_level_exes(&root).is_empty() {
        targets.push(root.clone());
    }
    targets.extend(subdirs);

    let total = targets.len() as i64;
    let mut out = Vec::new();
    for (index, dir) in targets.iter().enumerate() {
        if progress.cancelled() {
            return Err(AppError::Cancelled);
        }
        let exes = read_exes(dir);
        progress.report("scan", Some(index as i64 + 1), Some(total));
        if exes.is_empty() {
            continue;
        }
        let dir_path = dir.to_string_lossy().into_owned();
        let name = file_name_of(dir)?;
        let registered_hit = registered.contains(&dir_path.to_ascii_lowercase());
        out.push(ScanCandidate {
            entry_path: best_candidate(&exes, &name),
            exes,
            name,
            dir_path,
            registered: registered_hit,
        });
    }
    Ok(out)
}

fn inspect_file(target: &Path) -> Result<PathInspect> {
    let file_name = file_name_of(target)?;
    if !is_executable(&file_name) {
        return Err(AppError::BadRequest("所选路径不是可执行文件".into()));
    }
    let dir = target
        .parent()
        .ok_or_else(|| AppError::BadRequest("该文件没有父目录".into()))?;
    let rel = relative_entry(&dir.to_path_buf(), target);
    Ok(PathInspect {
        name: file_stem(&file_name),
        dir_path: dir.to_string_lossy().into_owned(),
        entry_path: Some(rel),
        exes: read_exes(dir),
    })
}

fn inspect_dir(target: &Path) -> Result<PathInspect> {
    let dir_name = file_name_of(target)?;
    let exes = read_exes(target);
    let entry = best_candidate(&exes, &dir_name);
    Ok(PathInspect {
        name: dir_name,
        dir_path: target.to_string_lossy().into_owned(),
        entry_path: entry,
        exes,
    })
}

fn best_candidate(exes: &[String], dir_name: &str) -> Option<String> {
    let want = dir_name.to_ascii_lowercase();
    let stem_matches = |rel: &str| {
        let file = rel.rsplit('/').next().unwrap_or(rel);
        file_stem(file).to_ascii_lowercase() == want
    };
    exes.iter()
        .find(|rel| !rel.contains('/') && stem_matches(rel))
        .or_else(|| exes.iter().find(|rel| !rel.contains('/')))
        .or_else(|| exes.first())
        .cloned()
}

fn file_name_of(path: &Path) -> Result<String> {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| AppError::BadRequest("路径没有末段名称".into()))
}

fn file_stem(file: &str) -> String {
    match file.rfind('.') {
        Some(dot) if dot > 0 => file[..dot].to_owned(),
        _ => file.to_owned(),
    }
}

fn top_level_exes(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            (entry.path().is_file() && is_executable(&name)).then_some(name)
        })
        .collect()
}

fn relative_entry(dir: &PathBuf, file: &Path) -> String {
    let rel = file
        .strip_prefix(dir)
        .unwrap_or(file)
        .to_string_lossy()
        .into_owned();
    rel.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 挑目录时同名入口优先_名称取目录名() {
        let dir = tempfile::tempdir().expect("临时目录");
        let root = dir.path().join("MyGame");
        std::fs::create_dir_all(&root).expect("建目录");
        std::fs::write(root.join("other.exe"), b"x").expect("写文件");
        std::fs::write(root.join("MyGame.exe"), b"x").expect("写文件");

        let info = inspect(root.to_str().expect("路径文本")).expect("识别成功");
        assert_eq!(info.name, "MyGame");
        assert_eq!(info.entry_path.as_deref(), Some("MyGame.exe"));
        assert_eq!(info.exes, vec!["MyGame.exe", "other.exe"]);
    }

    #[test]
    fn 挑目录没有同名入口时取根目录第一个() {
        let dir = tempfile::tempdir().expect("临时目录");
        let root = dir.path().join("MyGame");
        std::fs::create_dir_all(&root).expect("建目录");
        std::fs::write(root.join("b.exe"), b"x").expect("写文件");
        std::fs::write(root.join("a.exe"), b"x").expect("写文件");

        let info = inspect(root.to_str().expect("路径文本")).expect("识别成功");
        assert_eq!(info.entry_path.as_deref(), Some("a.exe"));
    }

    #[test]
    fn 同名入口大小写无关_不被字母序兜底抢先() {
        let dir = tempfile::tempdir().expect("临时目录");
        let root = dir.path().join("PCL");
        std::fs::create_dir_all(&root).expect("建目录");
        std::fs::write(root.join("HMCL-3.5.9.exe"), b"x").expect("写文件");
        std::fs::write(root.join("PCL.exe"), b"x").expect("写文件");

        let info = inspect(root.to_str().expect("路径文本")).expect("识别成功");
        assert_eq!(info.entry_path.as_deref(), Some("PCL.exe"));
    }

    #[test]
    fn 挑文件时名称取文件名_目录取父目录() {
        let dir = tempfile::tempdir().expect("临时目录");
        let root = dir.path().join("MyGame");
        std::fs::create_dir_all(&root).expect("建目录");
        let file = root.join("start.exe");
        std::fs::write(&file, b"x").expect("写文件");

        let info = inspect(file.to_str().expect("路径文本")).expect("识别成功");
        assert_eq!(info.name, "start");
        assert_eq!(info.dir_path, root.to_string_lossy());
        assert_eq!(info.entry_path.as_deref(), Some("start.exe"));
    }

    #[test]
    fn 挑非可执行文件报错_路径不存在报错() {
        let dir = tempfile::tempdir().expect("临时目录");
        let file = dir.path().join("readme.txt");
        std::fs::write(&file, b"x").expect("写文件");
        assert!(inspect(file.to_str().expect("路径文本")).is_err());
        assert!(inspect(dir.path().join("没有.exe").to_str().expect("路径文本")).is_err());
        assert!(inspect("").is_err());
    }

    struct StubProgress {
        cancelled: std::sync::atomic::AtomicBool,
    }

    impl StubProgress {
        fn new() -> Self {
            Self {
                cancelled: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    impl crate::tasks::Progress for StubProgress {
        fn report(&self, _phase: &str, _done: Option<i64>, _total: Option<i64>) {}
        fn report_error(&self, _phase: &str, _message: &str) {}
        fn cancelled(&self) -> bool {
            self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    #[test]
    fn 扫描根目录逐个子目录识别_空目录跳过_已登记标记() {
        let dir = tempfile::tempdir().expect("临时目录");
        let root = dir.path().join("Library");
        std::fs::create_dir_all(root.join("GameA")).expect("建目录");
        std::fs::write(root.join("GameA/MyGame.exe"), b"x").expect("写文件");
        std::fs::write(root.join("GameA/other.exe"), b"x").expect("写文件");
        std::fs::create_dir_all(root.join("EmptyDir")).expect("建目录");
        std::fs::create_dir_all(root.join("GameB")).expect("建目录");
        std::fs::write(root.join("GameB/unins000.exe"), b"x").expect("写文件");

        let mut registered = std::collections::HashSet::new();
        registered.insert(root.join("GameA").to_string_lossy().to_ascii_lowercase());

        let progress = StubProgress::new();
        let found = scan_root(root.to_str().expect("路径文本"), &registered, &progress)
            .expect("扫描成功");

        assert_eq!(found.len(), 1, "无候选入口的目录不进清单，根自身无 exe 不成行");
        assert_eq!(found[0].name, "GameA");
        assert!(found[0].registered, "已登记目录要打标");
        assert_eq!(found[0].entry_path.as_deref(), Some("MyGame.exe"));
        assert_eq!(
            found[0].exes,
            vec!["MyGame.exe".to_owned(), "other.exe".to_owned()]
        );
    }

    #[test]
    fn 扫描根自身有入口时排在首位() {
        let dir = tempfile::tempdir().expect("临时目录");
        let root = dir.path().join("PCL");
        std::fs::create_dir_all(root.join("PCL")).expect("建目录");
        std::fs::write(root.join("PCL.exe"), b"x").expect("写文件");
        std::fs::write(root.join("PCL/LatestLaunch.bat"), b"x").expect("写文件");

        let progress = StubProgress::new();
        let found =
            scan_root(root.to_str().expect("路径文本"), &Default::default(), &progress)
                .expect("扫描成功");

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].dir_path, root.to_string_lossy(), "根自身是第一行");
        assert_eq!(found[0].entry_path.as_deref(), Some("PCL.exe"));
        assert_eq!(found[1].entry_path.as_deref(), Some("LatestLaunch.bat"));
    }
}

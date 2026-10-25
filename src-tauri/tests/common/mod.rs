//! 每个测试文件自己开临时库与临时目录树。
//!
//! TempDir 析构时递归删目录；SQLite 的 WAL 附属文件要等连接关闭才不再占用，
//! 所以 Connection 必须活得比 TempDir 短。这里让 Db 持有一个 TempDir，
//! 字段声明顺序即 drop 顺序：conn 先关，dir 后删。
//!
//! 每个测试二进制只用到其中一部分，剩下的成员按需忽略。

#![allow(dead_code)]

use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;
use tempfile::{tempdir, TempDir};

use fishvn::db;
use fishvn::games::{create_game, update_game_state};
use fishvn::installations::{create_installation, update_installation};
use fishvn::types::{InstallationPatch, NewGame, NewInstallation, StatePatch};

pub struct Db {
    conn: Connection,
    dir: TempDir,
}

impl Db {
    pub fn new() -> Self {
        let dir = tempdir().expect("临时目录建不出来");
        let conn = db::open(dir.path()).expect("临时库开不了");
        Self { conn, dir }
    }
}

impl Deref for Db {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        &self.conn
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Shared {
    pub conn: Arc<Mutex<Connection>>,
    dir: TempDir,
}

impl Shared {
    pub fn new() -> Self {
        let dir = tempdir().expect("临时目录建不出来");
        let conn = db::open(dir.path()).expect("临时库开不了");
        Self {
            conn: Arc::new(Mutex::new(conn)),
            dir,
        }
    }

    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> T) -> T {
        let guard: MutexGuard<'_, Connection> = self.conn.lock().expect("库锁被毒化");
        f(guard.deref())
    }
}

pub fn new_game(shared: &Shared, name: &str) -> String {
    shared
        .with(|conn| {
            create_game(
                conn,
                &NewGame {
                    name: name.into(),
                    vndb_id: None,
                    metadata: None,
                },
            )
        })
        .expect("建作品失败")
        .id
}

pub fn add_launch(shared: &Shared, game: &str, dir: &Path, entry: &str) -> String {
    let id = shared
        .with(|conn| {
            create_installation(
                conn,
                &NewInstallation {
                    game_id: game.into(),
                    dir_path: dir.to_string_lossy().into_owned(),
                },
            )
        })
        .expect("建安装失败")
        .id;
    shared
        .with(|conn| {
            update_installation(
                conn,
                &id,
                &InstallationPatch {
                    game_id: None,
                    entry_path: Some(Some(entry.into())),
                    le_launch: None,
                    magpie: None,
                    launch_args: None,
                },
            )
        })
        .expect("设启动入口失败")
        .id
}

pub fn set_default(shared: &Shared, game: &str, installation: String) {
    shared
        .with(|conn| {
            update_game_state(
                conn,
                game,
                &StatePatch {
                    default_installation_id: Some(Some(installation)),
                    ..Default::default()
                },
            )
        })
        .expect("设默认启动失败");
}

/// 建一棵真实的目录树供文件系统相关用例使用。
/// make_dir 每次返回一个全新目录：dir_path 唯一约束会让复用目录的用例互相干扰。
pub struct Tree {
    pub root: PathBuf,
    dir: TempDir,
    seq: u32,
}

impl Tree {
    pub fn new() -> Self {
        let dir = tempdir().expect("临时目录建不出来");
        let root = dir.path().join("library");
        fs::create_dir_all(&root).expect("library 建不出来");
        Self { root, dir, seq: 0 }
    }

    /// files 的键是相对路径，值是文件内容
    pub fn make_dir(&mut self, files: &[(&str, &str)]) -> PathBuf {
        self.seq += 1;
        let dir = self.root.join(format!("case-{}", self.seq));
        for (rel, content) in files {
            let full = dir.join(rel);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("子目录建不出来");
            }
            fs::write(full, content).expect("文件写不出来");
        }
        dir
    }

    pub fn missing(&self) -> PathBuf {
        self.dir.path().join("不存在")
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

/// 错误变体取出来断言用：拿不到变体名就只能断言「是某种错误」，
/// 而 BadRequest / NotFound / Conflict 三者行为不同，混在一起就测不出区别。
pub fn variant(err: &fishvn::error::AppError) -> &'static str {
    use fishvn::error::AppError::*;
    match err {
        BadRequest(_) => "BadRequest",
        NotFound(_) => "NotFound",
        Conflict(_) => "Conflict",
        RateLimited => "RateLimited",
        Upstream(_) => "Upstream",
        Cancelled => "Cancelled",
        Db(_) | Json(_) | Io(_) => "db",
    }
}

pub fn message(err: &fishvn::error::AppError) -> String {
    err.to_string()
}

pub fn ghost() -> &'static str {
    "00000000-0000-0000-0000-000000000000"
}

pub fn path_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

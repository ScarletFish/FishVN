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

use rusqlite::Connection;
use tempfile::{tempdir, TempDir};

use fishvn::db;

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

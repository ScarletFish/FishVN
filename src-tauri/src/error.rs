use std::fmt;

use serde::{Serialize, Serializer};

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    RateLimited,
    Upstream(String),
    Db(rusqlite::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(m) | Self::NotFound(m) | Self::Conflict(m) => f.write_str(m),
            Self::Db(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::RateLimited => f.write_str("VNDB 请求太频繁，已触发限流，请稍后再试"),
            Self::Upstream(m) => write!(f, "VNDB 服务异常：{m}"),
        }
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

const SQLITE_CONSTRAINT_UNIQUE: i32 = 2067;

pub fn write_conflict(err: rusqlite::Error, message: &str) -> AppError {
    if let rusqlite::Error::SqliteFailure(inner, _) = &err {
        if inner.extended_code == SQLITE_CONSTRAINT_UNIQUE {
            return AppError::Conflict(message.to_owned());
        }
    }
    AppError::Db(err)
}

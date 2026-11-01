use std::fmt;

use serde::{Serialize, Serializer};

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    RateLimited,
    Upstream(String),
    Cancelled,
    Db(rusqlite::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "input",
            Self::NotFound(_) => "not-found",
            Self::Conflict(_) => "conflict",
            Self::RateLimited => "rate-limited",
            Self::Upstream(_) => "upstream",
            Self::Cancelled => "cancelled",
            Self::Db(_) => "db",
            Self::Json(_) => "json",
            Self::Io(_) => "io",
        }
    }

    pub fn is_fault(&self) -> bool {
        matches!(
            self,
            Self::Upstream(_) | Self::Db(_) | Self::Json(_) | Self::Io(_)
        )
    }
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
            Self::Cancelled => f.write_str("任务已取消"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 每个变体的类别与故障判定() {
        let cases: Vec<(AppError, &str, bool)> = vec![
            (AppError::BadRequest(String::new()), "input", false),
            (AppError::NotFound(String::new()), "not-found", false),
            (AppError::Conflict(String::new()), "conflict", false),
            (AppError::RateLimited, "rate-limited", false),
            (AppError::Cancelled, "cancelled", false),
            (AppError::Upstream(String::new()), "upstream", true),
            (
                AppError::Db(rusqlite::Error::QueryReturnedNoRows),
                "db",
                true,
            ),
            (
                AppError::Json(serde_json::Error::io(std::io::Error::other("x"))),
                "json",
                true,
            ),
            (AppError::Io(std::io::Error::other("x")), "io", true),
        ];
        for (err, kind, fault) in cases {
            assert_eq!(err.kind(), kind);
            assert_eq!(err.is_fault(), fault, "{kind}");
        }
    }
}

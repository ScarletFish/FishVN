use std::fmt;

#[derive(Debug)]
pub enum AppError {
    /// 输入不合法，对应 400
    BadRequest(String),
    /// 目标不存在，对应 404
    NotFound(String),
    /// 唯一约束撞了，对应 409
    Conflict(String),
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
        }
    }
}

impl std::error::Error for AppError {}

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

/// SQLITE_CONSTRAINT_UNIQUE 的扩展码，唯一约束与其它约束要分开报
const SQLITE_CONSTRAINT_UNIQUE: i32 = 2067;

/// 写入撞唯一约束时转 Conflict，其余错误原样上抛
pub fn write_conflict(err: rusqlite::Error, message: &str) -> AppError {
    if let rusqlite::Error::SqliteFailure(inner, _) = &err {
        if inner.extended_code == SQLITE_CONSTRAINT_UNIQUE {
            return AppError::Conflict(message.to_owned());
        }
    }
    AppError::Db(err)
}

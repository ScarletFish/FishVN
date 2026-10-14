pub mod commands;
pub mod db;
pub mod error;
pub mod fsinfo;
pub mod games;
pub mod installations;
pub mod metadata;
pub mod types;
pub mod vndb;
#[cfg(debug_assertions)]
pub mod devhttp;

/// 主键形态：带连字符的小写 uuid v4。已有行是这个形状，换生成方式要一并迁移
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

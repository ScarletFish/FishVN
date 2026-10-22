pub mod characters;
pub mod commands;
pub mod dashboard;
pub mod db;
#[cfg(debug_assertions)]
pub mod devhttp;
pub mod error;
pub mod fsinfo;
pub mod games;
pub mod installations;
pub mod inspect;
pub mod metadata;
pub mod sessions;
pub mod settings;
pub mod tagdict;
pub mod tasks;
pub mod thumbs;
pub mod types;
pub mod vndb;

#[cfg(not(debug_assertions))]
use tauri::Manager;

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    {
        let _ = app;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| std::io::Error::other("src-tauri 不在项目根下"))?;
        Ok(root.join("data"))
    }
    #[cfg(not(debug_assertions))]
    {
        Ok(app.path().app_data_dir()?)
    }
}

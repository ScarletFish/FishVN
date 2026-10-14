pub mod characters;
pub mod commands;
pub mod db;
pub mod error;
pub mod fsinfo;
pub mod games;
pub mod installations;
pub mod metadata;
pub mod thumbs;
pub mod types;
pub mod vndb;
#[cfg(debug_assertions)]
pub mod devhttp;

#[cfg(not(debug_assertions))]
use tauri::Manager;

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

/// 数据目录解析放在 lib，commands 模块也要调用它。
///
/// 开发期指向仓库根的 data/。打包后仓库根不在用户机器上，改用系统应用数据目录 ——
/// 两者不共用，免得开发数据混进安装版。
///
/// 落在 %USERPROFILE% 下（Windows 上 %APPDATA%\com.fishvn.app），所以在
/// tauri.conf.json 的 assetProtocol.scope "$HOME/**/*" 覆盖内，派生图不必另行授权。
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

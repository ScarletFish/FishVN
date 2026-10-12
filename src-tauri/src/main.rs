// 打包后不再弹控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use fishvn::commands;
use fishvn::db;
use fishvn::vndb::RateLimiter;
use tauri::Manager;

/// 开发期指向仓库根的 data/。打包后仓库根不在用户机器上，改用系统应用数据目录 ——
/// 两者不共用，免得开发数据混进安装版。
fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
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

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let conn = db::open(&data_dir(app.handle())?)?;
            app.manage(Mutex::new(conn));
            app.manage(RateLimiter::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_games,
            commands::get_game,
            commands::create_game,
            commands::update_game,
            commands::delete_game,
            commands::get_game_state,
            commands::update_game_state,
            commands::list_installations,
            commands::get_installation,
            commands::create_installation,
            commands::update_installation,
            commands::delete_installation,
            commands::search_vndb,
            commands::fetch_vn_metadata,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}

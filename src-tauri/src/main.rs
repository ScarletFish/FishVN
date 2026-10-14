// 打包后不再弹控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use fishvn::commands;
use fishvn::db;
use fishvn::vndb::RateLimiter;
use tauri::Manager;

/// 只绑回环地址：本机任何进程都能读写用户的库
#[cfg(debug_assertions)]
fn spawn_dev_channel(
    conn: Arc<Mutex<rusqlite::Connection>>,
    rate: Arc<RateLimiter>,
    app: tauri::AppHandle,
) {
    use fishvn::devhttp::{router, Dev, PORT};

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], PORT));
    tauri::async_runtime::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("开发通道没起来（{addr}）：{e}");
                return;
            }
        };
        println!("开发通道已开： http://{addr}");
        let _ = axum::serve(listener, router(Dev { conn, rate, app })).await;
    });
}

/// 开发期指向仓库根的 data/。打包后仓库根不在用户机器上，改用系统应用数据目录 ——
/// 两者不共用，免得开发数据混进安装版。
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let conn = Arc::new(Mutex::new(db::open(&fishvn::data_dir(app.handle())?)?));
            let rate = Arc::new(RateLimiter::new());
            // 这里是 Arc，命令侧就必须是 State<'_, Arc<...>>。注册值类型与 State 泛型参数
            // 按类型精确匹配，写错编译期不报，只在调用该命令时报 state not managed
            app.manage(conn.clone());
            app.manage(rate.clone());
            #[cfg(debug_assertions)]
            if std::env::var_os("FISHVN_DEV_HTTP").is_some() {
                spawn_dev_channel(conn, rate, app.handle().clone());
            }
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
            commands::get_game_metadata,
            commands::update_game_metadata,
            commands::search_vndb,
            commands::fetch_vn_metadata,
            commands::allow_local_file,
            commands::thumbnail,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}

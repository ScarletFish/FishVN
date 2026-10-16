#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use fishvn::commands;
use fishvn::db;
use fishvn::tasks::{EventLog, Tasks};
use fishvn::vndb::RateLimiter;
use tauri::Manager;

#[cfg(debug_assertions)]
fn spawn_dev_channel(
    conn: Arc<Mutex<rusqlite::Connection>>,
    rate: Arc<RateLimiter>,
    events: Arc<Mutex<EventLog>>,
    tasks: Arc<Tasks>,
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
        let _ = axum::serve(
            listener,
            router(Dev {
                conn,
                rate,
                events,
                tasks,
                app,
            }),
        )
        .await;
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let conn = Arc::new(Mutex::new(db::open(&fishvn::data_dir(app.handle())?)?));
            let rate = Arc::new(RateLimiter::new());
            let events = Arc::new(Mutex::new(EventLog::default()));
            let tasks = Arc::new(Tasks::default());
            app.manage(conn.clone());
            app.manage(rate.clone());
            app.manage(events.clone());
            app.manage(tasks.clone());
            #[cfg(debug_assertions)]
            if std::env::var_os("FISHVN_DEV_HTTP").is_some() {
                spawn_dev_channel(conn, rate, events, tasks, app.handle().clone());
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
            commands::list_game_characters,
            commands::save_game_characters,
            commands::search_vndb,
            commands::fetch_vn_metadata,
            commands::cancel_task,
            commands::allow_local_file,
            commands::thumbnail,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use fishvn::commands;
use fishvn::db;
use fishvn::tasks::{EventLog, Tasks};
use fishvn::vndb::RateLimiter;
use tauri::Manager;
use tauri_plugin_log::log;

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
    let started = std::time::Instant::now();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .max_file_size(5 * 1024 * 1024)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .build(),
        )
        .setup(move |app| {
            let default_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                log::error!(target: "panic", "{info}");
                default_hook(info);
            }));
            let dir = fishvn::data_dir(app.handle())?;
            fishvn::settings::init(&dir);
            let window_color = match fishvn::settings::current().theme {
                fishvn::settings::Theme::Dark => tauri::window::Color(0x12, 0x12, 0x12, 0xff),
                fishvn::settings::Theme::Light => tauri::window::Color(0xf1, 0xf0, 0xec, 0xff),
            };
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_background_color(Some(window_color));
            }
            let fallback = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1500));
                let Some(window) = fallback.get_webview_window("main") else {
                    return;
                };
                if window.is_visible().unwrap_or(true) {
                    return;
                }
                log::warn!(target: "startup", "兜底显示窗口：前端未在 1.5 秒内显示");
                let _ = window.show();
                let _ = window.set_focus();
            });
            let conn = db::open(&dir)?;
            fishvn::sessions::sweep(&conn)?;
            let conn = Arc::new(Mutex::new(conn));
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
            log::info!(target: "startup", "后端就绪：{} ms", started.elapsed().as_millis());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_games,
            commands::dashboard_stats,
            commands::dashboard_recent,
            commands::dashboard_tag_cloud,
            commands::dashboard_monthly_activity,
            commands::dashboard_playtime_top,
            commands::dashboard_release_eras,
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
            commands::save_installations,
            commands::launch_game,
            commands::list_active_sessions,
            commands::get_game_metadata,
            commands::update_game_metadata,
            commands::list_game_characters,
            commands::save_game_characters,
            commands::search_vndb,
            commands::fetch_vn_metadata,
            commands::cancel_task,
            commands::allow_local_file,
            commands::thumbnail,
            commands::tag_dictionary,
            commands::get_config,
            commands::update_config,
            commands::inspect_path,
            commands::scan_dir,
            commands::log_frontend,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}

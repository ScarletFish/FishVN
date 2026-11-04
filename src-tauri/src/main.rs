#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use fishvn::commands;
use fishvn::db;
use fishvn::tasks::{EventLog, Tasks};
use fishvn::vndb::RateLimiter;
use tauri::{Emitter, Manager};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
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

fn log_level() -> log::LevelFilter {
    std::env::var("FISHVN_LOG_LEVEL")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(log::LevelFilter::Info)
}

fn main() {
    let started = std::time::Instant::now();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log_level())
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
            let main_window = app.get_webview_window("main");
            let dark = match fishvn::settings::current().theme {
                fishvn::settings::Theme::Dark => true,
                fishvn::settings::Theme::Light => false,
                fishvn::settings::Theme::System => main_window
                    .as_ref()
                    .and_then(|w| w.theme().ok())
                    .is_some_and(|t| t == tauri::Theme::Dark),
            };
            let window_color = if dark {
                tauri::window::Color(0x12, 0x12, 0x12, 0xff)
            } else {
                tauri::window::Color(0xf1, 0xf0, 0xec, 0xff)
            };
            if let Some(window) = main_window {
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

            let handle = app.handle().clone();
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        match fishvn::settings::current().quit_behavior {
                            fishvn::settings::QuitBehavior::MinimizeToTray => {
                                let _ = handle.get_webview_window("main").map(|w| w.hide());
                            }
                            fishvn::settings::QuitBehavior::Quit => {
                                let _ = handle.emit("app-quit-requested", ());
                            }
                        }
                    }
                });
            }
            if let Some(icon) = app.default_window_icon() {
                let icon = icon.clone();
                let show_item = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
                let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
                let tray = TrayIconBuilder::new()
                    .icon(icon)
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                            let _ = app.emit("app-quit-requested", ());
                        }
                        _ => {}
                    })
                    .build(app)?;
                let _ = app.manage(tray);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_games,
            commands::query_games,
            commands::query_characters,
            commands::list_staff,
            commands::update_staff,
            commands::merge_staff,
            commands::dashboard_stats,
            commands::dashboard_recent,
            commands::dashboard_tag_cloud,
            commands::library_tags,
            commands::library_traits,
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
            commands::game_play_log,
            commands::get_game_metadata,
            commands::update_game_metadata,
            commands::list_game_characters,
            commands::save_game_characters,
            commands::search_vndb,
            commands::fetch_vn_metadata,
            commands::cancel_task,
            commands::events_since,
            commands::allow_local_file,
            commands::thumbnail,
            commands::tag_dictionary,
            commands::get_config,
            commands::update_config,
            commands::resolve_app_quit,
            commands::test_vndb_token,
            commands::sync_vndb,
            commands::apply_sync_plan,
            commands::start_backfill,
            commands::backfill_status,
            commands::pause_backfill,
            commands::resume_backfill,
            commands::inspect_path,
            commands::scan_dir,
            commands::log_frontend,
            commands::path_exists,
            commands::create_backup,
            commands::list_backups,
            commands::restore_backup,
            commands::delete_backup,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}

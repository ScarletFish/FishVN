use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;
use tauri::{Manager, State};

use crate::characters;
use crate::dashboard::{self, ActivityPoint, DashboardStats, EraPoint, PlaytimeRank, RecentPlay, TagCount};
use crate::db;
use crate::error::{AppError, Result};
use crate::games;
use crate::installations;
use crate::inspect;
use crate::inspect::{PathInspect, ScanCandidate};
use crate::metadata;
use crate::sessions;
use crate::settings::{self, AppConfig, ConfigPatch};
use crate::tasks::{EventLog, Task, Tasks};
use crate::thumbs;
use crate::types::{
    Game, GameCharacter, GameListItem, GameMetadata, GamePatch, Installation, InstallationInput,
    InstallationPatch, MetadataPatch, NewGame, NewInstallation, PlaySession, StatePatch,
    UserGameState,
};
use crate::vndb::{self, RateLimiter, VnSearchHit, VnSync};
use tauri_plugin_log::log;

type Shared<'a> = State<'a, Arc<Mutex<Connection>>>;

fn lock<'a>(shared: &'a Shared<'_>) -> Result<MutexGuard<'a, Connection>> {
    db::lock(shared.inner())
}

#[tauri::command]
pub fn list_games(shared: Shared<'_>) -> Result<Vec<GameListItem>> {
    games::list_games(&*lock(&shared)?)
}

#[tauri::command]
pub fn dashboard_stats(shared: Shared<'_>) -> Result<DashboardStats> {
    let config = settings::current();
    let week_start = crate::now() - 7 * 24 * 3600;
    dashboard::stats(&*lock(&shared)?, config.time_metric, week_start)
}

#[tauri::command]
pub fn dashboard_recent(shared: Shared<'_>) -> Result<Vec<RecentPlay>> {
    let config = settings::current();
    dashboard::recent(&*lock(&shared)?, config.time_metric, 6)
}

#[tauri::command]
pub fn dashboard_tag_cloud(shared: Shared<'_>) -> Result<Vec<TagCount>> {
    dashboard::tag_cloud(&*lock(&shared)?, 40)
}

#[tauri::command]
pub fn dashboard_playtime_top(shared: Shared<'_>) -> Result<Vec<PlaytimeRank>> {
    dashboard::playtime_top(&*lock(&shared)?, 6)
}

#[tauri::command]
pub fn dashboard_release_eras(shared: Shared<'_>) -> Result<Vec<EraPoint>> {
    dashboard::release_eras(&*lock(&shared)?)
}

#[tauri::command]
pub fn dashboard_monthly_activity(shared: Shared<'_>, months: i64) -> Result<Vec<ActivityPoint>> {
    dashboard::monthly_activity(&*lock(&shared)?, months)
}

#[tauri::command]
pub fn get_game(shared: Shared<'_>, id: String) -> Result<Game> {
    games::get_game(&*lock(&shared)?, &id)
}

#[tauri::command]
pub fn create_game(shared: Shared<'_>, input: NewGame) -> Result<Game> {
    games::create_game(&*lock(&shared)?, &input)
}

#[tauri::command]
pub fn update_game(shared: Shared<'_>, id: String, patch: GamePatch) -> Result<Game> {
    games::update_game(&*lock(&shared)?, &id, &patch)
}

#[tauri::command]
pub fn delete_game(shared: Shared<'_>, id: String) -> Result<()> {
    games::delete_game(&*lock(&shared)?, &id)
}

#[tauri::command]
pub fn get_game_state(shared: Shared<'_>, id: String) -> Result<UserGameState> {
    games::get_game_state(&*lock(&shared)?, &id)
}

#[tauri::command]
pub fn update_game_state(
    shared: Shared<'_>,
    id: String,
    patch: StatePatch,
) -> Result<UserGameState> {
    games::update_game_state(&*lock(&shared)?, &id, &patch)
}

#[tauri::command]
pub fn list_installations(
    shared: Shared<'_>,
    game_id: Option<String>,
) -> Result<Vec<Installation>> {
    installations::list_installations(&*lock(&shared)?, game_id.as_deref())
}

#[tauri::command]
pub fn get_installation(shared: Shared<'_>, id: String) -> Result<Installation> {
    installations::get_installation(&*lock(&shared)?, &id)
}

#[tauri::command]
pub fn create_installation(shared: Shared<'_>, input: NewInstallation) -> Result<Installation> {
    installations::create_installation(&*lock(&shared)?, &input)
}

#[tauri::command]
pub fn update_installation(
    shared: Shared<'_>,
    id: String,
    patch: InstallationPatch,
) -> Result<Installation> {
    installations::update_installation(&*lock(&shared)?, &id, &patch)
}

#[tauri::command]
pub fn delete_installation(shared: Shared<'_>, id: String) -> Result<()> {
    installations::delete_installation(&*lock(&shared)?, &id)
}

#[tauri::command]
pub fn save_installations(
    shared: Shared<'_>,
    game_id: String,
    entries: Vec<InstallationInput>,
) -> Result<Vec<Installation>> {
    installations::save_installations(&*lock(&shared)?, &game_id, &entries)
}

#[tauri::command]
pub fn launch_game(shared: Shared<'_>, id: String) -> Result<PlaySession> {
    sessions::launch(shared.inner().clone(), &id)
}

#[tauri::command]
pub fn list_active_sessions(shared: Shared<'_>) -> Result<Vec<PlaySession>> {
    sessions::list_active(&*lock(&shared)?)
}

#[tauri::command]
pub fn get_game_metadata(shared: Shared<'_>, id: String) -> Result<Option<GameMetadata>> {
    metadata::get(&*lock(&shared)?, &id)
}

#[tauri::command]
pub fn update_game_metadata(
    shared: Shared<'_>,
    id: String,
    patch: MetadataPatch,
) -> Result<GameMetadata> {
    metadata::upsert(&*lock(&shared)?, &id, &patch)
}

#[tauri::command]
pub fn save_game_characters(
    shared: Shared<'_>,
    id: String,
    characters: Vec<GameCharacter>,
) -> Result<Vec<GameCharacter>> {
    characters::replace_all(&*lock(&shared)?, &id, &characters)
}

#[tauri::command]
pub fn list_game_characters(shared: Shared<'_>, id: String) -> Result<Vec<GameCharacter>> {
    characters::list(&*lock(&shared)?, &id)
}

#[tauri::command]
pub async fn search_vndb(
    rate: State<'_, Arc<RateLimiter>>,
    keyword: String,
) -> Result<Vec<VnSearchHit>> {
    let keyword = vndb::validate_keyword(&keyword)?;
    rate.acquire().await;
    vndb::search(vndb::http(), &keyword).await
}

pub const VNDB_SYNC: &str = "vndb-sync";

#[tauri::command]
pub async fn fetch_vn_metadata(
    app: tauri::AppHandle,
    rate: State<'_, Arc<RateLimiter>>,
    tasks: State<'_, Arc<Tasks>>,
    log: State<'_, Arc<Mutex<EventLog>>>,
    vndb_id: String,
) -> Result<VnSync> {
    let vndb_id = vndb::validate_vn_id(&vndb_id)?;
    rate.acquire().await;
    rate.acquire().await;
    let dir =
        crate::data_dir(&app).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    let task = Task::new(
        VNDB_SYNC,
        app,
        log.inner().clone(),
        tasks.register(VNDB_SYNC),
    );
    let started = std::time::Instant::now();
    log::info!("VNDB 同步开始 {vndb_id}");
    let result = vndb::fetch_sync(vndb::http(), &vndb_id, &dir, &task).await;
    match &result {
        Ok(_) => log::info!("VNDB 同步完成 {vndb_id}：{} ms", started.elapsed().as_millis()),
        Err(AppError::Cancelled) => {
            log::info!("VNDB 同步取消 {vndb_id}：{} ms", started.elapsed().as_millis())
        }
        Err(e) => log::warn!("VNDB 同步失败 {vndb_id}：{e}"),
    }
    result
}

#[tauri::command]
pub fn cancel_task(tasks: State<'_, Arc<Tasks>>, task: String) -> bool {
    tasks.cancel(&task)
}

#[tauri::command]
pub async fn inspect_path(path: String) -> Result<PathInspect> {
    tauri::async_runtime::spawn_blocking(move || inspect::inspect(&path))
        .await
        .map_err(|e| AppError::Upstream(format!("识别任务失败：{e}")))?
}

pub const SCAN: &str = "scan";

#[tauri::command]
pub async fn scan_dir(
    app: tauri::AppHandle,
    shared: State<'_, Arc<Mutex<Connection>>>,
    tasks: State<'_, Arc<Tasks>>,
    log: State<'_, Arc<Mutex<EventLog>>>,
    path: String,
) -> Result<Vec<ScanCandidate>> {
    let task = Task::new(SCAN, app, log.inner().clone(), tasks.register(SCAN));
    let conn = shared.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let started = std::time::Instant::now();
        log::info!("目录扫描开始 {path}");
        let registered = {
            let guard = db::lock(&conn)?;
            installations::registered_dirs(&guard)?
        };
        let result = inspect::scan_root(&path, &registered, &task);
        match &result {
            Ok(found) => log::info!(
                "目录扫描完成 {path}：{} 个候选，{} ms",
                found.len(),
                started.elapsed().as_millis()
            ),
            Err(AppError::Cancelled) => {
                log::info!("目录扫描取消 {path}：{} ms", started.elapsed().as_millis())
            }
            Err(e) => log::warn!("目录扫描失败 {path}：{e}"),
        }
        result
    })
    .await
    .map_err(|e| AppError::Upstream(format!("扫描任务失败：{e}")))?
}

const FRONTEND_LOG_MAX_CHARS: usize = 2000;

#[tauri::command]
pub fn log_frontend(level: String, message: String) {
    let message = if message.chars().count() > FRONTEND_LOG_MAX_CHARS {
        let truncated: String = message.chars().take(FRONTEND_LOG_MAX_CHARS).collect();
        format!("{truncated}…")
    } else {
        message
    };
    match level.as_str() {
        "error" => log::error!(target: "webview", "{message}"),
        "warn" => log::warn!(target: "webview", "{message}"),
        _ => log::info!(target: "webview", "{message}"),
    }
}

#[tauri::command]
pub fn thumbnail(app: tauri::AppHandle, src: String) -> Result<String> {
    let dir =
        crate::data_dir(&app).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    let path = thumbs::cached(&src, &dir)?;
    Ok(path.to_string_lossy().into_owned())
}

fn data_dir_or_err(app: &tauri::AppHandle) -> Result<PathBuf> {
    crate::data_dir(app).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))
}

#[tauri::command]
pub fn get_config(_app: tauri::AppHandle) -> Result<AppConfig> {
    Ok(settings::current())
}

#[tauri::command]
pub fn update_config(app: tauri::AppHandle, patch: ConfigPatch) -> Result<AppConfig> {
    settings::patch(&data_dir_or_err(&app)?, patch)
}

#[tauri::command]
pub fn allow_local_file(app: tauri::AppHandle, path: String) -> Result<()> {
    let path = PathBuf::from(path.trim());
    let meta = std::fs::metadata(&path).map_err(|_| AppError::BadRequest("文件不存在".into()))?;
    if !meta.is_file() {
        return Err(AppError::BadRequest("不是文件".into()));
    }
    app.asset_protocol_scope()
        .allow_file(&path)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))
}

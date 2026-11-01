use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;
use tauri::{Emitter, Manager, State};

use crate::backfill;
use crate::backup;
use crate::characters;
use crate::dashboard::{self, ActivityPoint, DashboardStats, EraPoint, LibraryTag, LibraryTrait, PlaytimeRank, RecentPlay, TagCloudView};
use crate::db;
use crate::error::{AppError, Result};
use crate::games;
use crate::installations;
use crate::inspect;
use crate::inspect::{PathInspect, ScanCandidate};
use crate::metadata;
use crate::sessions;
use crate::settings::{self, AppConfig, ConfigPatch};
use crate::sync;
use crate::tasks::{EventLog, Task, Tasks};
use crate::thumbs;
use crate::types::{
    Game, GameCharacter, GameListItem, GameMetadata, GamePatch, Installation, InstallationInput,
    InstallationPatch, MetadataPatch, NewGame, NewInstallation, PlaySession, StatePatch,
    TaskEvent, UserGameState,
};
use crate::vndb::{self, RateLimiter, VnSearchHit, VnSync};
use tauri_plugin_log::log;

type Shared<'a> = State<'a, Arc<Mutex<Connection>>>;

fn lock<'a>(shared: &'a Shared<'_>) -> Result<MutexGuard<'a, Connection>> {
    db::lock(shared.inner())
}

fn trace<T>(cmd: &str, result: Result<T>) -> Result<T> {
    if let Err(e) = &result {
        if e.is_fault() {
            log::warn!(target: "command", "{cmd} 失败：[{}] {e}", e.kind());
        } else {
            log::debug!(target: "command", "{cmd} 返回：[{}] {e}", e.kind());
        }
    }
    result
}

fn traced<T>(cmd: &str, run: impl FnOnce() -> Result<T>) -> Result<T> {
    trace(cmd, run())
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
    dashboard::recent(&*lock(&shared)?, 6)
}

#[tauri::command]
pub fn dashboard_tag_cloud(shared: Shared<'_>) -> Result<TagCloudView> {
    let config = settings::current();
    dashboard::tag_cloud(
        &*lock(&shared)?,
        40,
        config.hide_r18_tags,
        config.spoiler_level,
        &config.excluded_tags,
    )
}

#[tauri::command]
pub fn library_tags(shared: Shared<'_>) -> Result<Vec<LibraryTag>> {
    let config = settings::current();
    dashboard::library_tags(
        &*lock(&shared)?,
        config.hide_r18_tags,
        config.spoiler_level,
        &config.excluded_tags,
    )
}

#[tauri::command]
pub fn library_traits(shared: Shared<'_>) -> Result<Vec<LibraryTrait>> {
    dashboard::library_traits(&*lock(&shared)?)
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
    traced("create_game", || {
        games::create_game(&*lock(&shared)?, &input)
    })
}

#[tauri::command]
pub fn update_game(shared: Shared<'_>, id: String, patch: GamePatch) -> Result<Game> {
    traced("update_game", || {
        games::update_game(&*lock(&shared)?, &id, &patch)
    })
}

#[tauri::command]
pub fn delete_game(shared: Shared<'_>, id: String) -> Result<()> {
    traced("delete_game", || games::delete_game(&*lock(&shared)?, &id))
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
    traced("update_game_state", || {
        games::update_game_state(&*lock(&shared)?, &id, &patch)
    })
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
    traced("create_installation", || {
        installations::create_installation(&*lock(&shared)?, &input)
    })
}

#[tauri::command]
pub fn update_installation(
    shared: Shared<'_>,
    id: String,
    patch: InstallationPatch,
) -> Result<Installation> {
    traced("update_installation", || {
        installations::update_installation(&*lock(&shared)?, &id, &patch)
    })
}

#[tauri::command]
pub fn delete_installation(shared: Shared<'_>, id: String) -> Result<()> {
    traced("delete_installation", || {
        installations::delete_installation(&*lock(&shared)?, &id)
    })
}

#[tauri::command]
pub fn save_installations(
    shared: Shared<'_>,
    game_id: String,
    entries: Vec<InstallationInput>,
) -> Result<Vec<Installation>> {
    traced("save_installations", || {
        installations::save_installations(&*lock(&shared)?, &game_id, &entries)
    })
}

#[tauri::command]
pub fn launch_game(shared: Shared<'_>, id: String, app: tauri::AppHandle) -> Result<PlaySession> {
    traced("launch_game", || {
        sessions::launch(shared.inner().clone(), &id, Some(app))
    })
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
    traced("update_game_metadata", || {
        metadata::upsert(&*lock(&shared)?, &id, &patch)
    })
}

#[tauri::command]
pub fn save_game_characters(
    shared: Shared<'_>,
    id: String,
    characters: Vec<GameCharacter>,
) -> Result<Vec<GameCharacter>> {
    traced("save_game_characters", || {
        characters::replace_all(&*lock(&shared)?, &id, &characters)
    })
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
pub const SYNC_PROGRESS: &str = "vndb-sync-progress";

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
        0,
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
    let task = Task::new(SCAN, app, log.inner().clone(), tasks.register(SCAN), 0);
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

#[tauri::command]
pub fn tag_dictionary() -> Result<crate::tagdict::TagDictionary> {
    Ok(crate::tagdict::dictionary())
}

fn data_dir_or_err(app: &tauri::AppHandle) -> Result<PathBuf> {
    crate::data_dir(app).map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))
}

#[tauri::command]
pub fn get_config(_app: tauri::AppHandle) -> Result<settings::ConfigView> {
    Ok(settings::view())
}

#[tauri::command]
pub fn update_config(app: tauri::AppHandle, patch: ConfigPatch) -> Result<AppConfig> {
    traced("update_config", || {
        settings::patch(&data_dir_or_err(&app)?, patch)
    })
}

#[tauri::command]
pub fn resolve_app_quit(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub async fn test_vndb_token(
    token: Option<String>,
    rate: State<'_, Arc<RateLimiter>>,
) -> Result<bool> {
    let resolved = match token.as_deref().map(str::trim) {
        Some(t) if !t.is_empty() => t.to_owned(),
        _ => settings::current()
            .vndb_token_obf
            .as_deref()
            .and_then(settings::deobfuscate_token)
            .ok_or_else(|| AppError::BadRequest("还没有可测试的 VNDB Token".into()))?,
    };
    rate.acquire().await;
    vndb::test_token(vndb::http(), &resolved).await
}

#[tauri::command]
pub async fn sync_vndb(
    app: tauri::AppHandle,
    log: State<'_, Arc<Mutex<EventLog>>>,
    shared: State<'_, Arc<Mutex<Connection>>>,
    rate: State<'_, Arc<RateLimiter>>,
) -> Result<sync::SyncPlan> {
    let result = sync::plan(shared.inner().clone(), rate.inner(), |phase, done, total| {
        let event = TaskEvent {
            phase: phase.to_owned(),
            done,
            total,
            error: None,
            run_id: 0,
            game_id: None,
        };
        let logged = match log.inner().lock() {
            Ok(mut log) => log.push(SYNC_PROGRESS, event),
            Err(_) => return,
        };
        let _ = app.emit(SYNC_PROGRESS, logged.event);
    })
    .await;
    trace("sync_vndb", result)
}

#[tauri::command]
pub async fn apply_sync_plan(
    app: tauri::AppHandle,
    log: State<'_, Arc<Mutex<EventLog>>>,
    shared: State<'_, Arc<Mutex<Connection>>>,
    rate: State<'_, Arc<RateLimiter>>,
    decision: sync::PlanDecision,
) -> Result<sync::ApplyReport> {
    let result: Result<sync::ApplyReport> = async {
        let pushes = decision.pushes.clone();
        let mut upload = None;
        if !pushes.is_empty() {
            let config = crate::settings::current();
            let token = config
                .vndb_token_obf
                .as_deref()
                .and_then(crate::settings::deobfuscate_token)
                .ok_or_else(|| AppError::BadRequest("还没有保存 VNDB 令牌".into()))?;
            let http = crate::vndb::http();
            if !crate::vndb::has_listwrite(http, &token).await? {
                return Err(AppError::BadRequest(
                    "Token 没有写入权限，无法上传到 VNDB".into(),
                ));
            }
            let labels = crate::vndb::fetch_label_map(http, &token).await?;
            upload = Some((token, labels));
        }
        let conn = shared.inner().clone();
        let apply_conn = conn.clone();
        let mut report = tauri::async_runtime::spawn_blocking(move || {
            let guard = crate::db::lock(&apply_conn)?;
            sync::apply(&guard, &decision)
        })
        .await
        .map_err(|e| AppError::Upstream(format!("同步应用失败：{e}")))??;

        let Some((token, labels)) = upload else {
            return Ok(report);
        };
        let http = crate::vndb::http();
        report.pushed = sync::push(conn, http, &token, &labels, &pushes, rate.inner(), |done, total| {
            let event = TaskEvent {
                phase: "push".to_owned(),
                done: Some(done as i64),
                total: Some(total as i64),
                error: None,
                run_id: 0,
                game_id: None,
            };
            let logged = match log.inner().lock() {
                Ok(mut log) => log.push(SYNC_PROGRESS, event),
                Err(_) => return,
            };
            let _ = app.emit(SYNC_PROGRESS, logged.event);
        })
        .await
        .map_err(|e| AppError::Upstream(format!("本地更改已应用，但写入 VNDB 时中断：{e}")))?;
        Ok(report)
    }
    .await;
    trace("apply_sync_plan", result)
}

#[tauri::command]
pub async fn start_backfill(
    app: tauri::AppHandle,
    shared: State<'_, Arc<Mutex<Connection>>>,
    rate: State<'_, Arc<RateLimiter>>,
    events: State<'_, Arc<Mutex<EventLog>>>,
    tasks: State<'_, Arc<Tasks>>,
    game_ids: Option<Vec<String>>,
) -> Result<backfill::BackfillReport> {
    traced("start_backfill", || {
        if backfill::status().running {
            return Err(AppError::Conflict("补全任务正在进行中".into()));
        }
        let items = {
            let guard = lock(&shared)?;
            match &game_ids {
                Some(ids) if !ids.is_empty() => backfill::items_by_ids(&guard, ids)?,
                _ => backfill::scan(&guard)?,
            }
        };
        let total = items.len();
        let run_id = backfill::reset(total as i64);
        tauri::async_runtime::spawn(backfill::run(
            shared.inner().clone(),
            rate.inner().clone(),
            app,
            events.inner().clone(),
            tasks.inner().clone(),
            run_id,
            items,
        ));
        Ok(backfill::BackfillReport {
            total,
            ..Default::default()
        })
    })
}

#[tauri::command]
pub fn backfill_status() -> backfill::BackfillStatus {
    backfill::status()
}

#[tauri::command]
pub fn pause_backfill() -> backfill::BackfillStatus {
    backfill::set_paused(true)
}

#[tauri::command]
pub fn resume_backfill() -> backfill::BackfillStatus {
    backfill::set_paused(false)
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

#[tauri::command]
pub fn path_exists(path: String) -> Result<bool> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }
    Ok(Path::new(trimmed).is_file())
}

#[tauri::command]
pub fn create_backup(shared: Shared<'_>, app: tauri::AppHandle) -> Result<backup::BackupMeta> {
    traced("create_backup", || {
        backup::create_backup(shared.inner(), &data_dir_or_err(&app)?)
    })
}

#[tauri::command]
pub fn list_backups(app: tauri::AppHandle) -> Result<Vec<backup::BackupMeta>> {
    backup::list_backups(&data_dir_or_err(&app)?)
}

#[tauri::command]
pub fn restore_backup(shared: Shared<'_>, app: tauri::AppHandle, id: String) -> Result<()> {
    traced("restore_backup", || {
        backup::restore_backup(shared.inner(), &data_dir_or_err(&app)?, &id)
    })
}

#[tauri::command]
pub fn delete_backup(app: tauri::AppHandle, id: String) -> Result<()> {
    traced("delete_backup", || {
        backup::delete_backup(&data_dir_or_err(&app)?, &id)
    })
}

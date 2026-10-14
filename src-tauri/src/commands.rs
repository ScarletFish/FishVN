//! Tauri 命令层：只做转发，业务逻辑全在 games / installations 里。
//!
//! Connection 是 Send 但不是 Sync，所以用 Mutex 包起来交给 State 管。
//! 锁毒化之后每次调用都会失败，转成错误上报而不是 unwrap。

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;
use tauri::{Manager, State};

use crate::error::{AppError, Result};
use crate::games;
use crate::installations;
use crate::metadata;
use crate::types::{
    Game, GameListItem, GameMetadata, GamePatch, Installation, InstallationPatch, MetadataPatch,
    NewGame, NewInstallation, StatePatch, UserGameState,
};
use crate::thumbs;
use crate::vndb::{self, RateLimiter, VnMetadata, VnSearchHit};

type Shared<'a> = State<'a, Arc<Mutex<Connection>>>;

fn lock<'a>(shared: &'a Shared<'_>) -> Result<MutexGuard<'a, Connection>> {
    shared
        .lock()
        .map_err(|_| AppError::Io(std::io::Error::other("数据库锁已毒化，之前的命令 panic 了")))
}

#[tauri::command]
pub fn list_games(shared: Shared<'_>) -> Result<Vec<GameListItem>> {
    games::list_games(&*lock(&shared)?)
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

/// 不取 Connection 锁：那是全局单锁，跨网络 I/O 持着会卡住所有库操作
#[tauri::command]
pub async fn search_vndb(
    rate: State<'_, Arc<RateLimiter>>,
    keyword: String,
) -> Result<Vec<VnSearchHit>> {
    let keyword = vndb::validate_keyword(&keyword)?;
    rate.acquire().await;
    vndb::search(vndb::http(), &keyword).await
}

#[tauri::command]
pub async fn fetch_vn_metadata(
    app: tauri::AppHandle,
    rate: State<'_, Arc<RateLimiter>>,
    vndb_id: String,
) -> Result<VnMetadata> {
    let vndb_id = vndb::validate_vn_id(&vndb_id)?;
    rate.acquire().await;
    let dir = crate::data_dir(&app)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    vndb::fetch_vn(vndb::http(), &vndb_id, &dir).await
}

#[tauri::command]
pub fn thumbnail(app: tauri::AppHandle, src: String) -> Result<String> {
    let dir = crate::data_dir(&app)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    let path = thumbs::cached(&src, &dir)?;
    Ok(path.to_string_lossy().into_owned())
}

/// dialog 插件只放行 fs_scope，不碰 asset 协议，所以本地封面渲染前要补一次授权。
/// 授权只活在本次进程，重启后靠 tauri.conf.json 的静态 scope 兜底。
#[tauri::command]
pub fn allow_local_file(app: tauri::AppHandle, path: String) -> Result<()> {
    let path = PathBuf::from(path.trim());
    let meta = std::fs::metadata(&path)
        .map_err(|_| AppError::BadRequest("文件不存在".into()))?;
    if !meta.is_file() {
        return Err(AppError::BadRequest("不是文件".into()));
    }
    app.asset_protocol_scope()
        .allow_file(&path)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))
}

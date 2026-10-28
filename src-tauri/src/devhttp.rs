use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::{Query, State as AxumState};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::Stream;
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, Result};
use crate::tasks::{EventLog, LoggedEvent, Task, Tasks};
use crate::types::{GameCharacter, InstallationInput, TaskEvent};
use crate::vndb::RateLimiter;
use crate::{characters, commands, backfill, backup, dashboard, games, installations, metadata, sessions, settings, vndb};

pub const PORT: u16 = 4546;

const TAIL_TICK: Duration = Duration::from_millis(100);

const KEEP_ALIVE: Duration = Duration::from_secs(15);

struct Fail(StatusCode, String);

impl axum::response::IntoResponse for Fail {
    fn into_response(self) -> axum::response::Response {
        axum::response::IntoResponse::into_response((self.0, self.1))
    }
}

impl From<AppError> for Fail {
    fn from(e: AppError) -> Self {
        Fail(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

type Reply = std::result::Result<Json<Value>, Fail>;

#[derive(Clone)]
pub struct Dev {
    pub conn: Arc<Mutex<Connection>>,
    pub rate: Arc<RateLimiter>,
    pub app: tauri::AppHandle,
    pub events: Arc<Mutex<EventLog>>,
    pub tasks: Arc<Tasks>,
}

#[derive(Default, serde::Deserialize)]
pub struct Args {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub input: Option<Value>,
    #[serde(default)]
    pub patch: Option<Value>,
    #[serde(default)]
    pub game_id: Option<String>,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub vndb_id: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub config: Option<Value>,
    #[serde(default)]
    pub characters: Option<Value>,
    #[serde(default)]
    pub entries: Option<Value>,
    #[serde(default)]
    pub task: Option<String>,
    #[serde(default)]
    pub months: Option<i64>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub statuses: Option<Value>,
    #[serde(default)]
    pub ratings: Option<Value>,
    #[serde(default)]
    pub decision: Option<Value>,
    #[serde(default)]
    pub game_ids: Option<Vec<String>>,
}

impl Args {
    fn need<'a>(&'a self, slot: Option<&'a String>, name: &str) -> Result<&'a str> {
        slot.map(|s| s.as_str())
            .ok_or_else(|| AppError::BadRequest(format!("缺少 {name}")))
    }

    pub fn id(&self) -> Result<&str> {
        self.need(self.id.as_ref(), "id")
    }

    pub fn keyword(&self) -> Result<&str> {
        self.need(self.keyword.as_ref(), "keyword")
    }

    pub fn vndb_id(&self) -> Result<&str> {
        self.need(self.vndb_id.as_ref(), "vndb_id")
    }

    pub fn parse<T: DeserializeOwned>(&self, slot: Option<&Value>, name: &str) -> Result<T> {
        let raw = slot.ok_or_else(|| AppError::BadRequest(format!("缺少 {name}")))?;
        serde_json::from_value(raw.clone()).map_err(AppError::from)
    }

    pub fn path(&self) -> Result<&str> {
        self.need(self.path.as_ref(), "path")
    }

    pub fn task(&self) -> Result<&str> {
        self.need(self.task.as_ref(), "task")
    }
}

fn finish<T: Serialize>(r: Result<T>) -> Reply {
    r.map(|v| Json(serde_json::to_value(v).unwrap_or(Value::Null)))
        .map_err(Fail::from)
}

fn run<T: Serialize>(dev: &Dev, f: impl FnOnce(&Connection) -> Result<T>) -> Reply {
    let guard = dev
        .conn
        .lock()
        .map_err(|_| Fail::from(AppError::Io(std::io::Error::other("数据库锁已毒化"))))?;
    finish(f(&guard))
}

macro_rules! handler {
    ($name:ident, |$conn:ident, $a:ident| $body:expr) => {
        async fn $name(AxumState(dev): AxumState<Dev>, Json($a): Json<Args>) -> Reply {
            run(&dev, |$conn| -> Result<_> { $body })
        }
    };
}

handler!(list_games, |c, _a| games::list_games(c));
handler!(dashboard_stats, |c, _a| {
    let config = settings::current();
    let week_start = crate::now() - 7 * 24 * 3600;
    dashboard::stats(c, config.time_metric, week_start)
});
handler!(dashboard_recent, |c, _a| {
    dashboard::recent(c, 6)
});
handler!(dashboard_tag_cloud, |c, _a| {
    let config = settings::current();
    dashboard::tag_cloud(c, 40, config.hide_r18_tags, config.spoiler_level, &config.excluded_tags)
});
handler!(library_tags, |c, _a| {
    let config = settings::current();
    dashboard::library_tags(c, config.hide_r18_tags, config.spoiler_level, &config.excluded_tags)
});
handler!(dashboard_monthly_activity, |c, a| {
    dashboard::monthly_activity(c, a.months.unwrap_or(12))
});
handler!(dashboard_playtime_top, |c, _a| dashboard::playtime_top(c, 6));
handler!(dashboard_release_eras, |c, _a| dashboard::release_eras(c));
handler!(get_game, |c, a| games::get_game(c, a.id()?));
handler!(create_game, |c, a| games::create_game(
    c,
    &a.parse(a.input.as_ref(), "input")?
));
handler!(update_game, |c, a| {
    games::update_game(c, a.id()?, &a.parse(a.patch.as_ref(), "patch")?)
});
handler!(delete_game, |c, a| games::delete_game(c, a.id()?));
handler!(get_game_state, |c, a| games::get_game_state(c, a.id()?));
handler!(update_game_state, |c, a| {
    games::update_game_state(c, a.id()?, &a.parse(a.patch.as_ref(), "patch")?)
});
handler!(list_installations, |c, a| {
    installations::list_installations(c, a.game_id.as_deref())
});
handler!(get_installation, |c, a| installations::get_installation(
    c,
    a.id()?
));
handler!(create_installation, |c, a| {
    installations::create_installation(c, &a.parse(a.input.as_ref(), "input")?)
});
handler!(update_installation, |c, a| {
    installations::update_installation(c, a.id()?, &a.parse(a.patch.as_ref(), "patch")?)
});
handler!(delete_installation, |c, a| {
    installations::delete_installation(c, a.id()?)
});
handler!(save_installations, |c, a| {
    let entries: Vec<InstallationInput> = a.parse(a.entries.as_ref(), "entries")?;
    installations::save_installations(c, a.need(a.game_id.as_ref(), "game_id")?, &entries)
});
handler!(get_game_metadata, |c, a| metadata::get(c, a.id()?));
handler!(update_game_metadata, |c, a| {
    metadata::upsert(c, a.id()?, &a.parse(a.patch.as_ref(), "patch")?)
});
handler!(list_game_characters, |c, a| characters::list(c, a.id()?));
handler!(save_game_characters, |c, a| {
    let list: Vec<GameCharacter> = a.parse(a.characters.as_ref(), "characters")?;
    characters::replace_all(c, a.id()?, &list)
});
handler!(list_active_sessions, |c, _a| sessions::list_active(c));
async fn launch_game(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    finish(sessions::launch(dev.conn.clone(), a.id()?, Some(dev.app.clone())))
}

async fn search_vndb(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let keyword = match a.keyword().and_then(vndb::validate_keyword) {
        Ok(k) => k,
        Err(e) => return Err(Fail(StatusCode::BAD_REQUEST, e.to_string())),
    };
    dev.rate.acquire().await;
    finish(vndb::search(vndb::http(), &keyword).await)
}

async fn fetch_vn_metadata(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let vndb_id = match a.vndb_id().and_then(vndb::validate_vn_id) {
        Ok(v) => v,
        Err(e) => return Err(Fail(StatusCode::BAD_REQUEST, e.to_string())),
    };
    dev.rate.acquire().await;
    dev.rate.acquire().await;
    let dir = match crate::data_dir(&dev.app) {
        Ok(d) => d,
        Err(e) => return Err(Fail(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };
    let task = Task::new(
        commands::VNDB_SYNC,
        dev.app.clone(),
        dev.events.clone(),
        dev.tasks.register(commands::VNDB_SYNC),
        0,
    );
    finish(vndb::fetch_sync(vndb::http(), &vndb_id, &dir, &task).await)
}

async fn cancel_task(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let ok = a.task().map(|name| dev.tasks.cancel(name)).unwrap_or(false);
    finish(Ok(ok))
}

async fn inspect_path(AxumState(_dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let path = a.path().map(str::to_owned);
    let result = tokio::task::spawn_blocking(move || {
        path.and_then(|p| crate::inspect::inspect(&p))
    })
    .await
    .unwrap_or_else(|e| Err(AppError::Upstream(format!("识别任务失败：{e}"))));
    finish(result)
}

async fn scan_dir(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let path = match a.path() {
        Ok(p) => p.to_owned(),
        Err(e) => return Err(Fail(StatusCode::BAD_REQUEST, e.to_string())),
    };
    let task = Task::new(
        commands::SCAN,
        dev.app.clone(),
        dev.events.clone(),
        dev.tasks.register(commands::SCAN),
        0,
    );
    let registered = {
        let guard = dev.conn.lock().unwrap_or_else(|poison| poison.into_inner());
        match installations::registered_dirs(&guard) {
            Ok(set) => set,
            Err(e) => return Err(Fail(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        }
    };
    let result = tokio::task::spawn_blocking(move || {
        crate::inspect::scan_root(&path, &registered, &task)
    })
    .await
    .unwrap_or_else(|e| Err(AppError::Upstream(format!("扫描任务失败：{e}"))));
    finish(result)
}

#[derive(Deserialize)]
struct SinceQuery {
    since: Option<u64>,
}

async fn events(
    AxumState(dev): AxumState<Dev>,
    headers: HeaderMap,
    Query(q): Query<SinceQuery>,
) -> Response {
    let after = last_event_id(&headers).or(q.since).unwrap_or(0);
    events_response(&dev.events, after)
}

fn last_event_id(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(axum::http::header::HeaderName::from_static("last-event-id"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
}

pub fn events_response(log: &Arc<Mutex<EventLog>>, after: u64) -> Response {
    Sse::new(tail(log.clone(), after))
        .keep_alive(KeepAlive::new().interval(KEEP_ALIVE))
        .into_response()
}

fn tail(
    log: Arc<Mutex<EventLog>>,
    after: u64,
) -> impl Stream<Item = std::result::Result<Event, axum::Error>> {
    futures_util::stream::unfold(
        (log, after, VecDeque::<LoggedEvent>::new()),
        |(log, mut after, mut pending)| async move {
            loop {
                if pending.is_empty() {
                    let fresh = log
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .since(after);
                    if let Some(last) = fresh.last() {
                        after = last.seq;
                    }
                    pending.extend(fresh);
                }
                if let Some(entry) = pending.pop_front() {
                    return Some((frame(&entry), (log, after, pending)));
                }
                tokio::time::sleep(TAIL_TICK).await;
            }
        },
    )
}

fn frame(entry: &LoggedEvent) -> std::result::Result<Event, axum::Error> {
    Event::default()
        .id(entry.seq.to_string())
        .event(&entry.name)
        .json_data(&entry.event)
}

async fn allow_local_file(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    finish(
        a.path()
            .and_then(|p| commands::allow_local_file(dev.app.clone(), p.to_owned())),
    )
}

async fn path_exists(AxumState(_dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    finish(a.path().and_then(|p| commands::path_exists(p.to_owned())))
}

fn dev_data_dir(dev: &Dev) -> std::result::Result<PathBuf, Fail> {
    crate::data_dir(&dev.app).map_err(|e| Fail(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn create_backup(AxumState(dev): AxumState<Dev>, Json(_a): Json<Args>) -> Reply {
    let dir = match dev_data_dir(&dev) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };
    finish(backup::create_backup(&dev.conn, &dir))
}

async fn list_backups(AxumState(dev): AxumState<Dev>, Json(_a): Json<Args>) -> Reply {
    let dir = match dev_data_dir(&dev) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };
    finish(backup::list_backups(&dir))
}

async fn restore_backup(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let dir = match dev_data_dir(&dev) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };
    let id = match a.id() {
        Ok(i) => i.to_owned(),
        Err(e) => return Err(Fail(StatusCode::BAD_REQUEST, e.to_string())),
    };
    finish(backup::restore_backup(&dev.conn, &dir, &id))
}

async fn delete_backup(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let dir = match dev_data_dir(&dev) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };
    let id = match a.id() {
        Ok(i) => i.to_owned(),
        Err(e) => return Err(Fail(StatusCode::BAD_REQUEST, e.to_string())),
    };
    finish(backup::delete_backup(&dir, &id))
}

async fn get_config(AxumState(_dev): AxumState<Dev>, Json(_a): Json<Args>) -> Reply {
    finish(Ok(settings::view()))
}

async fn tag_dictionary(AxumState(_dev): AxumState<Dev>, Json(_a): Json<Args>) -> Reply {
    finish(Ok(crate::tagdict::dictionary()))
}

async fn update_config(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let config: settings::AppConfig = a.parse(a.config.as_ref(), "config")?;
    let dir = crate::data_dir(&dev.app)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    finish(
        settings::update(&dir, config)
            .map(|_| settings::current()),
    )
}

handler!(apply_sync_plan, |c, a| {
    let decision: crate::sync::PlanDecision = a.parse(a.decision.as_ref(), "decision")?;
    crate::sync::apply(c, &decision)
});

async fn test_vndb_token(AxumState(_dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let token = match a.token.as_deref().map(str::trim) {
        Some(t) if !t.is_empty() => t.to_owned(),
        _ => settings::current()
            .vndb_token_obf
            .as_deref()
            .and_then(settings::deobfuscate_token)
            .ok_or_else(|| AppError::BadRequest("还没有可测试的 VNDB Token".into()))?,
    };
    finish(vndb::test_token(vndb::http(), &token).await)
}

async fn sync_vndb(AxumState(dev): AxumState<Dev>, Json(_a): Json<Args>) -> Reply {
    let conn = dev.conn.clone();
    let rate = dev.rate.clone();
    let events = dev.events.clone();
    let report = move |phase: &str, done: Option<i64>, total: Option<i64>| {
        if let Ok(mut log) = events.lock() {
            log.push(
                commands::SYNC_PROGRESS,
                TaskEvent {
                    phase: phase.to_owned(),
                    done,
                    total,
                    error: None,
                    run_id: 0,
                    game_id: None,
                },
            );
        }
    };
    finish(crate::sync::plan(conn, &rate, report).await)
}

async fn start_backfill(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    if backfill::status().running {
        return Err(Fail(StatusCode::CONFLICT, "补全任务正在进行中".into()));
    }
    let items = {
        let guard = dev
            .conn
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        match &a.game_ids {
            Some(ids) if !ids.is_empty() => match backfill::items_by_ids(&guard, ids) {
                Ok(items) => items,
                Err(e) => return Err(Fail::from(e)),
            },
            _ => match backfill::scan(&guard) {
                Ok(items) => items,
                Err(e) => return Err(Fail::from(e)),
            },
        }
    };
    let total = items.len();
    let run_id = backfill::reset(total as i64);
    tokio::spawn(backfill::run(
        dev.conn.clone(),
        dev.rate.clone(),
        dev.app.clone(),
        dev.events.clone(),
        dev.tasks.clone(),
        run_id,
        items,
    ));
    finish(Ok(backfill::BackfillReport {
        total,
        ..Default::default()
    }))
}

async fn backfill_status(AxumState(_dev): AxumState<Dev>, Json(_a): Json<Args>) -> Reply {
    finish(Ok(backfill::status()))
}

async fn pause_backfill(AxumState(_dev): AxumState<Dev>, Json(_a): Json<Args>) -> Reply {
    finish(Ok(backfill::set_paused(true)))
}

async fn resume_backfill(AxumState(_dev): AxumState<Dev>, Json(_a): Json<Args>) -> Reply {
    finish(Ok(backfill::set_paused(false)))
}

async fn thumbnail(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    finish(
        a.path()
            .and_then(|p| commands::thumbnail(dev.app.clone(), p.to_owned())),
    )
}

pub fn router(dev: Dev) -> Router {
    Router::new()
        .route("/list_games", post(list_games))
        .route("/dashboard_stats", post(dashboard_stats))
        .route("/dashboard_recent", post(dashboard_recent))
        .route("/dashboard_tag_cloud", post(dashboard_tag_cloud))
        .route("/library_tags", post(library_tags))
        .route("/dashboard_monthly_activity", post(dashboard_monthly_activity))
        .route("/dashboard_playtime_top", post(dashboard_playtime_top))
        .route("/dashboard_release_eras", post(dashboard_release_eras))
        .route("/get_game", post(get_game))
        .route("/create_game", post(create_game))
        .route("/update_game", post(update_game))
        .route("/delete_game", post(delete_game))
        .route("/get_game_state", post(get_game_state))
        .route("/update_game_state", post(update_game_state))
        .route("/list_installations", post(list_installations))
        .route("/get_installation", post(get_installation))
        .route("/create_installation", post(create_installation))
        .route("/update_installation", post(update_installation))
        .route("/delete_installation", post(delete_installation))
        .route("/save_installations", post(save_installations))
        .route("/launch_game", post(launch_game))
        .route("/list_active_sessions", post(list_active_sessions))
        .route("/get_game_metadata", post(get_game_metadata))
        .route("/update_game_metadata", post(update_game_metadata))
        .route("/list_game_characters", post(list_game_characters))
        .route("/save_game_characters", post(save_game_characters))
        .route("/search_vndb", post(search_vndb))
        .route("/fetch_vn_metadata", post(fetch_vn_metadata))
        .route("/cancel_task", post(cancel_task))
        .route("/inspect_path", post(inspect_path))
        .route("/scan_dir", post(scan_dir))
        .route("/allow_local_file", post(allow_local_file))
        .route("/path_exists", post(path_exists))
        .route("/create_backup", post(create_backup))
        .route("/list_backups", post(list_backups))
        .route("/restore_backup", post(restore_backup))
        .route("/delete_backup", post(delete_backup))
        .route("/thumbnail", post(thumbnail))
        .route("/get_config", post(get_config))
        .route("/tag_dictionary", post(tag_dictionary))
        .route("/update_config", post(update_config))
        .route("/test_vndb_token", post(test_vndb_token))
        .route("/sync_vndb", post(sync_vndb))
        .route("/apply_sync_plan", post(apply_sync_plan))
        .route("/start_backfill", post(start_backfill))
        .route("/backfill_status", post(backfill_status))
        .route("/pause_backfill", post(pause_backfill))
        .route("/resume_backfill", post(resume_backfill))
        .route("/events", get(events))
        .with_state(dev)
}

pub const COMMANDS: &[&str] = &[
    "list_games",
    "dashboard_stats",
    "dashboard_recent",
    "dashboard_tag_cloud",
    "library_tags",
    "dashboard_monthly_activity",
    "dashboard_playtime_top",
    "dashboard_release_eras",
    "get_game",
    "create_game",
    "update_game",
    "delete_game",
    "get_game_state",
    "update_game_state",
    "list_installations",
    "get_installation",
    "create_installation",
    "update_installation",
    "delete_installation",
    "save_installations",
    "launch_game",
    "list_active_sessions",
    "get_game_metadata",
    "update_game_metadata",
    "list_game_characters",
    "save_game_characters",
    "search_vndb",
    "fetch_vn_metadata",
    "cancel_task",
    "inspect_path",
    "scan_dir",
    "allow_local_file",
    "path_exists",
    "create_backup",
    "list_backups",
    "restore_backup",
    "delete_backup",
    "thumbnail",
    "get_config",
    "update_config",
    "tag_dictionary",
    "test_vndb_token",
    "sync_vndb",
    "apply_sync_plan",
    "start_backfill",
    "backfill_status",
    "pause_backfill",
    "resume_backfill",
];

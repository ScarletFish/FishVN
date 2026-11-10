use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use rusqlite::{Connection, params_from_iter};
use serde::Serialize;
use tauri_plugin_log::log;

use crate::characters;
use crate::error::{AppError, Result};
use crate::games;
use crate::generated::metadata::is_blank_current;
use crate::metadata;
use crate::tasks::{EventLog, Progress, Task, Tasks};
use crate::types::{GameMetadata, GamePatch, LocalizedTitle, MetadataPatch, MetadataStaff, MetadataTag};
use crate::vndb::{
    self, download_images, ImageJob, ImageKind, RateLimiter, VnMetadata, VnSync,
    IMAGE_CONCURRENCY,
};

pub const BACKFILL: &str = "vndb-backfill";

static RUN_ID: AtomicU64 = AtomicU64::new(0);
static SINGLE: AtomicBool = AtomicBool::new(false);

struct SingleGuard;

impl SingleGuard {
    fn acquire() -> Result<Self> {
        if SINGLE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(AppError::Conflict("单条补全正在进行中".into()));
        }
        Ok(Self)
    }
}

impl Drop for SingleGuard {
    fn drop(&mut self) {
        SINGLE.store(false, Ordering::SeqCst);
    }
}

pub fn busy() -> bool {
    status().running || SINGLE.load(Ordering::SeqCst)
}

const FREEZE: Duration = Duration::from_secs(300);
const FREEZE_ROUNDS: u32 = 3;
const PAUSE_TICK: Duration = Duration::from_millis(250);
const BATCH_ATTEMPTS: u32 = 2;

const SCAN_SQL: &str = "SELECT g.id, g.name, g.vndb_id
     FROM games g
     LEFT JOIN game_metadata m ON m.game_id = g.id
     WHERE m.game_id IS NULL
        OR COALESCE(m.description, '') = ''
        OR COALESCE(m.image_path, '') = ''
        OR NOT EXISTS (SELECT 1 FROM characters c WHERE c.game_id = g.id)";

const BY_IDS_PREFIX: &str = "SELECT g.id, g.name, g.vndb_id FROM games AS g WHERE g.id IN (";

#[derive(Debug, Clone, Serialize)]
pub struct BackfillItem {
    pub game_id: String,
    pub name: String,
    pub vndb_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemState {
    #[default]
    Pending,
    Active,
    Ok,
    Unmatched,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BindSource {
    Existing,
    Search,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackfillCandidate {
    pub id: String,
    pub title: String,
    pub released: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BackfillItemState {
    pub game_id: String,
    pub name: String,
    pub vndb_id: Option<String>,
    pub state: ItemState,
    pub bind_source: Option<BindSource>,
    pub hit_count: usize,
    pub hit_title: Option<String>,
    pub hit_released: Option<String>,
    pub candidates: Vec<BackfillCandidate>,
    pub filled: Vec<&'static str>,
    pub characters: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct BackfillReport {
    pub total: usize,
    pub fetched: usize,
    pub bound: usize,
    pub unmatched: usize,
    pub failed: usize,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BackfillStatus {
    pub running: bool,
    pub paused: bool,
    pub paused_idle: bool,
    pub done: i64,
    pub total: i64,
    pub unmatched: i64,
    pub failed: i64,
    pub rate_limited: bool,
    pub stage: String,
    pub images_done: i64,
    pub images_total: i64,
    pub run_id: u64,
    pub items: Vec<BackfillItemState>,
    pub pending_ids: Vec<String>,
    pub finished: Option<BackfillReport>,
}

fn status_cell() -> &'static Mutex<Option<BackfillStatus>> {
    static CELL: OnceLock<Mutex<Option<BackfillStatus>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

pub fn status() -> BackfillStatus {
    status_cell()
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_default()
}

pub fn item_state(game_id: &str) -> Option<BackfillItemState> {
    let guard = status_cell().lock().ok()?;
    guard
        .as_ref()?
        .items
        .iter()
        .find(|item| item.game_id == game_id)
        .cloned()
}

fn set_status(f: impl FnOnce(&mut BackfillStatus)) {
    if let Ok(mut guard) = status_cell().lock() {
        f(guard.get_or_insert_with(BackfillStatus::default));
    }
}

fn pending_states(items: &[BackfillItem]) -> Vec<BackfillItemState> {
    items
        .iter()
        .map(|item| BackfillItemState {
            game_id: item.game_id.clone(),
            name: item.name.clone(),
            vndb_id: item.vndb_id.clone(),
            bind_source: Some(if item.vndb_id.is_some() {
                BindSource::Existing
            } else {
                BindSource::Search
            }),
            ..Default::default()
        })
        .collect()
}

pub fn status_or_plan(conn: &Connection) -> Result<BackfillStatus> {
    let pending = scan(conn)?;
    let pending_ids = pending.iter().map(|it| it.game_id.clone()).collect();
    let current = status();
    if !current.items.is_empty() {
        return Ok(BackfillStatus {
            pending_ids,
            ..current
        });
    }
    Ok(BackfillStatus {
        total: pending.len() as i64,
        items: pending_states(&pending),
        pending_ids,
        ..Default::default()
    })
}

pub fn reset(items: &[BackfillItem]) -> u64 {
    let run_id = RUN_ID.fetch_add(1, Ordering::Relaxed) + 1;
    let states = pending_states(items);
    if let Ok(mut guard) = status_cell().lock() {
        *guard = Some(BackfillStatus {
            running: true,
            total: items.len() as i64,
            stage: "fetch".to_owned(),
            run_id,
            items: states,
            ..Default::default()
        });
    }
    run_id
}

fn set_item(game_id: &str, f: impl FnOnce(&mut BackfillItemState)) {
    set_status(|s| {
        if let Some(item) = s.items.iter_mut().find(|item| item.game_id == game_id) {
            f(item);
        }
    });
}

pub fn set_paused(paused: bool) -> BackfillStatus {
    set_status(|s| {
        s.paused = paused;
        if paused {
            s.paused_idle = false;
        }
    });
    status()
}

pub fn scan(conn: &Connection) -> Result<Vec<BackfillItem>> {
    let sql = format!("{SCAN_SQL} {}", crate::games::GAME_ORDER_SQL);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(BackfillItem {
            game_id: row.get("id")?,
            name: row.get("name")?,
            vndb_id: row.get("vndb_id")?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn items_by_ids(conn: &Connection, ids: &[String]) -> Result<Vec<BackfillItem>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "{BY_IDS_PREFIX}{}) {}",
        vec!["?"; ids.len()].join(", "),
        crate::games::GAME_ORDER_SQL,
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(ids.iter()), |row| {
        Ok(BackfillItem {
            game_id: row.get("id")?,
            name: row.get("name")?,
            vndb_id: row.get("vndb_id")?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

fn clean_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn fill<T>(blank: bool, remote: Option<T>) -> Option<Option<T>> {
    (blank && remote.is_some()).then_some(remote)
}

fn map_titles(vn: &VnMetadata) -> Option<Vec<LocalizedTitle>> {
    let titles = vn.titles.as_ref()?;
    let out: Vec<LocalizedTitle> = titles
        .iter()
        .filter(|t| t.title.as_deref().and_then(clean_text).is_some())
        .map(|t| LocalizedTitle {
            lang: t.lang.clone(),
            title: t.title.clone().unwrap_or_default(),
            latin: t.latin.clone(),
            official: t.official,
            main: t.main,
        })
        .collect();
    (!out.is_empty()).then_some(out)
}

fn map_tags(vn: &VnMetadata) -> Option<Vec<MetadataTag>> {
    let tags = vn.tags.as_ref()?;
    let out: Vec<MetadataTag> = tags
        .iter()
        .filter(|t| clean_text(&t.name).is_some())
        .map(|t| MetadataTag {
            id: Some(t.id.clone()),
            name: t.name.clone(),
            category: t.category.clone(),
            rating: t.rating,
            vn_count: t.vn_count,
            spoiler: t.spoiler,
            lie: t.lie,
        })
        .collect();
    (!out.is_empty()).then_some(out)
}

fn map_developers(vn: &VnMetadata) -> Option<Vec<String>> {
    let devs = vn.developers.as_ref()?;
    let out: Vec<String> = devs
        .iter()
        .filter_map(|d| {
            d.name
                .as_deref()
                .and_then(clean_text)
                .or_else(|| d.original.as_deref().and_then(clean_text))
        })
        .collect();
    (!out.is_empty()).then_some(out)
}

fn map_staff(vn: &VnMetadata) -> Option<Vec<MetadataStaff>> {
    let staff = vn.staff.as_ref()?;
    let out: Vec<MetadataStaff> = staff
        .iter()
        .filter(|s| {
            clean_text(s.name.as_deref().unwrap_or("")).is_some()
                || clean_text(s.original.as_deref().unwrap_or("")).is_some()
        })
        .cloned()
        .collect();
    (!out.is_empty()).then_some(out)
}

pub fn patch_for(
    current: Option<&GameMetadata>,
    sync: &VnSync,
    has_dict_tags: bool,
) -> Option<MetadataPatch> {
    let vn = &sync.vn;
    let cur = current.cloned().unwrap_or_default();
    let patch = MetadataPatch {
        alias: fill(
            is_blank_current(&cur, "alias"),
            vndb::chain(vn).into_iter().nth(1),
        ),
        released: fill(
            is_blank_current(&cur, "released"),
            vn.released.as_deref().and_then(clean_text),
        ),
        rating: fill(is_blank_current(&cur, "rating"), vn.rating),
        votecount: fill(is_blank_current(&cur, "votecount"), vn.votecount),
        length_minutes: fill(is_blank_current(&cur, "length_minutes"), vn.length_minutes),
        description: fill(
            is_blank_current(&cur, "description"),
            vn.description.as_deref().and_then(clean_text),
        ),
        image_path: fill(
            is_blank_current(&cur, "image_path"),
            vn.image
                .as_ref()
                .and_then(|i| i.local_path.as_deref())
                .and_then(clean_text),
        ),
        cover_focal_x: None,
        cover_focal_y: None,
        ambience_path: None,
        ambience_mode: None,
        ambience_plate: None,
        ambience_focal_x: None,
        ambience_focal_y: None,
        titles: fill(is_blank_current(&cur, "titles"), map_titles(vn)),
        tags: fill(!has_dict_tags, map_tags(vn)),
        developers: fill(is_blank_current(&cur, "developers"), map_developers(vn)),
        staff: fill(is_blank_current(&cur, "staff"), map_staff(vn)),
    };
    (!patch.filled_labels().is_empty()).then_some(patch)
}

async fn freeze(task: &Task) -> bool {
    task.report_error("rate-limited", "已触发 VNDB 限流，约 5 分钟后自动继续");
    set_status(|s| s.rate_limited = true);
    let mut left = FREEZE;
    while !left.is_zero() {
        if task.cancelled() {
            return false;
        }
        let step = left.min(Duration::from_secs(1));
        tokio::time::sleep(step).await;
        left -= step;
    }
    set_status(|s| s.rate_limited = false);
    true
}

async fn wait_resume(task: &Task) {
    if !status().paused {
        return;
    }
    set_status(|s| s.paused_idle = true);
    task.report("paused", None, None);
    while status().paused && !task.cancelled() {
        tokio::time::sleep(PAUSE_TICK).await;
    }
    set_status(|s| s.paused_idle = false);
}

fn fail(
    report: &mut BackfillReport,
    task: &Task,
    phase: &str,
    game_id: &str,
    name: &str,
    err: &AppError,
) {
    report.failed += 1;
    task.report_error(phase, &format!("「{name}」{err}"));
    set_status(|s| {
        s.failed += 1;
        if let Some(item) = s.items.iter_mut().find(|item| item.game_id == game_id) {
            item.state = ItemState::Failed;
            item.error = Some(err.to_string());
        }
    });
}

async fn resolve_vndb_id(
    task: &Task,
    rate: &RateLimiter,
    item: &BackfillItem,
    report: &mut BackfillReport,
) -> Result<Option<String>> {
    if let Some(id) = &item.vndb_id {
        return Ok(Some(id.clone()));
    }
    let mut rounds = FREEZE_ROUNDS;
    loop {
        rate.acquire().await;
        if task.cancelled() {
            return Err(AppError::Cancelled);
        }
        match vndb::search(vndb::http(), &item.name).await {
            Ok(hits) => {
                let Some(first) = hits.first() else {
                    report.unmatched += 1;
                    task.report_error(
                        "unmatched",
                        &format!("「{}」在 VNDB 没搜到结果，已跳过", item.name),
                    );
                    set_status(|s| {
                        s.unmatched += 1;
                        if let Some(it) = s.items.iter_mut().find(|it| it.game_id == item.game_id) {
                            it.state = ItemState::Unmatched;
                            it.error = Some("VNDB 未搜到结果".to_owned());
                        }
                    });
                    return Ok(None);
                };
                let hit_count = hits.len();
                let first_id = first.id.clone();
                let first_title = first.title.clone();
                let first_released = first.released.clone();
                let candidates: Vec<BackfillCandidate> = hits
                    .iter()
                    .take(3)
                    .map(|hit| BackfillCandidate {
                        id: hit.id.clone(),
                        title: hit.title.clone(),
                        released: hit.released.clone(),
                    })
                    .collect();
                task.report_note(
                    "resolve",
                    &format!(
                        "「{}」命中 {} 条，取 {}《{}》",
                        item.name, hit_count, first_id, first_title
                    ),
                );
                set_item(&item.game_id, |it| {
                    it.vndb_id = Some(first_id.clone());
                    it.bind_source = Some(BindSource::Search);
                    it.hit_count = hit_count;
                    it.hit_title = Some(first_title);
                    it.hit_released = first_released;
                    it.candidates = candidates;
                });
                return Ok(Some(first_id));
            }
            Err(AppError::RateLimited) if rounds > 0 => {
                rounds -= 1;
                if !freeze(task).await {
                    return Err(AppError::Cancelled);
                }
            }
            Err(e) => {
                fail(report, task, "failed", &item.game_id, &item.name, &e);
                return Ok(None);
            }
        }
    }
}

async fn apply_sync(
    conn: &Arc<Mutex<Connection>>,
    game_id: &str,
    sync: &VnSync,
) -> Result<(Vec<&'static str>, usize)> {
    tauri::async_runtime::spawn_blocking({
        let conn = conn.clone();
        let game_id = game_id.to_owned();
        let sync = sync.clone();
        move || {
            let guard = crate::db::lock(&conn)?;
            let current = metadata::get(&guard, &game_id)?;
            let mut filled = Vec::new();
            let has_dict_tags = crate::tagstore::has_dict_tags(&guard, &game_id)?;
            if let Some(patch) = patch_for(current.as_ref(), &sync, has_dict_tags) {
                filled = patch.filled_labels();
                metadata::upsert(&guard, &game_id, &patch)?;
            }
            let mut char_count = 0;
            if characters::list(&guard, &game_id)?.is_empty() && !sync.characters.is_empty() {
                characters::replace_all(&guard, &game_id, &sync.characters)?;
                char_count = sync.characters.len();
            }
            Ok::<(Vec<&'static str>, usize), AppError>((filled, char_count))
        }
    })
    .await
    .map_err(|e| AppError::Upstream(format!("补全任务失败：{e}")))?
}

async fn bind_item(
    task: &Task,
    conn: &Arc<Mutex<Connection>>,
    rate: &RateLimiter,
    item: &BackfillItem,
    report: &mut BackfillReport,
) -> Result<Option<String>> {
    let Some(vndb_id) = resolve_vndb_id(task, rate, item, report).await? else {
        return Ok(None);
    };
    if item.vndb_id.is_some() {
        return Ok(Some(vndb_id));
    }
    let bound = tauri::async_runtime::spawn_blocking({
        let conn = conn.clone();
        let game_id = item.game_id.clone();
        let id = vndb_id.clone();
        move || {
            let guard = crate::db::lock(&conn)?;
            games::update_game(
                &guard,
                &game_id,
                &GamePatch {
                    name: None,
                    vndb_id: Some(Some(id)),
                },
            )?;
            Ok::<(), AppError>(())
        }
    })
    .await
    .map_err(|e| AppError::Upstream(format!("补全任务失败：{e}")))?;
    match bound {
        Ok(()) => {
            report.bound += 1;
            Ok(Some(vndb_id))
        }
        Err(AppError::Conflict(_)) => {
            fail(
                report,
                task,
                "failed",
                &item.game_id,
                &item.name,
                &AppError::Conflict("vndb_id 已被其他游戏占用".into()),
            );
            Ok(None)
        }
        Err(e) => {
            fail(report, task, "failed", &item.game_id, &item.name, &e);
            Ok(None)
        }
    }
}

fn failed_single(game_id: &str, name: &str, err: &AppError) -> BackfillItemState {
    let message = err.to_string();
    set_item(game_id, |it| {
        it.state = ItemState::Failed;
        it.error = Some(message.clone());
    });
    BackfillItemState {
        game_id: game_id.to_owned(),
        name: name.to_owned(),
        state: ItemState::Failed,
        error: Some(message),
        ..Default::default()
    }
}

pub async fn fill_single(
    conn: Arc<Mutex<Connection>>,
    rate: Arc<RateLimiter>,
    app: tauri::AppHandle,
    events: Arc<Mutex<EventLog>>,
    tasks: Arc<Tasks>,
    game_id: String,
) -> Result<BackfillItemState> {
    if status().running {
        return Err(AppError::Conflict("补全任务正在进行中".into()));
    }
    let _guard = SingleGuard::acquire()?;
    let item = {
        let guard = crate::db::lock(&conn)?;
        items_by_ids(&guard, std::slice::from_ref(&game_id))?
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("游戏不存在".into()))?
    };
    let Some(vndb_id) = item.vndb_id.clone() else {
        return Err(AppError::Conflict("未绑定 VNDB 作品，先匹配作品".into()));
    };
    let dir = crate::data_dir(&app)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    let task = Task::new(BACKFILL, app, events, tasks.register(BACKFILL), 0);
    task.enter(&game_id);
    let mut rounds = FREEZE_ROUNDS;
    let (mut sync, jobs) = loop {
        rate.acquire().await;
        rate.acquire().await;
        match vndb::fetch_sync(vndb::http(), &game_id, &vndb_id, &task).await {
            Ok(pair) => break pair,
            Err(AppError::RateLimited) if rounds > 0 => {
                rounds -= 1;
                if !freeze(&task).await {
                    task.leave();
                    return Err(AppError::Cancelled);
                }
            }
            Err(e) => {
                task.leave();
                return Ok(failed_single(&game_id, &item.name, &e));
            }
        }
    };
    if let Ok(results) = download_images(vndb::image_client(), jobs, IMAGE_CONCURRENCY, &dir, &task).await {
        vndb::fill_image_paths(&mut sync, &results);
    }
    let applied = apply_sync(&conn, &game_id, &sync).await;
    task.leave();
    let (filled, char_count) = match applied {
        Ok(done) => done,
        Err(e) => return Ok(failed_single(&game_id, &item.name, &e)),
    };
    let vn_title = sync.vn.title.clone();
    let vn_released = sync.vn.released.clone();
    set_item(&game_id, |it| {
        it.state = ItemState::Ok;
        it.filled = filled.clone();
        it.characters = char_count;
        it.error = None;
        if it.vndb_id.is_none() {
            it.vndb_id = Some(vndb_id.clone());
        }
        if it.hit_title.is_none() {
            it.hit_title = Some(vn_title.clone());
        }
        if it.hit_released.is_none() {
            it.hit_released = vn_released.clone();
        }
    });
    let mut state = status()
        .items
        .into_iter()
        .find(|it| it.game_id == game_id)
        .unwrap_or_default();
    state.game_id = game_id;
    state.state = ItemState::Ok;
    state.filled = filled;
    state.characters = char_count;
    state.error = None;
    if state.vndb_id.is_none() {
        state.vndb_id = Some(vndb_id);
    }
    Ok(state)
}

pub async fn run(
    conn: Arc<Mutex<Connection>>,
    rate: Arc<RateLimiter>,
    app: tauri::AppHandle,
    events: Arc<Mutex<EventLog>>,
    tasks: Arc<Tasks>,
    run_id: u64,
    items: Vec<BackfillItem>,
) -> BackfillReport {
    let task = Task::new(BACKFILL, app.clone(), events, tasks.register(BACKFILL), run_id);
    let total = items.len() as i64;
    let mut report = BackfillReport {
        total: items.len(),
        ..Default::default()
    };
    let dir = match crate::data_dir(&app) {
        Ok(d) => d,
        Err(e) => {
            task.report_error("failed", &format!("数据目录不可用：{e}"));
            finish_run(&task, total, 0, report);
            return report;
        }
    };
    set_status(|s| s.stage = "fetch".to_owned());
    let names: std::collections::HashMap<&str, &str> = items
        .iter()
        .map(|item| (item.game_id.as_str(), item.name.as_str()))
        .collect();

    let mut processed = 0i64;
    let mut jobs: Vec<ImageJob> = Vec::new();
    let mut cancelled = false;

    let mut ready: Vec<vndb::BatchItem> = Vec::new();
    for item in &items {
        if task.cancelled() {
            cancelled = true;
            break;
        }
        wait_resume(&task).await;
        if task.cancelled() {
            cancelled = true;
            break;
        }
        let searching = item.vndb_id.is_none();
        if searching {
            task.enter(&item.game_id);
            set_item(&item.game_id, |it| it.state = ItemState::Active);
        }
        let outcome = bind_item(&task, &conn, &rate, item, &mut report).await;
        if searching {
            task.leave();
        }
        match outcome {
            Ok(Some(vndb_id)) => ready.push(vndb::BatchItem {
                game_id: item.game_id.clone(),
                vndb_id,
            }),
            Ok(None) => {
                if searching {
                    processed += 1;
                    set_status(|s| s.done = processed);
                }
            }
            Err(AppError::Cancelled) => {
                cancelled = true;
                if searching {
                    set_item(&item.game_id, |it| it.state = ItemState::Pending);
                }
                break;
            }
            Err(_) => {
                if searching {
                    processed += 1;
                    set_status(|s| s.done = processed);
                }
            }
        }
    }

    let mut start = 0usize;
    while !cancelled && start < ready.len() {
        if task.cancelled() {
            cancelled = true;
            break;
        }
        wait_resume(&task).await;
        if task.cancelled() {
            cancelled = true;
            break;
        }
        let end = (start + vndb::BATCH_SIZE).min(ready.len());
        let chunk = &ready[start..end];
        start = end;
        for entry in chunk {
            set_item(&entry.game_id, |it| it.state = ItemState::Active);
        }
        let started = std::time::Instant::now();
        match fetch_chunk(&task, &rate, chunk).await {
            Ok((synced, missing)) => {
                for vndb_id in missing {
                    let Some(entry) = chunk.iter().find(|entry| entry.vndb_id == vndb_id) else {
                        continue;
                    };
                    fail(
                        &mut report,
                        &task,
                        "failed",
                        &entry.game_id,
                        names.get(entry.game_id.as_str()).copied().unwrap_or_default(),
                        &AppError::NotFound("VNDB 未返回这条记录".into()),
                    );
                }
                for entry in &synced {
                    let name = names
                        .get(entry.game_id.as_str())
                        .copied()
                        .unwrap_or_default();
                    let vn_title = entry.sync.vn.title.clone();
                    let vn_released = entry.sync.vn.released.clone();
                    match apply_sync(&conn, &entry.game_id, &entry.sync).await {
                        Ok((filled, char_count)) => {
                            report.fetched += 1;
                            set_item(&entry.game_id, |it| {
                                it.state = ItemState::Ok;
                                it.filled = filled;
                                it.characters = char_count;
                                if it.vndb_id.is_none() {
                                    it.vndb_id = Some(entry.vndb_id.clone());
                                }
                                if it.hit_title.is_none() {
                                    it.hit_title = Some(vn_title);
                                }
                                if it.hit_released.is_none() {
                                    it.hit_released = vn_released;
                                }
                            });
                        }
                        Err(e) => {
                            fail(&mut report, &task, "failed", &entry.game_id, name, &e);
                        }
                    }
                    jobs.extend(entry.jobs.iter().cloned());
                }
            }
            Err(AppError::Cancelled) => {
                cancelled = true;
                break;
            }
            Err(e) => {
                for entry in chunk {
                    fail(
                        &mut report,
                        &task,
                        "failed",
                        &entry.game_id,
                        names.get(entry.game_id.as_str()).copied().unwrap_or_default(),
                        &e,
                    );
                }
            }
        }
        processed += chunk.len() as i64;
        set_status(|s| s.done = processed);
        for entry in chunk {
            task.report_item("backfill", Some(processed), Some(total), &entry.game_id);
        }
        log::debug!(
            target: "backfill",
            "批次完成：{} 部，累计 {processed}/{total}，{}ms",
            chunk.len(),
            started.elapsed().as_millis()
        );
    }

    if cancelled {
        set_status(|s| {
            for item in s.items.iter_mut() {
                if item.state == ItemState::Active {
                    item.state = ItemState::Pending;
                }
            }
        });
    }

    if !cancelled && !jobs.is_empty() {
        set_status(|s| {
            s.stage = "images".to_owned();
            s.images_done = 0;
            s.images_total = jobs.len() as i64;
        });
        let mirror = StatusMirror(&task);
        let touched = apply_images(&conn, jobs, &dir, &mirror).await;
        for id in touched {
            task.report_item("backfill", Some(processed), Some(total), &id);
        }
    }

    finish_run(&task, total, processed, report);
    report
}

async fn fetch_chunk(
    task: &Task,
    rate: &RateLimiter,
    chunk: &[vndb::BatchItem],
) -> Result<(Vec<vndb::BatchSync>, Vec<String>)> {
    let mut rounds = FREEZE_ROUNDS;
    let mut attempts = 0u32;
    loop {
        match vndb::fetch_batch(vndb::http(), rate, chunk, task).await {
            Ok(pair) => return Ok(pair),
            Err(AppError::RateLimited) if rounds > 0 => {
                rounds -= 1;
                if !freeze(task).await {
                    return Err(AppError::Cancelled);
                }
            }
            Err(AppError::Cancelled) => return Err(AppError::Cancelled),
            Err(e) if attempts < BATCH_ATTEMPTS => {
                attempts += 1;
                log::warn!(
                    target: "backfill",
                    "整批取数失败，第 {attempts} 次重试：{e}"
                );
                tokio::time::sleep(Duration::from_millis(500 * attempts as u64)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

struct StatusMirror<'a>(&'a Task);

impl Progress for StatusMirror<'_> {
    fn report(&self, phase: &str, done: Option<i64>, total: Option<i64>) {
        if phase == "images" {
            set_status(|s| {
                if let Some(done) = done {
                    s.images_done = done;
                }
                if let Some(total) = total {
                    s.images_total = total;
                }
            });
        }
        self.0.report(phase, done, total);
    }

    fn report_item(&self, phase: &str, done: Option<i64>, total: Option<i64>, game_id: &str) {
        self.0.report_item(phase, done, total, game_id);
    }

    fn report_error(&self, phase: &str, message: &str) {
        self.0.report_error(phase, message);
    }

    fn report_note(&self, phase: &str, detail: &str) {
        self.0.report_note(phase, detail);
    }

    fn cancelled(&self) -> bool {
        self.0.cancelled()
    }
}

async fn apply_images(
    conn: &Arc<Mutex<Connection>>,
    jobs: Vec<ImageJob>,
    dir: &Path,
    progress: &dyn Progress,
) -> Vec<String> {
    if jobs.is_empty() {
        return Vec::new();
    }
    let Ok(results) = download_images(vndb::image_client(), jobs, IMAGE_CONCURRENCY, dir, progress).await else {
        return Vec::new();
    };
    if results.is_empty() {
        return Vec::new();
    }
    let conn = conn.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Ok(guard) = conn.lock() else {
            return Vec::new();
        };
        let mut touched: Vec<String> = Vec::new();
        for (job, path) in results {
            let Some(path) = path else { continue };
            let changed = match &job.kind {
                ImageKind::Cover => guard.execute(
                    "UPDATE game_metadata SET image_path = ?1 WHERE game_id = ?2 AND COALESCE(image_path, '') = ''",
                    rusqlite::params![path, job.game_id],
                ),
                ImageKind::Character { character_id } => guard.execute(
                    "UPDATE characters SET image_path = ?1 WHERE game_id = ?2 AND id = ?3 AND COALESCE(image_path, '') = ''",
                    rusqlite::params![path, job.game_id, character_id],
                ),
            };
            if matches!(changed, Ok(n) if n > 0) && !touched.contains(&job.game_id) {
                touched.push(job.game_id.clone());
            }
        }
        touched
    })
    .await
    .unwrap_or_default()
}

fn finish_run(task: &Task, total: i64, done: i64, mut report: BackfillReport) {
    set_status(|s| s.stage.clear());
    task.report("backfill", Some(done), Some(total));
    task.report("finished", None, None);
    report.cancelled = task.cancelled();
    set_status(|s| {
        s.running = false;
        s.paused = false;
        s.finished = Some(report);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vndb::{VnDeveloper, VnImage, VnTag, VnTitle};

    fn vn() -> VnMetadata {
        VnMetadata {
            id: "v1".into(),
            title: "Fate/stay night".into(),
            alttitle: Some("フェイト/ステイナイト".into()),
            olang: Some("ja".into()),
            released: Some("2004-01-30".into()),
            rating: Some(82.34),
            votecount: Some(1200),
            length_minutes: Some(360),
            description: Some(
                "[spoiler]剧透[/spoiler]正文 [url=https://x.com]链接[/url] [b]粗体[/b]".into(),
            ),
        titles: Some(vec![VnTitle {
            lang: "zh-Hans".into(),
            title: Some("命运之夜".into()),
                latin: None,
                official: Some(true),
                main: Some(true),
            }]),
            image: Some(VnImage {
                id: Some("cv1".into()),
                url: None,
                local_path: Some("/covers/v1.jpg".into()),
                ..VnImage::default()
            }),
            tags: Some(vec![VnTag {
                id: "g1".into(),
                name: "Visual Novel".into(),
                category: Some("cont".into()),
                rating: Some(1.2),
                spoiler: Some(0),
                lie: None,
                vn_count: Some(999),
            }]),
            developers: Some(vec![VnDeveloper {
                id: "d1".into(),
                name: Some("TYPE-MOON".into()),
                original: None,
                kind: None,
            }]),
            staff: Some(vec![MetadataStaff {
                role: Some("scenario".into()),
                name: Some("奈须蘑菇".into()),
                original: None,
                note: None,
            }]),
            va: None,
            devstatus: None,
        }
    }

    fn sync() -> VnSync {
        VnSync {
            vn: vn(),
            characters: vec![],
        }
    }

    #[test]
    fn 标题链按中文日文罗马音回落() {
        let v = vn();
        assert_eq!(
            vndb::chain(&v),
            vec!["命运之夜", "フェイト/ステイナイト", "Fate/stay night"]
        );

        let mut no_zh = vn();
        no_zh.titles = None;
        assert_eq!(
            vndb::chain(&no_zh),
            vec!["フェイト/ステイナイト", "Fate/stay night"]
        );

        let mut en = vn();
        en.titles = None;
        en.olang = Some("en".into());
        en.alttitle = None;
        assert_eq!(vndb::chain(&en), vec!["Fate/stay night"]);
    }

    #[test]
    fn 中文原作的罗马字别名不算日语档() {
        let mut zh = vn();
        zh.titles = None;
        zh.olang = Some("zh".into());
        zh.alttitle = Some("Ming Yun Zhi Ye".into());
        assert_eq!(vndb::chain(&zh), vec!["Fate/stay night"]);
    }

    #[test]
    fn 日文标题与原文别名相同时只保留一次() {
        let mut v = vn();
        v.titles = Some(vec![VnTitle {
            lang: "ja".into(),
            title: Some("フェイト/ステイナイト".into()),
            ..VnTitle::default()
        }]);
        assert_eq!(
            vndb::chain(&v),
            vec!["フェイト/ステイナイト", "Fate/stay night"]
        );
    }

    #[test]
    fn 全新游戏所有空白字段一次填齐() {
        let patch = patch_for(None, &sync(), false).expect("空库必须产出补丁");
        assert_eq!(patch.alias, Some(Some("フェイト/ステイナイト".into())));
        assert_eq!(patch.released, Some(Some("2004-01-30".into())));
        assert_eq!(patch.rating, Some(Some(82.34)));
        assert_eq!(patch.votecount, Some(Some(1200)));
        assert_eq!(patch.length_minutes, Some(Some(360)));
        assert_eq!(
            patch.description,
            Some(Some(
                "[spoiler]剧透[/spoiler]正文 [url=https://x.com]链接[/url] [b]粗体[/b]".into()
            ))
        );
        assert_eq!(patch.image_path, Some(Some("/covers/v1.jpg".into())));
        assert!(patch.titles.as_ref().unwrap().is_some());
        assert!(patch.tags.as_ref().unwrap().is_some());
        assert!(patch.developers.as_ref().unwrap().is_some());
        assert!(patch.staff.as_ref().unwrap().is_some());
    }

    #[test]
    fn 已有内容的字段不动只补空白() {
        let current = GameMetadata {
            game_id: "g1".into(),
            alias: Some("用户自己起的别名".into()),
            description: Some("用户自己写的简介".into()),
            image_path: Some("/covers/local.jpg".into()),
            rating: Some(90.0),
            titles: Some(vec![]),
            ..GameMetadata::default()
        };
        let patch = patch_for(Some(&current), &sync(), false).expect("还有空白字段");
        assert_eq!(patch.alias, None);
        assert_eq!(patch.description, None);
        assert_eq!(patch.image_path, None);
        assert_eq!(patch.rating, None);
        assert_eq!(patch.released, Some(Some("2004-01-30".into())));
        assert!(patch.tags.is_some());
    }

    #[test]
    fn 没有任何空白时不需要落库() {
        let current = GameMetadata {
            game_id: "g1".into(),
            alias: Some("别名".into()),
            released: Some("2004-01-30".into()),
            rating: Some(80.0),
            votecount: Some(100),
            length_minutes: Some(300),
            description: Some("简介".into()),
            image_path: Some("/covers/x.jpg".into()),
            titles: Some(vec![LocalizedTitle {
                lang: "zh-Hans".into(),
                title: "命运之夜".into(),
                latin: None,
                official: Some(true),
                main: Some(true),
            }]),
            tags: Some(vec![MetadataTag {
                id: Some("g1".into()),
                name: "Visual Novel".into(),
                category: Some("cont".into()),
                rating: Some(1.2),
                vn_count: Some(999),
                spoiler: Some(0),
                lie: None,
            }]),
            developers: Some(vec!["TYPE-MOON".into()]),
            staff: Some(vec![MetadataStaff {
                role: Some("scenario".into()),
                name: Some("奈须蘑菇".into()),
                original: None,
                note: None,
            }]),
            ..GameMetadata::default()
        };
        assert!(patch_for(Some(&current), &sync(), true).is_none());
    }

    #[test]
    fn 远端空值不会清空本地() {
        let mut s = sync();
        s.vn.description = None;
        s.vn.image = None;
        s.vn.released = None;
        let patch = patch_for(None, &s, false).expect("还有别的字段可填");
        assert_eq!(patch.description, None);
        assert_eq!(patch.image_path, None);
        assert_eq!(patch.released, None);
        assert_eq!(patch.alias, Some(Some("フェイト/ステイナイト".into())));
    }

    fn items(n: usize) -> Vec<BackfillItem> {
        (0..n)
            .map(|i| BackfillItem {
                game_id: format!("g{i}"),
                name: format!("游戏{i}"),
                vndb_id: None,
            })
            .collect()
    }

    #[test]
    fn 重置按有无绑定二分标注() {
        let mut list = items(3);
        list[0].vndb_id = Some("v10".into());
        list[1].vndb_id = Some("v66".into());
        reset(&list);
        let fresh = status();
        assert_eq!(fresh.items[0].bind_source, Some(BindSource::Existing));
        assert_eq!(fresh.items[1].bind_source, Some(BindSource::Existing));
        assert_eq!(fresh.items[2].bind_source, Some(BindSource::Search));
    }

    #[test]
    fn 重置建立逐条状态并清掉暂停开关() {
        reset(&items(3));
        assert!(!status().paused);
        let fresh = status();
        assert_eq!(fresh.items.len(), 3);
        assert_eq!(fresh.total, 3);
        assert!(fresh.items.iter().all(|it| it.state == ItemState::Pending));

        set_item("g1", |it| {
            it.state = ItemState::Ok;
            it.filled = vec!["简介", "封面"];
        });
        let after = status();
        assert_eq!(after.items[0].state, ItemState::Pending);
        assert_eq!(after.items[1].state, ItemState::Ok);
        assert_eq!(after.items[1].filled, vec!["简介", "封面"]);

        let one = item_state("g1").expect("应取到该条");
        assert_eq!(one.state, ItemState::Ok);
        assert_eq!(one.filled, vec!["简介", "封面"]);
        assert!(item_state("nope").is_none());

        assert!(!set_paused(true).paused_idle);
        assert!(set_paused(true).paused);
        assert!(status().paused);
        assert!(!set_paused(false).paused);
        reset(&items(1));
        assert!(!status().paused);
        assert_eq!(status().items.len(), 1);
    }
}

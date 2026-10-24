use std::sync::{Arc, Mutex};

use rusqlite::{Connection, params};

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{AppError, Result};
use crate::types::Status;
use crate::vndb::{RateLimiter, UlistLabel};

#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncPlan {
    pub total: usize,
    pub skipped: usize,
    pub creates: Vec<PlanCreate>,
    pub updates: Vec<PlanUpdate>,
    pub aligns: Vec<PlanAlign>,
    pub conflicts: Vec<SyncConflict>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCreate {
    pub vndb_id: String,
    pub name: String,
    pub status: Status,
    #[serde(default)]
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanUpdate {
    pub game_id: String,
    pub name: String,
    pub vndb_id: String,
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "double_option")]
    pub rating: Option<Option<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAlign {
    pub game_id: String,
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<f64>,
}

fn double_option<'de, D>(d: D) -> std::result::Result<Option<Option<f64>>, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(Some)
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PlanDecision {
    #[serde(default)]
    pub creates: Vec<PlanCreate>,
    #[serde(default)]
    pub updates: Vec<PlanUpdate>,
    #[serde(default)]
    pub aligns: Vec<PlanAlign>,
    #[serde(default)]
    pub statuses: Vec<StatusDecision>,
    #[serde(default)]
    pub ratings: Vec<RatingDecision>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ApplyReport {
    pub created: usize,
    pub updated: usize,
    pub resolved: usize,
    pub created_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncConflict {
    pub game_id: String,
    pub name: String,
    pub vndb_id: String,
    pub status: Option<FieldConflict<Status>>,
    pub rating: Option<FieldConflict<Option<f64>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldConflict<T> {
    pub local: T,
    pub remote: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusDecision {
    pub game_id: String,
    pub status: Status,
    pub synced: Status,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RatingDecision {
    pub game_id: String,
    pub rating: Option<f64>,
    pub synced: Option<f64>,
}

struct LocalRow {
    game_id: String,
    name: String,
    status: Status,
    user_rating: Option<f64>,
    synced_status: Option<Status>,
    synced_rating: Option<f64>,
    touched: bool,
}

enum Verdict {
    Adopt,
    Keep,
    Conflict,
    Aligned,
}

fn judge<T: PartialEq>(local: &T, remote: &T, base: Option<&T>, touched: bool) -> Verdict {
    if local == remote {
        return Verdict::Aligned;
    }
    if let Some(b) = base {
        if b == local {
            return Verdict::Adopt;
        }
        if b == remote {
            return Verdict::Keep;
        }
        return Verdict::Conflict;
    }
    if touched {
        Verdict::Conflict
    } else {
        Verdict::Adopt
    }
}

fn normalize_rating(v: Option<f64>) -> Option<f64> {
    match v {
        Some(x) if x > 0.0 => Some(x),
        _ => None,
    }
}

enum RemoteState {
    Status(Status),
    Wishlist,
    NoStatus,
}

fn map_state(labels: &[UlistLabel]) -> (RemoteState, Option<String>) {
    for l in labels {
        let state = match l.label.to_ascii_lowercase().as_str() {
            "playing" => RemoteState::Status(Status::Playing),
            "finished" => RemoteState::Status(Status::Finished),
            "stalled" => RemoteState::Status(Status::Stalled),
            "dropped" => RemoteState::Status(Status::Dropped),
            "wishlist" | "planning" | "plan to play" => RemoteState::Wishlist,
            _ => continue,
        };
        return (state, None);
    }
    let unknown = labels.iter().any(|l| !l.label.eq_ignore_ascii_case("voted"));
    let warning = unknown.then(|| {
        format!(
            "VNDB 状态「{}」暂时无法对应，已跳过",
            labels
                .iter()
                .map(|l| l.label.as_str())
                .collect::<Vec<_>>()
                .join("、")
        )
    });
    (RemoteState::NoStatus, warning)
}

pub fn plan_ulist(
    conn: &Connection,
    entries: &[crate::vndb::UlistEntry],
    include_wishlist: bool,
) -> Result<SyncPlan> {
    let local = load_local(conn)?;
    let mut plan = SyncPlan {
        total: entries.len(),
        ..Default::default()
    };
    for entry in entries {
        let vndb_id = entry.id.trim();
        if vndb_id.is_empty() {
            continue;
        }
        let (state, unknown) = map_state(&entry.labels);
        if let Some(w) = unknown {
            plan.warnings.push(w);
        }
        let remote_status = match &state {
            RemoteState::Status(s) => *s,
            RemoteState::Wishlist if include_wishlist => Status::Wishlist,
            _ => {
                plan.skipped += 1;
                continue;
            }
        };
        let remote_rating = normalize_rating(entry.vote.map(|v| v as f64));
        let Some(row) = local.get(vndb_id) else {
            let name = entry
                .vn
                .title
                .clone()
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| vndb_id.to_owned());
            plan.creates.push(PlanCreate {
                vndb_id: vndb_id.to_owned(),
                name,
                status: remote_status,
                rating: remote_rating,
            });
            continue;
        };
        let local_rating = normalize_rating(row.user_rating);
        let base_rating = normalize_rating(row.synced_rating);
        let mut update = PlanUpdate {
            game_id: row.game_id.clone(),
            name: row.name.clone(),
            vndb_id: vndb_id.to_owned(),
            status: None,
            rating: None,
        };
        let mut align = PlanAlign {
            game_id: row.game_id.clone(),
            status: None,
            rating: None,
        };
        let mut conflict = SyncConflict {
            game_id: row.game_id.clone(),
            name: row.name.clone(),
            vndb_id: vndb_id.to_owned(),
            status: None,
            rating: None,
        };
        match judge(&row.status, &remote_status, row.synced_status.as_ref(), row.touched) {
            Verdict::Adopt => update.status = Some(remote_status),
            Verdict::Keep => {}
            Verdict::Conflict => {
                conflict.status = Some(FieldConflict {
                    local: row.status,
                    remote: remote_status,
                })
            }
            Verdict::Aligned => {
                if row.synced_status != Some(remote_status) {
                    align.status = Some(remote_status);
                }
            }
        }
        let base_wrapped = base_rating.map(Some);
        match judge(&local_rating, &remote_rating, base_wrapped.as_ref(), row.touched) {
            Verdict::Adopt => update.rating = Some(remote_rating),
            Verdict::Keep => {}
            Verdict::Conflict => {
                conflict.rating = Some(FieldConflict {
                    local: local_rating,
                    remote: remote_rating,
                })
            }
            Verdict::Aligned => {
                if base_rating != remote_rating {
                    align.rating = remote_rating;
                }
            }
        }
        if update.status.is_some() || update.rating.is_some() {
            plan.updates.push(update);
        }
        if align.status.is_some() || align.rating.is_some() {
            plan.aligns.push(align);
        }
        if conflict.status.is_some() || conflict.rating.is_some() {
            plan.conflicts.push(conflict);
        }
    }
    Ok(plan)
}

pub fn apply(conn: &Connection, decision: &PlanDecision) -> Result<ApplyReport> {
    let tx = conn.unchecked_transaction()?;
    let mut report = ApplyReport::default();
    for c in &decision.creates {
        if vndb_exists(&tx, &c.vndb_id)? {
            continue;
        }
        let id = crate::new_id();
        tx.execute(
            "INSERT INTO games (id, name, vndb_id, created_at) VALUES (?, ?, ?, ?)",
            params![id, c.name, c.vndb_id, crate::now()],
        )?;
        tx.execute(
            "INSERT INTO user_game_state (game_id, status, user_rating, synced_status, synced_rating)
             VALUES (?, ?, ?, ?, ?)",
            params![id, c.status.as_str(), c.rating, c.status.as_str(), c.rating],
        )?;
        report.created += 1;
        report.created_ids.push(id);
    }
    for u in &decision.updates {
        if let Some(status) = u.status {
            tx.execute(
                "UPDATE user_game_state
                 SET status = ?, synced_status = ?, state_modified_at = ?
                 WHERE game_id = ?",
                params![status.as_str(), status.as_str(), crate::now(), u.game_id],
            )?;
            report.updated += 1;
        }
        if let Some(rating) = u.rating {
            tx.execute(
                "UPDATE user_game_state
                 SET user_rating = ?, synced_rating = ?, state_modified_at = ?
                 WHERE game_id = ?",
                params![rating, rating, crate::now(), u.game_id],
            )?;
            report.updated += 1;
        }
    }
    for a in &decision.aligns {
        if let Some(status) = a.status {
            tx.execute(
                "UPDATE user_game_state SET synced_status = ? WHERE game_id = ?",
                params![status.as_str(), a.game_id],
            )?;
        }
        if let Some(rating) = a.rating {
            tx.execute(
                "UPDATE user_game_state SET synced_rating = ? WHERE game_id = ?",
                params![rating, a.game_id],
            )?;
        }
    }
    for d in &decision.statuses {
        let changes = tx.execute(
            "UPDATE user_game_state
             SET status = ?, synced_status = ?, state_modified_at = ?
             WHERE game_id = ?",
            params![d.status.as_str(), d.synced.as_str(), crate::now(), d.game_id],
        )?;
        if changes == 0 {
            return Err(AppError::NotFound("游戏不存在".into()));
        }
        report.resolved += 1;
    }
    for d in &decision.ratings {
        let changes = tx.execute(
            "UPDATE user_game_state
             SET user_rating = ?, synced_rating = ?, state_modified_at = ?
             WHERE game_id = ?",
            params![d.rating, d.synced, crate::now(), d.game_id],
        )?;
        if changes == 0 {
            return Err(AppError::NotFound("游戏不存在".into()));
        }
        report.resolved += 1;
    }
    tx.commit()?;
    invalidate_plan();
    Ok(report)
}

fn vndb_exists(tx: &Connection, vndb_id: &str) -> Result<bool> {
    let n: i64 = tx.query_row(
        "SELECT COUNT(*) FROM games WHERE vndb_id = ?",
        params![vndb_id],
        |row| row.get(0),
    )?;
    Ok(n > 0)
}

fn load_local(conn: &Connection) -> Result<std::collections::HashMap<String, LocalRow>> {
    let mut stmt = conn.prepare(
        "SELECT g.vndb_id, g.id, g.name, s.status, s.user_rating,
                s.synced_status, s.synced_rating, s.state_modified_at
         FROM games g
         JOIN user_game_state s ON s.game_id = g.id
         WHERE g.vndb_id IS NOT NULL",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>("vndb_id")?,
            LocalRow {
                game_id: row.get("id")?,
                name: row.get("name")?,
                status: row.get("status")?,
                user_rating: row.get("user_rating")?,
                synced_status: row.get("synced_status")?,
                synced_rating: row.get("synced_rating")?,
                touched: row.get::<_, Option<i64>>("state_modified_at")?.is_some(),
            },
        ))
    })?;
    let mut out = std::collections::HashMap::new();
    for row in rows {
        let (k, v) = row?;
        out.insert(k, v);
    }
    Ok(out)
}

pub async fn plan(conn: Arc<Mutex<Connection>>, rate: &RateLimiter) -> Result<SyncPlan> {
    let config = crate::settings::current();
    if config.vndb_sync_mode != crate::settings::VndbSyncMode::Download {
        return Err(AppError::BadRequest(
            "仅上传 / 双向同步还没做好，先在设置里选「仅下载」".into(),
        ));
    }
    let token = config
        .vndb_token_obf
        .as_deref()
        .and_then(crate::settings::deobfuscate_token)
        .ok_or_else(|| AppError::BadRequest("还没有保存 VNDB 令牌".into()))?;
    let include_wishlist = config.vndb_sync_import_wishlist;
    if let Some(cached) = cached_plan() {
        return Ok(cached);
    }
    let entries = crate::vndb::fetch_ulist(crate::vndb::http(), &token, rate).await?;
    let plan = tauri::async_runtime::spawn_blocking(move || {
        let guard = crate::db::lock(&conn)?;
        plan_ulist(&guard, &entries, include_wishlist)
    })
    .await
    .map_err(|e| AppError::Upstream(format!("同步任务失败：{e}")))??;
    store_plan(&plan);
    Ok(plan)
}

const PLAN_TTL_SECS: i64 = 60;

fn plan_cache() -> &'static Mutex<Option<(i64, SyncPlan)>> {
    static CELL: std::sync::OnceLock<Mutex<Option<(i64, SyncPlan)>>> = std::sync::OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

pub fn cached_plan() -> Option<SyncPlan> {
    cached_plan_at(crate::now())
}

pub fn cached_plan_at(now: i64) -> Option<SyncPlan> {
    let guard = plan_cache().lock().ok()?;
    let (at, plan) = guard.as_ref()?;
    (now - *at < PLAN_TTL_SECS).then(|| plan.clone())
}

pub fn store_plan(plan: &SyncPlan) -> i64 {
    let at = crate::now();
    if let Ok(mut guard) = plan_cache().lock() {
        *guard = Some((at, plan.clone()));
    }
    at
}

pub fn invalidate_plan() {
    if let Ok(mut guard) = plan_cache().lock() {
        *guard = None;
    }
}

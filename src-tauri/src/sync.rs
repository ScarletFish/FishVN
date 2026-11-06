use std::sync::{Arc, Mutex};

use rusqlite::{Connection, params};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::settings::VndbSyncMode;
use crate::types::{Status, double_option};
use crate::vndb::{RateLimiter, UlistLabel};

#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncPlan {
    pub total: usize,
    pub mode: VndbSyncMode,
    pub skipped: usize,
    pub creates: Vec<PlanCreate>,
    pub updates: Vec<PlanUpdate>,
    pub aligns: Vec<PlanAlign>,
    pub conflicts: Vec<SyncConflict>,
    pub pushes: Vec<PlanPush>,
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
    #[serde(default)]
    pub pushes: Vec<PlanPush>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ApplyReport {
    pub created: usize,
    pub updated: usize,
    pub resolved: usize,
    pub pushed: usize,
    pub created_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPush {
    pub game_id: String,
    pub vndb_id: String,
    pub name: String,
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "double_option")]
    pub rating: Option<Option<f64>>,
    #[serde(default)]
    pub remote_status: Option<Status>,
    #[serde(default)]
    pub remote_rating: Option<f64>,
    #[serde(default)]
    pub local_at: Option<i64>,
    #[serde(default)]
    pub remote_at: Option<i64>,
    #[serde(default)]
    pub overrides: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Prefer {
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncConflict {
    pub game_id: String,
    pub name: String,
    pub vndb_id: String,
    pub status: Option<FieldConflict<Status>>,
    pub rating: Option<FieldConflict<Option<f64>>>,
    pub local_at: Option<i64>,
    pub remote_at: Option<i64>,
    pub prefer: Prefer,
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
    modified_at: Option<i64>,
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
        if let Some(status) = crate::vndb::status_of_label(&l.label) {
            let state = if status == Status::Wishlist {
                RemoteState::Wishlist
            } else {
                RemoteState::Status(status)
            };
            return (state, None);
        }
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
    mode: VndbSyncMode,
    sync_rating: bool,
) -> Result<SyncPlan> {
    let local = load_local(conn)?;
    let mut plan = SyncPlan {
        total: entries.len(),
        mode,
        ..Default::default()
    };
    let mut seen = std::collections::HashSet::new();
    for entry in entries {
        let vndb_id = entry.id.trim();
        if vndb_id.is_empty() || !seen.insert(vndb_id.to_owned()) {
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
        let remote_at = entry.lastmod;
        let Some(row) = local.get(vndb_id) else {
            if !mode.downloads() {
                plan.skipped += 1;
                continue;
            }
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
        let touched = row.modified_at.is_some();
        let status_verdict = judge(
            &row.status,
            &remote_status,
            row.synced_status.as_ref(),
            touched,
        );
        let rating_verdict = sync_rating.then(|| {
            judge(
                &local_rating,
                &remote_rating,
                base_rating.map(Some).as_ref(),
                touched,
            )
        });
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
        let mut push = PlanPush {
            game_id: row.game_id.clone(),
            vndb_id: vndb_id.to_owned(),
            name: row.name.clone(),
            status: None,
            rating: None,
            remote_status: Some(remote_status),
            remote_rating,
            local_at: row.modified_at,
            remote_at,
            overrides: false,
        };
        let mut conflict = SyncConflict {
            game_id: row.game_id.clone(),
            name: row.name.clone(),
            vndb_id: vndb_id.to_owned(),
            status: None,
            rating: None,
            local_at: row.modified_at,
            remote_at,
            prefer: prefer_for(mode, row.modified_at, remote_at),
        };
        match status_verdict {
            Verdict::Aligned => {
                if row.synced_status != Some(remote_status) {
                    align.status = Some(remote_status);
                }
            }
            Verdict::Adopt => {
                if mode.downloads() {
                    update.status = Some(remote_status);
                }
            }
            Verdict::Keep => {
                if mode.uploads() {
                    push.status = Some(row.status);
                }
            }
            Verdict::Conflict => {
                if mode == VndbSyncMode::Upload {
                    push.status = Some(row.status);
                    push.overrides = true;
                } else {
                    conflict.status = Some(FieldConflict {
                        local: row.status,
                        remote: remote_status,
                    })
                }
            }
        }
        if let Some(verdict) = rating_verdict {
            match verdict {
                Verdict::Aligned => {
                    if base_rating != remote_rating {
                        align.rating = remote_rating;
                    }
                }
                Verdict::Adopt => {
                    if mode.downloads() {
                        update.rating = Some(remote_rating);
                    }
                }
                Verdict::Keep => {
                    if mode.uploads() {
                        push.rating = Some(local_rating);
                    }
                }
                Verdict::Conflict => {
                    if mode == VndbSyncMode::Upload {
                        push.rating = Some(local_rating);
                        push.overrides = true;
                    } else {
                        conflict.rating = Some(FieldConflict {
                            local: local_rating,
                            remote: remote_rating,
                        })
                    }
                }
            }
        }
        if update.status.is_some() || update.rating.is_some() {
            plan.updates.push(update);
        }
        if align.status.is_some() || align.rating.is_some() {
            plan.aligns.push(align);
        }
        if push.status.is_some() || push.rating.is_some() {
            plan.pushes.push(push);
        }
        if conflict.status.is_some() || conflict.rating.is_some() {
            plan.conflicts.push(conflict);
        }
    }
    if mode.uploads() {
        for (vndb_id, row) in &local {
            if seen.contains(vndb_id) {
                continue;
            }
            if row.status == Status::Wishlist && !include_wishlist {
                plan.skipped += 1;
                continue;
            }
            plan.pushes.push(PlanPush {
                game_id: row.game_id.clone(),
                vndb_id: vndb_id.clone(),
                name: row.name.clone(),
                status: Some(row.status),
                rating: if sync_rating && row.user_rating.is_some_and(|v| v > 0.0) {
                    Some(normalize_rating(row.user_rating))
                } else {
                    None
                },
                remote_status: None,
                remote_rating: None,
                local_at: row.modified_at,
                remote_at: None,
                overrides: false,
            });
        }
    }
    Ok(plan)
}

fn prefer_for(mode: VndbSyncMode, local_at: Option<i64>, remote_at: Option<i64>) -> Prefer {
    if mode != VndbSyncMode::Both {
        return Prefer::Local;
    }
    match (local_at, remote_at) {
        (Some(l), Some(r)) if r > l => Prefer::Remote,
        _ => Prefer::Local,
    }
}

fn update_one(tx: &Connection, sql: &str, values: impl rusqlite::Params) -> Result<()> {
    if tx.execute(sql, values)? == 0 {
        return Err(AppError::NotFound("游戏不存在".into()));
    }
    Ok(())
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
            update_one(
                &tx,
                "UPDATE user_game_state
                 SET status = ?, synced_status = ?, state_modified_at = ?
                 WHERE game_id = ?",
                params![status.as_str(), status.as_str(), crate::now(), u.game_id],
            )?;
            report.updated += 1;
        }
        if let Some(rating) = u.rating {
            update_one(
                &tx,
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
            update_one(
                &tx,
                "UPDATE user_game_state SET synced_status = ? WHERE game_id = ?",
                params![status.as_str(), a.game_id],
            )?;
        }
        if let Some(rating) = a.rating {
            update_one(
                &tx,
                "UPDATE user_game_state SET synced_rating = ? WHERE game_id = ?",
                params![rating, a.game_id],
            )?;
        }
    }
    for d in &decision.statuses {
        update_one(
            &tx,
            "UPDATE user_game_state
             SET status = ?, synced_status = ?, state_modified_at = ?
             WHERE game_id = ?",
            params![d.status.as_str(), d.synced.as_str(), crate::now(), d.game_id],
        )?;
        report.resolved += 1;
    }
    for d in &decision.ratings {
        update_one(
            &tx,
            "UPDATE user_game_state
             SET user_rating = ?, synced_rating = ?, state_modified_at = ?
             WHERE game_id = ?",
            params![d.rating, d.synced, crate::now(), d.game_id],
        )?;
        report.resolved += 1;
    }
    tx.commit()?;
    invalidate_plan();
    Ok(report)
}

pub async fn push(
    conn: Arc<Mutex<Connection>>,
    http: &reqwest::Client,
    token: &str,
    labels: &crate::vndb::LabelMap,
    pushes: &[PlanPush],
    rate: &RateLimiter,
    report: impl Fn(usize, usize),
) -> Result<usize> {
    let total = pushes.len();
    let mut done = 0;
    for p in pushes {
        rate.acquire().await;
        crate::vndb::set_ulist(http, token, &p.vndb_id, labels, p.status, p.rating).await?;
        {
            let guard = crate::db::lock(&conn)?;
            if let Some(status) = p.status {
                guard.execute(
                    "UPDATE user_game_state SET synced_status = ? WHERE game_id = ?",
                    params![status.as_str(), p.game_id],
                )?;
            }
            if let Some(rating) = p.rating {
                guard.execute(
                    "UPDATE user_game_state SET synced_rating = ? WHERE game_id = ?",
                    params![rating, p.game_id],
                )?;
            }
        }
        done += 1;
        report(done, total);
    }
    invalidate_plan();
    Ok(done)
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
                modified_at: row.get("state_modified_at")?,
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

pub async fn plan(
    conn: Arc<Mutex<Connection>>,
    rate: &RateLimiter,
    report: impl Fn(&str, Option<i64>, Option<i64>),
) -> Result<SyncPlan> {
    let config = crate::settings::current();
    let mode = config.vndb_sync_mode;
    let sync_rating = config.vndb_sync_rating;
    let token = config
        .vndb_token_obf
        .as_deref()
        .and_then(crate::settings::deobfuscate_token)
        .ok_or_else(|| AppError::BadRequest("还没有保存 VNDB 令牌".into()))?;
    let include_wishlist = config.vndb_sync_wishlist;
    if let Some(cached) = cached_plan() {
        return Ok(cached);
    }
    if mode.uploads()
        && !crate::vndb::has_listwrite(crate::vndb::http(), &token).await?
    {
        return Err(AppError::BadRequest(
            "Token 没有写入权限，无法上传。去 vndb.org 重新生成 Token 时勾选 listwrite".into(),
        ));
    }
    let entries = crate::vndb::fetch_ulist(crate::vndb::http(), &token, rate, |fetched, total| {
        report("fetch", Some(fetched as i64), total);
    })
    .await?;
    report("plan", None, None);
    let plan = tauri::async_runtime::spawn_blocking(move || {
        let guard = crate::db::lock(&conn)?;
        plan_ulist(&guard, &entries, include_wishlist, mode, sync_rating)
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
    (now >= *at && now.saturating_sub(*at) < PLAN_TTL_SECS).then(|| plan.clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vndb::{UlistEntry, UlistLabel, UlistVn};
    use rusqlite::Connection;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::ensure_schema(&conn).unwrap();
        conn
    }

    fn seed(
        conn: &Connection,
        vndb_id: &str,
        status: Status,
        rating: Option<f64>,
        base_status: Option<Status>,
        base_rating: Option<f64>,
        modified: Option<i64>,
    ) {
        conn.execute(
            "INSERT INTO games (id, name, vndb_id, created_at) VALUES (?, ?, ?, 0)",
            params![vndb_id, "本地名", vndb_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO user_game_state
             (game_id, status, user_rating, synced_status, synced_rating, state_modified_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                vndb_id,
                status.as_str(),
                rating,
                base_status.map(Status::as_str),
                base_rating,
                modified
            ],
        )
        .unwrap();
    }

    fn entry(vndb_id: &str, label: &str, vote: Option<i64>, lastmod: Option<i64>) -> UlistEntry {
        UlistEntry {
            id: vndb_id.into(),
            vn: UlistVn {
                title: Some("远端名".into()),
            },
            labels: vec![UlistLabel {
                id: 1,
                label: label.into(),
            }],
            vote,
            lastmod,
        }
    }

    #[test]
    fn 仅下载采纳远端改动() {
        let conn = db();
        seed(
            &conn,
            "v1",
            Status::Playing,
            Some(80.0),
            Some(Status::Playing),
            Some(80.0),
            Some(100),
        );
        let entries = vec![entry("v1", "finished", Some(90), Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Download, true).unwrap();
        assert_eq!(plan.updates.len(), 1);
        assert_eq!(plan.updates[0].status, Some(Status::Finished));
        assert_eq!(plan.updates[0].rating, Some(Some(90.0)));
        assert!(plan.pushes.is_empty());
    }

    #[test]
    fn 仅下载不动本地改动() {
        let conn = db();
        seed(
            &conn,
            "v1",
            Status::Finished,
            Some(80.0),
            Some(Status::Playing),
            Some(80.0),
            Some(100),
        );
        let entries = vec![entry("v1", "playing", Some(80), Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Download, true).unwrap();
        assert!(plan.updates.is_empty());
        assert!(plan.pushes.is_empty());
    }

    #[test]
    fn 仅上传推送本地改动() {
        let conn = db();
        seed(
            &conn,
            "v1",
            Status::Finished,
            Some(80.0),
            Some(Status::Playing),
            Some(80.0),
            Some(100),
        );
        let entries = vec![entry("v1", "playing", Some(80), Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Upload, true).unwrap();
        assert_eq!(plan.pushes.len(), 1);
        assert_eq!(plan.pushes[0].status, Some(Status::Finished));
        assert!(!plan.pushes[0].overrides);
        assert!(plan.updates.is_empty());
    }

    #[test]
    fn 仅上传不覆盖远端改动() {
        let conn = db();
        seed(
            &conn,
            "v1",
            Status::Playing,
            Some(80.0),
            Some(Status::Playing),
            Some(80.0),
            Some(100),
        );
        let entries = vec![entry("v1", "finished", Some(80), Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Upload, true).unwrap();
        assert!(plan.pushes.is_empty());
        assert!(plan.updates.is_empty());
    }

    #[test]
    fn 仅上传时两边都改的条目自动推送并标记覆盖() {
        let conn = db();
        seed(
            &conn,
            "v1",
            Status::Finished,
            Some(80.0),
            Some(Status::Playing),
            Some(80.0),
            Some(100),
        );
        let entries = vec![entry("v1", "dropped", Some(80), Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Upload, true).unwrap();
        assert!(plan.conflicts.is_empty());
        assert_eq!(plan.pushes.len(), 1);
        assert!(plan.pushes[0].overrides);
    }

    #[test]
    fn 双向冲突默认选较新的一侧() {
        let conn = db();
        seed(
            &conn,
            "v1",
            Status::Finished,
            Some(80.0),
            Some(Status::Playing),
            Some(80.0),
            Some(100),
        );
        let entries = vec![entry("v1", "dropped", Some(80), Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Both, true).unwrap();
        assert_eq!(plan.conflicts.len(), 1);
        let conflict = &plan.conflicts[0];
        assert_eq!(conflict.prefer, Prefer::Remote);
        assert!(conflict.status.is_some());
        assert!(conflict.rating.is_none());
    }

    #[test]
    fn 双向冲突本地更晚时选本地() {
        let conn = db();
        seed(
            &conn,
            "v1",
            Status::Finished,
            Some(80.0),
            Some(Status::Playing),
            Some(80.0),
            Some(300),
        );
        let entries = vec![entry("v1", "dropped", Some(80), Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Both, true).unwrap();
        assert_eq!(plan.conflicts[0].prefer, Prefer::Local);
    }

    #[test]
    fn 本地独有的条目在仅上传时新增() {
        let conn = db();
        seed(&conn, "v1", Status::Finished, Some(85.0), None, None, Some(100));
        let plan = plan_ulist(&conn, &[], false, VndbSyncMode::Upload, true).unwrap();
        assert_eq!(plan.pushes.len(), 1);
        assert_eq!(plan.pushes[0].status, Some(Status::Finished));
        assert_eq!(plan.pushes[0].rating, Some(Some(85.0)));
        assert!(!plan.pushes[0].overrides);
    }

    #[test]
    fn 本地独有的条目在仅下载时不出现() {
        let conn = db();
        seed(&conn, "v1", Status::Finished, Some(85.0), None, None, Some(100));
        let plan = plan_ulist(&conn, &[], false, VndbSyncMode::Download, true).unwrap();
        assert!(plan.pushes.is_empty());
    }

    #[test]
    fn 关闭评分同步后评分不参与() {
        let conn = db();
        seed(
            &conn,
            "v1",
            Status::Playing,
            Some(70.0),
            Some(Status::Playing),
            Some(80.0),
            Some(100),
        );
        let entries = vec![entry("v1", "playing", Some(80), Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Upload, false).unwrap();
        assert!(plan.pushes.iter().all(|p| p.rating.is_none()));
        assert!(plan.updates.iter().all(|u| u.rating.is_none()));
        assert!(plan.aligns.iter().all(|a| a.rating.is_none()));
        assert!(plan.conflicts.is_empty());
    }

    #[test]
    fn 评分清空也推送() {
        let conn = db();
        seed(
            &conn,
            "v1",
            Status::Playing,
            None,
            Some(Status::Playing),
            Some(80.0),
            Some(100),
        );
        let entries = vec![entry("v1", "playing", Some(80), Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Upload, true).unwrap();
        assert_eq!(plan.pushes.len(), 1);
        assert_eq!(plan.pushes[0].rating, Some(None));
    }

    #[test]
    fn 心愿单关闭时跳过() {
        let conn = db();
        let entries = vec![entry("v1", "wishlist", None, Some(200))];
        let plan = plan_ulist(&conn, &entries, false, VndbSyncMode::Download, true).unwrap();
        assert_eq!(plan.skipped, 1);
        assert!(plan.creates.is_empty());
    }
}

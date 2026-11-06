//! 同步内核单测：只覆盖 plan_ulist（判定）与 apply（落库），不碰网络。
//! UlistEntry 用测试值直接构造，替代真实 ulist 响应。

mod common;

use common::Db;
use fishvn::games::{create_game, delete_game, update_game, update_game_state};
use fishvn::settings::VndbSyncMode;
use fishvn::sync::{
    apply, cached_plan_at, invalidate_plan, plan_ulist, store_plan, PlanAlign, PlanCreate,
    PlanDecision, PlanUpdate, RatingDecision, StatusDecision, SyncPlan,
};
use fishvn::types::{GamePatch, NewGame, StatePatch, Status};
use fishvn::vndb::{UlistEntry, UlistLabel, UlistVn};
use rusqlite::params;

// 计划缓存是进程级全局，直接读写它的用例和 apply 落库（末尾即使之失效）的用例
// 并行执行会互相打断断言，这把锁把它们串起来。
static PLAN_CACHE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn plan_cache_guard() -> std::sync::MutexGuard<'static, ()> {
    PLAN_CACHE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn entry(vndb_id: &str, label: &str, vote: Option<i64>) -> UlistEntry {
    UlistEntry {
        id: vndb_id.to_owned(),
        vn: UlistVn {
            title: Some("测试作品".to_owned()),
        },
        labels: vec![UlistLabel {
            id: 1,
            label: label.to_owned(),
        }],
        vote,
        lastmod: None,
    }
}

fn voted_entry(vndb_id: &str, vote: Option<i64>) -> UlistEntry {
    UlistEntry {
        id: vndb_id.to_owned(),
        vn: UlistVn {
            title: Some("测试作品".to_owned()),
        },
        labels: vec![UlistLabel {
            id: 6,
            label: "Voted".to_owned(),
        }],
        vote,
        lastmod: None,
    }
}

fn new_game(db: &Db, name: &str, vndb_id: &str) -> String {
    create_game(
        db,
        &NewGame {
            name: name.into(),
            vndb_id: Some(vndb_id.into()),
            metadata: None,
        },
    )
    .expect("建作品失败")
    .id
}

fn set_state(db: &Db, id: &str, status: Option<Status>, rating: Option<Option<f64>>) {
    update_game_state(
        db,
        id,
        &StatePatch {
            status,
            user_rating: rating,
            ..Default::default()
        },
    )
    .expect("改状态失败");
}

fn set_synced(db: &Db, id: &str, status: Option<Status>, rating: Option<f64>) {
    db.execute(
        "UPDATE user_game_state SET synced_status = ?, synced_rating = ? WHERE game_id = ?",
        params![status, rating, id],
    )
    .expect("设基快照失败");
}

fn state_of(db: &Db, id: &str) -> (Status, Option<f64>, Option<Status>, Option<f64>) {
    db.query_row(
        "SELECT status, user_rating, synced_status, synced_rating
         FROM user_game_state WHERE game_id = ?",
        params![id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )
    .expect("读状态失败")
}

fn modified_of(db: &Db, id: &str) -> bool {
    db.query_row(
        "SELECT state_modified_at IS NOT NULL FROM user_game_state WHERE game_id = ?",
        params![id],
        |row| row.get(0),
    )
    .expect("读修改标记失败")
}

fn games_count(db: &Db) -> i64 {
    db.query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
        .expect("数作品失败")
}

type StateSnapshot = (String, Status, Option<f64>, Option<Status>, Option<f64>);

fn states_snapshot(db: &Db) -> Vec<StateSnapshot> {
    let mut stmt = db
        .prepare(
            "SELECT s.game_id, s.status, s.user_rating, s.synced_status, s.synced_rating
             FROM user_game_state s ORDER BY s.game_id",
        )
        .expect("准备快照失败");
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })
        .expect("查快照失败");
    rows.collect::<rusqlite::Result<_>>().expect("收快照失败")
}

// ---------- 判定：plan_ulist 不落库 ----------

#[test]
fn 仅_vndb_改过时计划采纳远端但不落库() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_synced(&db, &id, Some(Status::Wishlist), None);

    let plan = plan_ulist(&db, &[entry("v1", "Playing", None)], false, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.updates.len(), 1);
    assert_eq!(plan.updates[0].status, Some(Status::Playing));
    assert!(plan.conflicts.is_empty());
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Wishlist, "plan 阶段不写库");
}

#[test]
fn 仅本地改过时保留本地状态() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Playing), None);
    set_synced(&db, &id, Some(Status::Wishlist), None);

    let plan = plan_ulist(&db, &[entry("v1", "Wishlist", None)], true, VndbSyncMode::Download, true).unwrap();

    assert!(plan.updates.is_empty());
    assert!(plan.conflicts.is_empty());
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Playing);
}

#[test]
fn 两边都改过时状态进冲突且本地不动() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Finished), None);
    set_synced(&db, &id, Some(Status::Playing), None);

    let plan = plan_ulist(&db, &[entry("v1", "Dropped", None)], false, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.conflicts.len(), 1);
    let conflict = &plan.conflicts[0];
    assert_eq!(conflict.status.as_ref().unwrap().local, Status::Finished);
    assert_eq!(conflict.status.as_ref().unwrap().remote, Status::Dropped);
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Finished);
}

#[test]
fn 从未同步过的初始值直接采纳远端() {
    let db = Db::new();
    new_game(&db, "甲", "v1");

    let plan = plan_ulist(&db, &[entry("v1", "Playing", None)], false, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.updates.len(), 1);
    assert!(plan.conflicts.is_empty());
}

#[test]
fn 从未同步但本地手改过则判冲突() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Finished), None);

    let plan = plan_ulist(&db, &[entry("v1", "Playing", None)], false, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.conflicts.len(), 1);
    assert!(plan.conflicts[0].status.is_some());
}

#[test]
fn 两边改成同一值时计划只刷基快照() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Playing), None);
    set_synced(&db, &id, Some(Status::Wishlist), None);

    let plan = plan_ulist(&db, &[entry("v1", "Playing", None)], false, VndbSyncMode::Download, true).unwrap();

    assert!(plan.updates.is_empty());
    assert_eq!(plan.aligns.len(), 1);
    assert_eq!(plan.aligns[0].status, Some(Status::Playing));
    let (.., synced_status, _) = state_of(&db, &id);
    assert_eq!(synced_status, Some(Status::Wishlist), "plan 阶段不写库");
}

#[test]
fn 新_vndb_id_进创建计划并带齐字段() {
    let db = Db::new();

    let plan = plan_ulist(&db, &[entry("v9", "Playing", Some(85))], false, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.creates.len(), 1);
    let created = &plan.creates[0];
    assert_eq!(created.name, "测试作品");
    assert_eq!(created.vndb_id, "v9");
    assert_eq!(created.status, Status::Playing);
    assert_eq!(created.rating, Some(85.0));
    assert_eq!(games_count(&db), 0, "plan 阶段不建库");
}

#[test]
fn 新_vndb_id_查不到标题时用_id_兜底() {
    let db = Db::new();
    let mut item = entry("v7", "Finished", None);
    item.vn.title = None;

    let plan = plan_ulist(&db, &[item], false, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.creates.len(), 1);
    assert_eq!(plan.creates[0].name, "v7");
}

#[test]
fn 评分仅_vndb_改过时计划采纳远端() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_synced(&db, &id, Some(Status::Wishlist), None);

    let plan = plan_ulist(&db, &[entry("v1", "Wishlist", Some(85))], true, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.updates.len(), 1);
    assert_eq!(plan.updates[0].rating, Some(Some(85.0)));
    let (_, rating, ..) = state_of(&db, &id);
    assert_eq!(rating, None, "plan 阶段不写库");
}

#[test]
fn 评分仅本地改过时保留本地() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, None, Some(Some(85.0)));
    set_synced(&db, &id, Some(Status::Wishlist), Some(90.0));

    let plan = plan_ulist(&db, &[entry("v1", "Wishlist", Some(90))], true, VndbSyncMode::Download, true).unwrap();

    assert!(plan.updates.is_empty());
    let (_, rating, ..) = state_of(&db, &id);
    assert_eq!(rating, Some(85.0));
}

#[test]
fn 评分两边都改过时进冲突() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, None, Some(Some(70.0)));
    set_synced(&db, &id, Some(Status::Wishlist), Some(85.0));

    let plan = plan_ulist(&db, &[entry("v1", "Wishlist", Some(90))], true, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.conflicts.len(), 1);
    let rating = plan.conflicts[0].rating.as_ref().unwrap();
    assert_eq!(rating.local, Some(70.0));
    assert_eq!(rating.remote, Some(90.0));
    let (_, local_rating, ..) = state_of(&db, &id);
    assert_eq!(local_rating, Some(70.0));
}

#[test]
fn 本地显式清零评分与远端有分判冲突() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, None, Some(Some(0.0)));

    let plan = plan_ulist(&db, &[entry("v1", "Wishlist", Some(85))], true, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.conflicts.len(), 1);
    assert!(plan.conflicts[0].rating.is_some());
}

#[test]
fn 两端都未评分时无事发生() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_synced(&db, &id, Some(Status::Wishlist), None);

    let plan = plan_ulist(&db, &[entry("v1", "Wishlist", None)], true, VndbSyncMode::Download, true).unwrap();

    assert!(plan.updates.is_empty());
    assert!(plan.conflicts.is_empty());
    let (_, rating, ..) = state_of(&db, &id);
    assert_eq!(rating, None);
}

#[test]
fn 未知_vndb_状态跳过并警告不建壳() {
    let db = Db::new();

    let plan = plan_ulist(&db, &[entry("v1", "Weird", None)], true, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.skipped, 1);
    assert_eq!(plan.warnings.len(), 1);
    assert!(plan.warnings[0].contains("Weird"));
    assert!(plan.creates.is_empty());
    assert!(plan.updates.is_empty());
    assert!(plan.conflicts.is_empty());
}

#[test]
fn 未解决的冲突再次计划仍判冲突() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Finished), None);
    set_synced(&db, &id, Some(Status::Playing), None);

    let first = plan_ulist(&db, &[entry("v1", "Dropped", None)], false, VndbSyncMode::Download, true).unwrap();
    assert_eq!(first.conflicts.len(), 1);

    let second = plan_ulist(&db, &[entry("v1", "Dropped", None)], false, VndbSyncMode::Download, true).unwrap();
    assert_eq!(second.conflicts.len(), 1);
}

#[test]
fn 心愿单开关关闭时跳过新条目且不建壳() {
    let db = Db::new();

    let plan = plan_ulist(&db, &[entry("v9", "Wishlist", None)], false, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.skipped, 1);
    assert!(plan.creates.is_empty());
    assert!(plan.updates.is_empty());
    assert!(plan.conflicts.is_empty());
    assert!(plan.warnings.is_empty());
}

#[test]
fn 心愿单开关关闭时跳过已存在条目的更新() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Playing), None);
    set_synced(&db, &id, Some(Status::Playing), None);

    let plan = plan_ulist(&db, &[entry("v1", "Wishlist", None)], false, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.skipped, 1);
    assert!(plan.updates.is_empty());
    assert!(plan.conflicts.is_empty());
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Playing);
}

#[test]
fn 心愿单开关打开时新条目进创建计划() {
    let db = Db::new();

    let plan = plan_ulist(&db, &[entry("v9", "Wishlist", Some(80))], true, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.skipped, 0);
    assert_eq!(plan.creates.len(), 1);
    assert_eq!(plan.creates[0].status, Status::Wishlist);
    assert_eq!(plan.creates[0].rating, Some(80.0));
}

#[test]
fn 心愿单标签名为_planning_时同样识别() {
    let db = Db::new();

    let plan = plan_ulist(&db, &[entry("v1", "Planning", None)], true, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.skipped, 0);
    assert_eq!(plan.creates.len(), 1);
    assert_eq!(plan.creates[0].status, Status::Wishlist);
    assert!(plan.warnings.is_empty());
}

#[test]
fn 纯投票条目恒跳过且不警告() {
    let db = Db::new();

    let plan = plan_ulist(&db, &[voted_entry("v9", Some(90))], true, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.skipped, 1);
    assert!(plan.warnings.is_empty());
    assert!(plan.creates.is_empty());
    assert!(plan.updates.is_empty());
}

#[test]
fn 纯投票条目不覆盖本地已有状态与评分() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Playing), Some(Some(70.0)));
    set_synced(&db, &id, Some(Status::Playing), Some(70.0));

    let plan = plan_ulist(&db, &[voted_entry("v1", Some(90))], true, VndbSyncMode::Download, true).unwrap();

    assert_eq!(plan.skipped, 1);
    assert!(plan.updates.is_empty());
    let (status, rating, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Playing);
    assert_eq!(rating, Some(70.0));
}

#[test]
fn 跳过心愿单后打开开关_本地未再改则计划采纳远端() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Playing), None);
    set_synced(&db, &id, Some(Status::Playing), None);

    let first = plan_ulist(&db, &[entry("v1", "Wishlist", None)], false, VndbSyncMode::Download, true).unwrap();
    assert_eq!(first.skipped, 1);
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Playing);

    let second = plan_ulist(&db, &[entry("v1", "Wishlist", None)], true, VndbSyncMode::Download, true).unwrap();
    assert_eq!(second.skipped, 0);
    assert_eq!(second.updates.len(), 1);
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Playing, "plan 阶段不写库");
}

#[test]
fn 跳过心愿单后打开开关_本地改过则判冲突() {
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Playing), None);
    set_synced(&db, &id, Some(Status::Playing), None);

    let first = plan_ulist(&db, &[entry("v1", "Wishlist", None)], false, VndbSyncMode::Download, true).unwrap();
    assert_eq!(first.skipped, 1);

    set_state(&db, &id, Some(Status::Finished), None);
    let second = plan_ulist(&db, &[entry("v1", "Wishlist", None)], true, VndbSyncMode::Download, true).unwrap();
    assert_eq!(second.conflicts.len(), 1);
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Finished);
}

// ---------- 应用：apply 落库 ----------

#[test]
fn plan_全程不写库() {
    let db = Db::new();
    let existing = new_game(&db, "甲", "v1");
    set_state(&db, &existing, Some(Status::Finished), Some(Some(70.0)));
    set_synced(&db, &existing, Some(Status::Playing), Some(85.0));

    let before_games = games_count(&db);
    let before_states = states_snapshot(&db);

    let plan = plan_ulist(
        &db,
        &[
            entry("v1", "Dropped", Some(90)),  // 冲突
            entry("v2", "Playing", None),     // 新建
            entry("v3", "Wishlist", None),    // 开关关 → 跳过
            voted_entry("v4", Some(50)),      // 纯投票 → 跳过
        ],
        false, VndbSyncMode::Download, true)
    .unwrap();

    assert_eq!(plan.total, 4);
    assert_eq!(plan.conflicts.len(), 1);
    assert_eq!(plan.creates.len(), 1);
    assert_eq!(plan.skipped, 2);
    assert_eq!(before_games, games_count(&db));
    assert_eq!(before_states, states_snapshot(&db));
}

#[test]
fn apply_只建勾选的壳() {
    let _guard = plan_cache_guard();
    let db = Db::new();

    let plan = plan_ulist(
        &db,
        &[entry("v9", "Playing", Some(85)), entry("v10", "Finished", None)],
        false, VndbSyncMode::Download, true)
    .unwrap();
    assert_eq!(plan.creates.len(), 2);

    let report = apply(
        &db,
        &PlanDecision {
            creates: vec![plan.creates[0].clone()],
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(report.created, 1);
    assert_eq!(report.created_ids.len(), 1, "创建成功后要回传新条目 id，补全任务靠它接续");
    assert_eq!(games_count(&db), 1);
    let (status, rating, synced_status, synced_rating) = state_of(
        &db,
        &db.query_row(
            "SELECT id FROM games WHERE vndb_id = 'v9'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
    );
    assert_eq!(status, Status::Playing);
    assert_eq!(rating, Some(85.0));
    assert_eq!(synced_status, Some(Status::Playing));
    assert_eq!(synced_rating, Some(85.0));
}

#[test]
fn apply_空决策零副作用() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Finished), None);

    let before = states_snapshot(&db);
    apply(&db, &PlanDecision::default()).unwrap();

    assert_eq!(before, states_snapshot(&db));
}

#[test]
fn apply_采纳更新会写值并回写基快照() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_synced(&db, &id, Some(Status::Wishlist), None);

    let plan = plan_ulist(&db, &[entry("v1", "Playing", Some(85))], false, VndbSyncMode::Download, true).unwrap();
    assert_eq!(plan.updates.len(), 1);

    apply(
        &db,
        &PlanDecision {
            updates: plan.updates.clone(),
            ..Default::default()
        },
    )
    .unwrap();

    let (status, rating, synced_status, synced_rating) = state_of(&db, &id);
    assert_eq!(status, Status::Playing);
    assert_eq!(rating, Some(85.0));
    assert_eq!(synced_status, Some(Status::Playing));
    assert_eq!(synced_rating, Some(85.0));
    assert!(modified_of(&db, &id));
}

#[test]
fn apply_可清空评分() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, None, Some(Some(70.0)));
    set_synced(&db, &id, Some(Status::Wishlist), Some(85.0));

    apply(
        &db,
        &PlanDecision {
            updates: vec![PlanUpdate {
                game_id: id.clone(),
                name: "甲".into(),
                vndb_id: "v1".into(),
                status: None,
                rating: Some(None),
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let (_, rating, _, synced_rating) = state_of(&db, &id);
    assert_eq!(rating, None);
    assert_eq!(synced_rating, None);
}

#[test]
fn apply_刷基快照不动本地值() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Playing), None);
    set_synced(&db, &id, Some(Status::Wishlist), None);

    let plan = plan_ulist(&db, &[entry("v1", "Playing", None)], false, VndbSyncMode::Download, true).unwrap();
    assert_eq!(plan.aligns.len(), 1);

    apply(
        &db,
        &PlanDecision {
            aligns: plan.aligns.clone(),
            ..Default::default()
        },
    )
    .unwrap();

    let (status, _, synced_status, _) = state_of(&db, &id);
    assert_eq!(status, Status::Playing);
    assert_eq!(synced_status, Some(Status::Playing));
}

#[test]
fn apply_解决冲突选本地保留本地且基快照对齐_vndb_已知值() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Finished), None);

    apply(
        &db,
        &PlanDecision {
            statuses: vec![StatusDecision {
                game_id: id.clone(),
                status: Status::Finished,
                synced: Status::Dropped,
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let (status, _, synced_status, _) = state_of(&db, &id);
    assert_eq!(status, Status::Finished);
    assert_eq!(synced_status, Some(Status::Dropped));
}

#[test]
fn apply_解决冲突选远端写入并回写基快照() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");

    apply(
        &db,
        &PlanDecision {
            statuses: vec![StatusDecision {
                game_id: id.clone(),
                status: Status::Dropped,
                synced: Status::Dropped,
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let (status, _, synced_status, _) = state_of(&db, &id);
    assert_eq!(status, Status::Dropped);
    assert_eq!(synced_status, Some(Status::Dropped));
}

#[test]
fn apply_解决评分可选空值() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");

    apply(
        &db,
        &PlanDecision {
            ratings: vec![RatingDecision {
                game_id: id.clone(),
                rating: None,
                synced: None,
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let (_, rating, _, synced_rating) = state_of(&db, &id);
    assert_eq!(rating, None);
    assert_eq!(synced_rating, None);
}

#[test]
fn apply_解决后的冲突不再重复出现() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Finished), None);
    set_synced(&db, &id, Some(Status::Playing), None);

    let first = plan_ulist(&db, &[entry("v1", "Dropped", None)], false, VndbSyncMode::Download, true).unwrap();
    assert_eq!(first.conflicts.len(), 1);

    apply(
        &db,
        &PlanDecision {
            statuses: vec![StatusDecision {
                game_id: id.clone(),
                status: Status::Finished,
                synced: Status::Dropped,
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let second = plan_ulist(&db, &[entry("v1", "Dropped", None)], false, VndbSyncMode::Download, true).unwrap();
    assert!(second.conflicts.is_empty());
    assert!(second.updates.is_empty());
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Finished);
}

#[test]
fn 拒绝后_vndb_真改了才再判冲突() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");
    set_state(&db, &id, Some(Status::Finished), None);
    set_synced(&db, &id, Some(Status::Playing), None);

    let first = plan_ulist(&db, &[entry("v1", "Dropped", None)], false, VndbSyncMode::Download, true).unwrap();
    assert_eq!(first.conflicts.len(), 1);

    apply(
        &db,
        &PlanDecision {
            statuses: vec![StatusDecision {
                game_id: id.clone(),
                status: Status::Finished,
                synced: Status::Dropped,
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let second = plan_ulist(&db, &[entry("v1", "Playing", None)], false, VndbSyncMode::Download, true).unwrap();
    assert_eq!(second.conflicts.len(), 1);
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Finished);
}

#[test]
fn apply_跳过库里已存在的_vndb_id() {
    let _guard = plan_cache_guard();
    let db = Db::new();
    let id = new_game(&db, "甲", "v1");

    apply(
        &db,
        &PlanDecision {
            creates: vec![PlanCreate {
                vndb_id: "v1".into(),
                name: "重复壳".into(),
                status: Status::Playing,
                rating: None,
            }],
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(games_count(&db), 1);
    let (status, ..) = state_of(&db, &id);
    assert_eq!(status, Status::Wishlist);
}

// ---------- 决策对象的 JSON 契约 ----------

#[test]
fn decision_缺省数组按空处理() {
    let decision: PlanDecision = serde_json::from_str("{}").unwrap();
    assert!(decision.creates.is_empty());
    assert!(decision.updates.is_empty());
    assert!(decision.aligns.is_empty());
    assert!(decision.statuses.is_empty());
    assert!(decision.ratings.is_empty());
}

#[test]
fn decision_评分双层_option_区分不动与清空() {
    let keep = PlanUpdate {
        game_id: "g1".into(),
        name: "甲".into(),
        vndb_id: "v1".into(),
        status: None,
        rating: None,
    };
    let clear = PlanUpdate {
        rating: Some(None),
        ..keep.clone()
    };
    let set = PlanUpdate {
        rating: Some(Some(88.0)),
        ..keep.clone()
    };

    let keep_json = serde_json::to_value(&keep).unwrap();
    assert!(keep_json.get("rating").is_none(), "不动评分时字段应缺席");

    let back_keep: PlanUpdate = serde_json::from_value(keep_json).unwrap();
    assert_eq!(back_keep.rating, None);

    let back_clear: PlanUpdate = serde_json::from_value(serde_json::to_value(&clear).unwrap()).unwrap();
    assert_eq!(back_clear.rating, Some(None));

    let back_set: PlanUpdate = serde_json::from_value(serde_json::to_value(&set).unwrap()).unwrap();
    assert_eq!(back_set.rating, Some(Some(88.0)));
}

#[test]
fn decision_里_align_可只带状态() {
    let align = PlanAlign {
        game_id: "g1".into(),
        status: Some(Status::Playing),
        rating: None,
    };
    let json = serde_json::to_value(&align).unwrap();
    assert!(json.get("rating").is_none());
    let back: PlanAlign = serde_json::from_value(json).unwrap();
    assert_eq!(back.status, Some(Status::Playing));
}

#[test]
fn 计划缓存_ttl_内命中_过期即失效() {
    let _guard = plan_cache_guard();
    invalidate_plan();
    assert!(cached_plan_at(0).is_none(), "初始无缓存");

    let at = store_plan(&SyncPlan::default());
    assert!(cached_plan_at(at).is_some(), "刚落库的计划应命中");
    assert!(cached_plan_at(at - 1).is_none(), "时钟回拨时旧计划作废");
    assert!(cached_plan_at(at + 59).is_some(), "TTL 边缘内仍命中");
    assert!(cached_plan_at(at + 60).is_none(), "超过 TTL 必须重新拉取");

    invalidate_plan();
    assert!(cached_plan_at(at + 61).is_none(), "手动失效后不再命中");
}

#[test]
fn 计划缓存_存的是计划的克隆_互不影响() {
    let _guard = plan_cache_guard();
    invalidate_plan();
    let at = store_plan(&SyncPlan::default());

    let mut edited = cached_plan_at(at).expect("应命中");
    edited.total = 999;
    edited.skipped = 999;

    let again = cached_plan_at(at).expect("应命中");
    assert_eq!(again.total, 0);
    assert_eq!(again.skipped, 0);
    invalidate_plan();
}

#[test]
fn 应用计划后缓存失效() {
    let _guard = plan_cache_guard();
    invalidate_plan();
    let at = store_plan(&SyncPlan::default());
    assert!(cached_plan_at(at).is_some());

    let db = Db::new();
    let id = new_game(&db, "壳", "v1");
    let decision = PlanDecision {
        creates: vec![],
        updates: vec![],
        aligns: vec![],
        statuses: vec![StatusDecision {
            game_id: id,
            status: Status::Finished,
            synced: Status::Finished,
        }],
        ratings: vec![],
        pushes: vec![],
    };
    apply(&db, &decision).unwrap();

    assert!(
        cached_plan_at(at).is_none(),
        "apply 落库后旧计划必须失效，否则用户看到的计划与库不一致"
    );
}

#[test]
fn 本地写操作后计划缓存失效() {
    type Action = (&'static str, fn(&Db, &str));

    let _guard = plan_cache_guard();
    let actions: Vec<Action> = vec![
        (
            "改状态后旧计划应失效，否则弹窗拿着旧本地值算判定",
            |db, id| set_state(db, id, Some(Status::Playing), None),
        ),
        (
            "删作品后旧计划应失效，否则导入名单会认错行",
            |db, id| {
                delete_game(db, id).unwrap();
            },
        ),
        (
            "重绑 vndb 后旧计划应失效，否则条目会归错桶",
            |db, id| {
                update_game(
                    db,
                    id,
                    &GamePatch {
                        name: None,
                        vndb_id: Some(Some("v2".into())),
                    },
                )
                .unwrap();
            },
        ),
    ];

    for (label, act) in actions {
        invalidate_plan();
        let db = Db::new();
        let id = new_game(&db, "壳", "v1");
        let at = store_plan(&SyncPlan::default());
        act(&db, &id);
        assert!(cached_plan_at(at).is_none(), "{label}");
    }
}

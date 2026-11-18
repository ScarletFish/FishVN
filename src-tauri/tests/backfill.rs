mod common;

use common::{new_game_in as new_game, Db};
use fishvn::backfill::{
    items_by_ids, patch_for, reset, scan, status_or_plan, BindSource, ItemState,
};
use fishvn::metadata;
use fishvn::types::MetadataPatch;
use fishvn::vndb::VnSync;

fn full_meta_patch() -> MetadataPatch {
    MetadataPatch {
        description: Some(Some("简介齐全".into())),
        image_path: Some(Some("/covers/x.jpg".into())),
        ..Default::default()
    }
}

#[test]
fn 库内列表按名称排序且任务顺序锚定同一规则() {
    let db = Db::new();
    for name in ["Zulu", "alpha", "中文", "Beta", "Aurora"] {
        new_game(&db, name);
    }
    let games = fishvn::games::list_games(&db).unwrap();
    let names: Vec<String> = games.iter().map(|g| g.name.clone()).collect();
    let mut sorted = names.clone();
    sorted.sort_by_key(|n| n.to_lowercase());
    assert_eq!(names, sorted, "库内列表应按名称大小写不敏感排序");

    let listed: Vec<String> = games.into_iter().map(|g| g.id).collect();
    let scanned: Vec<String> = scan(&db).unwrap().into_iter().map(|i| i.game_id).collect();
    assert_eq!(scanned, listed, "批量扫描顺序应与列表一致");
    let by_ids: Vec<String> = items_by_ids(&db, &listed)
        .unwrap()
        .into_iter()
        .map(|i| i.game_id)
        .collect();
    assert_eq!(by_ids, listed, "按 id 启动的顺序应与列表一致");
}

#[test]
fn 扫描只收还没有元数据行的作品() {
    let db = Db::new();
    let bare = new_game(&db, "啥都没有");
    let partial = new_game(&db, "只有封面");
    let complete = new_game(&db, "齐全");

    metadata::upsert(
        &db,
        &partial,
        &MetadataPatch {
            image_path: Some(Some("/covers/a.jpg".into())),
            ..Default::default()
        },
        None,
    )
    .unwrap();
    metadata::upsert(&db, &complete, &full_meta_patch(), None).unwrap();

    let picked: Vec<String> = scan(&db).unwrap().into_iter().map(|i| i.game_id).collect();
    assert!(picked.contains(&bare), "没有元数据行的作品要收");
    assert!(!picked.contains(&partial), "有元数据行就不收，哪怕只填了一项");
    assert!(!picked.contains(&complete), "元数据齐全的不收");
}

#[test]
fn 空元数据行也让作品离开待处理() {
    let db = Db::new();
    let id = new_game(&db, "远端啥都没有");
    assert!(scan(&db).unwrap().iter().any(|i| i.game_id == id));

    metadata::ensure_row(&db, &id).unwrap();

    assert!(
        !scan(&db).unwrap().iter().any(|i| i.game_id == id),
        "建了行就不再进待处理"
    );
}

#[test]
fn 扫描带上未绑定作品由匹配阶段补_id() {
    let db = Db::new();
    let id = new_game(&db, "没绑 VNDB");

    let items = scan(&db).unwrap();
    let item = items
        .iter()
        .find(|i| i.game_id == id)
        .expect("未绑定也要收");
    assert_eq!(item.name, "没绑 VNDB");
    assert_eq!(item.vndb_id, None);
}

#[test]
fn 没跑过补全时状态里先摆出计划清单() {
    let db = Db::new();
    let bare = new_game(&db, "啥都没有");
    metadata::upsert(&db, &bare, &full_meta_patch(), None).unwrap();
    fishvn::characters::replace_all(
        &db,
        &bare,
        &[fishvn::types::GameCharacter {
            id: "c1".into(),
            name: Some("角色".into()),
            sort_order: 0,
            ..Default::default()
        }],
    )
    .unwrap();
    let pending = new_game(&db, "缺东西");

    let plan = status_or_plan(&db).unwrap();
    assert!(!plan.running);
    assert_eq!(plan.total, 1);
    let ids: Vec<&str> = plan.items.iter().map(|i| i.game_id.as_str()).collect();
    assert_eq!(ids, vec![pending.as_str()]);
    assert_eq!(plan.items[0].state, ItemState::Pending);
    assert_eq!(plan.items[0].bind_source, Some(BindSource::Search));

    let run_items = items_by_ids(&db, std::slice::from_ref(&bare)).unwrap();
    reset(&run_items);
    let live = status_or_plan(&db).unwrap();
    assert_eq!(live.items.len(), 1);
    assert_eq!(live.items[0].game_id, bare);
    assert_eq!(live.items[0].bind_source, Some(BindSource::Search));
    assert_eq!(live.pending_ids, vec![pending.clone()]);
}

#[test]
fn 按_id_取条目过滤且空列表不求值() {
    let db = Db::new();
    let a = new_game(&db, "甲");
    let b = new_game(&db, "乙");

    let items = items_by_ids(&db, &[a.clone(), b.clone()]).unwrap();
    assert_eq!(items.len(), 2);
    let ids: Vec<&str> = items.iter().map(|i| i.game_id.as_str()).collect();
    assert!(ids.contains(&a.as_str()) && ids.contains(&b.as_str()));

    let one = items_by_ids(&db, std::slice::from_ref(&b)).unwrap();
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].game_id, b);

    assert!(items_by_ids(&db, &[]).unwrap().is_empty());
}

#[test]
fn 按空白判定的补丁落库后旧值不动新值填上() {
    let db = Db::new();
    let id = new_game(&db, "半填");
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            alias: Some(Some("用户别名".into())),
            description: Some(Some("用户简介".into())),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    let sync = VnSync {
        vn: fishvn::vndb::VnMetadata {
            id: "v9".into(),
            title: "远端标题".into(),
            released: Some("2020-05-01".into()),
            rating: Some(80.5),
            ..Default::default()
        },
        characters: vec![],
    };
    let patch =
        patch_for(metadata::get(&db, &id).unwrap().as_ref(), &sync, false).expect("还有空白字段");
    metadata::upsert(&db, &id, &patch, None).unwrap();

    let after = metadata::get(&db, &id).unwrap().unwrap();
    assert_eq!(
        after.alias.as_deref(),
        Some("用户别名"),
        "用户填过的别名不能被覆盖"
    );
    assert_eq!(
        after.description.as_deref(),
        Some("用户简介"),
        "用户填过的简介不能被覆盖"
    );
    assert_eq!(
        after.released.as_deref(),
        Some("2020-05-01"),
        "空白的发行日要填上"
    );
    assert_eq!(after.rating, Some(80.5), "空白的评分要填上");
}

fn event(phase: &str) -> fishvn::types::TaskEvent {
    fishvn::types::TaskEvent {
        phase: phase.to_owned(),
        ts: 1,
        ..Default::default()
    }
}

#[test]
fn 一次运行落一行并可读回事件() {
    let db = Db::new();
    let report = fishvn::backfill::BackfillReport {
        total: 3,
        fetched: 2,
        ..Default::default()
    };
    fishvn::backfill::save_run(&db, 100, 200, &report, &[event("resolve"), event("backfill")])
        .unwrap();

    let list = fishvn::backfill::runs(&db, 20).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].started_at, 100);
    assert_eq!(list[0].ended_at, 200);
    assert_eq!(list[0].summary.fetched, 2);

    let detail = fishvn::backfill::run_detail(&db, list[0].id)
        .unwrap()
        .expect("应取到该次运行");
    assert_eq!(detail.events.len(), 2);
    assert_eq!(detail.events[0].phase, "resolve");
}

#[test]
fn 图片中间进度只留最后一条() {
    let db = Db::new();
    let mut events = vec![event("images"), event("images"), event("images")];
    events[1].done = Some(1);
    events[2].done = Some(2);
    fishvn::backfill::save_run(
        &db,
        0,
        1,
        &fishvn::backfill::BackfillReport::default(),
        &events,
    )
    .unwrap();

    let id = fishvn::backfill::runs(&db, 1).unwrap()[0].id;
    let detail = fishvn::backfill::run_detail(&db, id).unwrap().unwrap();
    assert_eq!(detail.events.len(), 1);
    assert_eq!(detail.events[0].done, Some(2));
}

#[test]
fn 只保留最近二十次运行() {
    let db = Db::new();
    for step in 0..22 {
        fishvn::backfill::save_run(
            &db,
            step,
            step,
            &fishvn::backfill::BackfillReport::default(),
            &[],
        )
        .unwrap();
    }

    let list = fishvn::backfill::runs(&db, 20).unwrap();
    assert_eq!(list.len(), 20);
    assert_eq!(list[0].started_at, 21, "最新的在最前");
    assert_eq!(list[19].started_at, 2, "最旧的两条被清掉");
}

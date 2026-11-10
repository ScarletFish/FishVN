mod common;

use common::{new_game_in as new_game, Db};
use fishvn::backfill::{BindSource, ItemState, items_by_ids, patch_for, reset, scan, status_or_plan};
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
fn 扫描只收资料有空白的作品() {
    let db = Db::new();
    let bare = new_game(&db, "啥都没有");
    let no_desc = new_game(&db, "没简介");
    let no_cover = new_game(&db, "没封面");
    let no_characters = new_game(&db, "没角色");
    let complete = new_game(&db, "齐全");

    metadata::upsert(&db, &no_desc, &MetadataPatch {
        image_path: Some(Some("/covers/a.jpg".into())),
        ..Default::default()
    })
    .unwrap();
    metadata::upsert(&db, &no_cover, &MetadataPatch {
        description: Some(Some("有简介".into())),
        ..Default::default()
    })
    .unwrap();
    metadata::upsert(&db, &no_characters, &full_meta_patch()).unwrap();
    metadata::upsert(&db, &complete, &full_meta_patch()).unwrap();
    fishvn::characters::replace_all(
        &db,
        &complete,
        &[fishvn::types::GameCharacter {
            id: "c1".into(),
            name: Some("角色".into()),
            sort_order: 0,
            ..Default::default()
        }],
    )
    .unwrap();

    let picked: Vec<String> = scan(&db).unwrap().into_iter().map(|i| i.game_id).collect();
    assert!(picked.contains(&bare), "没有任何元数据的作品要收");
    assert!(picked.contains(&no_desc), "缺简介的要收");
    assert!(picked.contains(&no_cover), "缺封面的要收");
    assert!(picked.contains(&no_characters), "缺角色的要收");
    assert!(!picked.contains(&complete), "资料齐全的不该收");
}

#[test]
fn 扫描带上未绑定作品由匹配阶段补_id() {
    let db = Db::new();
    let id = new_game(&db, "没绑 VNDB");

    let items = scan(&db).unwrap();
    let item = items.iter().find(|i| i.game_id == id).expect("未绑定也要收");
    assert_eq!(item.name, "没绑 VNDB");
    assert_eq!(item.vndb_id, None);
}

#[test]
fn 没跑过补全时状态里先摆出计划清单() {
    let db = Db::new();
    let bare = new_game(&db, "啥都没有");
    metadata::upsert(&db, &bare, &full_meta_patch()).unwrap();
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
    metadata::upsert(&db, &id, &MetadataPatch {
        alias: Some(Some("用户别名".into())),
        description: Some(Some("用户简介".into())),
        ..Default::default()
    })
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
    let patch = patch_for(metadata::get(&db, &id).unwrap().as_ref(), &sync, false).expect("还有空白字段");
    metadata::upsert(&db, &id, &patch).unwrap();

    let after = metadata::get(&db, &id).unwrap().unwrap();
    assert_eq!(after.alias.as_deref(), Some("用户别名"), "用户填过的别名不能被覆盖");
    assert_eq!(after.description.as_deref(), Some("用户简介"), "用户填过的简介不能被覆盖");
    assert_eq!(after.released.as_deref(), Some("2020-05-01"), "空白的发行日要填上");
    assert_eq!(after.rating, Some(80.5), "空白的评分要填上");
}

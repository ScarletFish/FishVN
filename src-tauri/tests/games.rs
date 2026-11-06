#![allow(non_ascii_idents)]

mod common;

use common::{message, new_game_in as new_game, variant, Db, Tree};
use fishvn::games::{
    create_game, delete_game, get_game, list_games, update_game, update_game_state,
};
use fishvn::installations::create_installation;
use fishvn::metadata;
use fishvn::types::{GameListItem, GamePatch, MetadataPatch, NewGame, StatePatch, Status};

fn find(db: &Db, id: &str) -> Option<GameListItem> {
    list_games(db)
        .expect("列作品失败")
        .into_iter()
        .find(|g| g.id == id)
}

#[test]
fn 只给名字就能建条目其余字段留空() {
    let db = Db::new();
    let game = create_game(
        &db,
        &NewGame {
            name: "  星空  ".into(),
            vndb_id: None,
            metadata: None,
        },
    )
    .expect("建作品失败");

    assert_eq!(game.name, "星空", "两端空白要修掉");
    assert_eq!(game.vndb_id, None);
    assert!(!game.id.is_empty(), "id 应是 uuid");
    assert!(game.created_at > 0, "created_at 应该是 unix 秒");
}

#[test]
fn name为空返回_bad_request() {
    let db = Db::new();
    let err = create_game(
        &db,
        &NewGame {
            name: "   ".into(),
            vndb_id: None,
            metadata: None,
        },
    )
    .unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
}

#[test]
fn vndb_id格式非法返回_bad_request() {
    let db = Db::new();
    for bad in ["14", "v", "v14x", "vv1", "V14", ""] {
        let err = create_game(
            &db,
            &NewGame {
                name: "X".into(),
                vndb_id: Some(bad.into()),
                metadata: None,
            },
        )
        .unwrap_err();
        assert_eq!(variant(&err), "BadRequest", "vndb_id={bad:?} 应该被拒");
        assert_eq!(message(&err), "vndb_id 必须是 v+数字");
    }
}

#[test]
fn name与vndb_id同时非法时报name那个() {
    // 校验顺序：先 name 后 vndb_id。顺序反了错误信息就变了
    let db = Db::new();
    let err = create_game(
        &db,
        &NewGame {
            name: "   ".into(),
            vndb_id: Some("bad".into()),
            metadata: None,
        },
    )
    .unwrap_err();
    assert_eq!(message(&err), "name 不能为空");
}

#[test]
fn 建条目时可以带上vndb_id() {
    let db = Db::new();
    let game = create_game(
        &db,
        &NewGame {
            name: "命中".into(),
            vndb_id: Some("v20001".into()),
            metadata: None,
        },
    )
    .expect("建作品失败");
    assert_eq!(game.vndb_id.as_deref(), Some("v20001"));
}

#[test]
fn 建条目撞上已占用的vndb_id返回_conflict_事务不留下残行() {
    let db = Db::new();
    create_game(
        &db,
        &NewGame {
            name: "先到".into(),
            vndb_id: Some("v20010".into()),
            metadata: None,
        },
    )
    .expect("建作品失败");

    let err = create_game(
        &db,
        &NewGame {
            name: "后到".into(),
            vndb_id: Some("v20010".into()),
            metadata: None,
        },
    )
    .unwrap_err();
    assert_eq!(variant(&err), "Conflict");

    let games = list_games(&db).expect("列作品失败");
    assert!(
        !games.iter().any(|g| g.name == "后到"),
        "失败的作品不应留下 games 行"
    );
}

#[test]
fn get列出全部条目带状态与安装概况() {
    let db = Db::new();
    for n in ["一", "二", "三", "四"] {
        new_game(&db, n);
    }

    let games = list_games(&db).expect("列作品失败");
    assert_eq!(games.len(), 4);
    assert!(!games[0].can_launch, "没有安装时不该可启动");
    assert_eq!(games[0].installation_count, 0);
}

#[test]
fn 列表里的状态跟随状态表变化() {
    let db = Db::new();
    let id = new_game(&db, "列表状态");
    assert_eq!(find(&db, &id).unwrap().status, Status::Wishlist);

    update_game_state(
        &db,
        &id,
        &StatePatch {
            status: Some(Status::Playing),
            ..Default::default()
        },
    )
    .expect("改状态失败");

    assert_eq!(find(&db, &id).unwrap().status, Status::Playing);
}

#[test]
fn 列表带出封面路径_没有元数据行时为_none() {
    let db = Db::new();
    let bare = new_game(&db, "无封面");
    let covered = new_game(&db, "有封面");

    assert_eq!(
        find(&db, &bare).unwrap().cover_path,
        None,
        "没有元数据行时不能报错，也不能拿空串当路径"
    );

    metadata::upsert(
        &db,
        &covered,
        &MetadataPatch {
            image_path: Some(Some("D:\\covers\\vndb\\cv75598.jpg".into())),
            ..Default::default()
        },
    )
    .expect("写元数据失败");

    assert_eq!(
        find(&db, &covered).unwrap().cover_path.as_deref(),
        Some("D:\\covers\\vndb\\cv75598.jpg")
    );
}

#[test]
fn 联表封面不放大安装计数() {
    let db = Db::new();
    let mut tree = Tree::new();
    let id = new_game(&db, "计数");

    let dir = tree.make_dir(&[("a.exe", "a"), ("b.exe", "b")]);
    create_installation(
        &db,
        &fishvn::types::NewInstallation {
            game_id: id.clone(),
            dir_path: common::path_str(&dir),
        },
    )
    .expect("建安装失败");
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            image_path: Some(Some("D:\\covers\\cv1.jpg".into())),
            ..Default::default()
        },
    )
    .expect("写元数据失败");

    assert_eq!(find(&db, &id).unwrap().installation_count, 1);
}

#[test]
fn 列表统计本地安装数与可启动判定() {
    let db = Db::new();
    let mut tree = Tree::new();
    let id = new_game(&db, "安装统计");

    let before = find(&db, &id).unwrap();
    assert_eq!(before.installation_count, 0);
    assert!(!before.can_launch);

    let dir_a = tree.make_dir(&[("a.exe", "a")]);
    let inst = create_installation(
        &db,
        &fishvn::types::NewInstallation {
            game_id: id.clone(),
            dir_path: common::path_str(&dir_a),
        },
    )
    .expect("建安装失败");
    assert_eq!(
        inst.entry_path.as_deref(),
        Some("a.exe"),
        "候选唯一应自动填入口"
    );

    let dir_b = tree.make_dir(&[("b.exe", "b")]);
    create_installation(
        &db,
        &fishvn::types::NewInstallation {
            game_id: id.clone(),
            dir_path: common::path_str(&dir_b),
        },
    )
    .expect("建第二个安装失败");

    let after = find(&db, &id).unwrap();
    assert_eq!(after.installation_count, 2);
    assert!(after.can_launch);
}

#[test]
fn patch改name与绑定vndb_id() {
    let db = Db::new();
    let id = new_game(&db, "原名");
    update_game(
        &db,
        &id,
        &GamePatch {
            name: Some("新名".into()),
            vndb_id: Some(Some("v20002".into())),
        },
    )
    .expect("改作品失败");

    let game = get_game(&db, &id).expect("读作品失败");
    assert_eq!(game.name, "新名");
    assert_eq!(game.vndb_id.as_deref(), Some("v20002"));
}

#[test]
fn patch绑到已被占用的vndb_id返回_conflict() {
    let db = Db::new();
    let holder = create_game(
        &db,
        &NewGame {
            name: "占用者".into(),
            vndb_id: Some("v20003".into()),
            metadata: None,
        },
    )
    .unwrap();
    let latecomer = create_game(
        &db,
        &NewGame {
            name: "后来者".into(),
            vndb_id: None,
            metadata: None,
        },
    )
    .unwrap();

    let err = update_game(
        &db,
        &latecomer.id,
        &GamePatch {
            name: None,
            vndb_id: Some(Some("v20003".into())),
        },
    )
    .unwrap_err();

    assert_eq!(variant(&err), "Conflict");
    assert!(message(&err).contains("占用"), "错误信息要说明是占用");
    assert_eq!(
        get_game(&db, &holder.id).unwrap().vndb_id.as_deref(),
        Some("v20003"),
        "占用者不该被顶掉"
    );
}

#[test]
fn patch_vndb_id置null后另一条可以绑同一个号() {
    let db = Db::new();
    let a = create_game(
        &db,
        &NewGame {
            name: "原持有".into(),
            vndb_id: Some("v20004".into()),
            metadata: None,
        },
    )
    .unwrap();
    let b = create_game(
        &db,
        &NewGame {
            name: "接盘".into(),
            vndb_id: None,
            metadata: None,
        },
    )
    .unwrap();

    update_game(
        &db,
        &a.id,
        &GamePatch {
            name: None,
            vndb_id: Some(None),
        },
    )
    .expect("置空失败");
    assert_eq!(get_game(&db, &a.id).unwrap().vndb_id, None);

    update_game(
        &db,
        &b.id,
        &GamePatch {
            name: None,
            vndb_id: Some(Some("v20004".into())),
        },
    )
    .expect("接盘失败");
    assert_eq!(
        get_game(&db, &b.id).unwrap().vndb_id.as_deref(),
        Some("v20004")
    );
}

#[test]
fn patch不存在的id返回_not_found() {
    let db = Db::new();
    let err = update_game(
        &db,
        common::ghost(),
        &GamePatch {
            name: Some("x".into()),
            vndb_id: None,
        },
    )
    .unwrap_err();
    assert_eq!(variant(&err), "NotFound");
}

#[test]
fn delete删掉条目列表里不再出现() {
    let db = Db::new();
    let id = new_game(&db, "待删");
    assert!(find(&db, &id).is_some());

    delete_game(&db, &id).expect("删除失败");
    assert!(find(&db, &id).is_none(), "列表里不该再有这条");
}

#[test]
fn delete同一id第二次返回_not_found() {
    let db = Db::new();
    let id = new_game(&db, "删一次");
    delete_game(&db, &id).expect("首次删除失败");

    let err = delete_game(&db, &id).unwrap_err();
    assert_eq!(variant(&err), "NotFound");
    assert!(message(&err).contains("不存在"));
}

#[test]
fn delete不存在的id返回_not_found() {
    let db = Db::new();
    assert_eq!(
        variant(&delete_game(&db, common::ghost()).unwrap_err()),
        "NotFound"
    );
}

#[test]
fn patch空body返回_bad_request() {
    let db = Db::new();
    let id = new_game(&db, "空改");
    let err = update_game(&db, &id, &GamePatch::default()).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert_eq!(message(&err), "没有可更新的字段");
}

#[test]
fn patch只给vndb_id置null不动name() {
    let db = Db::new();
    let id = new_game(&db, "只置空");
    update_game(
        &db,
        &id,
        &GamePatch {
            name: None,
            vndb_id: Some(Some("v30001".into())),
        },
    )
    .expect("绑定失败");

    update_game(
        &db,
        &id,
        &GamePatch {
            name: None,
            vndb_id: Some(None),
        },
    )
    .expect("置空失败");

    let game = get_game(&db, &id).unwrap();
    assert_eq!(game.vndb_id, None);
    assert_eq!(game.name, "只置空", "没给的字段不该被动到");
}

mod common;

use common::{variant, Db};
use fishvn::games::{create_game, delete_game};
use fishvn::metadata;
use fishvn::types::{
    LocalizedTitle, MetadataPatch, MetadataTag, NewGame, StatePatch, UserGameState,
};

fn new_game(db: &Db, name: &str) -> String {
    create_game(
        db,
        &NewGame {
            name: name.into(),
            vndb_id: None,
            metadata: None,
        },
    )
    .expect("建作品失败")
    .id
}

fn alias_patch(alias: &str) -> MetadataPatch {
    MetadataPatch {
        alias: Some(Some(alias.to_owned())),
        ..Default::default()
    }
}

#[test]
fn 删作品连带删掉元数据() {
    let db = Db::new();
    let id = new_game(&db, "待删");
    metadata::upsert(&db, &id, &alias_patch("别名")).expect("写元数据失败");

    delete_game(&db, &id).expect("删除失败");

    let left: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM game_metadata WHERE game_id = ?",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .expect("数元数据失败");
    assert_eq!(left, 0, "元数据不该留在库里");
}

#[test]
fn 一部作品只有一行重复写是更新() {
    let db = Db::new();
    let id = new_game(&db, "重复写");

    metadata::upsert(&db, &id, &alias_patch("第一次")).unwrap();
    metadata::upsert(&db, &id, &alias_patch("第二次")).unwrap();

    assert_eq!(
        metadata::get(&db, &id).unwrap().unwrap().alias.as_deref(),
        Some("第二次")
    );
    let rows: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM game_metadata WHERE game_id = ?",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(rows, 1);
}

#[test]
fn 不给元数据就没有元数据行() {
    let db = Db::new();
    let id = new_game(&db, "只填了名字");

    assert_eq!(metadata::get(&db, &id).unwrap(), None);
}

#[test]
fn 纯本地作品只填几个字段也存得下() {
    let db = Db::new();
    let id = new_game(&db, "本地作品");

    let stored = metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            description: Some(Some("用户自己写的简介".into())),
            developers: Some(Some(vec!["同人社团".into()])),
            ..Default::default()
        },
    )
    .expect("存本地资料失败");

    assert_eq!(stored.alias, None, "没填的别名该是空");
    assert_eq!(
        stored.votecount, None,
        "没同步过的评分人数该是空，不能拿 0 充数"
    );
    assert_eq!(stored.description.as_deref(), Some("用户自己写的简介"));
    assert_eq!(stored.developers.unwrap(), vec!["同人社团".to_owned()]);
}

#[test]
fn 外层不给就不动内层置空才清掉() {
    let db = Db::new();
    let id = new_game(&db, "取舍");

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            alias: Some(Some("别名".into())),
            released: Some(Some("2004-01-30".into())),
            ..Default::default()
        },
    )
    .unwrap();

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            alias: Some(None),
            ..Default::default()
        },
    )
    .expect("只给别名就该成功");
    let after = metadata::get(&db, &id).unwrap().unwrap();
    assert_eq!(after.alias, None, "显式置空要真的清掉");
    assert_eq!(
        after.released.as_deref(),
        Some("2004-01-30"),
        "没给的字段不该被动到"
    );
}

#[test]
fn 空补丁没有可更新的字段() {
    let db = Db::new();
    let id = new_game(&db, "空补丁");

    let err = metadata::upsert(&db, &id, &MetadataPatch::default()).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
}

#[test]
fn 作品不存在时两个入口都报_not_found() {
    let db = Db::new();
    let ghost = common::ghost();

    assert_eq!(variant(&metadata::get(&db, ghost).unwrap_err()), "NotFound");
    assert_eq!(
        variant(&metadata::upsert(&db, ghost, &alias_patch("别名")).unwrap_err()),
        "NotFound"
    );
}

#[test]
fn 标签带排序键往返不断() {
    let db = Db::new();
    let id = new_game(&db, "带标签");

    let tags = vec![
        MetadataTag {
            id: Some("p1".into()),
            name: "作品".into(),
            category: Some("cont".into()),
            rating: Some(92.5),
            vn_count: Some(19334),
            spoiler: Some(0),
            lie: Some(false),
        },
        // 用户自己加的标签没有 VNDB 编号，也没有排序键
        MetadataTag {
            name: "重玩向".into(),
            ..Default::default()
        },
    ];

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            tags: Some(Some(tags.clone())),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(
        metadata::get(&db, &id).unwrap().unwrap().tags.unwrap(),
        tags
    );
}

#[test]
fn 标签整列替换() {
    let db = Db::new();
    let id = new_game(&db, "删标签");

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            tags: Some(Some(vec![
                MetadataTag {
                    name: "甲".into(),
                    ..Default::default()
                },
                MetadataTag {
                    name: "乙".into(),
                    ..Default::default()
                },
            ])),
            ..Default::default()
        },
    )
    .unwrap();

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            tags: Some(Some(vec![MetadataTag {
                name: "乙".into(),
                ..Default::default()
            }])),
            ..Default::default()
        },
    )
    .unwrap();

    let tags = metadata::get(&db, &id).unwrap().unwrap().tags.unwrap();
    assert_eq!(tags.len(), 1, "删掉的标签不该因为 id 相同而复活");
    assert_eq!(tags[0].name, "乙");
}

#[test]
fn 本地化标题原样往返() {
    let db = Db::new();
    let id = new_game(&db, "多语言");

    let titles = vec![
        LocalizedTitle {
            lang: "zh-Hans".into(),
            title: "中文标题".into(),
            official: Some(true),
            main: Some(true),
            ..Default::default()
        },
        LocalizedTitle {
            lang: "ja".into(),
            title: "フェイト／ステイナイト".into(),
            ..Default::default()
        },
    ];

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            titles: Some(Some(titles.clone())),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(
        metadata::get(&db, &id).unwrap().unwrap().titles.unwrap(),
        titles
    );
}

#[test]
fn 建作品时一并写资料() {
    let db = Db::new();

    let game = create_game(
        &db,
        &NewGame {
            name: "带资料建的".into(),
            vndb_id: Some("v11".into()),
            metadata: Some(MetadataPatch {
                alias: Some(Some("别名".into())),
                rating: Some(Some(86.3)),
                ..Default::default()
            }),
        },
    )
    .expect("建作品失败");

    let stored = metadata::get(&db, &game.id)
        .unwrap()
        .expect("资料行该建出来");
    assert_eq!(stored.alias.as_deref(), Some("别名"));
    assert_eq!(stored.rating, Some(86.3));
}

#[test]
fn 资料写失败时作品与状态都不留行() {
    let db = Db::new();

    let err = create_game(
        &db,
        &NewGame {
            name: "资料不合法".into(),
            vndb_id: None,
            // 空补丁没有可写的列，upsert 在执行 SQL 前就报错
            metadata: Some(MetadataPatch::default()),
        },
    )
    .unwrap_err();
    assert_eq!(variant(&err), "BadRequest");

    for (table, label) in [("games", "作品"), ("user_game_state", "状态")] {
        let left: i64 = db
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap_or_else(|e| panic!("数{label}失败：{e}"));
        assert_eq!(left, 0, "{label}行不该留在库里");
    }
}

#[test]
fn 没资料也能改状态() {
    let db = Db::new();
    let id = new_game(&db, "只改状态");

    let state = fishvn::games::update_game_state(
        &db,
        &id,
        &StatePatch {
            status: Some(fishvn::types::Status::Playing),
            ..Default::default()
        },
    )
    .expect("改状态失败");

    assert!(matches!(state, UserGameState { .. }));
    assert_eq!(metadata::get(&db, &id).unwrap(), None);
}

mod common;

use common::{message, new_game_in as new_game, variant, Db};
use fishvn::games::{create_game, delete_game};
use fishvn::metadata;
use fishvn::types::{
    LocalizedTitle, MetadataPatch, MetadataStaff, MetadataTag, NewGame, StatePatch,
};

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
    metadata::upsert(&db, &id, &alias_patch("别名"), None).expect("写元数据失败");

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

    metadata::upsert(&db, &id, &alias_patch("第一次"), None).unwrap();
    metadata::upsert(&db, &id, &alias_patch("第二次"), None).unwrap();

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
        None,
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
        None,
    )
    .unwrap();

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            alias: Some(None),
            ..Default::default()
        },
        None,
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

    let err = metadata::upsert(&db, &id, &MetadataPatch::default(), None).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert_eq!(message(&err), "没有可更新的字段");
    assert!(
        metadata::get(&db, &id).unwrap().is_none(),
        "拒绝后资料行保持为空"
    );
}

#[test]
fn 作品不存在时两个入口都报_not_found() {
    let db = Db::new();
    let ghost = common::ghost();

    assert_eq!(variant(&metadata::get(&db, ghost).unwrap_err()), "NotFound");
    assert_eq!(
        variant(&metadata::upsert(&db, ghost, &alias_patch("别名"), None).unwrap_err()),
        "NotFound"
    );
}

#[test]
fn 标签往返保留关联属性() {
    let db = Db::new();
    let id = new_game(&db, "带标签");

    let tags = vec![
        MetadataTag {
            id: Some("g96".into()),
            name: "Romance".into(),
            rating: Some(92.5),
            spoiler: Some(1),
            ..Default::default()
        },
        MetadataTag {
            name: "重玩向".into(),
            ..Default::default()
        },
    ];

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            tags: Some(Some(tags)),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    let got = metadata::get(&db, &id).unwrap().unwrap().tags.unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].id.as_deref(), Some("g96"));
    assert_eq!(got[0].name, "Romance");
    assert_eq!(got[0].rating, Some(92.5));
    assert_eq!(got[0].spoiler, Some(1));
    assert_eq!(got[0].category.as_deref(), Some("cont"));
    assert!(got[0].vn_count.is_some());
    assert_eq!(got[1].name, "重玩向");
    assert!(got[1].id.as_deref().is_some_and(|v| v.starts_with('u')));
}

#[test]
fn 只给名字的标签分配本地id() {
    let db = Db::new();
    let id = new_game(&db, "作品");

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            tags: Some(Some(vec![MetadataTag {
                name: "只有名字".into(),
                ..Default::default()
            }])),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    let tag = metadata::get(&db, &id)
        .unwrap()
        .unwrap()
        .tags
        .unwrap()
        .remove(0);
    assert_eq!(tag.name, "只有名字");
    assert_eq!(tag.rating, None);
    assert_eq!(tag.id.as_deref(), Some("u1"));
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
        None,
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
        None,
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
        None,
    )
    .unwrap();

    assert_eq!(
        metadata::get(&db, &id).unwrap().unwrap().titles.unwrap(),
        titles
    );
}

#[test]
fn 制作人员整列替换且往返不断() {
    let db = Db::new();
    let id = new_game(&db, "带制作");

    let staff = vec![
        MetadataStaff {
            role: Some("scenario".into()),
            name: Some("Uchikoshi Koutarou".into()),
            original: Some("打越 鋼太郎".into()),
            note: Some("Main scenario".into()),
        },
        MetadataStaff {
            role: Some("music".into()),
            name: Some("Amao".into()),
            original: None,
            note: None,
        },
    ];

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            staff: Some(Some(staff.clone())),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            staff: Some(Some(vec![MetadataStaff {
                role: Some("art".into()),
                name: None,
                original: Some("甲田屋人".into()),
                note: None,
            }])),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    let stored = metadata::get(&db, &id).unwrap().unwrap().staff.unwrap();
    assert_eq!(stored.len(), 1, "staff 是整列替换，旧条目不残留");
    assert_eq!(stored[0].role, Some("art".into()));
    assert_eq!(stored[0].original.as_deref(), Some("甲田屋人"));
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
fn 评分人数带小数反序列化失败() {
    // 输入框给的是字符串，转数字可能带小数；i64 接不住，整个包都读不出来
    assert!(
        serde_json::from_str::<MetadataPatch>(r#"{"votecount":12.5}"#).is_err(),
        "浮点评分人数不该被当成 i64 收下"
    );
    assert!(serde_json::from_str::<MetadataPatch>(r#"{"votecount":12}"#).is_ok());
}

#[test]
fn 没资料也能改状态() {
    let db = Db::new();
    let id = new_game(&db, "只改状态");

    fishvn::games::update_game_state(
        &db,
        &id,
        &StatePatch {
            status: Some(fishvn::types::Status::Playing),
            ..Default::default()
        },
    )
    .expect("改状态失败");

    assert_eq!(metadata::get(&db, &id).unwrap(), None);
}

#[test]
fn 换封面时焦点重置为居中() {
    let db = Db::new();
    let id = new_game(&db, "换封面");
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            image_path: Some(Some("/covers/a.jpg".into())),
            cover_focal_x: Some(Some(80)),
            cover_focal_y: Some(Some(20)),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    let stored = metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            image_path: Some(Some("/covers/b.jpg".into())),
            cover_focal_x: Some(Some(10)),
            cover_focal_y: Some(Some(90)),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(
        (stored.cover_focal_x, stored.cover_focal_y),
        (Some(50), Some(50)),
        "换封面后旧焦点不该留在新图上，patch 里带的焦点值也不作数"
    );
}

#[test]
fn 重复选同一张氛围图时焦点保持() {
    let db = Db::new();
    let id = new_game(&db, "同图");
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(Some("/a.png".into())),
            ..Default::default()
        },
        None,
    )
    .unwrap();
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(Some("/a.png".into())),
            ambience_focal_x: Some(Some(80)),
            ambience_focal_y: Some(Some(10)),
            showcase_focal_x: Some(Some(70)),
            showcase_focal_y: Some(Some(20)),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    let stored = metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(Some("/a.png".into())),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(
        (
            stored.ambience_focal_x,
            stored.ambience_focal_y,
            stored.showcase_focal_x,
            stored.showcase_focal_y
        ),
        (80, 10, 70, 20),
        "图没换，之前调过的焦点该留着"
    );
}

#[test]
fn 换氛围图时两处焦点归位() {
    let db = Db::new();
    let id = new_game(&db, "换图");
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(Some("/a.png".into())),
            ..Default::default()
        },
        None,
    )
    .unwrap();
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(Some("/a.png".into())),
            ambience_focal_x: Some(Some(80)),
            ambience_focal_y: Some(Some(10)),
            showcase_focal_x: Some(Some(70)),
            showcase_focal_y: Some(Some(20)),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    let stored = metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(Some("/b.png".into())),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(
        (
            stored.ambience_focal_x,
            stored.ambience_focal_y,
            stored.showcase_focal_x,
            stored.showcase_focal_y
        ),
        (50, 22, 50, 22),
        "图换了，两处焦点该归位"
    );
}

#[test]
fn 清空氛围图也算换图() {
    let db = Db::new();
    let id = new_game(&db, "清空");
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(Some("/a.png".into())),
            ..Default::default()
        },
        None,
    )
    .unwrap();
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(Some("/a.png".into())),
            ambience_focal_x: Some(Some(80)),
            ambience_focal_y: Some(Some(10)),
            showcase_focal_x: Some(Some(70)),
            showcase_focal_y: Some(Some(20)),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    let stored = metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(None),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert_eq!((stored.ambience_focal_x, stored.showcase_focal_y), (50, 22));
}

#[test]
fn 不换封面只调焦点正常生效() {
    let db = Db::new();
    let id = new_game(&db, "调焦点");
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            image_path: Some(Some("/covers/a.jpg".into())),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    let stored = metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            cover_focal_x: Some(Some(80)),
            cover_focal_y: Some(Some(20)),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(stored.image_path.as_deref(), Some("/covers/a.jpg"));
    assert_eq!(
        (stored.cover_focal_x, stored.cover_focal_y),
        (Some(80), Some(20))
    );
}

#[test]
fn 清空自定义横幅写回空值() {
    let db = Db::new();
    let id = new_game(&db, "横幅");
    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(Some("D:/kv.png".into())),
            ..Default::default()
        },
        None,
    )
    .expect("写横幅失败");

    metadata::upsert(
        &db,
        &id,
        &MetadataPatch {
            ambience_path: Some(None),
            ..Default::default()
        },
        None,
    )
    .expect("清横幅失败");

    let stored = metadata::get(&db, &id)
        .expect("读元数据失败")
        .expect("元数据不该是空的");
    assert_eq!(stored.ambience_path, None, "清空后应回到跟随封面");
}

#[test]
fn 保存时把库外封面复制进游戏目录() {
    let db = Db::new();
    let dir = tempfile::tempdir().unwrap();
    let id = new_game(&db, "导入封面");
    let outside = dir.path().join("选中的封面.jpg");
    std::fs::write(&outside, b"fake jpeg bytes").unwrap();

    let patch = MetadataPatch {
        image_path: Some(Some(outside.to_string_lossy().into_owned())),
        ..Default::default()
    };
    metadata::upsert(&db, &id, &patch, Some(dir.path())).unwrap();

    let stored = metadata::get(&db, &id).unwrap().unwrap();
    let path = stored.image_path.unwrap();
    assert!(
        path.contains(&id) && path.contains("source"),
        "路径应改写为库内：{path}"
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"fake jpeg bytes");
}

#[test]
fn 图片不存在时保存失败且不落库() {
    let db = Db::new();
    let dir = tempfile::tempdir().unwrap();
    let id = new_game(&db, "坏图");
    let missing = dir.path().join("不存在.jpg");

    let patch = MetadataPatch {
        image_path: Some(Some(missing.to_string_lossy().into_owned())),
        alias: Some(Some("不该写进去".into())),
        ..Default::default()
    };
    let err = metadata::upsert(&db, &id, &patch, Some(dir.path())).unwrap_err();
    assert!(
        message(&err).contains("图片文件不存在"),
        "错误应说明原因：{}",
        message(&err)
    );

    let stored = metadata::get(&db, &id).unwrap();
    assert!(
        stored.and_then(|m| m.alias).is_none(),
        "保存失败时不应写入任何字段"
    );
}

#[test]
fn 导入多张图其中一张失败时不留已复制的文件() {
    let db = Db::new();
    let dir = tempfile::tempdir().unwrap();
    let id = new_game(&db, "半途失败");
    let good = dir.path().join("好的封面.jpg");
    std::fs::write(&good, b"good bytes").unwrap();
    let missing = dir.path().join("坏的横幅.jpg");

    let patch = MetadataPatch {
        image_path: Some(Some(good.to_string_lossy().into_owned())),
        ambience_path: Some(Some(missing.to_string_lossy().into_owned())),
        ..Default::default()
    };
    assert!(metadata::upsert(&db, &id, &patch, Some(dir.path())).is_err());

    let source = dir.path().join("covers").join(&id).join("source");
    let leftover: Vec<_> = std::fs::read_dir(&source)
        .map(|entries| entries.flatten().collect())
        .unwrap_or_default();
    assert!(
        leftover.is_empty(),
        "失败后不该留下已复制的图：{leftover:?}"
    );
}

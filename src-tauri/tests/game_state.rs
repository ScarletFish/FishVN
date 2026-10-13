#![allow(non_ascii_idents)]

mod common;

use common::{message, variant, Db};
use fishvn::games::{create_game, delete_game, get_game_state, update_game, update_game_state};
use fishvn::types::{NewGame, StatePatch, Status};

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

#[test]
fn 建条目时自动带一条wishlist状态() {
    let db = Db::new();
    let id = new_game(&db, "默认状态");

    let state = get_game_state(&db, &id).expect("读状态失败");
    assert_eq!(state.status, Status::Wishlist);
    assert_eq!(state.game_id, id);
    assert_eq!(state.user_rating, None);
    assert_eq!(state.notes, None);
    assert_eq!(state.default_installation_id, None);
    assert_eq!(state.last_played_at, None);
    assert_eq!(state.play_count, 0);
}

#[test]
fn 带vndb_id建条目时状态同样是wishlist() {
    let db = Db::new();
    let id = new_game(&db, "已绑定");
    update_game(
        &db,
        &id,
        &fishvn::types::GamePatch {
            name: None,
            vndb_id: Some(Some("v20001".into())),
        },
    )
    .expect("绑定失败");

    assert_eq!(get_game_state(&db, &id).unwrap().status, Status::Wishlist);
}

#[test]
fn 四个状态值都能写入() {
    let db = Db::new();
    let id = new_game(&db, "状态流转");

    for value in [
        Status::Wishlist,
        Status::Playing,
        Status::Finished,
        Status::Dropped,
    ] {
        let state = update_game_state(
            &db,
            &id,
            &StatePatch {
                status: Some(value),
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("{value:?} 应可写入：{e}"));
        assert_eq!(state.status, value);
    }
}

#[test]
fn 表外的状态值反序列化就失败() {
    // Status 是枚举，非法值进不到领域函数：serde 这一层就是校验
    for bad in [
        r#""on_hold""#,
        r#""WISHLIST""#,
        r#""playing ""#,
        r#""""#,
        "1",
    ] {
        let payload = format!(r#"{{"status":{bad}}}"#);
        assert!(
            serde_json::from_str::<StatePatch>(&payload).is_err(),
            "status={bad} 应在反序列化就被拒"
        );
    }
}

#[test]
fn status给null等于没给_状态列不允许清空() {
    // status 是 NOT NULL，没有「清空」语义，所以 null 走 Option 的「未提供」解释。
    // 代价：status 单独给 null 会落到「没有可更新的字段」而不是别的错误信息。
    let parsed: StatePatch = serde_json::from_str(r#"{"status":null}"#).expect("null 视为未提供");
    assert_eq!(parsed.status, None);
    assert_eq!(parsed, StatePatch::default());
}

#[test]
fn 非法状态写入不应改变状态() {
    let db = Db::new();
    let id = new_game(&db, "非法状态");
    for bad in [
        r#""on_hold""#,
        r#""WISHLIST""#,
        r#""playing ""#,
        r#""""#,
        "1",
    ] {
        let payload = format!(r#"{{"status":{bad}}}"#);
        assert!(
            serde_json::from_str::<StatePatch>(&payload).is_err(),
            "status={bad} 应在反序列化就被拒"
        );
    }
    assert_eq!(get_game_state(&db, &id).unwrap().status, Status::Wishlist);
}

#[test]
fn patch改user_rating与notes() {
    let db = Db::new();
    let id = new_game(&db, "评分笔记");

    let state = update_game_state(
        &db,
        &id,
        &StatePatch {
            user_rating: Some(Some(8.5)),
            notes: Some(Some("通关了".into())),
            ..Default::default()
        },
    )
    .expect("写评分失败");
    assert_eq!(state.user_rating, Some(8.5));
    assert_eq!(state.notes.as_deref(), Some("通关了"));

    let cleared = update_game_state(
        &db,
        &id,
        &StatePatch {
            user_rating: Some(None),
            notes: Some(None),
            ..Default::default()
        },
    )
    .expect("清空失败");
    assert_eq!(cleared.user_rating, None);
    assert_eq!(cleared.notes, None);
}

#[test]
fn user_rating与notes类型错反序列化就失败() {
    assert!(serde_json::from_str::<StatePatch>(r#"{"user_rating":"8"}"#).is_err());
    assert!(serde_json::from_str::<StatePatch>(r#"{"user_rating":true}"#).is_err());
    assert!(serde_json::from_str::<StatePatch>(r#"{"notes":42}"#).is_err());
}

#[test]
fn patch状态空body返回_bad_request() {
    let db = Db::new();
    let id = new_game(&db, "空改状态");
    let err = update_game_state(&db, &id, &StatePatch::default()).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert_eq!(message(&err), "没有可更新的字段");
}

#[test]
fn 读改不存在的作品状态返回_not_found() {
    let db = Db::new();
    assert_eq!(
        variant(&get_game_state(&db, common::ghost()).unwrap_err()),
        "NotFound"
    );

    let err = update_game_state(
        &db,
        common::ghost(),
        &StatePatch {
            status: Some(Status::Playing),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(variant(&err), "NotFound");
}

#[test]
fn 删作品时状态随外键级联清除() {
    let db = Db::new();
    let id = new_game(&db, "随主删除");
    delete_game(&db, &id).expect("删除失败");

    assert_eq!(variant(&get_game_state(&db, &id).unwrap_err()), "NotFound");
}

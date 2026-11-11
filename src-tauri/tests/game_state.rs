#![allow(non_ascii_idents)]

mod common;

use common::{message, new_game_in as new_game, variant, Db};
use fishvn::games::{delete_game, get_game_state, update_game, update_game_state};
use fishvn::types::{StatePatch, Status};

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
    assert_eq!(state.foreground_seconds, 0);
    assert_eq!(state.alive_seconds, 0);
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

    let clear: StatePatch =
        serde_json::from_str(r#"{"user_rating":null,"notes":null}"#).expect("null 应解析为清空");
    assert_eq!(clear.user_rating, Some(None));
    assert_eq!(clear.notes, Some(None));

    let cleared = update_game_state(&db, &id, &clear).expect("清空失败");
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

    let err = get_game_state(&db, &id).unwrap_err();
    assert_eq!(variant(&err), "NotFound");
    assert_eq!(message(&err), "游戏不存在");
}

fn state_modified_at(db: &Db, id: &str) -> Option<i64> {
    db.query_row(
        "SELECT state_modified_at FROM user_game_state WHERE game_id = ?",
        [id],
        |row| row.get(0),
    )
    .expect("读状态行失败")
}

#[test]
fn 改状态或评分会留下本地修改时间() {
    let db = Db::new();
    let id = new_game(&db, "修改时间");
    assert_eq!(state_modified_at(&db, &id), None, "建条目时没有本地修改记录");

    update_game_state(
        &db,
        &id,
        &StatePatch {
            status: Some(Status::Playing),
            ..Default::default()
        },
    )
    .expect("改状态失败");
    let stamped = state_modified_at(&db, &id);
    assert!(stamped.is_some(), "改状态要留下本地修改时间");

    update_game_state(
        &db,
        &id,
        &StatePatch {
            user_rating: Some(Some(80.0)),
            ..Default::default()
        },
    )
    .expect("改评分失败");
    assert!(state_modified_at(&db, &id).is_some(), "改评分同样要留下时间");
}

#[test]
fn 只改备注不刷新本地修改时间() {
    let db = Db::new();
    let id = new_game(&db, "只改备注");
    update_game_state(
        &db,
        &id,
        &StatePatch {
            status: Some(Status::Playing),
            ..Default::default()
        },
    )
    .expect("改状态失败");
    let stamped = state_modified_at(&db, &id).expect("改状态后应有时间");

    update_game_state(
        &db,
        &id,
        &StatePatch {
            notes: Some(Some("随手记".into())),
            ..Default::default()
        },
    )
    .expect("改备注失败");
    assert_eq!(
        state_modified_at(&db, &id),
        Some(stamped),
        "备注不在同步字段集里，改了也不算本地状态变更"
    );
}

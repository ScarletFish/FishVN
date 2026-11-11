mod common;

use common::new_game_in as new_game;
use fishvn::{db, games};

#[test]
fn 建表跑两遍仍保住原有作品() {
    let db = common::Db::new();
    let id = new_game(&db, "已存在");

    db::ensure_schema(&db).expect("第二遍 ensure_schema 不该报错");

    let listed = games::list_games(&db).expect("列作品不该失败");
    assert_eq!(listed.len(), 1, "重复建表后作品数量保持");
    assert_eq!(listed[0].id, id, "重复建表后作品身份保持");
}

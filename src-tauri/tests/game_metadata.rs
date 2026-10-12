mod common;

use common::Db;
use fishvn::games::{create_game, delete_game};
use fishvn::types::NewGame;

/// 建一条元数据，用裸 SQL —— 领域函数随同步命令一起进来
fn put(db: &Db, game_id: &str, title: &str) {
    db.execute(
        "INSERT INTO game_metadata (game_id, title, votecount) VALUES (?, ?, 0)",
        rusqlite::params![game_id, title],
    )
    .expect("写元数据失败");
}

fn count(db: &Db, game_id: &str) -> i64 {
    db.query_row(
        "SELECT COUNT(*) FROM game_metadata WHERE game_id = ?",
        rusqlite::params![game_id],
        |row| row.get(0),
    )
    .expect("数元数据失败")
}

#[test]
fn 元数据可空列留空() {
    let db = Db::new();
    let game = create_game(&db, &NewGame { name: "只填标题".into(), vndb_id: None }).unwrap();
    put(&db, &game.id, "Fate/stay night");

    let row: (String, Option<String>, Option<f64>, Option<String>) = db
        .query_row(
            "SELECT title, description, rating, image_path FROM game_metadata WHERE game_id = ?",
            rusqlite::params![game.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("读元数据失败");

    assert_eq!(row.0, "Fate/stay night");
    assert_eq!(row.1, None, "description 应为 NULL");
    assert_eq!(row.2, None, "rating 应为 NULL");
    assert_eq!(row.3, None, "image_path 应为 NULL");
}

/// 外键要真的生效才测得到级联，而 SQLite 默认关闭外键，靠 ensure_schema 打开。
/// 少了那行 PRAGMA 这条会失败 —— 孤行会留在库里、作品列表再也看不到它。
#[test]
fn 删作品连带删掉元数据() {
    let db = Db::new();
    let game = create_game(&db, &NewGame { name: "待删".into(), vndb_id: None }).unwrap();
    put(&db, &game.id, "标题");
    assert_eq!(count(&db, &game.id), 1);

    delete_game(&db, &game.id).expect("删除失败");
    assert_eq!(count(&db, &game.id), 0, "元数据不该留在库里");
}

/// 一部作品一行：同步要更新而不是插入第二份
#[test]
fn 同一部作品只能有一份元数据() {
    let db = Db::new();
    let game = create_game(&db, &NewGame { name: "重复写".into(), vndb_id: None }).unwrap();
    put(&db, &game.id, "第一次");
    db.execute(
        "UPDATE game_metadata SET title = ? WHERE game_id = ?",
        rusqlite::params!["第二次", game.id],
    )
    .expect("更新失败");

    assert_eq!(count(&db, &game.id), 1);
    let title: String = db
        .query_row(
            "SELECT title FROM game_metadata WHERE game_id = ?",
            rusqlite::params![game.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(title, "第二次");
}
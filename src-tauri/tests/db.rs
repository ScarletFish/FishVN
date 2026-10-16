mod common;

use fishvn::db;

#[test]
fn 建表跑两遍不报错() {
    let db = common::Db::new();

    db::ensure_schema(&db).expect("第二遍 ensure_schema 不该报错");
}

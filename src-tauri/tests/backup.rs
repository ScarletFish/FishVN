mod common;

use std::fs;

use fishvn::backup;
use fishvn::db;
use fishvn::games;
use fishvn::settings;

fn game_count(shared: &common::Shared) -> usize {
    shared
        .with(games::list_games)
        .expect("列作品不该失败")
        .len()
}

fn list_ids(dir: &std::path::Path) -> Vec<String> {
    backup::list_backups(dir)
        .expect("列备份不该失败")
        .into_iter()
        .map(|m| m.id)
        .collect()
}

#[test]
fn 创建备份后文件与列表一致且可还原() {
    let shared = common::Shared::new();
    let dir = shared.dir().to_path_buf();

    common::new_game(&shared, "还原前");
    assert_eq!(game_count(&shared), 1);

    let meta = backup::create_backup(&shared.conn, &dir).expect("建备份不该失败");
    let backup_db = dir.join("backups").join(&meta.id).join(db::DB_FILE);
    assert!(backup_db.is_file(), "备份库文件应存在");
    assert!(meta.size_bytes > 0, "备份体积应大于 0");
    assert_eq!(list_ids(&dir).len(), 1);

    common::new_game(&shared, "还原后新增");
    assert_eq!(game_count(&shared), 2);

    backup::restore_backup(&shared.conn, &dir, &meta.id).expect("还原不该失败");
    let restored = shared
        .with(games::list_games)
        .expect("列作品不该失败");
    assert_eq!(restored.len(), 1, "还原后应回到备份时刻的一部");
    assert_eq!(restored[0].name, "还原前", "还原后应为备份里的那部");
}

#[test]
fn 删除备份后从列表消失() {
    let shared = common::Shared::new();
    let dir = shared.dir().to_path_buf();

    let meta = backup::create_backup(&shared.conn, &dir).expect("建备份不该失败");
    assert_eq!(list_ids(&dir).len(), 1);

    backup::delete_backup(&dir, &meta.id).expect("删备份不该失败");
    assert!(list_ids(&dir).is_empty(), "删除后列表应为空");
}

#[test]
fn 非法标识被拒绝且缺失备份报未找到() {
    let shared = common::Shared::new();
    let dir = shared.dir().to_path_buf();
    common::new_game(&shared, "还原前");
    let meta = backup::create_backup(&shared.conn, &dir).expect("建备份不该失败");

    let bad = backup::restore_backup(&shared.conn, &dir, "../etc").unwrap_err();
    assert_eq!(common::message(&bad), "备份标识非法");
    let missing = backup::restore_backup(&shared.conn, &dir, "不存在").unwrap_err();
    assert_eq!(common::message(&missing), "备份不存在");
    let del = backup::delete_backup(&dir, "..").unwrap_err();
    assert_eq!(common::message(&del), "备份标识非法");

    assert_eq!(game_count(&shared), 1, "还原失败后库内数据保持");
    assert!(list_ids(&dir).contains(&meta.id), "还原失败后备份仍在");
}

#[test]
fn 还原中途失败后磁盘库与共享连接都不受损() {
    let shared = common::Shared::new();
    let dir = shared.dir().to_path_buf();

    common::new_game(&shared, "还原前");
    let meta = backup::create_backup(&shared.conn, &dir).expect("建备份不该失败");
    fs::remove_file(dir.join("backups").join(&meta.id).join(db::DB_FILE))
        .expect("删备份库不该失败");

    assert!(backup::restore_backup(&shared.conn, &dir, &meta.id).is_err());
    assert_eq!(game_count(&shared), 1, "失败后连接不能停在空内存库");
    common::new_game(&shared, "失败后仍能写");
    assert_eq!(game_count(&shared), 2, "失败后磁盘库仍可写");
}

#[test]
fn 超过保留天数的旧备份在新建时被清掉() {
    let shared = common::Shared::new();
    let dir = shared.dir().to_path_buf();

    fs::write(dir.join("config.json"), r#"{"backup_retention_days":1}"#)
        .expect("写配置不该失败");
    settings::init(&dir);

    let old_id = format!("{:010}000000000", 1_000_000_000);
    fs::create_dir_all(dir.join("backups").join(&old_id)).expect("建旧备份不该失败");
    let fresh_id = format!("{:010}000000000", 2_000_000_000);
    fs::create_dir_all(dir.join("backups").join(&fresh_id)).expect("建新备份不该失败");

    backup::create_backup(&shared.conn, &dir).expect("建备份不该失败");

    let ids = list_ids(&dir);
    assert!(!ids.contains(&old_id), "旧备份应被清理");
    assert!(ids.contains(&fresh_id), "新备份应保留");
}

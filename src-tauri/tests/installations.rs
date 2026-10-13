#![allow(non_ascii_idents)]

mod common;

use common::{message, path_str, variant, Db, Tree};
use fishvn::games::{create_game, delete_game, get_game_state, update_game_state};
use fishvn::installations::{
    create_installation, delete_installation, get_installation, list_installations,
    update_installation,
};
use fishvn::types::{InstallationPatch, NewGame, NewInstallation, StatePatch};

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

fn add(db: &Db, game_id: &str, dir: &std::path::Path) -> fishvn::error::Result<String> {
    create_installation(
        db,
        &NewInstallation {
            game_id: game_id.to_owned(),
            dir_path: path_str(dir),
        },
    )
    .map(|i| i.id)
}

#[test]
fn 建安装时读目录填入候选入口与大小() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "多候选游戏");

    let dir = tree.make_dir(&[
        ("game.exe", "aaaa"),
        ("setup.exe", "bb"),
        ("readme.txt", "cccccccc"),
        ("data/nested.exe", "d"),
    ]);

    let inst = create_installation(
        &db,
        &NewInstallation {
            game_id: game_id.clone(),
            dir_path: path_str(&dir),
        },
    )
    .expect("建安装失败");

    assert_eq!(inst.game_id.as_deref(), Some(game_id.as_str()));
    // 层数浅的在前：两个顶层 exe，再是嵌套的
    assert_eq!(
        inst.exes,
        Some(vec![
            "game.exe".to_owned(),
            "setup.exe".to_owned(),
            "data/nested.exe".to_owned()
        ]),
        "候选入口要按层数排序"
    );
    assert_eq!(inst.size_bytes, Some(4 + 2 + 8 + 1), "大小要累加全部深度");
    assert_eq!(inst.entry_path, None, "候选不止一个时入口留空");
}

#[test]
fn 候选唯一时自动填入口() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "单候选游戏");
    let dir = tree.make_dir(&[("only.bat", "e")]);

    let inst = create_installation(
        &db,
        &NewInstallation {
            game_id,
            dir_path: path_str(&dir),
        },
    )
    .expect("建安装失败");
    assert_eq!(inst.exes, Some(vec!["only.bat".to_owned()]));
    assert_eq!(inst.entry_path.as_deref(), Some("only.bat"));
}

#[test]
fn 没有可执行文件时入口留空等用户指认() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "无入口游戏");
    let dir = tree.make_dir(&[("readme.txt", "ffff")]);

    let inst = create_installation(
        &db,
        &NewInstallation {
            game_id,
            dir_path: path_str(&dir),
        },
    )
    .expect("建安装失败");
    assert_eq!(inst.exes.as_deref(), Some([].as_slice()));
    assert_eq!(inst.entry_path, None);
}

#[test]
fn 目录不存在返回_bad_request() {
    let db = Db::new();
    let tree = Tree::new();
    let game_id = new_game(&db, "路径错误");

    let err = add(&db, &game_id, &tree.missing()).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert!(message(&err).contains("不存在"));
}

#[test]
fn 路径是文件而非目录返回_bad_request() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "传了文件");
    let file = tree.make_dir(&[("x.txt", "x")]).join("x.txt");

    let err = add(&db, &game_id, &file).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert!(message(&err).contains("不是目录"));
}

#[test]
fn 缺game_id或dir_path返回_bad_request() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "参数缺失");
    let dir = tree.make_dir(&[("a.exe", "a")]);

    let no_game = create_installation(
        &db,
        &NewInstallation {
            game_id: String::new(),
            dir_path: path_str(&dir),
        },
    )
    .unwrap_err();
    assert_eq!(variant(&no_game), "BadRequest");
    assert_eq!(message(&no_game), "game_id 必填");

    let no_dir = create_installation(
        &db,
        &NewInstallation {
            game_id: game_id.clone(),
            dir_path: String::new(),
        },
    )
    .unwrap_err();
    assert_eq!(variant(&no_dir), "BadRequest");
    assert_eq!(message(&no_dir), "dir_path 必填");

    let blank = create_installation(
        &db,
        &NewInstallation {
            game_id,
            dir_path: "   ".into(),
        },
    )
    .unwrap_err();
    assert_eq!(variant(&blank), "BadRequest");
    assert_eq!(message(&blank), "dir_path 必填");
}

#[test]
fn 作品不存在返回_not_found() {
    let db = Db::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("a.exe", "a")]);

    let err = add(&db, common::ghost(), &dir).unwrap_err();
    assert_eq!(variant(&err), "NotFound");
}

#[test]
fn 同一目录重复建安装返回_conflict() {
    let db = Db::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("a.exe", "a")]);
    let first = new_game(&db, "占位者");
    let second = new_game(&db, "撞号者");

    add(&db, &first, &dir).expect("首次登记失败");
    let err = add(&db, &second, &dir).unwrap_err();
    assert_eq!(variant(&err), "Conflict");
    assert!(message(&err).contains("已有安装记录"));
}

#[test]
fn patch改入口与改归属作品() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "原属");
    let other = new_game(&db, "新属");
    let dir = tree.make_dir(&[
        ("game.exe", "a"),
        ("setup.exe", "b"),
        ("data/nested.exe", "c"),
    ]);
    let id = add(&db, &game_id, &dir).expect("建安装失败");

    let inst = update_installation(
        &db,
        &id,
        &InstallationPatch {
            game_id: None,
            entry_path: Some(Some("data/nested.exe".into())),
        },
    )
    .expect("改入口失败");
    assert_eq!(inst.entry_path.as_deref(), Some("data/nested.exe"));

    let moved = update_installation(
        &db,
        &id,
        &InstallationPatch {
            game_id: Some(Some(other.clone())),
            entry_path: None,
        },
    )
    .expect("改归属失败");
    assert_eq!(moved.game_id.as_deref(), Some(other.as_str()));
    assert_eq!(
        moved.entry_path.as_deref(),
        Some("data/nested.exe"),
        "没给的字段不该被动到"
    );
}

#[test]
fn patch把归属改成不存在的作品返回_not_found() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "待改归属");
    let dir = tree.make_dir(&[("a.exe", "a")]);
    let id = add(&db, &game_id, &dir).expect("建安装失败");

    let err = update_installation(
        &db,
        &id,
        &InstallationPatch {
            game_id: Some(Some(common::ghost().to_owned())),
            entry_path: None,
        },
    )
    .unwrap_err();
    assert_eq!(variant(&err), "NotFound");
}

#[test]
fn patch归属给空串返回_not_found_不是外键错误() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "空串归属");
    let dir = tree.make_dir(&[("a.exe", "a")]);
    let id = add(&db, &game_id, &dir).expect("建安装失败");

    let err = update_installation(
        &db,
        &id,
        &InstallationPatch {
            game_id: Some(Some(String::new())),
            entry_path: None,
        },
    )
    .unwrap_err();
    assert_eq!(
        variant(&err),
        "NotFound",
        "空串当真实 id 查，查不到就是不存在"
    );
    assert!(!message(&err).contains("FOREIGN"), "不该漏成数据库外键错误");
}

#[test]
fn patch_entry_path为空串返回_bad_request() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "字段校验");
    let dir = tree.make_dir(&[("a.exe", "a")]);
    let id = add(&db, &game_id, &dir).expect("建安装失败");

    let empty = update_installation(
        &db,
        &id,
        &InstallationPatch {
            game_id: None,
            entry_path: Some(Some(String::new())),
        },
    )
    .unwrap_err();
    assert_eq!(variant(&empty), "BadRequest");
    assert_eq!(message(&empty), "entry_path 是非空字符串或 null");
}

#[test]
fn 归属与入口同时非法时报输入问题不是不存在() {
    // 校验顺序：entry_path 的合法性先判，归属查不到是后面的事。
    // 顺序反了同一个请求会从「输入有问题」变成「不存在」
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "双错");
    let dir = tree.make_dir(&[("a.exe", "a")]);
    let id = add(&db, &game_id, &dir).expect("建安装失败");

    let err = update_installation(
        &db,
        &id,
        &InstallationPatch {
            game_id: Some(Some(common::ghost().to_owned())),
            entry_path: Some(Some(String::new())),
        },
    )
    .unwrap_err();
    assert_eq!(
        variant(&err),
        "BadRequest",
        "两个都非法时该报 entry_path 那个"
    );
}

#[test]
fn patch空body返回_bad_request() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "空改安装");
    let dir = tree.make_dir(&[("a.exe", "a")]);
    let id = add(&db, &game_id, &dir).expect("建安装失败");

    let err = update_installation(&db, &id, &InstallationPatch::default()).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert_eq!(message(&err), "没有可更新的字段");
}

#[test]
fn patch不存在的安装返回_not_found() {
    let db = Db::new();
    let err = update_installation(
        &db,
        common::ghost(),
        &InstallationPatch {
            game_id: None,
            entry_path: Some(Some("x.exe".into())),
        },
    )
    .unwrap_err();
    assert_eq!(variant(&err), "NotFound");
}

#[test]
fn get可按game_id过滤() {
    let db = Db::new();
    let mut tree = Tree::new();
    let mine = new_game(&db, "过滤甲");
    let theirs = new_game(&db, "过滤乙");
    let one = add(&db, &mine, &tree.make_dir(&[("a.exe", "a")])).unwrap();
    let two = add(&db, &theirs, &tree.make_dir(&[("b.exe", "b")])).unwrap();

    let filtered = list_installations(&db, Some(&mine)).expect("按作品过滤失败");
    let ids: Vec<&str> = filtered.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, [one.as_str()]);
    assert!(!ids.contains(&two.as_str()), "别作品的安装不该出现");
}

#[test]
fn get无过滤时列出全部() {
    let db = Db::new();
    let mut tree = Tree::new();
    let mine = new_game(&db, "全量甲");
    let theirs = new_game(&db, "全量乙");
    add(&db, &mine, &tree.make_dir(&[("a.exe", "a")])).unwrap();
    add(&db, &theirs, &tree.make_dir(&[("b.exe", "b")])).unwrap();

    assert!(list_installations(&db, None).expect("列全部失败").len() >= 2);
}

#[test]
fn get找不到时返回空数组() {
    let db = Db::new();
    let found = list_installations(&db, Some(common::ghost())).expect("查空结果失败");
    assert!(found.is_empty());
}

#[test]
fn delete删记录不删磁盘文件可重新登记() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "待删安装");
    let dir = tree.make_dir(&[("only.exe", "a")]);
    let id = add(&db, &game_id, &dir).expect("建安装失败");

    delete_installation(&db, &id).expect("删除失败");
    assert_eq!(
        variant(&delete_installation(&db, &id).unwrap_err()),
        "NotFound"
    );

    // 目录仍在，可以重新登记
    assert!(dir.exists(), "删记录不该动磁盘");
    add(&db, &game_id, &dir).expect("重新登记失败");
}

#[test]
fn 默认安装只接受属于该作品的安装() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "默认安装甲");
    let other = new_game(&db, "默认安装乙");
    let mine = add(&db, &game_id, &tree.make_dir(&[("a.exe", "a")])).unwrap();
    let foreign = add(&db, &other, &tree.make_dir(&[("b.exe", "b")])).unwrap();

    let wrong = update_game_state(
        &db,
        &game_id,
        &StatePatch {
            default_installation_id: Some(Some(foreign)),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(variant(&wrong), "BadRequest");
    assert!(message(&wrong).contains("不属于这部作品"));

    let missing = update_game_state(
        &db,
        &game_id,
        &StatePatch {
            default_installation_id: Some(Some(common::ghost().to_owned())),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(variant(&missing), "NotFound");

    let ok = update_game_state(
        &db,
        &game_id,
        &StatePatch {
            default_installation_id: Some(Some(mine.clone())),
            ..Default::default()
        },
    )
    .expect("绑定自己的安装应成功");
    assert_eq!(ok.default_installation_id.as_deref(), Some(mine.as_str()));
}

#[test]
fn 默认安装给空串返回_bad_request_不是外键错误() {
    // 空串会被 SQLite 当成真实 id 去查，撞外键后报成未处理的数据库错误
    let db = Db::new();
    let game_id = new_game(&db, "空串默认安装");

    let err = update_game_state(
        &db,
        &game_id,
        &StatePatch {
            default_installation_id: Some(Some(String::new())),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert_eq!(message(&err), "default_installation_id 是非空字符串或 null");
}

#[test]
fn 删掉默认安装后状态里的引用置空() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "级联置空");
    let id = add(&db, &game_id, &tree.make_dir(&[("a.exe", "a")])).unwrap();

    let set = update_game_state(
        &db,
        &game_id,
        &StatePatch {
            default_installation_id: Some(Some(id.clone())),
            ..Default::default()
        },
    )
    .expect("设默认安装失败");
    assert_eq!(set.default_installation_id.as_deref(), Some(id.as_str()));

    delete_installation(&db, &id).expect("删安装失败");
    assert_eq!(
        get_game_state(&db, &game_id)
            .unwrap()
            .default_installation_id,
        None
    );
}

#[test]
fn 删作品时它的安装退回待认领而非消失() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "作品将删");
    let id = add(&db, &game_id, &tree.make_dir(&[("a.exe", "a")])).unwrap();

    delete_game(&db, &game_id).expect("删作品失败");

    let row = get_installation(&db, &id).expect("安装记录应仍在");
    assert_eq!(row.game_id, None, "应退回待认领");
}

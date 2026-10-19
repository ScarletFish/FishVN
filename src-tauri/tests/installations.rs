#![allow(non_ascii_idents)]

mod common;

use common::{message, path_str, variant, Db, Tree};
use fishvn::fsinfo::resolve_path;
use fishvn::games::{create_game, delete_game, get_game_state, update_game_state};
use fishvn::installations::{
    create_installation, delete_installation, get_installation, list_installations,
    save_installations, update_installation,
};
use fishvn::types::{Installation, InstallationInput, InstallationPatch, NewGame, NewInstallation, StatePatch};

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

fn fresh(dir: &std::path::Path) -> InstallationInput {
    InstallationInput {
        id: None,
        dir_path: path_str(dir),
        entry_path: None,
    }
}

fn kept(inst: &Installation) -> InstallationInput {
    InstallationInput {
        id: Some(inst.id.clone()),
        dir_path: inst.dir_path.clone(),
        entry_path: inst.entry_path.clone(),
    }
}

fn ids(list: &[Installation]) -> Vec<String> {
    list.iter().map(|i| i.id.clone()).collect()
}

fn entry(dir: &std::path::Path, name: &str) -> InstallationInput {
    InstallationInput {
        id: None,
        dir_path: path_str(dir),
        entry_path: Some(name.to_owned()),
    }
}

fn with_entry(inst: &Installation, dir: &std::path::Path, name: &str) -> InstallationInput {
    InstallationInput {
        id: Some(inst.id.clone()),
        dir_path: path_str(dir),
        entry_path: Some(name.to_owned()),
    }
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
    // 层数浅的在前；setup.exe 属安装器，不进候选但计入大小
    assert_eq!(
        inst.exes,
        Some(vec!["game.exe".to_owned(), "data/nested.exe".to_owned()]),
        "候选入口按层数排序，安装器类文件排除"
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
fn 删作品时它的安装一并消失() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "作品将删");
    let id = add(&db, &game_id, &tree.make_dir(&[("a.exe", "a")])).unwrap();

    delete_game(&db, &game_id).expect("删作品失败");

    assert_eq!(
        get_installation(&db, &id).unwrap_err().to_string(),
        "安装记录不存在",
        "安装记录随作品级联删除"
    );
}

fn norm(p: &std::path::Path) -> String {
    resolve_path(&path_str(p)).to_string_lossy().into_owned()
}

#[test]
fn 整包保存按数组顺序写回并读出同序() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "整包排序");
    let first = tree.make_dir(&[("a.exe", "a")]);
    let second = tree.make_dir(&[("b.exe", "b")]);
    let third = tree.make_dir(&[("c.exe", "c")]);

    let saved = save_installations(
        &db,
        &game_id,
        &[fresh(&third), fresh(&first), fresh(&second)],
    )
    .expect("整包保存失败");

    let dirs: Vec<String> = saved.iter().map(|i| i.dir_path.clone()).collect();
    assert_eq!(
        dirs,
        [norm(&third), norm(&first), norm(&second)],
        "读取顺序跟草稿数组一致"
    );
    assert_eq!(
        ids(&list_installations(&db, Some(&game_id)).unwrap()),
        ids(&saved)
    );
}

#[test]
fn 整包保存删掉草稿里没有的安装() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "整包删减");
    let keep_dir = tree.make_dir(&[("a.exe", "a")]);
    let drop_dir = tree.make_dir(&[("b.exe", "b")]);

    let saved = save_installations(&db, &game_id, &[fresh(&keep_dir), fresh(&drop_dir)])
        .expect("整包保存失败");
    let kept_one = saved.iter().find(|i| i.dir_path == norm(&keep_dir)).unwrap();

    let after = save_installations(&db, &game_id, &[kept(kept_one)]).expect("整包保存失败");
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].id, kept_one.id);
    assert!(
        drop_dir.exists(),
        "移除安装项不该动磁盘"
    );
}

#[test]
fn 整包保存保住已有安装的_id() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "整包保号");
    let one = tree.make_dir(&[("a.exe", "a")]);
    let two = tree.make_dir(&[("b.exe", "b")]);

    let first = save_installations(&db, &game_id, &[fresh(&one), fresh(&two)]).expect("首次失败");
    let before = ids(&first);

    let again = save_installations(
        &db,
        &game_id,
        &[kept(&first[1]), kept(&first[0])],
    )
    .expect("重排失败");

    assert_eq!(ids(&again), [before[1].clone(), before[0].clone()], "重排只换顺序");
    assert_eq!(again[0].dir_path, norm(&two));
}

#[test]
fn 删作品后同目录可被新作品登记() {
    let db = Db::new();
    let mut tree = Tree::new();
    let owner = new_game(&db, "原作品");
    let newcomer = new_game(&db, "后来者");
    let dir = tree.make_dir(&[("a.exe", "a")]);
    add(&db, &owner, &dir).expect("登记失败");
    delete_game(&db, &owner).expect("删作品失败");

    let saved = save_installations(&db, &newcomer, &[fresh(&dir)]).expect("登记失败");
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].game_id.as_deref(), Some(newcomer.as_str()));
}

#[test]
fn 整包保存撞别作品的目录返回_conflict() {
    let db = Db::new();
    let mut tree = Tree::new();
    let mine = new_game(&db, "我的作品");
    let theirs = new_game(&db, "别人作品");
    let shared = tree.make_dir(&[("a.exe", "a")]);
    add(&db, &theirs, &shared).expect("别人先登记");

    let err = save_installations(&db, &mine, &[fresh(&shared)]).unwrap_err();
    assert_eq!(variant(&err), "Conflict");
    assert!(message(&err).contains("已有安装记录"));
}

#[test]
fn 整包保存同批重复目录返回_conflict() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "重复目录");
    let dir = tree.make_dir(&[("a.exe", "a")]);

    let err = save_installations(&db, &game_id, &[fresh(&dir), fresh(&dir)]).unwrap_err();
    assert_eq!(variant(&err), "Conflict");
    assert!(message(&err).contains("重复添加"));
}

#[test]
fn 整包保存重复出现同一安装项返回_bad_request() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "重复项");
    let one = tree.make_dir(&[("a.exe", "a")]);
    let two = tree.make_dir(&[("b.exe", "b")]);
    let saved = save_installations(&db, &game_id, &[fresh(&one), fresh(&two)]).expect("首次失败");

    let err = save_installations(
        &db,
        &game_id,
        &[
            kept(&saved[0]),
            InstallationInput {
                id: Some(saved[0].id.clone()),
                dir_path: path_str(&two),
                entry_path: None,
            },
        ],
    )
    .unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert!(message(&err).contains("重复出现"));
}

#[test]
fn 整包保存带别作品的安装_id返回_conflict() {
    let db = Db::new();
    let mut tree = Tree::new();
    let mine = new_game(&db, "我的");
    let theirs = new_game(&db, "别人的");
    let dir = tree.make_dir(&[("a.exe", "a")]);
    let foreign = add(&db, &theirs, &dir).expect("登记失败");

    let err = save_installations(
        &db,
        &mine,
        &[InstallationInput {
            id: Some(foreign),
            dir_path: path_str(&dir),
            entry_path: None,
        }],
    )
    .unwrap_err();
    assert_eq!(variant(&err), "Conflict");
    assert!(message(&err).contains("不属于这部作品"));
}

#[test]
fn 整包保存带不存在的安装_id返回_not_found() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "幽灵安装");
    let dir = tree.make_dir(&[("a.exe", "a")]);

    let err = save_installations(
        &db,
        &game_id,
        &[InstallationInput {
            id: Some(common::ghost().to_owned()),
            dir_path: path_str(&dir),
            entry_path: None,
        }],
    )
    .unwrap_err();
    assert_eq!(variant(&err), "NotFound");
}

#[test]
fn 整包保存缺目录或空_id返回_bad_request() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "参数缺失");
    let dir = tree.make_dir(&[("a.exe", "a")]);

    let blank_dir = save_installations(
        &db,
        &game_id,
        &[InstallationInput {
            id: None,
            dir_path: "   ".into(),
            entry_path: None,
        }],
    )
    .unwrap_err();
    assert_eq!(variant(&blank_dir), "BadRequest");
    assert_eq!(message(&blank_dir), "dir_path 必填");

    let blank_id = save_installations(
        &db,
        &game_id,
        &[InstallationInput {
            id: Some("  ".into()),
            dir_path: path_str(&dir),
            entry_path: None,
        }],
    )
    .unwrap_err();
    assert_eq!(variant(&blank_id), "BadRequest");
    assert!(message(&blank_id).contains("空串"));
}

#[test]
fn 整包保存作品不存在返回_not_found() {
    let db = Db::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("a.exe", "a")]);

    let err = save_installations(&db, common::ghost(), &[fresh(&dir)]).unwrap_err();
    assert_eq!(variant(&err), "NotFound");
}

#[test]
fn 整包保存新目录不存在返回_bad_request() {
    let db = Db::new();
    let tree = Tree::new();
    let game_id = new_game(&db, "目录不在");
    let missing = tree.missing();

    let err = save_installations(&db, &game_id, &[fresh(&missing)]).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert!(message(&err).contains("不存在"));
}

#[test]
fn 整包保存入口越出目录返回_bad_request() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "越级入口");
    let dir = tree.make_dir(&[("a.exe", "a")]);

    for bad in ["", "../outside.exe", "sub/../../up.exe"] {
        let err = save_installations(
            &db,
            &game_id,
            &[InstallationInput {
                id: None,
                dir_path: path_str(&dir),
                entry_path: Some(bad.into()),
            }],
        )
        .unwrap_err();
        assert_eq!(variant(&err), "BadRequest", "入口 {bad:?} 应被拒");
    }
}

#[test]
fn 整包保存接受候选之外的手输入口() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "手输入口");
    let dir = tree.make_dir(&[("launcher.bat", "a")]);

    let saved = save_installations(
        &db,
        &game_id,
        &[InstallationInput {
            id: None,
            dir_path: path_str(&dir),
            entry_path: Some("bin/start.exe".into()),
        }],
    )
    .expect("整包保存失败");
    assert_eq!(saved[0].entry_path.as_deref(), Some("bin/start.exe"));
}

#[test]
fn 整包保存新行不写目录大小与候选同步() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "大小留空");
    let dir = tree.make_dir(&[("game.exe", "a"), ("data/nested.exe", "b")]);

    let saved = save_installations(&db, &game_id, &[fresh(&dir)]).expect("整包保存失败");
    assert_eq!(
        saved[0].exes,
        Some(vec!["game.exe".to_owned(), "data/nested.exe".to_owned()]),
        "候选入口由后端重算"
    );
    assert_eq!(saved[0].size_bytes, None, "本轮不写目录大小");
}

#[test]
fn 整包保存换目录时重算候选并清掉旧的目录大小() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "换目录");
    let old = tree.make_dir(&[("old.exe", "a")]);
    let new = tree.make_dir(&[("new.exe", "b"), ("app/newer.exe", "c")]);

    let before = create_installation(
        &db,
        &NewInstallation {
            game_id: game_id.clone(),
            dir_path: path_str(&old),
        },
    )
    .expect("首次登记失败");
    assert!(before.size_bytes.is_some(), "旧路径留下了大小");

    let after = save_installations(
        &db,
        &game_id,
        &[InstallationInput {
            id: Some(before.id.clone()),
            dir_path: path_str(&new),
            entry_path: None,
        }],
    )
    .expect("换目录失败");

    assert_eq!(after[0].id, before.id, "换目录不换行");
    assert_eq!(after[0].dir_path, norm(&new));
    assert_eq!(
        after[0].exes,
        Some(vec!["new.exe".to_owned(), "app/newer.exe".to_owned()])
    );
    assert_eq!(after[0].size_bytes, None, "旧目录的大小已失效");
}

#[test]
fn 整包保存不重新读已登记的旧目录() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "目录已搬走");
    let dir = tree.make_dir(&[("gone.exe", "a")]);
    let id = add(&db, &game_id, &dir).expect("登记失败");
    let before = get_installation(&db, &id).expect("查安装失败");

    std::fs::remove_dir_all(&dir).expect("删目录失败");

    let after = save_installations(&db, &game_id, &[kept(&before)]).expect("整包保存失败");
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].id, id);
    assert_eq!(after[0].exes, before.exes, "目录未变就不重读");
    assert_eq!(after[0].size_bytes, before.size_bytes);
}

#[test]
fn 整包保存空数组清空该作品的安装() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "清空安装");
    let dir = tree.make_dir(&[("a.exe", "a")]);
    save_installations(&db, &game_id, &[fresh(&dir)]).expect("首次失败");

    let after = save_installations(&db, &game_id, &[]).expect("清空失败");
    assert!(after.is_empty());
    assert!(list_installations(&db, Some(&game_id)).unwrap().is_empty());
    assert!(dir.exists(), "清空安装项不该动磁盘");
}

#[test]
fn 整包保存不动别作品的安装() {
    let db = Db::new();
    let mut tree = Tree::new();
    let mine = new_game(&db, "只动自己的");
    let theirs = new_game(&db, "别碰我");
    let my_dir = tree.make_dir(&[("a.exe", "a")]);
    let their_dir = tree.make_dir(&[("b.exe", "b")]);
    let foreign = add(&db, &theirs, &their_dir).expect("登记失败");

    save_installations(&db, &mine, &[fresh(&my_dir)]).expect("整包保存失败");
    assert_eq!(
        get_installation(&db, &foreign).unwrap().game_id.as_deref(),
        Some(theirs.as_str())
    );
}

#[test]
fn 同一目录下的两个入口各占一条安装() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "原版与汉化");
    let dir = tree.make_dir(&[("game.exe", "a"), ("game_zh.exe", "b")]);

    let saved = save_installations(
        &db,
        &game_id,
        &[entry(&dir, "game.exe"), entry(&dir, "game_zh.exe")],
    )
    .expect("同目录两条失败");

    assert_eq!(saved.len(), 2);
    assert_eq!(saved[0].dir_path, norm(&dir));
    assert_eq!(saved[1].dir_path, norm(&dir));
    assert_ne!(saved[0].id, saved[1].id, "两条各是独立记录");
    assert_eq!(saved[0].entry_path.as_deref(), Some("game.exe"));
    assert_eq!(saved[1].entry_path.as_deref(), Some("game_zh.exe"));
    assert_eq!(
        ids(&list_installations(&db, Some(&game_id)).unwrap()),
        ids(&saved),
        "两条都要读得回来"
    );
}

#[test]
fn 同一目录的同一入口重复保存不新增行() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "重复入口");
    let dir = tree.make_dir(&[("game.exe", "a"), ("game_zh.exe", "b")]);

    let first = save_installations(&db, &game_id, &[entry(&dir, "game.exe")]).expect("首次失败");
    let again = save_installations(&db, &game_id, &[entry(&dir, "game.exe")]).expect("再次失败");

    assert_eq!(again.len(), 1, "同一目录的同一入口只留一条");
    assert_eq!(again[0].id, first[0].id, "再存一次还是原来那行");

    let same_batch = save_installations(
        &db,
        &game_id,
        &[entry(&dir, "game_zh.exe"), entry(&dir, "game_zh.exe")],
    )
    .unwrap_err();
    assert_eq!(variant(&same_batch), "Conflict");
    assert!(message(&same_batch).contains("重复添加"));
}

#[test]
fn 同目录换入口也认得出是同一条安装() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "换入口");
    let dir = tree.make_dir(&[("game.exe", "a"), ("game_zh.exe", "b")]);

    let saved = save_installations(&db, &game_id, &[entry(&dir, "game.exe")]).expect("首次失败");
    let switched =
        save_installations(&db, &game_id, &[with_entry(&saved[0], &dir, "game_zh.exe")])
            .expect("换入口失败");

    assert_eq!(switched.len(), 1);
    assert_eq!(switched[0].id, saved[0].id, "换入口不换行");
    assert_eq!(switched[0].entry_path.as_deref(), Some("game_zh.exe"));
    assert_eq!(
        switched[0].exes.as_deref(),
        Some(
            [
                "game.exe".to_owned(),
                "game_zh.exe".to_owned()
            ]
            .as_slice()
        ),
        "同一目录不重读，候选保持原样"
    );
}

#[test]
fn 目录归了别的作品时换入口也返回_conflict() {
    let db = Db::new();
    let mut tree = Tree::new();
    let mine = new_game(&db, "我的");
    let theirs = new_game(&db, "别人的");
    let dir = tree.make_dir(&[("game.exe", "a"), ("game_zh.exe", "b")]);
    add(&db, &theirs, &dir).expect("别人先登记");

    let err = save_installations(&db, &mine, &[entry(&dir, "game_zh.exe")]).unwrap_err();
    assert_eq!(variant(&err), "Conflict");
    assert!(message(&err).contains("已有安装记录"));
}

#[test]
fn 同目录多入口时读取顺序跟草稿一致() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "顺序");
    let one = tree.make_dir(&[("a.exe", "a"), ("b.exe", "b")]);
    let two = tree.make_dir(&[("c.exe", "c"), ("d.exe", "d")]);

    let saved = save_installations(
        &db,
        &game_id,
        &[
            entry(&two, "d.exe"),
            entry(&one, "a.exe"),
            entry(&two, "c.exe"),
            entry(&one, "b.exe"),
        ],
    )
    .expect("保存失败");

    let pairs: Vec<(String, Option<String>)> = saved
        .iter()
        .map(|i| (i.dir_path.clone(), i.entry_path.clone()))
        .collect();
    assert_eq!(
        pairs,
        [
            (norm(&two), Some("d.exe".to_owned())),
            (norm(&one), Some("a.exe".to_owned())),
            (norm(&two), Some("c.exe".to_owned())),
            (norm(&one), Some("b.exe".to_owned())),
        ],
        "同目录的多条各按自己的下标排"
    );
}

#[test]
fn patch把入口改成同目录已有的那条返回_conflict() {
    let db = Db::new();
    let mut tree = Tree::new();
    let game_id = new_game(&db, "撞入口");
    let dir = tree.make_dir(&[("game.exe", "a"), ("game_zh.exe", "b")]);

    let saved = save_installations(
        &db,
        &game_id,
        &[entry(&dir, "game.exe"), entry(&dir, "game_zh.exe")],
    )
    .expect("保存失败");

    let err = update_installation(
        &db,
        &saved[1].id,
        &InstallationPatch {
            game_id: None,
            entry_path: Some(Some("game.exe".into())),
        },
    )
    .unwrap_err();
    assert_eq!(variant(&err), "Conflict");
    assert!(message(&err).contains("已有安装记录"));

    let same = update_installation(
        &db,
        &saved[1].id,
        &InstallationPatch {
            game_id: None,
            entry_path: Some(Some("game_zh.exe".into())),
        },
    )
    .expect("改成自己原来的值该放行");
    assert_eq!(same.entry_path.as_deref(), Some("game_zh.exe"));
}

mod common;

use std::fs;
use std::thread;
use std::time::{Duration, Instant};

use common::{add_launch, message, new_game, path_str, set_default, variant, Shared, Tree};
use fishvn::installations::update_installation;
use fishvn::sessions;
use fishvn::settings::{self, ConfigPatch};
use fishvn::types::{InstallationPatch, MagpieMode};

static TOOLS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn exclusive() -> std::sync::MutexGuard<'static, ()> {
    TOOLS_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn set_tools(magpie: Option<&str>, le: Option<&str>) {
    let dir = tempfile::tempdir().expect("临时目录建不出来");
    settings::patch(
        dir.path(),
        ConfigPatch {
            magpie_path: magpie.map(str::to_owned),
            le_path: le.map(str::to_owned),
            ..Default::default()
        },
    )
    .expect("设工具路径失败");
}

fn patch_install(shared: &Shared, installation: &str, patch: InstallationPatch) {
    shared
        .with(|conn| update_installation(conn, installation, &patch))
        .expect("改安装启动方式失败");
}

fn flags(le: bool, magpie: MagpieMode) -> InstallationPatch {
    InstallationPatch {
        game_id: None,
        entry_path: None,
        le_launch: Some(le),
        magpie: Some(magpie),
        launch_args: None,
    }
}

fn wait_until_settled(shared: &Shared) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !shared.with(sessions::list_active).unwrap().is_empty() {
        assert!(Instant::now() < deadline, "会话没在预期时间内收尾");
        thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn 转区开着但没登记路径时拦住启动且不起进程() {
    let _guard = exclusive();
    set_tools(None, None);
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("game.exe", "dummy")]);
    let game = new_game(&shared, "转区没配路径");
    let installation = add_launch(&shared, &game, &dir, "game.exe");
    patch_install(&shared, &installation, flags(true, MagpieMode::Off));

    let err = sessions::launch(shared.conn.clone(), &game, None).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert!(message(&err).contains("Locale Emulator"));
    assert!(shared.with(sessions::list_active).unwrap().is_empty());
}

#[test]
fn 放大开着但没登记路径时拦住启动且不起进程() {
    let _guard = exclusive();
    set_tools(None, None);
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("game.exe", "dummy")]);
    let game = new_game(&shared, "放大没配路径");
    let installation = add_launch(&shared, &game, &dir, "game.exe");
    patch_install(&shared, &installation, flags(false, MagpieMode::Full));

    let err = sessions::launch(shared.conn.clone(), &game, None).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert!(message(&err).contains("Magpie"));
    assert!(shared.with(sessions::list_active).unwrap().is_empty());
}

#[test]
fn 窗口化缩放开着但没登记路径时拦住启动且不起进程() {
    let _guard = exclusive();
    set_tools(None, None);
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("game.exe", "dummy")]);
    let game = new_game(&shared, "窗口化没配路径");
    let installation = add_launch(&shared, &game, &dir, "game.exe");
    patch_install(&shared, &installation, flags(false, MagpieMode::Window));

    let err = sessions::launch(shared.conn.clone(), &game, None).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert!(message(&err).contains("Magpie"));
    assert!(shared.with(sessions::list_active).unwrap().is_empty());
}

#[test]
fn 转区启动只收exe入口() {
    let _guard = exclusive();
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("start.bat", "@echo off\r\nexit\r\n")]);
    let le = tree.make_dir(&[("le.bat", "@echo off\r\nexit\r\n")]);
    set_tools(None, Some(&path_str(&le.join("le.bat"))));
    let game = new_game(&shared, "转区配了脚本入口");
    let installation = add_launch(&shared, &game, &dir, "start.bat");
    patch_install(&shared, &installation, flags(true, MagpieMode::Off));

    let err = sessions::launch(shared.conn.clone(), &game, None).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert!(message(&err).contains("转区启动仅支持 exe 入口"));
    assert!(shared.with(sessions::list_active).unwrap().is_empty());
}

#[test]
#[ignore = "真启进程，走 npm run test:launch"]
fn 转区启动把入口路径原样交给登记的程序() {
    let _guard = exclusive();
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[
        ("game.exe", "dummy"),
        ("le.bat", "@echo off\r\necho %1>le-args.txt\r\n"),
    ]);
    set_tools(None, Some(&path_str(&dir.join("le.bat"))));
    let game = new_game(&shared, "转区启动");
    let installation = add_launch(&shared, &game, &dir, "game.exe");
    patch_install(&shared, &installation, flags(true, MagpieMode::Off));
    set_default(&shared, &game, installation);

    sessions::launch(shared.conn.clone(), &game, None).expect("启动失败");
    wait_until_settled(&shared);

    let received = fs::read_to_string(dir.join("le-args.txt")).expect("假转区程序没收到参数");
    assert!(
        received.contains(&path_str(&dir.join("game.exe"))),
        "转区程序拿到的应该是入口绝对路径，实际是 {received:?}"
    );
}

#[test]
#[ignore = "真启进程，走 npm run test:launch"]
fn 附加参数按空格与引号拆开原样进进程() {
    let _guard = exclusive();
    set_tools(None, None);
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("game.bat", "@echo off\r\necho [%*]>args.txt\r\n")]);
    let game = new_game(&shared, "带参数启动");
    let installation = add_launch(&shared, &game, &dir, "game.bat");
    patch_install(
        &shared,
        &installation,
        InstallationPatch {
            launch_args: Some(Some("-window \"D:\\My Games\\One\"".into())),
            ..flags(false, MagpieMode::Off)
        },
    );
    set_default(&shared, &game, installation);

    sessions::launch(shared.conn.clone(), &game, None).expect("启动失败");
    wait_until_settled(&shared);

    let received = fs::read_to_string(dir.join("args.txt")).expect("进程没收到参数");
    assert!(received.contains("-window"), "实际是 {received:?}");
    assert!(received.contains("D:\\My Games\\One"), "实际是 {received:?}");
}

#[test]
#[ignore = "真启进程，走 npm run test:launch"]
fn 附加参数留空时按直接启动处理() {
    let _guard = exclusive();
    set_tools(None, None);
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("game.bat", "@echo off\r\necho [%*]>args.txt\r\n")]);
    let game = new_game(&shared, "空参数启动");
    let installation = add_launch(&shared, &game, &dir, "game.bat");
    patch_install(
        &shared,
        &installation,
        InstallationPatch {
            launch_args: Some(Some("   ".into())),
            ..flags(false, MagpieMode::Off)
        },
    );
    set_default(&shared, &game, installation);

    sessions::launch(shared.conn.clone(), &game, None).expect("启动失败");
    wait_until_settled(&shared);

    let received = fs::read_to_string(dir.join("args.txt")).expect("进程没收到参数");
    assert_eq!(received.trim(), "[]", "实际是 {received:?}");
}

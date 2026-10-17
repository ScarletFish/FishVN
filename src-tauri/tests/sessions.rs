#![allow(non_ascii_idents)]

mod common;

use std::fs;
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use common::{message, variant, Tree};
use fishvn::db;
use fishvn::games::{create_game, get_game_state, update_game_state};
use fishvn::installations::{create_installation, update_installation};
use fishvn::sessions;
use fishvn::types::{InstallationPatch, NewGame, NewInstallation, StatePatch, Status};

struct Shared {
    conn: Arc<Mutex<Connection>>,
    #[allow(dead_code)]
    dir: TempDir,
}

impl Shared {
    fn new() -> Self {
        let dir = tempdir().expect("临时目录建不出来");
        let conn = db::open(dir.path()).expect("临时库开不了");
        Self {
            conn: Arc::new(Mutex::new(conn)),
            dir,
        }
    }

    fn with<T>(&self, f: impl FnOnce(&Connection) -> T) -> T {
        let guard: MutexGuard<'_, Connection> = self.conn.lock().expect("库锁被毒化");
        f(guard.deref())
    }
}

fn new_game(shared: &Shared, name: &str) -> String {
    shared
        .with(|conn| {
            create_game(
                conn,
                &NewGame {
                    name: name.into(),
                    vndb_id: None,
                    metadata: None,
                },
            )
        })
        .expect("建作品失败")
        .id
}

fn add_launch(shared: &Shared, game: &str, dir: &Path, entry: &str) -> String {
    let id = shared
        .with(|conn| {
            create_installation(
                conn,
                &NewInstallation {
                    game_id: game.into(),
                    dir_path: dir.to_string_lossy().into_owned(),
                },
            )
        })
        .expect("建安装失败")
        .id;
    shared
        .with(|conn| {
            update_installation(
                conn,
                &id,
                &InstallationPatch {
                    game_id: None,
                    entry_path: Some(Some(entry.into())),
                },
            )
        })
        .expect("设启动入口失败")
        .id
}

fn set_default(shared: &Shared, game: &str, installation: String) {
    shared
        .with(|conn| {
            update_game_state(
                conn,
                game,
                &StatePatch {
                    default_installation_id: Some(Some(installation)),
                    ..Default::default()
                },
            )
        })
        .expect("设默认启动失败");
}

fn insert_active(shared: &Shared, id: &str, game: &str, started_at: i64, last_seen_at: i64) {
    shared
        .with(|conn| {
            conn.execute(
                "INSERT INTO play_sessions
                   (id, game_id, installation_id, started_at, last_seen_at, ended_at, foreground_seconds)
                 VALUES (?1, ?2, NULL, ?3, ?4, NULL, 90)",
                params![id, game, started_at, last_seen_at],
            )
        })
        .expect("插会话失败");
}

fn wait_until_settled(shared: &Shared) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !shared.with(sessions::list_active).unwrap().is_empty() {
        assert!(Instant::now() < deadline, "会话没在预期时间内收尾");
        thread::sleep(Duration::from_millis(200));
    }
}

fn count_sessions(conn: &Connection, game: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM play_sessions WHERE game_id = ?",
        params![game],
        |row| row.get(0),
    )
    .expect("数会话失败")
}

fn settled_session(conn: &Connection, game: &str) -> (Option<i64>, i64) {
    conn.query_row(
        "SELECT ended_at, COALESCE(ended_at, last_seen_at) - started_at
           FROM play_sessions WHERE game_id = ?",
        params![game],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .expect("读会话失败")
}

#[test]
fn 没玩过的作品派生值都是零与空() {
    let shared = Shared::new();
    let game = new_game(&shared, "没玩过");

    let state = shared.with(|conn| get_game_state(conn, &game)).unwrap();
    assert_eq!(state.play_count, 0);
    assert_eq!(state.last_played_at, None);
    assert_eq!(state.foreground_seconds, 0);
    assert_eq!(state.alive_seconds, 0);
    assert_eq!(state.play_seconds, 0);
    assert!(shared.with(sessions::list_active).unwrap().is_empty());
    assert_eq!(shared.with(|conn| count_sessions(conn, &game)), 0);
}

#[test]
fn 收尾把没结束的会话按最后在场时间结算() {
    let shared = Shared::new();
    let game = new_game(&shared, "上次没收尾");
    insert_active(&shared, "s1", &game, 100, 250);

    shared.with(sessions::sweep).expect("收尾失败");

    let state = shared.with(|conn| get_game_state(conn, &game)).unwrap();
    assert_eq!(state.play_count, 1);
    assert_eq!(state.alive_seconds, 150);
    assert_eq!(state.foreground_seconds, 90);
    assert_eq!(state.play_seconds, 90);
    assert!(shared.with(sessions::list_active).unwrap().is_empty());
}

#[test]
fn 总时长想填多少就多少() {
    let shared = Shared::new();
    let game = new_game(&shared, "随便填");

    let wrote = shared
        .with(|conn| {
            update_game_state(
                conn,
                &game,
                &StatePatch {
                    play_seconds: Some(4321),
                    ..Default::default()
                },
            )
        })
        .expect("写总时长失败");
    assert_eq!(wrote.play_seconds, 4321);

    insert_active(&shared, "s1", &game, 100, 250);
    shared.with(sessions::sweep).expect("收尾失败");

    let played = shared.with(|conn| get_game_state(conn, &game)).unwrap();
    assert_eq!(played.foreground_seconds, 90);
    assert_eq!(
        played.play_seconds,
        4321 + 90,
        "结算把这次的前景时间加在既有的总时长上"
    );

    let lowered = shared
        .with(|conn| {
            update_game_state(
                conn,
                &game,
                &StatePatch {
                    play_seconds: Some(10),
                    ..Default::default()
                },
            )
        })
        .expect("改小总时长失败");
    assert_eq!(lowered.play_seconds, 10);
}

#[test]
fn 总时长不能是负数() {
    let shared = Shared::new();
    let game = new_game(&shared, "负数");

    let err = shared
        .with(|conn| {
            update_game_state(
                conn,
                &game,
                &StatePatch {
                    play_seconds: Some(-1),
                    ..Default::default()
                },
            )
        })
        .unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert_eq!(message(&err), "总时长不能是负数");
}

const SLEEP_BAT: &str = "ping -n 4 127.0.0.1 >nul\n";

#[test]
fn 启动一条后记会话并按进程退出收尾() {
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("fake.bat", SLEEP_BAT)]);
    let game = new_game(&shared, "会启动的作品");
    let installation = add_launch(&shared, &game, &dir, "fake.bat");
    set_default(&shared, &game, installation);

    let session = sessions::launch(shared.conn.clone(), &game).expect("启动失败");
    assert_eq!(session.ended_at, None);
    assert_eq!(session.game_id, game);
    assert_eq!(shared.with(sessions::list_active).unwrap().len(), 1);

    let again = sessions::launch(shared.conn.clone(), &game).unwrap_err();
    assert_eq!(message(&again), "这部作品已经在游玩中");

    wait_until_settled(&shared);

    let (ended_at, alive) = shared.with(|conn| settled_session(conn, &game));
    assert!(ended_at.is_some());
    assert!(
        alive >= 3,
        "存活时长要覆盖 bat 的实际运行时间，拿到 {alive}"
    );

    let state = shared.with(|conn| get_game_state(conn, &game)).unwrap();
    assert_eq!(state.status, Status::Playing);
    assert_eq!(state.play_count, 1);
    assert_eq!(state.last_played_at, Some(session.started_at));
    assert!(state.foreground_seconds <= state.alive_seconds);
    assert_eq!(
        state.play_seconds, state.foreground_seconds,
        "收尾时把这次的前景时间加进总时长"
    );
}

#[test]
fn 已通的作品不会被启动改写状态() {
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("fake.bat", SLEEP_BAT)]);
    let game = new_game(&shared, "已经通了");
    let installation = add_launch(&shared, &game, &dir, "fake.bat");
    set_default(&shared, &game, installation);
    shared
        .with(|conn| {
            update_game_state(
                conn,
                &game,
                &StatePatch {
                    status: Some(Status::Finished),
                    ..Default::default()
                },
            )
        })
        .expect("标已通失败");

    sessions::launch(shared.conn.clone(), &game).expect("启动失败");
    wait_until_settled(&shared);

    assert_eq!(
        shared.with(|conn| get_game_state(conn, &game)).unwrap().status,
        Status::Finished
    );
}

#[test]
fn 没有启动文件的安装不能启动() {
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("readme.txt", "只有说明")]);
    let game = new_game(&shared, "只有目录");
    shared
        .with(|conn| {
            create_installation(
                conn,
                &NewInstallation {
                    game_id: game.clone(),
                    dir_path: dir.to_string_lossy().into_owned(),
                },
            )
        })
        .expect("建安装失败");

    let err = sessions::launch(shared.conn.clone(), &game).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert_eq!(message(&err), "这部作品还没有指定启动文件");
}

#[test]
fn 启动文件不在时拒绝启动() {
    let shared = Shared::new();
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("gone.exe", "不是真的可执行文件")]);
    let game = new_game(&shared, "入口被删");
    let installation = add_launch(&shared, &game, &dir, "gone.exe");
    set_default(&shared, &game, installation);
    fs::remove_file(dir.join("gone.exe")).expect("删入口失败");

    let err = sessions::launch(shared.conn.clone(), &game).unwrap_err();
    assert_eq!(variant(&err), "BadRequest");
    assert!(
        message(&err).starts_with("启动文件不存在："),
        "{}",
        message(&err)
    );
}

#[test]
fn 启动不存在的作品返回_not_found() {
    let shared = Shared::new();
    let err = sessions::launch(shared.conn.clone(), common::ghost()).unwrap_err();
    assert_eq!(variant(&err), "NotFound");
}

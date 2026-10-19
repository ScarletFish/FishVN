#![allow(non_ascii_idents)]

mod common;

use std::fs;

use rusqlite::{params, Connection};

use common::{add_launch, message, new_game, set_default, variant, Shared, Tree};
use fishvn::games::{get_game_state, update_game_state};
use fishvn::installations::create_installation;
use fishvn::sessions;
use fishvn::types::{NewInstallation, StatePatch};

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

fn count_sessions(conn: &Connection, game: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM play_sessions WHERE game_id = ?",
        params![game],
        |row| row.get(0),
    )
    .expect("数会话失败")
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
    assert_eq!(message(&err), "这个游戏还没有指定启动文件");
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

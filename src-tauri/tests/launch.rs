mod common;

use std::thread;
use std::time::{Duration, Instant};

use rusqlite::{params, Connection};

use common::{add_launch, message, new_game, set_default, Shared, Tree};
use fishvn::games::{get_game_state, update_game_state};
use fishvn::sessions;
use fishvn::types::{StatePatch, Status};

const SLEEP_BAT: &str = "ping -n 5 127.0.0.1 >nul\n";

fn wait_until_settled(shared: &Shared) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !shared.with(sessions::list_active).unwrap().is_empty() {
        assert!(Instant::now() < deadline, "会话没在预期时间内收尾");
        thread::sleep(Duration::from_millis(100));
    }
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
#[ignore = "真启进程，走 npm run test:launch"]
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
    assert_eq!(message(&again), "这个游戏已经在游玩中");

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
#[ignore = "真启进程，走 npm run test:launch"]
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

use rusqlite::Connection;
use serde::Serialize;

use crate::error::Result;
use crate::settings::TimeMetric;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DashboardStats {
    pub total: i64,
    pub playing: i64,
    pub wishlist: i64,
    pub finished: i64,
    pub dropped: i64,
    pub total_seconds: i64,
    pub week_seconds: i64,
    pub play_count: i64,
}

pub fn stats(conn: &Connection, metric: TimeMetric, week_start: i64) -> Result<DashboardStats> {
    let mut stmt = conn.prepare(
        "SELECT
            (SELECT COUNT(*) FROM games) AS total,
            (SELECT COALESCE(SUM(s.status = 'playing'), 0) FROM user_game_state s) AS playing,
            (SELECT COALESCE(SUM(s.status = 'wishlist'), 0) FROM user_game_state s) AS wishlist,
            (SELECT COALESCE(SUM(s.status = 'finished'), 0) FROM user_game_state s) AS finished,
            (SELECT COALESCE(SUM(s.status = 'dropped'), 0) FROM user_game_state s) AS dropped,
            (SELECT COALESCE(SUM(s.play_seconds), 0) FROM user_game_state s) AS total_seconds,
            (SELECT COUNT(*) FROM play_sessions) AS play_count",
    )?;
    let mut out = stmt.query_row([], |row| {
        Ok(DashboardStats {
            total: row.get("total")?,
            playing: row.get("playing")?,
            wishlist: row.get("wishlist")?,
            finished: row.get("finished")?,
            dropped: row.get("dropped")?,
            total_seconds: row.get("total_seconds")?,
            week_seconds: 0,
            play_count: row.get("play_count")?,
        })
    })?;
    out.week_seconds = week_seconds(conn, metric, week_start)?;
    Ok(out)
}

pub fn week_seconds(conn: &Connection, metric: TimeMetric, week_start: i64) -> Result<i64> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(SUM(
            CASE WHEN ?1 = 'alive'
                THEN COALESCE(ended_at, last_seen_at) - started_at
                ELSE foreground_seconds
            END), 0)
        FROM play_sessions WHERE started_at >= ?2",
    )?;
    let metric_text = match metric {
        TimeMetric::Foreground => "foreground",
        TimeMetric::Alive => "alive",
    };
    let value: i64 = stmt.query_row(rusqlite::params![metric_text, week_start], |row| {
        row.get(0)
    })?;
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecentPlay {
    pub id: String,
    pub name: String,
    pub cover_path: Option<String>,
    pub last_played_at: i64,
    pub play_seconds: i64,
}

pub fn recent(conn: &Connection, metric: TimeMetric, limit: i64) -> Result<Vec<RecentPlay>> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, m.image_path AS cover_path,
                MAX(p.started_at) AS last_played_at,
                CASE WHEN ?2 = 'alive'
                    THEN COALESCE(SUM(COALESCE(p.ended_at, p.last_seen_at) - p.started_at), 0)
                    ELSE COALESCE(SUM(p.foreground_seconds), 0)
                END AS play_seconds
         FROM games g
         JOIN user_game_state s ON s.game_id = g.id
         JOIN play_sessions p ON p.game_id = g.id
         LEFT JOIN game_metadata m ON m.game_id = g.id
         WHERE s.status = 'playing'
           AND EXISTS (
               SELECT 1 FROM installations i
               WHERE i.game_id = g.id AND i.entry_path IS NOT NULL
           )
         GROUP BY g.id, m.image_path
         ORDER BY last_played_at DESC
         LIMIT ?1",
    )?;
    let metric_text = match metric {
        TimeMetric::Foreground => "foreground",
        TimeMetric::Alive => "alive",
    };
    let rows = stmt.query_map(rusqlite::params![limit, metric_text], |row| {
        Ok(RecentPlay {
            id: row.get("id")?,
            name: row.get("name")?,
            cover_path: row.get("cover_path")?,
            last_played_at: row.get("last_played_at")?,
            play_seconds: row.get("play_seconds")?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::games::create_game;
    use crate::types::NewGame;
    use rusqlite::Connection;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::ensure_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn 空库统计全为零() {
        let conn = db();
        let stats = stats(&conn, TimeMetric::Foreground, 0).unwrap();
        assert_eq!(stats, DashboardStats::default());
    }

    #[test]
    fn 状态计数与累计时长按库内数据聚合() {
        let conn = db();
        let a = create_game(&conn, &NewGame { name: "甲".into(), vndb_id: None, metadata: None }).unwrap().id;
        let b = create_game(&conn, &NewGame { name: "乙".into(), vndb_id: None, metadata: None }).unwrap().id;
        conn.execute(
            "UPDATE user_game_state SET status = 'finished', play_seconds = 3600 WHERE game_id = ?1",
            rusqlite::params![a],
        )
        .unwrap();
        conn.execute(
            "UPDATE user_game_state SET status = 'playing', play_seconds = 60 WHERE game_id = ?1",
            rusqlite::params![b],
        )
        .unwrap();

        let stats = stats(&conn, TimeMetric::Foreground, 0).unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.playing, 1);
        assert_eq!(stats.finished, 1);
        assert_eq!(stats.wishlist, 0);
        assert_eq!(stats.total_seconds, 3660);
    }

    #[test]
    fn 周时长按口径取对应列() {
        let conn = db();
        let id = create_game(&conn, &NewGame { name: "丙".into(), vndb_id: None, metadata: None }).unwrap().id;
        let now = crate::now();
        conn.execute(
            "INSERT INTO play_sessions (id, game_id, started_at, last_seen_at, ended_at, foreground_seconds)
             VALUES ('s1', ?1, ?2, ?3, ?3, 120)",
            rusqlite::params![id, now - 60, now],
        )
        .unwrap();

        let week_start = now - 7 * 24 * 3600;
        assert_eq!(week_seconds(&conn, TimeMetric::Foreground, week_start).unwrap(), 120);
        assert_eq!(week_seconds(&conn, TimeMetric::Alive, week_start).unwrap(), 60);
        assert_eq!(week_seconds(&conn, TimeMetric::Foreground, now + 1).unwrap(), 0);
    }

    fn add_installation(conn: &Connection, game_id: &str, entry: bool) {
        conn.execute(
            "INSERT INTO installations (id, game_id, dir_path, entry_path)
             VALUES (?2, ?1, 'C:/game', ?3)",
            rusqlite::params![game_id, format!("i-{game_id}"), entry.then(|| format!("{game_id}.exe"))],
        )
        .unwrap();
    }

    fn add_session(conn: &Connection, game_id: &str, id: &str, started_at: i64, foreground: i64) {
        conn.execute(
            "INSERT INTO play_sessions (id, game_id, started_at, last_seen_at, ended_at, foreground_seconds)
             VALUES (?2, ?1, ?3, ?4, ?4, ?5)",
            rusqlite::params![game_id, format!("p-{id}"), started_at, started_at + foreground, foreground],
        )
        .unwrap();
    }

    #[test]
    fn 继续游玩只含游玩中有入口有会话的并按最近排序() {
        let conn = db();
        let now = crate::now();
        let make = |name: &str| {
            create_game(&conn, &NewGame { name: name.into(), vndb_id: None, metadata: None })
                .unwrap()
                .id
        };
        let a = make("甲");
        let b = make("乙");
        let c = make("丙");
        let d = make("丁");
        let _e = make("戊");
        add_installation(&conn, &a, true);
        add_installation(&conn, &b, true);
        add_installation(&conn, &c, false);
        add_installation(&conn, &d, true);
        add_installation(&conn, &_e, true);
        add_session(&conn, &a, "a1", now - 3_600, 1_800);
        add_session(&conn, &a, "a2", now - 60, 600);
        add_session(&conn, &b, "b1", now - 30, 120);
        add_session(&conn, &b, "b2", now - 120, 60);
        add_session(&conn, &c, "c1", now - 30, 60);
        add_session(&conn, &d, "d1", now - 10, 60);
        conn.execute(
            "UPDATE user_game_state SET status = 'playing' WHERE game_id IN (?1, ?2, ?3, ?4)",
            rusqlite::params![a, b, c, _e],
        )
        .unwrap();
        conn.execute(
            "UPDATE user_game_state SET status = 'finished' WHERE game_id = ?1",
            rusqlite::params![d],
        )
        .unwrap();

        let recent = recent(&conn, TimeMetric::Foreground, 6).unwrap();
        assert_eq!(
            recent.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
            vec![b.as_str(), a.as_str()]
        );
        assert_eq!(recent[0].play_seconds, 180);
        assert_eq!(recent[1].play_seconds, 2_400);
        assert_eq!(recent[1].last_played_at, now - 60);
    }

    #[test]
    fn 继续游玩受限数量并按口径取时长() {
        let conn = db();
        let now = crate::now();
        let a = create_game(&conn, &NewGame { name: "甲".into(), vndb_id: None, metadata: None }).unwrap().id;
        add_installation(&conn, &a, true);
        conn.execute(
            "UPDATE user_game_state SET status = 'playing' WHERE game_id = ?1",
            rusqlite::params![a],
        )
        .unwrap();
        add_session(&conn, &a, "a1", now - 120, 1_800);
        conn.execute(
            "UPDATE play_sessions SET ended_at = ?1, last_seen_at = ?1 WHERE id = 'p-a1'",
            rusqlite::params![now],
        )
        .unwrap();

        let items = recent(&conn, TimeMetric::Alive, 1).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].play_seconds, 120);
        let items = recent(&conn, TimeMetric::Foreground, 1).unwrap();
        assert_eq!(items[0].play_seconds, 1_800);
    }
}

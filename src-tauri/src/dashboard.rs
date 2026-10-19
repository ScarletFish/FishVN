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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::games::create_game;
    use crate::types::NewGame;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};

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
}

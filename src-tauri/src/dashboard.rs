use std::collections::HashMap;

use rusqlite::Connection;
use serde::Serialize;

use crate::error::Result;
use crate::settings::{SpoilerLevel, TimeMetric};
use crate::tagdict::{self, TagScope};

pub const SHOWCASE_LIMIT: i64 = 30;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DashboardStats {
    pub total: i64,
    pub playing: i64,
    pub wishlist: i64,
    pub finished: i64,
    pub dropped: i64,
    pub stalled: i64,
    pub total_seconds: i64,
    pub week_seconds: i64,
    pub play_count: i64,
}

fn metric_text(metric: TimeMetric) -> &'static str {
    match metric {
        TimeMetric::Foreground => "foreground",
        TimeMetric::Alive => "alive",
    }
}

pub fn stats(conn: &Connection, metric: TimeMetric, week_start: i64) -> Result<DashboardStats> {
    let mut stmt = conn.prepare(
        "SELECT
            (SELECT COUNT(*) FROM games) AS total,
            (SELECT COALESCE(SUM(s.status = 'playing'), 0) FROM user_game_state s) AS playing,
            (SELECT COALESCE(SUM(s.status = 'wishlist'), 0) FROM user_game_state s) AS wishlist,
            (SELECT COALESCE(SUM(s.status = 'finished'), 0) FROM user_game_state s) AS finished,
            (SELECT COALESCE(SUM(s.status = 'dropped'), 0) FROM user_game_state s) AS dropped,
            (SELECT COALESCE(SUM(s.status = 'stalled'), 0) FROM user_game_state s) AS stalled,
            (SELECT COALESCE(SUM(s.play_seconds), 0) FROM user_game_state s) AS total_seconds,
            (SELECT COUNT(*) FROM play_sessions) AS play_count,
            (SELECT COALESCE(SUM(
                CASE WHEN ?1 = 'alive'
                    THEN COALESCE(ended_at, last_seen_at) - started_at
                    ELSE foreground_seconds
                END), 0)
             FROM play_sessions WHERE started_at >= ?2) AS week_seconds",
    )?;
    let out = stmt.query_row(rusqlite::params![metric_text(metric), week_start], |row| {
        Ok(DashboardStats {
            total: row.get("total")?,
            playing: row.get("playing")?,
            wishlist: row.get("wishlist")?,
            finished: row.get("finished")?,
            dropped: row.get("dropped")?,
            stalled: row.get("stalled")?,
            total_seconds: row.get("total_seconds")?,
            week_seconds: row.get("week_seconds")?,
            play_count: row.get("play_count")?,
        })
    })?;
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecentPlay {
    pub id: String,
    pub name: String,
    pub cover_path: Option<String>,
    pub cover_focal_x: Option<i64>,
    pub cover_focal_y: Option<i64>,
    pub last_played_at: i64,
    pub total_seconds: i64,
}

pub fn recent(conn: &Connection, limit: i64) -> Result<Vec<RecentPlay>> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, m.image_path AS cover_path,
                m.cover_focal_x AS cover_focal_x,
                m.cover_focal_y AS cover_focal_y,
                MAX(p.started_at) AS last_played_at,
                s.play_seconds AS total_seconds
         FROM games g
         JOIN user_game_state s ON s.game_id = g.id
         JOIN play_sessions p ON p.game_id = g.id
         LEFT JOIN game_metadata m ON m.game_id = g.id
         WHERE s.status = 'playing'
           AND EXISTS (
               SELECT 1 FROM installations i
               WHERE i.game_id = g.id AND i.entry_path IS NOT NULL
           )
         GROUP BY g.id, m.image_path, m.cover_focal_x, m.cover_focal_y
         ORDER BY last_played_at DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![limit], |row| {
        Ok(RecentPlay {
            id: row.get("id")?,
            name: row.get("name")?,
            cover_path: row.get("cover_path")?,
            cover_focal_x: row.get("cover_focal_x")?,
            cover_focal_y: row.get("cover_focal_y")?,
            last_played_at: row.get("last_played_at")?,
            total_seconds: row.get("total_seconds")?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ShowcaseItem {
    pub id: String,
    pub name: String,
    pub play_seconds: i64,
    pub can_launch: bool,
    pub last_played_at: i64,
    pub state_modified_at: i64,
    pub cover_path: Option<String>,
    pub cover_focal_x: Option<i64>,
    pub cover_focal_y: Option<i64>,
    pub ambience_path: Option<String>,
    pub showcase_focal_x: i64,
    pub showcase_focal_y: i64,
    pub showcase_mode: String,
    pub ambience_mode: String,
}

pub fn showcase_items(conn: &Connection, limit: i64) -> Result<Vec<ShowcaseItem>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM (
            SELECT g.id AS id, g.name AS name,
                   COALESCE(s.play_seconds, 0) AS play_seconds,
                   COALESCE(MAX(CASE WHEN i.entry_path IS NOT NULL THEN 1 ELSE 0 END), 0) AS can_launch,
                   COALESCE((SELECT MAX(p.started_at) FROM play_sessions p WHERE p.game_id = g.id), 0) AS last_played_at,
                   COALESCE(s.state_modified_at, 0) AS state_modified_at,
                   m.image_path AS cover_path,
                   m.cover_focal_x AS cover_focal_x,
                   m.cover_focal_y AS cover_focal_y,
                   m.ambience_path AS ambience_path,
                   COALESCE(m.showcase_focal_x, 50) AS showcase_focal_x,
                   COALESCE(m.showcase_focal_y, 22) AS showcase_focal_y,
                   COALESCE(m.showcase_mode, 'soft') AS showcase_mode,
                   COALESCE(m.ambience_mode, 'soft') AS ambience_mode
            FROM games g
            JOIN user_game_state s ON s.game_id = g.id
            LEFT JOIN installations i ON i.game_id = g.id
            LEFT JOIN game_metadata m ON m.game_id = g.id
            WHERE s.status = 'playing'
               OR EXISTS (SELECT 1 FROM play_sessions p WHERE p.game_id = g.id)
            GROUP BY g.id, m.image_path, m.cover_focal_x, m.cover_focal_y,
                     m.ambience_path, m.showcase_focal_x, m.showcase_focal_y,
                     m.showcase_mode, m.ambience_mode
         )
         ORDER BY last_played_at DESC,
                  state_modified_at DESC,
                  name COLLATE NOCASE
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![limit], |row| {
        Ok(ShowcaseItem {
            id: row.get("id")?,
            name: row.get("name")?,
            play_seconds: row.get("play_seconds")?,
            can_launch: row.get::<_, i64>("can_launch")? == 1,
            last_played_at: row.get("last_played_at")?,
            state_modified_at: row.get("state_modified_at")?,
            cover_path: row.get("cover_path")?,
            cover_focal_x: row.get("cover_focal_x")?,
            cover_focal_y: row.get("cover_focal_y")?,
            ambience_path: row.get("ambience_path")?,
            showcase_focal_x: row.get("showcase_focal_x")?,
            showcase_focal_y: row.get("showcase_focal_y")?,
            showcase_mode: row.get("showcase_mode")?,
            ambience_mode: row.get("ambience_mode")?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TagCount {
    pub id: String,
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LibraryTag {
    pub id: String,
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TagCloudView {
    pub tags: Vec<TagCount>,
    pub candidates: i64,
    pub excluded: i64,
}

struct TagAgg {
    id: String,
    name: String,
    count: i64,
}

const CLOUD_MIN_COUNT: i64 = 2;

fn keep_tag_id(id: &str, hide_r18_tags: bool) -> bool {
    let dict = tagdict::dictionary();
    if dict.tag_groups.scope(Some(id)) != TagScope::Full {
        return false;
    }
    !(hide_r18_tags
        && dict
            .tags
            .get(id)
            .and_then(|entry| entry.category.as_deref())
            == Some("ero"))
}

pub fn library_tags(
    conn: &Connection,
    hide_r18_tags: bool,
    spoiler_level: SpoilerLevel,
    excluded: &[String],
) -> Result<Vec<LibraryTag>> {
    let counts = count_tags(conn, spoiler_level)?;
    let mut pool: Vec<LibraryTag> = counts
        .into_values()
        .filter(|a| keep_tag_id(&a.id, hide_r18_tags))
        .filter(|a| !excluded.iter().any(|entry| entry == &a.id))
        .map(|a| LibraryTag {
            id: a.id,
            name: a.name,
            count: a.count,
        })
        .collect();
    pool.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    Ok(pool)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LibraryTrait {
    pub id: String,
    pub name: String,
    pub group: Option<String>,
    pub count: i64,
}

pub fn library_traits(conn: &Connection) -> Result<Vec<LibraryTrait>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.grp, COUNT(*) AS n
           FROM character_trait ct JOIN traits t ON t.id = ct.trait_id
          GROUP BY t.id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(LibraryTrait {
            id: row.get(0)?,
            name: row.get(1)?,
            group: row.get(2)?,
            count: row.get(3)?,
        })
    })?;
    let mut out: Vec<LibraryTrait> = rows.flatten().collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    Ok(out)
}

pub fn tag_cloud(
    conn: &Connection,
    limit: i64,
    hide_r18_tags: bool,
    spoiler_level: SpoilerLevel,
    excluded: &[String],
) -> Result<TagCloudView> {
    let counts = count_tags(conn, spoiler_level)?;
    let mut pool: Vec<TagAgg> = counts
        .into_values()
        .filter(|a| keep_tag_id(&a.id, hide_r18_tags))
        .filter(|a| a.count >= CLOUD_MIN_COUNT)
        .collect();
    pool.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    let candidates = pool.len() as i64;
    let (hidden, visible): (Vec<TagAgg>, Vec<TagAgg>) = pool
        .into_iter()
        .partition(|a| excluded.iter().any(|entry| entry == &a.id));
    let mut out: Vec<TagCount> = visible
        .into_iter()
        .map(|a| TagCount {
            id: a.id,
            name: a.name,
            count: a.count,
        })
        .collect();
    out.truncate(limit.max(0) as usize);
    Ok(TagCloudView {
        tags: out,
        candidates,
        excluded: hidden.len() as i64,
    })
}

fn count_tags(
    conn: &Connection,
    spoiler_level: SpoilerLevel,
) -> Result<HashMap<String, TagAgg>> {
    let threshold = match spoiler_level {
        SpoilerLevel::HideAll => 1,
        SpoilerLevel::HideMajor => 2,
        SpoilerLevel::ShowAll => i64::MAX,
    };
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name,
                SUM(CASE WHEN COALESCE(gt.lie, 0) = 0
                          AND COALESCE(gt.spoiler, 0) < ?1
                         THEN 1 ELSE 0 END) AS n
           FROM game_tag gt JOIN tags t ON t.id = gt.tag_id
          GROUP BY t.id
         HAVING n > 0",
    )?;
    let rows = stmt.query_map([threshold], |row| {
        Ok(TagAgg {
            id: row.get(0)?,
            name: row.get(1)?,
            count: row.get(2)?,
        })
    })?;
    Ok(rows.flatten().map(|a| (a.id.clone(), a)).collect())
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlaytimeRank {
    pub id: String,
    pub name: String,
    pub seconds: i64,
}

pub fn playtime_top(conn: &Connection, limit: i64) -> Result<Vec<PlaytimeRank>> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, COALESCE(s.play_seconds, 0) AS seconds
         FROM games g
         JOIN user_game_state s ON s.game_id = g.id
         WHERE s.play_seconds > 0
         ORDER BY s.play_seconds DESC, g.name
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![limit.max(0)], |row| {
        Ok(PlaytimeRank {
            id: row.get("id")?,
            name: row.get("name")?,
            seconds: row.get("seconds")?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EraPoint {
    pub key: String,
    pub count: i64,
}

const ERA_KEYS: [&str; 4] = ["pre2000", "y2000", "y2010", "y2020"];

pub fn release_eras(conn: &Connection, status: Option<&str>) -> Result<Vec<EraPoint>> {
    let mut stmt = conn.prepare(
        "SELECT CASE
            WHEN CAST(substr(m.released, 1, 4) AS INTEGER) < 2000 THEN 'pre2000'
            WHEN CAST(substr(m.released, 1, 4) AS INTEGER) < 2010 THEN 'y2000'
            WHEN CAST(substr(m.released, 1, 4) AS INTEGER) < 2020 THEN 'y2010'
            ELSE 'y2020'
         END AS era, COUNT(*) AS count
         FROM game_metadata m
         LEFT JOIN user_game_state s ON s.game_id = m.game_id
         WHERE m.released IS NOT NULL
           AND length(m.released) >= 4
           AND CAST(substr(m.released, 1, 4) AS INTEGER) BETWEEN 1900 AND 2100
           AND (?1 IS NULL OR s.status = ?1)
         GROUP BY era",
    )?;
    let rows = stmt.query_map(rusqlite::params![status], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    let counts: HashMap<String, i64> = rows.collect::<rusqlite::Result<HashMap<_, _>>>()?;
    Ok(ERA_KEYS
        .iter()
        .map(|key| EraPoint {
            key: (*key).to_string(),
            count: counts.get(*key).copied().unwrap_or(0),
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActivityPoint {
    pub bucket: String,
    pub seconds: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Granularity {
    Week,
    Month,
    Year,
}

impl Granularity {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "week" => Some(Self::Week),
            "month" => Some(Self::Month),
            "year" => Some(Self::Year),
            _ => None,
        }
    }
}

pub const ACTIVITY_BUCKETS: i64 = 12;

fn activity_sql(granularity: Granularity) -> String {
    let (anchor, bucket, step) = match granularity {
        Granularity::Week => (
            "date(?1, 'unixepoch', 'weekday 0', '-6 days')",
            "date(p.started_at, 'unixepoch', 'weekday 0', '-6 days')",
            "-7 days",
        ),
        Granularity::Month => (
            "strftime('%Y-%m-01', ?1, 'unixepoch')",
            "strftime('%Y-%m-01', p.started_at, 'unixepoch')",
            "-1 month",
        ),
        Granularity::Year => (
            "strftime('%Y-01-01', ?1, 'unixepoch')",
            "strftime('%Y-01-01', p.started_at, 'unixepoch')",
            "-1 year",
        ),
    };
    format!(
        "WITH RECURSIVE seq(k) AS (
            SELECT {anchor}
            UNION ALL
            SELECT date(k, '{step}') FROM seq LIMIT ?2
         )
         SELECT seq.k AS k, COALESCE(SUM(
                CASE WHEN ?3 = 'alive'
                    THEN COALESCE(p.ended_at, p.last_seen_at) - p.started_at
                    ELSE p.foreground_seconds
                END), 0) AS s
         FROM seq
         LEFT JOIN play_sessions p ON {bucket} = seq.k
         GROUP BY seq.k
         ORDER BY seq.k"
    )
}

pub fn activity(
    conn: &Connection,
    now: i64,
    buckets: i64,
    granularity: Granularity,
    metric: TimeMetric,
) -> Result<Vec<ActivityPoint>> {
    let mut stmt = conn.prepare(&activity_sql(granularity))?;
    let rows = stmt.query_map(
        rusqlite::params![now, buckets.max(1), metric_text(metric)],
        |row| {
            Ok(ActivityPoint {
                bucket: row.get("k")?,
                seconds: row.get("s")?,
            })
        },
    )?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GameTrendSeries {
    pub game_id: String,
    pub name: String,
    pub total: i64,
    pub values: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GameTrend {
    pub buckets: Vec<String>,
    pub series: Vec<GameTrendSeries>,
}

pub const TREND_GAMES: i64 = 6;

pub fn game_trend(
    conn: &Connection,
    now: i64,
    months: i64,
    top: i64,
    metric: TimeMetric,
) -> Result<GameTrend> {
    let months = months.max(1);
    let top = top.max(1);
    let buckets: Vec<String> = {
        let mut stmt = conn.prepare(
            "WITH RECURSIVE seq(k) AS (
                SELECT strftime('%Y-%m-01', ?1, 'unixepoch')
                UNION ALL
                SELECT date(k, '-1 month') FROM seq LIMIT ?2
             )
             SELECT k FROM seq ORDER BY k",
        )?;
        let rows = stmt.query_map(rusqlite::params![now, months], |row| row.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let start = buckets.first().cloned().unwrap_or_default();

    let leaders: Vec<(String, String, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT g.id AS id, g.name AS name, SUM(
                CASE WHEN ?3 = 'alive'
                    THEN COALESCE(p.ended_at, p.last_seen_at) - p.started_at
                    ELSE p.foreground_seconds
                END) AS total
             FROM play_sessions p
             JOIN games g ON g.id = p.game_id
             WHERE strftime('%Y-%m-01', p.started_at, 'unixepoch') >= ?1
             GROUP BY g.id
             ORDER BY total DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![start, top, metric_text(metric)], |row| {
            Ok((row.get("id")?, row.get("name")?, row.get("total")?))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    let mut lookup: HashMap<(String, String), i64> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT p.game_id AS id,
                    strftime('%Y-%m-01', p.started_at, 'unixepoch') AS k,
                    SUM(CASE WHEN ?2 = 'alive'
                        THEN COALESCE(p.ended_at, p.last_seen_at) - p.started_at
                        ELSE p.foreground_seconds
                    END) AS s
             FROM play_sessions p
             WHERE strftime('%Y-%m-01', p.started_at, 'unixepoch') >= ?1
             GROUP BY p.game_id, k",
        )?;
        let rows = stmt.query_map(rusqlite::params![start, metric_text(metric)], |row| {
            Ok((
                (row.get::<_, String>("id")?, row.get::<_, String>("k")?),
                row.get::<_, i64>("s")?,
            ))
        })?;
        for row in rows {
            let (key, seconds) = row?;
            lookup.insert(key, seconds);
        }
    }

    let series = leaders
        .into_iter()
        .map(|(game_id, name, total)| {
            let values = buckets
                .iter()
                .map(|b| lookup.get(&(game_id.clone(), b.clone())).copied().unwrap_or(0))
                .collect();
            GameTrendSeries { game_id, name, total, values }
        })
        .collect();

    Ok(GameTrend { buckets, series })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LengthComparePoint {
    pub id: String,
    pub name: String,
    pub expected: i64,
    pub actual: i64,
}

pub fn length_compare(conn: &Connection) -> Result<Vec<LengthComparePoint>> {
    let mut stmt = conn.prepare(
        "SELECT g.id AS id, g.name AS name,
                m.length_minutes * 60 AS expected,
                s.play_seconds AS actual
         FROM user_game_state s
         JOIN games g ON g.id = s.game_id
         JOIN game_metadata m ON m.game_id = g.id
         WHERE s.play_seconds > 0 AND m.length_minutes > 0
         ORDER BY actual DESC, g.name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(LengthComparePoint {
            id: row.get("id")?,
            name: row.get("name")?,
            expected: row.get("expected")?,
            actual: row.get("actual")?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LengthBucket {
    pub key: String,
    pub count: i64,
}

const LENGTH_KEYS: [&str; 5] = ["b1", "b2", "b3", "b4", "b5"];

pub fn length_buckets(conn: &Connection) -> Result<Vec<LengthBucket>> {
    let mut stmt = conn.prepare(
        "SELECT CASE
            WHEN m.length_minutes < 600 THEN 'b1'
            WHEN m.length_minutes < 1200 THEN 'b2'
            WHEN m.length_minutes < 1800 THEN 'b3'
            WHEN m.length_minutes < 3000 THEN 'b4'
            ELSE 'b5'
         END AS k, COUNT(*) AS n
         FROM game_metadata m
         WHERE m.length_minutes > 0
         GROUP BY k",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>("k")?, row.get::<_, i64>("n")?))
    })?;
    let mut counts: HashMap<String, i64> = HashMap::new();
    for row in rows {
        let (key, count) = row?;
        counts.insert(key, count);
    }
    Ok(LENGTH_KEYS
        .iter()
        .map(|key| LengthBucket {
            key: key.to_string(),
            count: counts.get(*key).copied().unwrap_or(0),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::types::{CharacterTrait, MetadataTag};
    use crate::games::create_game;
    use crate::types::NewGame;
    use rusqlite::Connection;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::ensure_schema(&conn).unwrap();
        conn
    }

    fn make_game(conn: &Connection, name: &str) -> String {
        create_game(conn, &NewGame { name: name.into(), vndb_id: None, metadata: None })
            .unwrap()
            .id
    }

    fn set_status(conn: &Connection, game_id: &str, status: &str) {
        conn.execute(
            "UPDATE user_game_state SET status = ?2 WHERE game_id = ?1",
            rusqlite::params![game_id, status],
        )
        .unwrap();
    }

    fn set_play_seconds(conn: &Connection, game_id: &str, seconds: i64) {
        conn.execute(
            "UPDATE user_game_state SET play_seconds = ?2 WHERE game_id = ?1",
            rusqlite::params![game_id, seconds],
        )
        .unwrap();
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
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        set_status(&conn, &a, "finished");
        set_play_seconds(&conn, &a, 3600);
        set_status(&conn, &b, "playing");
        set_play_seconds(&conn, &b, 60);

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
        let id = make_game(&conn, "丙");
        let now = crate::now();
        conn.execute(
            "INSERT INTO play_sessions (id, game_id, started_at, last_seen_at, ended_at, foreground_seconds)
             VALUES ('s1', ?1, ?2, ?3, ?3, 120)",
            rusqlite::params![id, now - 60, now],
        )
        .unwrap();

        let week_start = now - 7 * 24 * 3600;
        assert_eq!(stats(&conn, TimeMetric::Foreground, week_start).unwrap().week_seconds, 120);
        assert_eq!(stats(&conn, TimeMetric::Alive, week_start).unwrap().week_seconds, 60);
        assert_eq!(stats(&conn, TimeMetric::Foreground, now + 1).unwrap().week_seconds, 0);
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

    fn ts(conn: &Connection, date: &str, time: &str) -> i64 {
        conn.query_row(
            "SELECT CAST(strftime('%s', ?1 || 'T' || ?2, 'utc') AS INTEGER)",
            rusqlite::params![date, time],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn 展厅候选集只含在玩与玩过且计划不参与() {
        let conn = db();
        let now = crate::now();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        let c = make_game(&conn, "丙");
        let d = make_game(&conn, "丁");
        set_status(&conn, &a, "finished");
        set_status(&conn, &b, "playing");
        set_status(&conn, &c, "wishlist");
        set_status(&conn, &d, "stalled");
        set_play_seconds(&conn, &a, 3600);
        add_session(&conn, &a, "a1", now - 60, 60);
        add_session(&conn, &b, "b1", now - 120, 60);
        add_installation(&conn, &b, true);

        let items = showcase_items(&conn, SHOWCASE_LIMIT).unwrap();
        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&a.as_str()));
        assert!(ids.contains(&b.as_str()));
        assert!(!ids.contains(&c.as_str()));
        assert!(!ids.contains(&d.as_str()));
        assert_eq!(items[0].id, a);
        assert_eq!(items[0].play_seconds, 3600);
        assert_eq!(items[0].last_played_at, now - 60);
        assert!(items.iter().find(|i| i.id == b).unwrap().can_launch);
        assert_eq!(items.iter().find(|i| i.id == b).unwrap().ambience_mode, "soft");
    }

    #[test]
    fn 展厅在玩部分按最近改状态排序() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        let c = make_game(&conn, "丙");
        set_status(&conn, &a, "playing");
        set_status(&conn, &b, "playing");
        set_status(&conn, &c, "playing");
        conn.execute(
            "UPDATE user_game_state SET state_modified_at = 300 WHERE game_id = ?1",
            rusqlite::params![a],
        )
        .unwrap();
        conn.execute(
            "UPDATE user_game_state SET state_modified_at = 100 WHERE game_id = ?1",
            rusqlite::params![b],
        )
        .unwrap();

        let items = showcase_items(&conn, SHOWCASE_LIMIT).unwrap();
        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert_eq!(ids, vec![a.as_str(), b.as_str(), c.as_str()]);
    }

    #[test]
    fn 展厅投影带自定义图与观感并尊重限量() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        set_status(&conn, &a, "playing");
        set_status(&conn, &b, "playing");
        conn.execute(
            "INSERT INTO game_metadata
                (game_id, image_path, ambience_path, ambience_mode,
                 showcase_focal_x, showcase_focal_y, showcase_mode)
             VALUES (?1, 'cover.jpg', 'cg.png', 'clear', 70, 80, 'soft')
             ON CONFLICT(game_id) DO UPDATE SET
                image_path = 'cover.jpg', ambience_path = 'cg.png', ambience_mode = 'clear',
                showcase_focal_x = 70, showcase_focal_y = 80, showcase_mode = 'soft'",
            rusqlite::params![a],
        )
        .unwrap();

        let all = showcase_items(&conn, SHOWCASE_LIMIT).unwrap();
        assert_eq!(all.len(), 2);
        let item = all.iter().find(|i| i.id == a).unwrap();
        assert_eq!(item.cover_path.as_deref(), Some("cover.jpg"));
        assert_eq!(item.ambience_path.as_deref(), Some("cg.png"));
        assert_eq!(item.ambience_mode, "clear");
        assert_eq!(item.showcase_focal_x, 70);
        assert_eq!(item.showcase_focal_y, 80);
        assert_eq!(item.ambience_mode, "clear");
        assert_eq!(item.showcase_mode, "soft");
        let other = all.iter().find(|i| i.id == b).unwrap();
        assert_eq!(other.ambience_mode, "soft");
        assert_eq!(other.showcase_mode, "soft");
        assert_eq!(other.showcase_focal_x, 50);
        assert_eq!(other.showcase_focal_y, 22);
        assert_eq!(showcase_items(&conn, 1).unwrap().len(), 1);
    }

    #[test]
    fn 继续游玩只含游玩中有入口有会话的并按最近排序() {
        let conn = db();
        let now = crate::now();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        let c = make_game(&conn, "丙");
        let d = make_game(&conn, "丁");
        let e = make_game(&conn, "戊");
        add_installation(&conn, &a, true);
        add_installation(&conn, &b, true);
        add_installation(&conn, &c, false);
        add_installation(&conn, &d, true);
        add_installation(&conn, &e, true);
        add_session(&conn, &a, "a1", now - 3_600, 1_800);
        add_session(&conn, &a, "a2", now - 60, 600);
        add_session(&conn, &b, "b1", now - 30, 120);
        add_session(&conn, &b, "b2", now - 120, 60);
        add_session(&conn, &c, "c1", now - 30, 60);
        add_session(&conn, &d, "d1", now - 10, 60);
        for id in [&a, &b, &c, &e] {
            set_status(&conn, id, "playing");
        }
        set_status(&conn, &d, "finished");
        set_play_seconds(&conn, &a, 2_400);
        set_play_seconds(&conn, &b, 99);

        let recent = recent(&conn, 6).unwrap();
        assert_eq!(
            recent.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
            vec![b.as_str(), a.as_str()]
        );
        assert_eq!(recent[0].total_seconds, 99);
        assert_eq!(recent[1].total_seconds, 2_400);
        assert_eq!(recent[1].last_played_at, now - 60);
    }

    #[test]
    fn 继续游玩取用户可改的总时长而非按口径聚合会话() {
        let conn = db();
        let now = crate::now();
        let a = make_game(&conn, "甲");
        add_installation(&conn, &a, true);
        set_status(&conn, &a, "playing");
        set_play_seconds(&conn, &a, 777);
        add_session(&conn, &a, "a1", now - 120, 1_800);
        conn.execute(
            "UPDATE play_sessions SET ended_at = ?1, last_seen_at = ?1 WHERE id = 'p-a1'",
            rusqlite::params![now],
        )
        .unwrap();

        let items = recent(&conn, 1).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].total_seconds, 777);
    }

    fn set_tags(conn: &Connection, game_id: &str, json: &str) {
        let tags: Vec<MetadataTag> = serde_json::from_str(json).unwrap();
        crate::tagstore::replace_tags(conn, game_id, &tags).unwrap();
    }

    fn set_released(conn: &Connection, game_id: &str, released: &str) {
        conn.execute(
            "INSERT INTO game_metadata (game_id, released) VALUES (?1, ?2)",
            rusqlite::params![game_id, released],
        )
        .unwrap();
    }

    fn set_length(conn: &Connection, game_id: &str, minutes: i64) {
        conn.execute(
            "INSERT INTO game_metadata (game_id, length_minutes) VALUES (?1, ?2)",
            rusqlite::params![game_id, minutes],
        )
        .unwrap();
    }

    #[test]
    fn 词云按频次计数并过滤剧透标签() {
        let conn = db();
        let a = make_game(&conn, "甲");
        set_tags(
            &conn,
            &a,
            r#"[{"id":"g1","name":"恋爱","category":"theme","rating":3.0,"vn_count":10,"spoiler":0,"lie":false},{"id":"g2","name":"战斗","category":"acts","rating":2.0,"vn_count":5,"spoiler":0,"lie":false}]"#,
        );
        let b = make_game(&conn, "乙");
        set_tags(
            &conn,
            &b,
            r#"[{"id":"g1","name":"恋爱","category":"theme","rating":3.0,"vn_count":10,"spoiler":0,"lie":false},{"id":"g3","name":"伏笔回收","category":"theme","rating":1.0,"vn_count":1,"spoiler":2,"lie":false}]"#,
        );

        let cloud = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(cloud.candidates, 1);
        assert_eq!(cloud.excluded, 0);
        assert_eq!(cloud.tags.len(), 1);
        assert_eq!(cloud.tags[0].id, "g1");
        assert_eq!(cloud.tags[0].name, "恋爱");
        assert_eq!(cloud.tags[0].count, 2);
    }

    #[test]
    fn 同名标签按id分成两条不合并() {
        let conn = db();
        let a = make_game(&conn, "甲");
        set_tags(
            &conn,
            &a,
            r#"[{"id":"g96","name":"恋爱","spoiler":0,"lie":false},{"name":"恋爱","spoiler":0,"lie":false}]"#,
        );

        let all = library_tags(&conn, false, SpoilerLevel::ShowAll, &[]).unwrap();
        let loves: Vec<_> = all.iter().filter(|t| t.name == "恋爱").collect();
        assert_eq!(loves.len(), 2, "{all:?}");
        assert_ne!(loves[0].id, loves[1].id);
    }

    #[test]
    fn 词云截断到上限() {
        let conn = db();
        let mut shared = String::new();
        for i in 0..4 {
            if i > 0 {
                shared.push(',');
            }
            shared.push_str(&format!(r#"{{"id":"t{i}","name":"标签{i}","spoiler":0,"lie":false}}"#));
        }
        let a = make_game(&conn, "甲");
        set_tags(&conn, &a, &format!("[{shared}]"));
        let b = make_game(&conn, "乙");
        set_tags(&conn, &b, &format!("[{shared},{{\"id\":\"t9\",\"name\":\"乙独占\",\"spoiler\":0,\"lie\":false}}]"));

        let view = tag_cloud(&conn, 3, false, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(view.candidates, 4);
        assert_eq!(view.tags.len(), 3);
        assert_eq!(tag_cloud(&conn, 0, false, SpoilerLevel::HideAll, &[]).unwrap().tags.len(), 0);
    }

    #[test]
    fn 词云在隐藏r18时排除成人标签() {
        let conn = db();
        for name in ["甲", "乙"] {
            let id = make_game(&conn, name);
            set_tags(
                &conn,
                &id,
                r#"[{"id":"g96","name":"恋爱","spoiler":0,"lie":false},{"id":"g82","name":"女性之间做爱","spoiler":0,"lie":false}]"#,
            );
        }

        let shown = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(shown.tags.len(), 2);
        let hidden = tag_cloud(&conn, 40, true, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(hidden.tags.len(), 1);
        assert_eq!(hidden.tags[0].name, "恋爱");
    }

    #[test]
    fn 词云排除技术类标签() {
        let conn = db();
        let tech = r#"[{"id":"g32","name":"ADV","category":"tech","rating":3.0,"vn_count":10,"spoiler":0,"lie":false},{"id":"g452","name":"Skip Scenes","category":"tech","rating":2.0,"vn_count":5,"spoiler":0,"lie":false},{"id":"g904","name":"Super Deformed CGs","category":"tech","rating":2.0,"vn_count":5,"spoiler":0,"lie":false},{"id":"g2","name":"奇幻","category":"cont","rating":2.0,"vn_count":9,"spoiler":0,"lie":false},{"name":"手输标签","spoiler":0,"lie":false}]"#;
        for name in ["甲", "乙"] {
            let id = make_game(&conn, name);
            set_tags(&conn, &id, tech);
        }

        let cloud = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(cloud.candidates, 2);
        assert_eq!(cloud.tags.len(), 2);
        assert_eq!(cloud.tags[0].name, "奇幻");
        assert_eq!(cloud.tags[1].name, "手输标签");
    }

    #[test]
    fn 词云按剧透级别过滤() {
        let conn = db();
        let tags = r#"[{"id":"g1","name":"恋爱","category":"cont","rating":3.0,"vn_count":10,"spoiler":0,"lie":false},{"id":"g5","name":"轻微剧透","category":"cont","rating":2.0,"vn_count":5,"spoiler":1,"lie":false},{"id":"g6","name":"严重剧透","category":"cont","rating":2.0,"vn_count":5,"spoiler":2,"lie":false}]"#;
        for name in ["甲", "乙"] {
            let id = make_game(&conn, name);
            set_tags(&conn, &id, tags);
        }

        let all = tag_cloud(&conn, 40, false, SpoilerLevel::ShowAll, &[]).unwrap();
        assert_eq!(all.tags.len(), 3);
        let major = tag_cloud(&conn, 40, false, SpoilerLevel::HideMajor, &[]).unwrap();
        assert_eq!(major.tags.len(), 2);
        assert!(major.tags.iter().all(|t| t.name != "严重剧透"));
        let none = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(none.tags.len(), 1);
        assert_eq!(none.tags[0].name, "恋爱");
    }

    #[test]
    fn 词云只收题材与角色根下的标签() {
        let conn = db();
        let tags = r#"[{"id":"g2","name":"奇幻","category":"cont","rating":3.0,"vn_count":10,"spoiler":0,"lie":false},{"id":"g64","name":"大和抚子女主角","category":"cont","rating":2.0,"vn_count":5,"spoiler":0,"lie":false},{"id":"g145","name":"无分支","category":"cont","rating":1.0,"vn_count":1,"spoiler":0,"lie":false},{"id":"g48","name":"高中","category":"cont","rating":1.0,"vn_count":1,"spoiler":0,"lie":false}]"#;
        for name in ["甲", "乙"] {
            let id = make_game(&conn, name);
            set_tags(&conn, &id, tags);
        }

        let cloud = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(cloud.candidates, 2);
        assert_eq!(
            cloud.tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
            vec!["大和抚子女主角", "奇幻"]
        );
    }

    #[test]
    fn 词云排除池按id命中() {
        let conn = db();
        let tags = r#"[{"id":"g96","name":"恋爱","category":"cont","rating":3.0,"vn_count":10,"spoiler":0,"lie":false},{"id":"g64","name":"大和抚子女主角","category":"cont","rating":2.0,"vn_count":5,"spoiler":0,"lie":false},{"name":"手输备注","spoiler":0,"lie":false}]"#;
        for name in ["甲", "乙"] {
            let id = make_game(&conn, name);
            set_tags(&conn, &id, tags);
        }

        let pool = vec!["g96".to_owned(), "u1".to_owned()];
        let cloud = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &pool).unwrap();
        assert_eq!(cloud.candidates, 3);
        assert_eq!(cloud.excluded, 2);
        assert_eq!(cloud.tags.len(), 1);
        assert_eq!(cloud.tags[0].name, "大和抚子女主角");

        let by_name = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &["恋爱".to_owned()]).unwrap();
        assert_eq!(by_name.excluded, 0);
        assert_eq!(by_name.tags.len(), 3);
    }

    #[test]
    fn 库内标签查询返回全部标签与次数() {
        let conn = db();
        let a = make_game(&conn, "甲");
        set_tags(
            &conn,
            &a,
            r#"[{"id":"g2","name":"奇幻","spoiler":2,"lie":false},{"name":"支线","spoiler":0,"lie":false},{"name":"手输标签","spoiler":0,"lie":false}]"#,
        );
        let b = make_game(&conn, "乙");
        set_tags(&conn, &b, r#"[{"id":"g2","name":"奇幻","spoiler":0,"lie":false}]"#);

        let all = library_tags(&conn, false, SpoilerLevel::ShowAll, &[]).unwrap();
        assert_eq!(
            all.iter().map(|t| (t.name.as_str(), t.count)).collect::<Vec<_>>(),
            vec![("奇幻", 2), ("手输标签", 1), ("支线", 1)]
        );
        assert_eq!(all[0].id, "g2");
        assert!(all[1].id.starts_with('u'));
        assert!(all[2].id.starts_with('u'));
    }

    #[test]
    fn 库内标签查询遵循排除配置() {
        let conn = db();
        let id = make_game(&conn, "甲");
        set_tags(
            &conn,
            &id,
            r#"[{"id":"g2","name":"奇幻","spoiler":0,"lie":false},{"id":"g32","name":"ADV","spoiler":0,"lie":false},{"id":"g96","name":"恋爱","spoiler":0,"lie":false}]"#,
        );
        let out = library_tags(&conn, false, SpoilerLevel::ShowAll, &["g96".to_owned()]).unwrap();
        let got: Vec<&str> = out.iter().map(|t| t.name.as_str()).collect();
        assert!(got.contains(&"奇幻"), "正常标签应保留: {:?}", got);
        assert!(!got.contains(&"ADV"), "分组排除标签应隐藏: {:?}", got);
        assert!(!got.contains(&"恋爱"), "排除池标签应隐藏: {:?}", got);
    }

    #[test]
    fn 无标签时词云为空() {
        let conn = db();
        make_game(&conn, "甲");
        assert!(
            tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &[])
                .unwrap()
                .tags
                .is_empty()
        );
    }

    #[test]
    fn 活跃度按粒度聚合补齐空桶() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let now = ts(&conn, "2026-03-15", "12:00:00");
        add_session(&conn, &a, "x1", ts(&conn, "2026-03-02", "20:00:00"), 3600);
        add_session(&conn, &a, "x2", ts(&conn, "2026-02-10", "20:00:00"), 1800);
        add_session(&conn, &a, "x3", ts(&conn, "2026-01-05", "20:00:00"), 600);

        let months = activity(&conn, now, 12, Granularity::Month, TimeMetric::Foreground).unwrap();
        assert_eq!(months.len(), 12);
        assert_eq!(months[0].bucket, "2025-04-01");
        assert_eq!(months[11].bucket, "2026-03-01");
        assert_eq!(months[11].seconds, 3600);
        assert_eq!(months[10].seconds, 1800);
        assert_eq!(months[9].seconds, 600);
        assert_eq!(months.iter().map(|p| p.seconds).sum::<i64>(), 6000);

        let years = activity(&conn, now, 12, Granularity::Year, TimeMetric::Foreground).unwrap();
        assert_eq!(years.len(), 12);
        assert_eq!(years[0].bucket, "2015-01-01");
        assert_eq!(years[11].bucket, "2026-01-01");
        assert_eq!(years[11].seconds, 6000);
    }

    #[test]
    fn 周档按周一对齐跨年同周不拆桶() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let now = ts(&conn, "2026-01-08", "12:00:00");
        add_session(&conn, &a, "w1", ts(&conn, "2025-12-31", "22:00:00"), 600);
        add_session(&conn, &a, "w2", ts(&conn, "2026-01-01", "02:00:00"), 1200);

        let weeks = activity(&conn, now, 4, Granularity::Week, TimeMetric::Foreground).unwrap();
        assert_eq!(weeks.len(), 4);
        assert_eq!(weeks[0].bucket, "2025-12-15");
        assert_eq!(weeks[3].bucket, "2026-01-05");
        assert_eq!(weeks[2].bucket, "2025-12-29");
        assert_eq!(weeks[2].seconds, 1800);
    }

    #[test]
    fn 活跃度按口径取对应列() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let now = ts(&conn, "2026-03-15", "12:00:00");
        let started = ts(&conn, "2026-03-02", "20:00:00");
        conn.execute(
            "INSERT INTO play_sessions (id, game_id, started_at, last_seen_at, ended_at, foreground_seconds)
             VALUES ('p-x1', ?1, ?2, ?3, ?3, 900)",
            rusqlite::params![a, started, started + 2400],
        )
        .unwrap();

        let foreground = activity(&conn, now, 1, Granularity::Month, TimeMetric::Foreground).unwrap();
        assert_eq!(foreground[0].seconds, 900);
        let alive = activity(&conn, now, 1, Granularity::Month, TimeMetric::Alive).unwrap();
        assert_eq!(alive[0].seconds, 2400);
    }

    #[test]
    fn 游戏趋势按月分桶并按时长取前几名() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        let c = make_game(&conn, "丙");
        let now = ts(&conn, "2026-03-15", "12:00:00");
        add_session(&conn, &a, "a1", ts(&conn, "2026-03-02", "20:00:00"), 3600);
        add_session(&conn, &a, "a2", ts(&conn, "2026-01-10", "20:00:00"), 1800);
        add_session(&conn, &b, "b1", ts(&conn, "2026-02-20", "20:00:00"), 600);
        add_session(&conn, &c, "c1", ts(&conn, "2026-03-05", "20:00:00"), 120);

        let trend = game_trend(&conn, now, 12, 2, TimeMetric::Foreground).unwrap();
        assert_eq!(trend.buckets.len(), 12);
        assert_eq!(trend.buckets[0], "2025-04-01");
        assert_eq!(trend.buckets[11], "2026-03-01");
        assert_eq!(trend.series.len(), 2);
        assert_eq!(trend.series[0].game_id, a);
        assert_eq!(trend.series[0].total, 5400);
        assert_eq!(trend.series[0].values[9], 1800);
        assert_eq!(trend.series[0].values[11], 3600);
        assert_eq!(trend.series[1].game_id, b);
    }

    #[test]
    fn 通关时长对照只取标称与实际都有值的作品() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        let c = make_game(&conn, "丙");
        let d = make_game(&conn, "丁");
        set_length(&conn, &a, 600);
        set_play_seconds(&conn, &a, 36000);
        set_length(&conn, &b, 600);
        set_play_seconds(&conn, &b, 0);
        set_length(&conn, &c, 0);
        set_play_seconds(&conn, &c, 7200);
        set_play_seconds(&conn, &d, 1800);

        let points = length_compare(&conn).unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].id, a);
        assert_eq!(points[0].expected, 36000);
        assert_eq!(points[0].actual, 36000);
    }

    #[test]
    fn 通关时长按档位分桶并补零() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        let c = make_game(&conn, "丙");
        let d = make_game(&conn, "丁");
        make_game(&conn, "戊");
        set_length(&conn, &a, 300);
        set_length(&conn, &b, 720);
        set_length(&conn, &c, 1500);
        set_length(&conn, &d, 3000);

        let buckets = length_buckets(&conn).unwrap();
        assert_eq!(
            buckets.iter().map(|b| b.key.as_str()).collect::<Vec<_>>(),
            vec!["b1", "b2", "b3", "b4", "b5"]
        );
        assert_eq!(
            buckets.iter().map(|b| b.count).collect::<Vec<_>>(),
            vec![1, 1, 1, 0, 1]
        );
    }

    #[test]
    fn 时长排行按时长降序且不含零时长() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        make_game(&conn, "丙");
        set_play_seconds(&conn, &a, 7200);
        set_play_seconds(&conn, &b, 3600);

        let top = playtime_top(&conn, 6).unwrap();
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].id, a);
        assert_eq!(top[0].seconds, 7200);
        assert_eq!(top[1].id, b);
        assert_eq!(top[1].seconds, 3600);
        assert_eq!(playtime_top(&conn, 1).unwrap().len(), 1);
        assert_eq!(playtime_top(&conn, 0).unwrap().len(), 0);
    }

    #[test]
    fn 发行年代按十年分桶并补零() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        let c = make_game(&conn, "丙");
        let d = make_game(&conn, "丁");
        let e = make_game(&conn, "戊");
        set_released(&conn, &a, "1998-06-12");
        set_released(&conn, &b, "2004-01-01");
        set_released(&conn, &c, "2016-11-30");
        set_released(&conn, &d, "2024-05-24");
        set_released(&conn, &e, "2026-01-09");

        let eras = release_eras(&conn, None).unwrap();
        assert_eq!(
            eras.iter().map(|e| e.key.as_str()).collect::<Vec<_>>(),
            vec!["pre2000", "y2000", "y2010", "y2020"]
        );
        assert_eq!(
            eras.iter().map(|e| e.count).collect::<Vec<_>>(),
            vec![1, 1, 1, 2]
        );
    }

    #[test]
    fn 发行年代过滤无效年份() {
        let conn = db();
        let a = make_game(&conn, "甲");
        set_released(&conn, &a, "abcd");
        let eras = release_eras(&conn, None).unwrap();
        assert_eq!(eras.iter().map(|e| e.count).sum::<i64>(), 0);
    }

    #[test]
    fn 发行年代可按状态筛选() {
        let conn = db();
        let a = make_game(&conn, "甲");
        let b = make_game(&conn, "乙");
        let c = make_game(&conn, "丙");
        set_released(&conn, &a, "1998-06-12");
        set_released(&conn, &b, "2016-11-30");
        set_released(&conn, &c, "2024-05-24");
        set_status(&conn, &a, "finished");
        set_status(&conn, &b, "finished");
        set_status(&conn, &c, "dropped");

        let all = release_eras(&conn, None).unwrap();
        assert_eq!(all.iter().map(|e| e.count).collect::<Vec<_>>(), vec![1, 0, 1, 1]);

        let finished = release_eras(&conn, Some("finished")).unwrap();
        assert_eq!(
            finished.iter().map(|e| e.count).collect::<Vec<_>>(),
            vec![1, 0, 1, 0]
        );

        let dropped = release_eras(&conn, Some("dropped")).unwrap();
        assert_eq!(
            dropped.iter().map(|e| e.count).collect::<Vec<_>>(),
            vec![0, 0, 0, 1]
        );
    }

    fn add_character_traits(conn: &Connection, game: &str, id: &str, traits: &str) {
        conn.execute(
            "INSERT INTO characters (game_id, id, name) VALUES (?1, ?2, ?2)",
            rusqlite::params![game, id],
        )
        .unwrap();
        let Ok(parsed) = serde_json::from_str::<Vec<CharacterTrait>>(traits) else {
            return;
        };
        crate::tagstore::replace_traits(conn, game, id, &parsed).unwrap();
    }

    #[test]
    fn 库内特征按出现次数聚合() {
        let conn = db();
        let game = make_game(&conn, "测试");
        add_character_traits(
            &conn,
            &game,
            "c1",
            r#"[{"id":"i4","name":"黑色","group_name":"头发"},{"id":"i2","name":"长发","group_name":"头发"}]"#,
        );
        add_character_traits(
            &conn,
            &game,
            "c2",
            r#"[{"id":"i4","name":"黑色","group_name":"头发"},{"name":"自定义"}]"#,
        );

        let out = library_traits(&conn).unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(
            out.iter().map(|t| (t.name.as_str(), t.count)).collect::<Vec<_>>(),
            vec![("黑色", 2), ("自定义", 1), ("长发", 1)]
        );
        assert_eq!(out[0].group.as_deref(), Some("头发"));
        assert_eq!(out[0].id, "i4");
        assert!(out[1].id.starts_with('u'));
    }

    #[test]
    fn 库内特征跳过无法解析的条目() {
        let conn = db();
        let game = make_game(&conn, "测试");
        add_character_traits(&conn, &game, "c1", "坏数据");
        add_character_traits(&conn, &game, "c2", r#"[{"name":""}]"#);
        assert!(library_traits(&conn).unwrap().is_empty());
    }
}

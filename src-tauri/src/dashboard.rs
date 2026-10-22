use std::collections::HashMap;

use rusqlite::Connection;
use serde::Serialize;

use crate::error::Result;
use crate::settings::{SpoilerLevel, TimeMetric};
use crate::tagdict::{self, TagScope};
use crate::types::MetadataTag;

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
    let metric_text = metric_text(metric);
    let rows = stmt.query_map(rusqlite::params![limit, metric_text], |row| {
        Ok(RecentPlay {
            id: row.get("id")?,
            name: row.get("name")?,
            cover_path: row.get("cover_path")?,
            last_played_at: row.get("last_played_at")?,
            play_seconds: row.get("play_seconds")?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TagCount {
    pub id: Option<String>,
    pub tag: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LibraryTag {
    pub id: Option<String>,
    pub tag: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TagCloudView {
    pub tags: Vec<TagCount>,
    pub candidates: i64,
    pub excluded: i64,
}

struct TagAgg {
    id: Option<String>,
    name: String,
    count: i64,
}

const CLOUD_MIN_COUNT: i64 = 2;

pub fn library_tags(conn: &Connection) -> Result<Vec<LibraryTag>> {
    let mut out: Vec<LibraryTag> = count_tags(conn, |_| true)?
        .into_values()
        .map(|a| LibraryTag {
            id: a.id,
            tag: a.name,
            count: a.count,
        })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.tag.cmp(&b.tag)));
    Ok(out)
}

pub fn tag_cloud(
    conn: &Connection,
    limit: i64,
    hide_r18_tags: bool,
    spoiler_level: SpoilerLevel,
    excluded: &[String],
) -> Result<TagCloudView> {
    let groups = tagdict::dictionary().tag_groups;
    let counts = count_tags(conn, |tag| {
        let hidden = tag.lie.unwrap_or(false)
            || spoiler_level.hides(tag.spoiler.unwrap_or(0))
            || (hide_r18_tags && tag.category.as_deref() == Some("ero"))
            || groups.scope(tag.id.as_deref()) != TagScope::Full;
        !hidden
    })?;
    let mut pool: Vec<TagAgg> = counts
        .into_values()
        .filter(|a| a.count >= CLOUD_MIN_COUNT)
        .collect();
    pool.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    let candidates = pool.len() as i64;
    let (hidden, visible): (Vec<TagAgg>, Vec<TagAgg>) = pool
        .into_iter()
        .partition(|a| is_excluded(a.id.as_deref(), &a.name, excluded));
    let mut out: Vec<TagCount> = visible
        .into_iter()
        .map(|a| TagCount {
            id: a.id,
            tag: a.name,
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

fn count_tags<F>(conn: &Connection, keep: F) -> Result<HashMap<String, TagAgg>>
where
    F: Fn(&MetadataTag) -> bool,
{
    let mut counts: HashMap<String, TagAgg> = HashMap::new();
    for tags in iter_metadata_tags(conn)? {
        for tag in tags {
            if !keep(&tag) {
                continue;
            }
            let entry = counts.entry(tag.name.clone()).or_insert_with(|| TagAgg {
                id: None,
                name: tag.name.clone(),
                count: 0,
            });
            if entry.id.is_none() {
                entry.id = tag.id.clone();
            }
            entry.count += 1;
        }
    }
    Ok(counts)
}

fn iter_metadata_tags(conn: &Connection) -> Result<Vec<Vec<MetadataTag>>> {
    let mut stmt = conn.prepare("SELECT tags FROM game_metadata WHERE tags IS NOT NULL")?;
    let rows = stmt.query_map([], |row| row.get::<_, Option<String>>(0))?;
    let mut out = Vec::new();
    for row in rows {
        let Some(text) = row? else { continue };
        if let Ok(tags) = serde_json::from_str::<Vec<MetadataTag>>(&text) {
            out.push(tags);
        }
    }
    Ok(out)
}

fn is_excluded(id: Option<&str>, name: &str, pool: &[String]) -> bool {
    match id {
        Some(id) => pool.iter().any(|entry| entry == id),
        None => pool.iter().any(|entry| entry == name),
    }
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

pub fn release_eras(conn: &Connection) -> Result<Vec<EraPoint>> {
    let mut stmt = conn.prepare(
        "SELECT CASE
            WHEN CAST(substr(m.released, 1, 4) AS INTEGER) < 2000 THEN 'pre2000'
            WHEN CAST(substr(m.released, 1, 4) AS INTEGER) < 2010 THEN 'y2000'
            WHEN CAST(substr(m.released, 1, 4) AS INTEGER) < 2020 THEN 'y2010'
            ELSE 'y2020'
         END AS era, COUNT(*) AS count
         FROM game_metadata m
         WHERE m.released IS NOT NULL
           AND length(m.released) >= 4
           AND CAST(substr(m.released, 1, 4) AS INTEGER) BETWEEN 1900 AND 2100
         GROUP BY era",
    )?;
    let rows = stmt.query_map([], |row| {
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
    pub month: String,
    pub seconds: i64,
}

pub fn monthly_activity(conn: &Connection, months: i64) -> Result<Vec<ActivityPoint>> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE seq(ym) AS (
            SELECT strftime('%Y-%m', ?1, 'unixepoch')
            UNION ALL
            SELECT strftime('%Y-%m', date(ym || '-01', '-1 month')) FROM seq LIMIT ?2
         )
         SELECT seq.ym AS ym, COALESCE(SUM(p.foreground_seconds), 0) AS s
         FROM seq
         LEFT JOIN play_sessions p ON strftime('%Y-%m', p.started_at, 'unixepoch') = seq.ym
         GROUP BY seq.ym
         ORDER BY seq.ym",
    )?;
    let rows = stmt.query_map(rusqlite::params![crate::now(), months.max(1)], |row| {
        Ok(ActivityPoint {
            month: row.get("ym")?,
            seconds: row.get("s")?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
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
        let a = make_game(&conn, "甲");
        add_installation(&conn, &a, true);
        set_status(&conn, &a, "playing");
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

    fn set_tags(conn: &Connection, game_id: &str, json: &str) {
        conn.execute(
            "INSERT INTO game_metadata (game_id, tags) VALUES (?1, ?2)",
            rusqlite::params![game_id, json],
        )
        .unwrap();
    }

    fn set_released(conn: &Connection, game_id: &str, released: &str) {
        conn.execute(
            "INSERT INTO game_metadata (game_id, released) VALUES (?1, ?2)",
            rusqlite::params![game_id, released],
        )
        .unwrap();
    }

    fn month_index(ym: &str) -> i64 {
        let y: i64 = ym[0..4].parse().unwrap();
        let m: i64 = ym[5..7].parse().unwrap();
        y * 12 + m
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
        assert_eq!(cloud.tags[0].id.as_deref(), Some("g1"));
        assert_eq!(cloud.tags[0].tag, "恋爱");
        assert_eq!(cloud.tags[0].count, 2);
    }

    #[test]
    fn 词云合并手输与同步的同名标签() {
        let conn = db();
        let a = make_game(&conn, "甲");
        set_tags(
            &conn,
            &a,
            r#"[{"id":"g1","name":"恋爱","spoiler":0,"lie":false},{"name":"手输独占","spoiler":0,"lie":false}]"#,
        );
        let b = make_game(&conn, "乙");
        set_tags(
            &conn,
            &b,
            r#"[{"name":"恋爱","spoiler":0,"lie":false},{"name":"手输独占","spoiler":0,"lie":false}]"#,
        );

        let cloud = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(cloud.tags.len(), 2);
        assert_eq!(cloud.tags[0].tag, "恋爱");
        assert_eq!(cloud.tags[0].id.as_deref(), Some("g1"));
        assert_eq!(cloud.tags[0].count, 2);
        assert_eq!(cloud.tags[1].tag, "手输独占");
        assert_eq!(cloud.tags[1].id, None);
        assert_eq!(cloud.tags[1].count, 2);
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
                r#"[{"id":"g1","name":"恋爱","category":"cont","rating":3.0,"vn_count":10,"spoiler":0,"lie":false},{"id":"g4","name":"性描写","category":"ero","rating":1.0,"vn_count":1,"spoiler":0,"lie":false}]"#,
            );
        }

        let shown = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(shown.tags.len(), 2);
        let hidden = tag_cloud(&conn, 40, true, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(hidden.tags.len(), 1);
        assert_eq!(hidden.tags[0].tag, "恋爱");
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
        assert_eq!(cloud.tags[0].tag, "奇幻");
        assert_eq!(cloud.tags[1].tag, "手输标签");
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
        assert!(major.tags.iter().all(|t| t.tag != "严重剧透"));
        let none = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &[]).unwrap();
        assert_eq!(none.tags.len(), 1);
        assert_eq!(none.tags[0].tag, "恋爱");
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
            cloud.tags.iter().map(|t| t.tag.as_str()).collect::<Vec<_>>(),
            vec!["大和抚子女主角", "奇幻"]
        );
    }

    #[test]
    fn 词云排除池按id与手输名命中() {
        let conn = db();
        let tags = r#"[{"id":"g96","name":"恋爱","category":"cont","rating":3.0,"vn_count":10,"spoiler":0,"lie":false},{"id":"g64","name":"大和抚子女主角","category":"cont","rating":2.0,"vn_count":5,"spoiler":0,"lie":false},{"name":"手输备注","spoiler":0,"lie":false}]"#;
        for name in ["甲", "乙"] {
            let id = make_game(&conn, name);
            set_tags(&conn, &id, tags);
        }

        let pool = vec!["g96".to_owned(), "手输备注".to_owned()];
        let cloud = tag_cloud(&conn, 40, false, SpoilerLevel::HideAll, &pool).unwrap();
        assert_eq!(cloud.candidates, 3);
        assert_eq!(cloud.excluded, 2);
        assert_eq!(cloud.tags.len(), 1);
        assert_eq!(cloud.tags[0].tag, "大和抚子女主角");

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
            r#"[{"id":"g2","name":"奇幻","spoiler":2,"lie":false},{"id":"g145","name":"无分支","spoiler":0,"lie":false},{"name":"手输标签","spoiler":0,"lie":false}]"#,
        );
        let b = make_game(&conn, "乙");
        set_tags(&conn, &b, r#"[{"id":"g2","name":"奇幻","spoiler":0,"lie":false}]"#);

        let all = library_tags(&conn).unwrap();
        assert_eq!(
            all.iter().map(|t| (t.tag.as_str(), t.count)).collect::<Vec<_>>(),
            vec![("奇幻", 2), ("手输标签", 1), ("无分支", 1)]
        );
        assert_eq!(all[0].id.as_deref(), Some("g2"));
        assert_eq!(all[1].id, None);
        assert_eq!(all[2].id.as_deref(), Some("g145"));
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
    fn 月度活跃按月份聚合前景时间并补齐空月() {
        let conn = db();
        let now = crate::now();
        let a = make_game(&conn, "甲");
        add_session(&conn, &a, "x1", now - 30 * 86400, 3600);
        add_session(&conn, &a, "x2", now - 60 * 86400, 1800);

        let points = monthly_activity(&conn, 12).unwrap();
        assert_eq!(points.len(), 12);
        let total: i64 = points.iter().map(|p| p.seconds).sum();
        assert_eq!(total, 5400);
        assert_eq!(month_index(&points[11].month) - month_index(&points[0].month), 11);
        assert_eq!(monthly_activity(&conn, 1).unwrap().len(), 1);
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

        let eras = release_eras(&conn).unwrap();
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
        let eras = release_eras(&conn).unwrap();
        assert_eq!(eras.iter().map(|e| e.count).sum::<i64>(), 0);
    }
}

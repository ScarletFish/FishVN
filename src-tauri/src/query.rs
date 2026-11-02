use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::games;
use crate::types::{GameListItem, MetadataTag, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuerySubject {
    Game,
    Character,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagMatchMode {
    Any,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricSource {
    Vndb,
    User,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QueryRow {
    TagInclude {
        mode: TagMatchMode,
        #[serde(default)]
        tags: Vec<String>,
    },
    TagExclude {
        #[serde(default)]
        tags: Vec<String>,
    },
    Status {
        #[serde(default)]
        values: Vec<Status>,
    },
    Rating {
        source: MetricSource,
        min: Option<f64>,
        max: Option<f64>,
    },
    Released {
        from: Option<String>,
        to: Option<String>,
    },
    Developer {
        name: String,
    },
    Length {
        source: MetricSource,
        min: Option<i64>,
        max: Option<i64>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySpec {
    pub subject: QuerySubject,
    #[serde(default)]
    pub rows: Vec<QueryRow>,
}

struct GameFacts {
    tags: Vec<MetadataTag>,
    developers: Vec<String>,
    released: Option<String>,
    vndb_rating: Option<f64>,
    length_minutes: Option<i64>,
    status: Status,
    user_rating: Option<f64>,
    play_seconds: i64,
}

fn load_facts(conn: &Connection) -> Result<HashMap<String, GameFacts>> {
    let mut stmt = conn.prepare(
        "SELECT s.game_id, m.tags, m.developers, m.released, m.rating, m.length_minutes,
                s.status, s.user_rating, s.play_seconds
         FROM user_game_state s
         LEFT JOIN game_metadata m ON m.game_id = s.game_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<f64>>(4)?,
            row.get::<_, Option<i64>>(5)?,
            row.get::<_, Status>(6)?,
            row.get::<_, Option<f64>>(7)?,
            row.get::<_, i64>(8)?,
        ))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (
            game_id,
            tags,
            developers,
            released,
            vndb_rating,
            length_minutes,
            status,
            user_rating,
            play_seconds,
        ) = row?;
        let tags = tags
            .and_then(|raw| serde_json::from_str::<Vec<MetadataTag>>(&raw).ok())
            .unwrap_or_default();
        let developers = developers
            .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
            .unwrap_or_default();
        map.insert(
            game_id,
            GameFacts {
                tags,
                developers,
                released,
                vndb_rating,
                length_minutes,
                status,
                user_rating,
                play_seconds,
            },
        );
    }
    Ok(map)
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..].iter().all(u8::is_ascii_digit)
}

fn tag_names(facts: &GameFacts) -> HashSet<&str> {
    facts.tags.iter().map(|t| t.name.as_str()).collect()
}

fn row_matches(facts: &GameFacts, row: &QueryRow) -> bool {
    match row {
        QueryRow::TagInclude { mode, tags } => {
            if tags.is_empty() {
                return true;
            }
            let names = tag_names(facts);
            match mode {
                TagMatchMode::Any => tags.iter().any(|t| names.contains(t.as_str())),
                TagMatchMode::All => tags.iter().all(|t| names.contains(t.as_str())),
            }
        }
        QueryRow::TagExclude { tags } => {
            if tags.is_empty() {
                return true;
            }
            let names = tag_names(facts);
            !tags.iter().any(|t| names.contains(t.as_str()))
        }
        QueryRow::Status { values } => values.is_empty() || values.contains(&facts.status),
        QueryRow::Rating { source, min, max } => {
            let value = match source {
                MetricSource::Vndb => facts.vndb_rating,
                MetricSource::User => facts.user_rating,
            };
            let Some(value) = value else {
                return false;
            };
            min.is_none_or(|m| value >= m) && max.is_none_or(|m| value <= m)
        }
        QueryRow::Released { from, to } => {
            if from.is_none() && to.is_none() {
                return true;
            }
            let Some(released) = facts.released.as_deref() else {
                return false;
            };
            if !is_iso_date(released) {
                return false;
            }
            from.as_deref().is_none_or(|f| released >= f)
                && to.as_deref().is_none_or(|t| released <= t)
        }
        QueryRow::Developer { name } => {
            let needle = name.trim().to_lowercase();
            needle.is_empty()
                || facts
                    .developers
                    .iter()
                    .any(|d| d.to_lowercase().contains(&needle))
        }
        QueryRow::Length { source, min, max } => {
            let minutes = match source {
                MetricSource::Vndb => facts.length_minutes,
                MetricSource::User => Some(facts.play_seconds / 60),
            };
            let Some(minutes) = minutes else {
                return false;
            };
            min.is_none_or(|m| minutes >= m) && max.is_none_or(|m| minutes <= m)
        }
    }
}

pub fn query_games(conn: &Connection, spec: QuerySpec) -> Result<Vec<GameListItem>> {
    if spec.subject == QuerySubject::Character {
        return Err(AppError::BadRequest("角色级查询尚未实现".into()));
    }
    let items = games::list_games(conn)?;
    if spec.rows.is_empty() {
        return Ok(items);
    }
    let facts = load_facts(conn)?;
    Ok(items
        .into_iter()
        .filter(|item| {
            facts
                .get(&item.id)
                .is_some_and(|f| spec.rows.iter().all(|row| row_matches(f, row)))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_schema;
    use rusqlite::params;

    fn tag(id: Option<&str>, name: &str) -> MetadataTag {
        MetadataTag {
            id: id.map(String::from),
            name: String::from(name),
            ..Default::default()
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_game(
        conn: &Connection,
        id: &str,
        status: &str,
        user_rating: Option<f64>,
        play_seconds: i64,
        tags: &[MetadataTag],
        developers: &[&str],
        released: Option<&str>,
        vndb_rating: Option<f64>,
        length_minutes: Option<i64>,
    ) {
        conn.execute(
            "INSERT INTO games (id, name, created_at) VALUES (?1, ?2, 0)",
            params![id, id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO user_game_state (game_id, status, user_rating, play_seconds)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, status, user_rating, play_seconds],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO game_metadata (game_id, tags, developers, released, rating, length_minutes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                serde_json::to_string(tags).unwrap(),
                serde_json::to_string(developers).unwrap(),
                released,
                vndb_rating,
                length_minutes
            ],
        )
        .unwrap();
    }

    fn spec(rows: Vec<QueryRow>) -> QuerySpec {
        QuerySpec {
            subject: QuerySubject::Game,
            rows,
        }
    }

    fn ids(conn: &Connection, s: QuerySpec) -> Vec<String> {
        let mut out = query_games(conn, s)
            .unwrap()
            .into_iter()
            .map(|i| i.id)
            .collect::<Vec<_>>();
        out.sort();
        out
    }

    fn fixture() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        add_game(
            &conn,
            "g1",
            "playing",
            Some(80.0),
            3600,
            &[
                tag(Some("g100"), "泣きゲー"),
                tag(None, "自制收藏"),
            ],
            &["Key"],
            Some("2016-04-28"),
            Some(8.0),
            Some(120),
        );
        add_game(
            &conn,
            "g2",
            "finished",
            None,
            7200,
            &[tag(Some("g200"), "百合")],
            &["key production"],
            Some("TBA"),
            Some(6.5),
            None,
        );
        add_game(
            &conn,
            "g3",
            "wishlist",
            Some(50.0),
            0,
            &[tag(Some("g100"), "泣きゲー"), tag(Some("g200"), "百合")],
            &[],
            Some("2020-01-01"),
            None,
            Some(30),
        );
        conn
    }

    #[test]
    fn tag_include_any_matches_either() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::TagInclude {
                mode: TagMatchMode::Any,
                tags: vec!["泣きゲー".into(), "自制收藏".into()],
            }]),
        );
        assert_eq!(got, vec!["g1", "g3"]);
    }

    #[test]
    fn tag_include_all_requires_every_tag() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::TagInclude {
                mode: TagMatchMode::All,
                tags: vec!["泣きゲー".into(), "百合".into()],
            }]),
        );
        assert_eq!(got, vec!["g3"]);
    }

    #[test]
    fn tag_exclude_removes_hit() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::TagExclude {
                tags: vec!["百合".into()],
            }]),
        );
        assert_eq!(got, vec!["g1"]);
    }

    #[test]
    fn custom_tag_matches_by_name() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::TagInclude {
                mode: TagMatchMode::Any,
                tags: vec!["自制收藏".into()],
            }]),
        );
        assert_eq!(got, vec!["g1"]);
    }

    #[test]
    fn status_row_filters() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::Status {
                values: vec![Status::Playing, Status::Finished],
            }]),
        );
        assert_eq!(got, vec!["g1", "g2"]);
    }

    #[test]
    fn rating_bounds_follow_source() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::Rating {
                source: MetricSource::Vndb,
                min: Some(7.0),
                max: Some(9.0),
            }]),
        );
        assert_eq!(got, vec!["g1"]);
        let got = ids(
            &conn,
            spec(vec![QueryRow::Rating {
                source: MetricSource::User,
                min: Some(60.0),
                max: None,
            }]),
        );
        assert_eq!(got, vec!["g1"]);
    }

    #[test]
    fn rating_requires_value() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::Rating {
                source: MetricSource::Vndb,
                min: None,
                max: Some(100.0),
            }]),
        );
        assert_eq!(got, vec!["g1", "g2"]);
    }

    #[test]
    fn released_range_excludes_invalid_dates() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::Released {
                from: Some("2016-01-01".into()),
                to: Some("2020-12-31".into()),
            }]),
        );
        assert_eq!(got, vec!["g1", "g3"]);
    }

    #[test]
    fn developer_matches_case_insensitive() {
        let conn = fixture();
        let got = ids(&conn, spec(vec![QueryRow::Developer { name: "KEY".into() }]));
        assert_eq!(got, vec!["g1", "g2"]);
    }

    #[test]
    fn length_follows_source() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::Length {
                source: MetricSource::Vndb,
                min: Some(90),
                max: Some(180),
            }]),
        );
        assert_eq!(got, vec!["g1"]);
        let got = ids(
            &conn,
            spec(vec![QueryRow::Length {
                source: MetricSource::User,
                min: Some(60),
                max: None,
            }]),
        );
        assert_eq!(got, vec!["g1", "g2"]);
    }

    #[test]
    fn empty_spec_returns_all() {
        let conn = fixture();
        assert_eq!(ids(&conn, spec(vec![])).len(), 3);
    }

    #[test]
    fn rows_combine_with_and() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![
                QueryRow::TagInclude {
                    mode: TagMatchMode::Any,
                    tags: vec!["泣きゲー".into()],
                },
                QueryRow::Status {
                    values: vec![Status::Wishlist],
                },
            ]),
        );
        assert_eq!(got, vec!["g3"]);
    }

    #[test]
    fn character_subject_rejected() {
        let conn = fixture();
        let err = query_games(
            &conn,
            QuerySpec {
                subject: QuerySubject::Character,
                rows: vec![],
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}

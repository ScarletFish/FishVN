use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::games;
use crate::tagdict;
use crate::types::{CharacterHit, CharacterTrait, GameListItem, MetadataTag, Status};

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
    TraitInclude {
        mode: TagMatchMode,
        #[serde(default)]
        traits: Vec<String>,
    },
    TraitExclude {
        #[serde(default)]
        traits: Vec<String>,
    },
    Role {
        #[serde(default)]
        values: Vec<String>,
    },
    Gender {
        #[serde(default)]
        values: Vec<String>,
    },
    NameContains {
        #[serde(default)]
        value: String,
    },
    Va {
        #[serde(default)]
        staff_ids: Vec<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySpec {
    pub subject: QuerySubject,
    #[serde(default)]
    pub rows: Vec<QueryRow>,
    #[serde(default)]
    pub spoiler_threshold: Option<i64>,
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
    let dict = tagdict::dictionary();
    let mut tags_by_game: HashMap<String, Vec<MetadataTag>> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT gt.game_id, t.id, t.name, gt.rating, gt.spoiler, gt.lie
               FROM game_tag gt JOIN tags t ON t.id = gt.tag_id
              ORDER BY gt.rowid",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<f64>>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<i64>>(5)?,
            ))
        })?;
        for (game_id, id, name, rating, spoiler, lie) in rows.flatten() {
            let entry = dict.tags.get(&id);
            tags_by_game.entry(game_id).or_default().push(MetadataTag {
                category: entry.and_then(|e| e.category.clone()),
                vn_count: entry.and_then(|e| e.vn_count),
                id: Some(id),
                name,
                rating,
                spoiler,
                lie: lie.map(|v| v != 0),
            });
        }
    }

    let mut stmt = conn.prepare(
        "SELECT s.game_id, m.developers, m.released, m.rating, m.length_minutes,
                s.status, s.user_rating, s.play_seconds
         FROM user_game_state s
         LEFT JOIN game_metadata m ON m.game_id = s.game_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<f64>>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, Status>(5)?,
            row.get::<_, Option<f64>>(6)?,
            row.get::<_, i64>(7)?,
        ))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (
            game_id,
            developers,
            released,
            vndb_rating,
            length_minutes,
            status,
            user_rating,
            play_seconds,
        ) = row?;
        let developers = developers
            .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
            .unwrap_or_default();
        map.insert(
            game_id.clone(),
            GameFacts {
                tags: tags_by_game.remove(&game_id).unwrap_or_default(),
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

fn visible_tag_ids(facts: &GameFacts, threshold: Option<i64>) -> HashSet<&str> {
    facts
        .tags
        .iter()
        .filter(|t| t.spoiler.unwrap_or(0) < threshold.unwrap_or(i64::MAX))
        .filter_map(|t| t.id.as_deref())
        .collect()
}

fn row_matches(facts: &GameFacts, row: &QueryRow, threshold: Option<i64>) -> bool {
    match row {
        QueryRow::TagInclude { mode, tags } => {
            if tags.is_empty() {
                return true;
            }
            let ids = visible_tag_ids(facts, threshold);
            match mode {
                TagMatchMode::Any => tags.iter().any(|t| ids.contains(t.as_str())),
                TagMatchMode::All => tags.iter().all(|t| ids.contains(t.as_str())),
            }
        }
        QueryRow::TagExclude { tags } => {
            if tags.is_empty() {
                return true;
            }
            let ids = visible_tag_ids(facts, threshold);
            !tags.iter().any(|t| ids.contains(t.as_str()))
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
        QueryRow::TraitInclude { .. }
        | QueryRow::TraitExclude { .. }
        | QueryRow::Role { .. }
        | QueryRow::Gender { .. }
        | QueryRow::NameContains { .. }
        | QueryRow::Va { .. } => true,
    }
}

const CHARACTER_QUERY_COLUMNS: &[&str] = &[
    "game_id",
    "id",
    "name",
    "original",
    "aliases",
    "gender",
    "role",
];

struct CharacterFacts {
    id: String,
    name: Option<String>,
    original: Option<String>,
    aliases: Vec<String>,
    gender: Vec<String>,
    role: Option<String>,
    traits: Vec<CharacterTrait>,
    va_staff: Vec<String>,
}

fn load_character_facts(conn: &Connection) -> Result<HashMap<String, Vec<CharacterFacts>>> {
    let mut traits_by: HashMap<(String, String), Vec<CharacterTrait>> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT ct.game_id, ct.character_id, t.id, t.name, t.grp, ct.spoiler, ct.lie
               FROM character_trait ct JOIN traits t ON t.id = ct.trait_id
              ORDER BY ct.rowid",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<i64>>(6)?,
            ))
        })?;
        for (game_id, character_id, id, name, grp, spoiler, lie) in rows.flatten() {
            traits_by
                .entry((game_id, character_id))
                .or_default()
                .push(CharacterTrait {
                    id: Some(id),
                    name: Some(name),
                    group_name: grp,
                    spoiler,
                    lie: lie.map(|v| v != 0),
                });
        }
    }

    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM characters ORDER BY game_id, sort_order, id",
        CHARACTER_QUERY_COLUMNS.join(", ")
    ))?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
        ))
    })?;
    let mut map: HashMap<String, Vec<CharacterFacts>> = HashMap::new();
    for row in rows {
        let (game_id, id, name, original, aliases, gender, role) = row?;
        let aliases = aliases
            .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
            .unwrap_or_default();
        let gender = gender
            .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
            .unwrap_or_default();
        let traits = traits_by
            .remove(&(game_id.clone(), id.clone()))
            .unwrap_or_default();
        map.entry(game_id).or_default().push(CharacterFacts {
            id,
            name,
            original,
            aliases,
            gender,
            role,
            traits,
            va_staff: Vec::new(),
        });
    }
    let mut stmt = conn.prepare("SELECT game_id, character_id, staff_id FROM character_va")?;
    let va_rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    for row in va_rows {
        let (game_id, character_id, staff_id) = row?;
        if let Some(list) = map.get_mut(&game_id) {
            if let Some(facts) = list.iter_mut().find(|c| c.id == character_id) {
                facts.va_staff.push(staff_id);
            }
        }
    }
    Ok(map)
}

fn is_character_row(row: &QueryRow) -> bool {
    matches!(
        row,
        QueryRow::TraitInclude { .. }
            | QueryRow::TraitExclude { .. }
            | QueryRow::Role { .. }
            | QueryRow::Gender { .. }
            | QueryRow::NameContains { .. }
            | QueryRow::Va { .. }
    )
}

fn partition_rows(rows: Vec<QueryRow>) -> (Vec<QueryRow>, Vec<QueryRow>) {
    let mut game_rows = Vec::new();
    let mut char_rows = Vec::new();
    for row in rows {
        if is_character_row(&row) {
            char_rows.push(row);
        } else {
            game_rows.push(row);
        }
    }
    (game_rows, char_rows)
}

fn trait_matches(trait_: &CharacterTrait, selector: &str) -> bool {
    trait_.id.as_deref() == Some(selector) || trait_.name.as_deref() == Some(selector)
}

fn visible_traits(
    facts: &CharacterFacts,
    threshold: Option<i64>,
) -> impl Iterator<Item = &CharacterTrait> {
    facts
        .traits
        .iter()
        .filter(move |t| t.spoiler.unwrap_or(0) < threshold.unwrap_or(i64::MAX))
}

fn char_row_matches(facts: &CharacterFacts, row: &QueryRow, threshold: Option<i64>) -> bool {
    match row {
        QueryRow::TraitInclude { mode, traits } => {
            if traits.is_empty() {
                return true;
            }
            match mode {
                TagMatchMode::Any => traits
                    .iter()
                    .any(|sel| visible_traits(facts, threshold).any(|t| trait_matches(t, sel))),
                TagMatchMode::All => traits
                    .iter()
                    .all(|sel| visible_traits(facts, threshold).any(|t| trait_matches(t, sel))),
            }
        }
        QueryRow::TraitExclude { traits } => {
            if traits.is_empty() {
                return true;
            }
            !traits
                .iter()
                .any(|sel| visible_traits(facts, threshold).any(|t| trait_matches(t, sel)))
        }
        QueryRow::Role { values } => {
            values.is_empty()
                || facts
                    .role
                    .as_deref()
                    .is_some_and(|r| values.iter().any(|v| v == r))
        }
        QueryRow::Gender { values } => {
            values.is_empty() || facts.gender.iter().any(|g| values.contains(g))
        }
        QueryRow::Va { staff_ids } => {
            staff_ids.is_empty() || facts.va_staff.iter().any(|s| staff_ids.contains(s))
        }
        QueryRow::NameContains { value } => {
            let needle = value.trim().to_lowercase();
            needle.is_empty()
                || facts
                    .name
                    .as_deref()
                    .is_some_and(|n| n.to_lowercase().contains(&needle))
                || facts
                    .original
                    .as_deref()
                    .is_some_and(|n| n.to_lowercase().contains(&needle))
                || facts
                    .aliases
                    .iter()
                    .any(|a| a.to_lowercase().contains(&needle))
        }
        _ => unreachable!("非角色级行不应进入 char_row_matches"),
    }
}

fn game_satisfies_rows(
    facts: Option<&GameFacts>,
    game_rows: &[QueryRow],
    threshold: Option<i64>,
) -> bool {
    if game_rows.is_empty() {
        return true;
    }
    let Some(facts) = facts else {
        return false;
    };
    game_rows.iter().all(|row| row_matches(facts, row, threshold))
}

fn game_has_matching_character(
    chars: &HashMap<String, Vec<CharacterFacts>>,
    game_id: &str,
    char_rows: &[QueryRow],
    threshold: Option<i64>,
) -> bool {
    let Some(list) = chars.get(game_id) else {
        return false;
    };
    list.iter()
        .any(|c| char_rows.iter().all(|row| char_row_matches(c, row, threshold)))
}

pub fn query_games(conn: &Connection, spec: QuerySpec) -> Result<Vec<GameListItem>> {
    if spec.subject != QuerySubject::Game {
        return Err(AppError::BadRequest(
            "subject 与命令不匹配：query_games 需要 subject=game".into(),
        ));
    }
    let threshold = spec.spoiler_threshold;
    let (game_rows, char_rows) = partition_rows(spec.rows);
    let items = games::list_games(conn)?;
    if game_rows.is_empty() && char_rows.is_empty() {
        return Ok(items);
    }
    let facts = if game_rows.is_empty() {
        HashMap::new()
    } else {
        load_facts(conn)?
    };
    let chars = if char_rows.is_empty() {
        HashMap::new()
    } else {
        load_character_facts(conn)?
    };
    Ok(items
        .into_iter()
        .filter(|item| {
            game_satisfies_rows(facts.get(&item.id), &game_rows, threshold)
                && (char_rows.is_empty()
                    || game_has_matching_character(&chars, &item.id, &char_rows, threshold))
        })
        .collect())
}

pub fn query_characters(conn: &Connection, spec: QuerySpec) -> Result<Vec<CharacterHit>> {
    if spec.subject != QuerySubject::Character {
        return Err(AppError::BadRequest(
            "subject 与命令不匹配：query_characters 需要 subject=character".into(),
        ));
    }
    let threshold = spec.spoiler_threshold;
    let (game_rows, char_rows) = partition_rows(spec.rows);
    let items = games::list_games(conn)?;
    let facts = if game_rows.is_empty() {
        HashMap::new()
    } else {
        load_facts(conn)?
    };
    let chars = load_character_facts(conn)?;
    let mut out = Vec::new();
    for game in items {
        if !game_satisfies_rows(facts.get(&game.id), &game_rows, threshold) {
            continue;
        }
        let Some(list) = chars.get(&game.id) else {
            continue;
        };
        for c in list {
            if char_rows.iter().all(|row| char_row_matches(c, row, threshold)) {
                out.push(CharacterHit {
                    game_id: game.id.clone(),
                    game_title: game.name.clone(),
                    character_id: c.id.clone(),
                    name: c.name.clone(),
                    original: c.original.clone(),
                    role: c.role.clone(),
                    gender: if c.gender.is_empty() {
                        None
                    } else {
                        Some(c.gender.clone())
                    },
                    traits: Some(visible_traits(c, threshold).cloned().collect()),
                });
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_schema;
    use rusqlite::params;

    const CHARACTER_FIXTURE_COLUMNS: &[&str] = &[
        "game_id",
        "id",
        "name",
        "original",
        "role",
        "gender",
        "aliases",
    ];

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
            "INSERT INTO game_metadata (game_id, developers, released, rating, length_minutes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                serde_json::to_string(developers).unwrap(),
                released,
                vndb_rating,
                length_minutes
            ],
        )
        .unwrap();
        crate::tagstore::replace_tags(conn, id, tags).unwrap();
    }

    fn spec(rows: Vec<QueryRow>) -> QuerySpec {
        QuerySpec {
            subject: QuerySubject::Game,
            rows,
            spoiler_threshold: None,
        }
    }

    fn char_spec(
        rows: Vec<QueryRow>,
        spoiler_threshold: Option<i64>,
    ) -> QuerySpec {
        QuerySpec {
            subject: QuerySubject::Character,
            rows,
            spoiler_threshold,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_character(
        conn: &Connection,
        game_id: &str,
        id: &str,
        name: Option<&str>,
        original: Option<&str>,
        role: Option<&str>,
        gender: &[&str],
        aliases: &[&str],
        traits: &[CharacterTrait],
    ) {
        let gender: Vec<String> = gender.iter().map(|s| s.to_string()).collect();
        let aliases: Vec<String> = aliases.iter().map(|s| s.to_string()).collect();
        conn.execute(
            &format!(
                "INSERT INTO characters ({}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                CHARACTER_FIXTURE_COLUMNS.join(", ")
            ),
            params![
                game_id,
                id,
                name,
                original,
                role,
                serde_json::to_string(&gender).unwrap(),
                serde_json::to_string(&aliases).unwrap(),
            ],
        )
        .unwrap();
        crate::tagstore::replace_traits(conn, game_id, id, traits).unwrap();
    }

    fn trait_(id: Option<&str>, name: &str, spoiler: Option<i64>) -> CharacterTrait {
        CharacterTrait {
            id: id.map(String::from),
            name: Some(String::from(name)),
            spoiler,
            ..Default::default()
        }
    }

    fn hit_keys(conn: &Connection, s: QuerySpec) -> Vec<String> {
        let mut out = query_characters(conn, s)
            .unwrap()
            .into_iter()
            .map(|h| format!("{}/{}", h.game_id, h.character_id))
            .collect::<Vec<_>>();
        out.sort();
        out
    }

    fn add_va(conn: &Connection, game_id: &str, character_id: &str, staff_id: &str) {
        conn.execute(
            "INSERT OR IGNORE INTO staffs (id, name, original) VALUES (?1, ?2, NULL)",
            params![staff_id, staff_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO character_va (game_id, character_id, staff_id, note)
             VALUES (?1, ?2, ?3, '')",
            params![game_id, character_id, staff_id],
        )
        .unwrap();
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
                tag(Some("u1"), "自制收藏"),
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
        add_character(
            &conn,
            "g1",
            "c1",
            Some("白毛静音"),
            Some("シロガネ"),
            Some("main"),
            &["female"],
            &[],
            &[
                trait_(Some("t100"), "白毛", None),
                trait_(Some("t200"), "傲娇", None),
            ],
        );
        add_character(
            &conn,
            "g1",
            "c2",
            Some("路人甲"),
            None,
            Some("side"),
            &["female"],
            &["モブ"],
            &[
                trait_(Some("t300"), "黑毛", None),
                trait_(Some("t100"), "白毛", Some(2)),
            ],
        );
        add_character(
            &conn,
            "g2",
            "c3",
            Some("学姐"),
            Some("センパイ"),
            Some("main"),
            &["female"],
            &["Senpai"],
            &[trait_(Some("t400"), "百合", Some(1))],
        );
        add_va(&conn, "g1", "c1", "s1");
        add_va(&conn, "g1", "c2", "s2");
        add_va(&conn, "g2", "c3", "s1");
        conn
    }

    #[test]
    fn 角色查询与夹具所用列名都在规格列清单里() {
        let columns = crate::generated::characters::CHARACTER_COLUMNS
            .split(',')
            .map(str::trim)
            .chain(["game_id"])
            .collect::<HashSet<_>>();
        for list in [CHARACTER_QUERY_COLUMNS, CHARACTER_FIXTURE_COLUMNS] {
            for column in list {
                assert!(columns.contains(column), "{column} 不在角色列清单里");
            }
        }
        for column in ["game_id", "sort_order", "id"] {
            assert!(columns.contains(column), "排序列 {column} 不在角色列清单里");
        }
    }

    #[test]
    fn va_row_matches_by_staff_id() {
        let conn = fixture();
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::Va {
                    staff_ids: vec!["s1".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c1", "g2/c3"]);
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::Va {
                    staff_ids: vec!["s2".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c2"]);
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::Va {
                    staff_ids: vec!["s1".into(), "s2".into()],
                }],
                None,
            ),
        );
        assert_eq!(got.len(), 3);
    }

    #[test]
    fn game_subject_va_row_uses_exists_semantics() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::Va {
                staff_ids: vec!["s1".into()],
            }]),
        );
        assert_eq!(got, vec!["g1", "g2"]);
        let got = ids(
            &conn,
            spec(vec![
                QueryRow::Va {
                    staff_ids: vec!["s1".into()],
                },
                QueryRow::Status {
                    values: vec![Status::Playing],
                },
            ]),
        );
        assert_eq!(got, vec!["g1"]);
    }

    #[test]
    fn tag_include_any_matches_either() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::TagInclude {
                mode: TagMatchMode::Any,
                tags: vec!["g100".into(), "u1".into()],
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
                tags: vec!["g100".into(), "g200".into()],
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
                tags: vec!["g200".into()],
            }]),
        );
        assert_eq!(got, vec!["g1"]);
    }

    #[test]
    fn custom_tag_matches_by_id() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::TagInclude {
                mode: TagMatchMode::Any,
                tags: vec!["u1".into()],
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
                    tags: vec!["g100".into()],
                },
                QueryRow::Status {
                    values: vec![Status::Wishlist],
                },
            ]),
        );
        assert_eq!(got, vec!["g3"]);
    }

    #[test]
    fn game_subject_rejected_in_character_query() {
        let conn = fixture();
        let err = query_characters(&conn, spec(vec![])).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn character_subject_rejected() {
        let conn = fixture();
        let err = query_games(
            &conn,
            QuerySpec {
                subject: QuerySubject::Character,
                rows: vec![],
                spoiler_threshold: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn character_subject_empty_rows_returns_all() {
        let conn = fixture();
        let got = hit_keys(&conn, char_spec(vec![], None));
        assert_eq!(got, vec!["g1/c1", "g1/c2", "g2/c3"]);
    }

    #[test]
    fn trait_include_matches_by_id_and_name() {
        let conn = fixture();
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TraitInclude {
                    mode: TagMatchMode::Any,
                    traits: vec!["t100".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c1", "g1/c2"]);
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TraitInclude {
                    mode: TagMatchMode::Any,
                    traits: vec!["黑毛".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c2"]);
    }

    #[test]
    fn trait_include_all_requires_every_trait() {
        let conn = fixture();
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TraitInclude {
                    mode: TagMatchMode::All,
                    traits: vec!["t100".into(), "t200".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c1"]);
    }

    #[test]
    fn trait_exclude_removes_hit() {
        let conn = fixture();
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TraitExclude {
                    traits: vec!["t200".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c2", "g2/c3"]);
    }

    #[test]
    fn spoiler_threshold_filters_traits_on_match_side() {
        let conn = fixture();
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TraitInclude {
                    mode: TagMatchMode::Any,
                    traits: vec!["白毛".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c1", "g1/c2"]);
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TraitInclude {
                    mode: TagMatchMode::Any,
                    traits: vec!["t100".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c1", "g1/c2"]);
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TraitInclude {
                    mode: TagMatchMode::Any,
                    traits: vec!["t100".into()],
                }],
                Some(2),
            ),
        );
        assert_eq!(got, vec!["g1/c1"]);
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TraitExclude {
                    traits: vec!["百合".into()],
                }],
                Some(2),
            ),
        );
        assert_eq!(got, vec!["g1/c1", "g1/c2"]);
    }

    #[test]
    fn spoiler_threshold_filters_game_tags_too() {
        let conn = fixture();
        add_game(
            &conn,
            "g4",
            "playing",
            None,
            0,
            &[MetadataTag {
                id: Some(String::from("u2")),
                name: String::from("剧透神器"),
                spoiler: Some(2),
                ..Default::default()
            }],
            &[],
            None,
            None,
            None,
        );
        add_character(&conn, "g4", "c4", Some("c4"), None, None, &[], &[], &[]);
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TagInclude {
                    mode: TagMatchMode::Any,
                    tags: vec!["u2".into()],
                }],
                Some(2),
            ),
        );
        assert!(got.is_empty());
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::TagInclude {
                    mode: TagMatchMode::Any,
                    tags: vec!["u2".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g4/c4"]);
    }

    #[test]
    fn role_and_gender_rows_filter() {
        let conn = fixture();
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::Role {
                    values: vec!["main".into()],
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c1", "g2/c3"]);
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::Gender {
                    values: vec!["male".into()],
                }],
                None,
            ),
        );
        assert!(got.is_empty());
    }

    #[test]
    fn name_contains_matches_all_name_fields() {
        let conn = fixture();
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::NameContains {
                    value: "senpai".into(),
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g2/c3"]);
        let got = hit_keys(
            &conn,
            char_spec(
                vec![QueryRow::NameContains {
                    value: "シロ".into(),
                }],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c1"]);
    }

    #[test]
    fn character_subject_combines_game_rows_and_character_rows() {
        let conn = fixture();
        let got = hit_keys(
            &conn,
            char_spec(
                vec![
                    QueryRow::Status {
                        values: vec![Status::Playing],
                    },
                    QueryRow::Role {
                        values: vec!["main".into()],
                    },
                ],
                None,
            ),
        );
        assert_eq!(got, vec!["g1/c1"]);
    }

    #[test]
    fn game_subject_character_rows_use_exists_semantics() {
        let conn = fixture();
        let got = ids(
            &conn,
            spec(vec![QueryRow::TraitInclude {
                mode: TagMatchMode::Any,
                traits: vec!["白毛".into()],
            }]),
        );
        assert_eq!(got, vec!["g1"]);
        let got = ids(
            &conn,
            spec(vec![
                QueryRow::Status {
                    values: vec![Status::Playing],
                },
                QueryRow::TraitInclude {
                    mode: TagMatchMode::Any,
                    traits: vec!["t100".into()],
                },
            ]),
        );
        assert_eq!(got, vec!["g1"]);
    }

    #[test]
    fn character_hits_carry_game_context() {
        let conn = fixture();
        let hits = query_characters(
            &conn,
            char_spec(
                vec![QueryRow::Role {
                    values: vec!["main".into()],
                }],
                None,
            ),
        )
        .unwrap();
        let hit = hits.iter().find(|h| h.character_id == "c3").unwrap();
        assert_eq!(hit.game_id, "g2");
        assert_eq!(hit.game_title, "g2");
        assert_eq!(hit.name.as_deref(), Some("学姐"));
        assert_eq!(hit.original.as_deref(), Some("センパイ"));
        assert_eq!(hit.gender.as_deref(), Some(&["female".to_string()][..]));
    }

    #[test]
    fn deserializes_wire_json() {
        let raw = r#"{
            "subject": "game",
            "rows": [
                { "kind": "tag_include", "mode": "all", "tags": ["百合"] },
                { "kind": "tag_exclude", "tags": ["NTR"] },
                { "kind": "status", "values": ["playing", "finished"] },
                { "kind": "rating", "source": "user", "min": 60, "max": null },
                { "kind": "released", "from": "2016-01-01", "to": null },
                { "kind": "developer", "name": "Key" },
                { "kind": "length", "source": "vndb", "min": null, "max": 300 }
            ]
        }"#;
        let spec: QuerySpec = serde_json::from_str(raw).unwrap();
        assert_eq!(spec.subject, QuerySubject::Game);
        assert_eq!(spec.rows.len(), 7);
        assert_eq!(spec.spoiler_threshold, None);
    }

    #[test]
    fn deserializes_character_rows_wire_json() {
        let raw = r#"{
            "subject": "character",
            "spoilerThreshold": 2,
            "rows": [
                { "kind": "trait_include", "mode": "any", "traits": ["t100", "白毛"] },
                { "kind": "trait_exclude", "traits": ["NTR"] },
                { "kind": "role", "values": ["main", "side"] },
                { "kind": "gender", "values": ["female"] },
                { "kind": "name_contains", "value": "senpai" },
                { "kind": "va", "staff_ids": ["s1", "s113"] }
            ]
        }"#;
        let spec: QuerySpec = serde_json::from_str(raw).unwrap();
        assert_eq!(spec.subject, QuerySubject::Character);
        assert_eq!(spec.spoiler_threshold, Some(2));
        assert_eq!(spec.rows.len(), 6);
    }
}

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::Result;
use crate::tagdict;
use crate::types::{CharacterTrait, MetadataTag};

pub fn read_tags(conn: &Connection, game_id: &str) -> Result<Vec<MetadataTag>> {
    let dict = tagdict::dictionary();
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, gt.rating, gt.spoiler, gt.lie
           FROM game_tag gt JOIN tags t ON t.id = gt.tag_id
          WHERE gt.game_id = ?1
          ORDER BY gt.rowid",
    )?;
    let rows = stmt.query_map(params![game_id], |row| {
        let id: String = row.get(0)?;
        let entry = dict.tags.get(&id);
        Ok(MetadataTag {
            category: entry.and_then(|e| e.category.clone()),
            vn_count: entry.and_then(|e| e.vn_count),
            id: Some(id),
            name: row.get(1)?,
            rating: row.get(2)?,
            spoiler: row.get(3)?,
            lie: row.get::<_, Option<i64>>(4)?.map(|v| v != 0),
        })
    })?;
    Ok(rows.flatten().collect())
}

pub fn replace_tags(conn: &Connection, game_id: &str, tags: &[MetadataTag]) -> Result<()> {
    conn.execute("DELETE FROM game_tag WHERE game_id = ?1", params![game_id])?;
    let mut seen = std::collections::HashSet::new();
    for tag in tags {
        if tag.name.trim().is_empty() {
            continue;
        }
        let is_dict = tag.id.is_some();
        let id = match &tag.id {
            Some(id) => id.clone(),
            None => create_local_tag(conn, &tag.name)?,
        };
        if !seen.insert(id.clone()) {
            continue;
        }
        conn.execute(
            "INSERT OR IGNORE INTO tags (id, name, source) VALUES (?1, ?2, ?3)",
            params![id, tag.name, if is_dict { "vndb" } else { "local" }],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO game_tag (game_id, tag_id, rating, spoiler, lie)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                game_id,
                id,
                tag.rating,
                tag.spoiler,
                tag.lie.map(i64::from)
            ],
        )?;
    }
    Ok(())
}

pub fn has_dict_tags(conn: &Connection, game_id: &str) -> Result<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM game_tag gt JOIN tags t ON t.id = gt.tag_id
              WHERE gt.game_id = ?1 AND t.source = 'vndb' LIMIT 1",
            params![game_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

pub fn create_local_tag(conn: &Connection, name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(crate::error::AppError::BadRequest("标签名不能为空".into()));
    }
    if let Some(id) = conn
        .query_row(
            "SELECT id FROM tags WHERE source = 'local' AND name = ?1 LIMIT 1",
            params![trimmed],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(id);
    }
    let id = format!("u{}", next_seq(conn, "tags")?);
    conn.execute(
        "INSERT INTO tags (id, name, source) VALUES (?1, ?2, 'local')",
        params![id, trimmed],
    )?;
    Ok(id)
}

pub fn read_traits(
    conn: &Connection,
    game_id: &str,
    character_id: &str,
) -> Result<Vec<CharacterTrait>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.grp, ct.spoiler, ct.lie
           FROM character_trait ct JOIN traits t ON t.id = ct.trait_id
          WHERE ct.game_id = ?1 AND ct.character_id = ?2
          ORDER BY ct.rowid",
    )?;
    let rows = stmt.query_map(params![game_id, character_id], |row| {
        Ok(CharacterTrait {
            id: Some(row.get(0)?),
            name: Some(row.get(1)?),
            group_name: row.get(2)?,
            spoiler: row.get(3)?,
            lie: row.get::<_, Option<i64>>(4)?.map(|v| v != 0),
        })
    })?;
    Ok(rows.flatten().collect())
}

pub fn replace_traits(
    conn: &Connection,
    game_id: &str,
    character_id: &str,
    traits: &[CharacterTrait],
) -> Result<()> {
    conn.execute(
        "DELETE FROM character_trait WHERE game_id = ?1 AND character_id = ?2",
        params![game_id, character_id],
    )?;
    let mut seen = std::collections::HashSet::new();
    for tr in traits {
        let name = tr.name.clone().unwrap_or_default();
        if name.trim().is_empty() {
            continue;
        }
        let is_dict = tr.id.is_some();
        let id = match &tr.id {
            Some(id) => id.clone(),
            None => create_local_trait(conn, &name, tr.group_name.as_deref())?,
        };
        if !seen.insert(id.clone()) {
            continue;
        }
        conn.execute(
            "INSERT OR IGNORE INTO traits (id, name, grp, source) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, tr.group_name, if is_dict { "vndb" } else { "local" }],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO character_trait
             (game_id, character_id, trait_id, spoiler, lie) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                game_id,
                character_id,
                id,
                tr.spoiler,
                tr.lie.map(i64::from)
            ],
        )?;
    }
    Ok(())
}

pub fn create_local_trait(
    conn: &Connection,
    name: &str,
    group: Option<&str>,
) -> Result<String> {
    let trimmed = name.trim();
    if let Some(id) = conn
        .query_row(
            "SELECT id FROM traits WHERE source = 'local' AND name = ?1 LIMIT 1",
            params![trimmed],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(id);
    }
    let id = format!("u{}", next_seq(conn, "traits")?);
    conn.execute(
        "INSERT INTO traits (id, name, grp, source) VALUES (?1, ?2, ?3, 'local')",
        params![id, trimmed, group],
    )?;
    Ok(id)
}

fn next_seq(conn: &Connection, table: &str) -> Result<u32> {
    let sql = format!("SELECT id FROM {table} WHERE id LIKE 'u%'");
    let mut stmt = conn.prepare(&sql)?;
    let ids = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut max = 0;
    for id in ids.flatten() {
        if let Ok(n) = id.trim_start_matches('u').parse::<u32>() {
            max = max.max(n);
        }
    }
    Ok(max + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Integer,
    Real,
    Bool,
    Titles,
    Tags,
    Strings,
    Staff,
    Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlankRule {
    TrimEmpty,
    NullOnly,
    EmptyList,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    Upstream,
    Local,
    System,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldSpec {
    pub key: &'static str,
    pub sql: &'static str,
    pub ty: FieldType,
    pub clearable: bool,
    pub blank: BlankRule,
    pub label: Option<&'static str>,
    pub source: ValueSource,
    pub editable: bool,
    pub in_patch: bool,
    pub joined: bool,
}

use crate::error::Result;
use crate::types::{LocalizedTitle, MetadataStaff, MetadataTag};
use rusqlite::types::Value;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GameMetadata {
    pub game_id: String,
    pub image_id: Option<String>,
    pub alias: Option<String>,
    pub released: Option<String>,
    pub rating: Option<f64>,
    pub votecount: Option<i64>,
    pub length_minutes: Option<i64>,
    pub description: Option<String>,
    pub image_path: Option<String>,
    pub titles: Option<Vec<LocalizedTitle>>,
    pub tags: Option<Vec<MetadataTag>>,
    pub developers: Option<Vec<String>>,
    pub staff: Option<Vec<MetadataStaff>>,
    pub cover_focal_x: Option<i64>,
    pub cover_focal_y: Option<i64>,
    pub ambience_path: Option<String>,
    pub ambience_mode: String,
    pub ambience_plate: bool,
    pub ambience_focal_x: i64,
    pub ambience_focal_y: i64,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
pub struct MetadataPatch {
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub alias: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub released: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub rating: Option<Option<f64>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub votecount: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub length_minutes: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub image_path: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub titles: Option<Option<Vec<LocalizedTitle>>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub tags: Option<Option<Vec<MetadataTag>>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub developers: Option<Option<Vec<String>>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub staff: Option<Option<Vec<MetadataStaff>>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub cover_focal_x: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub cover_focal_y: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub ambience_path: Option<Option<String>>,
    pub ambience_mode: Option<String>,
    pub ambience_plate: Option<bool>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub ambience_focal_x: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::types::double_option")]
    pub ambience_focal_y: Option<Option<i64>>,
}

impl MetadataPatch {
    pub fn filled_labels(&self) -> Vec<&'static str> {
        METADATA_FIELDS
            .iter()
            .filter_map(|field| field.label.filter(|_| is_set(self, field.key)))
            .collect()
    }
}

pub const METADATA_FIELDS: &[FieldSpec] = &[
    FieldSpec {
        key: "game_id",
        sql: "game_id",
        ty: FieldType::Text,
        clearable: false,
        blank: BlankRule::NotApplicable,
        label: None,
        source: ValueSource::System,
        editable: false,
        in_patch: false,
        joined: false,
    },
    FieldSpec {
        key: "image_id",
        sql: "image_id",
        ty: FieldType::Text,
        clearable: false,
        blank: BlankRule::NotApplicable,
        label: None,
        source: ValueSource::System,
        editable: false,
        in_patch: false,
        joined: false,
    },
    FieldSpec {
        key: "alias",
        sql: "alias",
        ty: FieldType::Text,
        clearable: true,
        blank: BlankRule::TrimEmpty,
        label: Some("别名"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "released",
        sql: "released",
        ty: FieldType::Text,
        clearable: true,
        blank: BlankRule::TrimEmpty,
        label: Some("发售日"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "rating",
        sql: "rating",
        ty: FieldType::Real,
        clearable: true,
        blank: BlankRule::NullOnly,
        label: Some("评分"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "votecount",
        sql: "votecount",
        ty: FieldType::Integer,
        clearable: true,
        blank: BlankRule::NullOnly,
        label: Some("评分人数"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "length_minutes",
        sql: "length_minutes",
        ty: FieldType::Integer,
        clearable: true,
        blank: BlankRule::NullOnly,
        label: Some("时长"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "description",
        sql: "description",
        ty: FieldType::Text,
        clearable: true,
        blank: BlankRule::TrimEmpty,
        label: Some("简介"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "image_path",
        sql: "image_path",
        ty: FieldType::Text,
        clearable: true,
        blank: BlankRule::TrimEmpty,
        label: Some("封面"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "titles",
        sql: "titles",
        ty: FieldType::Titles,
        clearable: true,
        blank: BlankRule::EmptyList,
        label: Some("标题"),
        source: ValueSource::Upstream,
        editable: false,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "tags",
        sql: "tags",
        ty: FieldType::Tags,
        clearable: true,
        blank: BlankRule::EmptyList,
        label: Some("标签"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: true,
    },
    FieldSpec {
        key: "developers",
        sql: "developers",
        ty: FieldType::Strings,
        clearable: true,
        blank: BlankRule::EmptyList,
        label: Some("开发商"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "staff",
        sql: "staff",
        ty: FieldType::Staff,
        clearable: true,
        blank: BlankRule::EmptyList,
        label: Some("制作组"),
        source: ValueSource::Upstream,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "cover_focal_x",
        sql: "cover_focal_x",
        ty: FieldType::Integer,
        clearable: true,
        blank: BlankRule::NotApplicable,
        label: None,
        source: ValueSource::Local,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "cover_focal_y",
        sql: "cover_focal_y",
        ty: FieldType::Integer,
        clearable: true,
        blank: BlankRule::NotApplicable,
        label: None,
        source: ValueSource::Local,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "ambience_path",
        sql: "ambience_path",
        ty: FieldType::Text,
        clearable: true,
        blank: BlankRule::NotApplicable,
        label: None,
        source: ValueSource::Local,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "ambience_mode",
        sql: "ambience_mode",
        ty: FieldType::Mode,
        clearable: false,
        blank: BlankRule::NotApplicable,
        label: None,
        source: ValueSource::Local,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "ambience_plate",
        sql: "ambience_plate",
        ty: FieldType::Bool,
        clearable: false,
        blank: BlankRule::NotApplicable,
        label: None,
        source: ValueSource::Local,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "ambience_focal_x",
        sql: "ambience_focal_x",
        ty: FieldType::Integer,
        clearable: true,
        blank: BlankRule::NotApplicable,
        label: None,
        source: ValueSource::Local,
        editable: true,
        in_patch: true,
        joined: false,
    },
    FieldSpec {
        key: "ambience_focal_y",
        sql: "ambience_focal_y",
        ty: FieldType::Integer,
        clearable: true,
        blank: BlankRule::NotApplicable,
        label: None,
        source: ValueSource::Local,
        editable: true,
        in_patch: true,
        joined: false,
    },
];

pub fn is_set(patch: &crate::types::MetadataPatch, key: &str) -> bool {
    match key {
        "alias" => patch.alias.is_some(),
        "released" => patch.released.is_some(),
        "rating" => patch.rating.is_some(),
        "votecount" => patch.votecount.is_some(),
        "length_minutes" => patch.length_minutes.is_some(),
        "description" => patch.description.is_some(),
        "image_path" => patch.image_path.is_some(),
        "titles" => patch.titles.is_some(),
        "tags" => patch.tags.is_some(),
        "developers" => patch.developers.is_some(),
        "staff" => patch.staff.is_some(),
        "cover_focal_x" => patch.cover_focal_x.is_some(),
        "cover_focal_y" => patch.cover_focal_y.is_some(),
        "ambience_path" => patch.ambience_path.is_some(),
        "ambience_mode" => patch.ambience_mode.is_some(),
        "ambience_plate" => patch.ambience_plate.is_some(),
        "ambience_focal_x" => patch.ambience_focal_x.is_some(),
        "ambience_focal_y" => patch.ambience_focal_y.is_some(),
        _ => false,
    }
}

pub fn field(key: &str) -> Option<&'static FieldSpec> {
    METADATA_FIELDS.iter().find(|f| f.key == key)
}

pub const COLUMNS: &str = "game_id, image_id, alias, released, rating, votecount, length_minutes, description, image_path, titles, developers, staff, cover_focal_x, cover_focal_y, ambience_path, ambience_mode, ambience_plate, ambience_focal_x, ambience_focal_y";

pub const DDL_COLUMNS: &str = r#"          game_id      TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
          image_id     TEXT,
          alias        TEXT,
          released     TEXT,
          rating       REAL,
          votecount    INTEGER,
          length_minutes INTEGER,
          description  TEXT,
          image_path   TEXT,
          titles       TEXT,
          developers   TEXT,
          staff        TEXT,
          cover_focal_x INTEGER NOT NULL DEFAULT 50,
          cover_focal_y INTEGER NOT NULL DEFAULT 50,
          ambience_path TEXT,
          ambience_mode TEXT NOT NULL DEFAULT 'soft',
          ambience_plate INTEGER NOT NULL DEFAULT 0,
          ambience_focal_x INTEGER NOT NULL DEFAULT 50,
          ambience_focal_y INTEGER NOT NULL DEFAULT 22
"#;

pub fn patch_columns(patch: &MetadataPatch) -> Result<(Vec<&'static str>, Vec<Value>)> {
    let mut columns: Vec<&'static str> = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    if let Some(raw) = &patch.alias {
        columns.push("alias");
        values.push(match raw {
            Some(v) => Value::Text(v.clone()),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.released {
        columns.push("released");
        values.push(match raw {
            Some(v) => Value::Text(v.clone()),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.rating {
        columns.push("rating");
        values.push(match raw {
            Some(v) => Value::Real(*v),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.votecount {
        columns.push("votecount");
        values.push(match raw {
            Some(v) => Value::Integer(*v),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.length_minutes {
        columns.push("length_minutes");
        values.push(match raw {
            Some(v) => Value::Integer(*v),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.description {
        columns.push("description");
        values.push(match raw {
            Some(v) => Value::Text(v.clone()),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.image_path {
        columns.push("image_path");
        values.push(match raw {
            Some(v) => Value::Text(v.clone()),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.titles {
        columns.push("titles");
        values.push(match raw {
            Some(v) => Value::Text(serde_json::to_string(v)?),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.developers {
        columns.push("developers");
        values.push(match raw {
            Some(v) => Value::Text(serde_json::to_string(v)?),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.staff {
        columns.push("staff");
        values.push(match raw {
            Some(v) => Value::Text(serde_json::to_string(v)?),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.cover_focal_x {
        columns.push("cover_focal_x");
        values.push(match raw {
            Some(v) => Value::Integer(*v),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.cover_focal_y {
        columns.push("cover_focal_y");
        values.push(match raw {
            Some(v) => Value::Integer(*v),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.ambience_path {
        columns.push("ambience_path");
        values.push(match raw {
            Some(v) => Value::Text(v.clone()),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.ambience_mode {
        columns.push("ambience_mode");
        values.push(Value::Text(raw.clone()));
    }
    if let Some(raw) = &patch.ambience_plate {
        columns.push("ambience_plate");
        values.push(Value::Integer(i64::from(*raw)));
    }
    if let Some(raw) = &patch.ambience_focal_x {
        columns.push("ambience_focal_x");
        values.push(match raw {
            Some(v) => Value::Integer(*v),
            None => Value::Null,
        });
    }
    if let Some(raw) = &patch.ambience_focal_y {
        columns.push("ambience_focal_y");
        values.push(match raw {
            Some(v) => Value::Integer(*v),
            None => Value::Null,
        });
    }
    Ok((columns, values))
}

pub fn is_blank_current(meta: &GameMetadata, key: &str) -> bool {
    match key {
        "alias" => meta.alias.as_deref().is_none_or(|v| v.trim().is_empty()),
        "released" => meta.released.as_deref().is_none_or(|v| v.trim().is_empty()),
        "rating" => meta.rating.is_none(),
        "votecount" => meta.votecount.is_none(),
        "length_minutes" => meta.length_minutes.is_none(),
        "description" => meta.description.as_deref().is_none_or(|v| v.trim().is_empty()),
        "image_path" => meta.image_path.as_deref().is_none_or(|v| v.trim().is_empty()),
        "titles" => meta.titles.as_ref().is_none_or(|v| v.is_empty()),
        "developers" => meta.developers.as_ref().is_none_or(|v| v.is_empty()),
        "staff" => meta.staff.as_ref().is_none_or(|v| v.is_empty()),
        _ => false,
    }
}

pub struct MetaRow {
    pub game_id: String,
    pub image_id: Option<String>,
    pub alias: Option<String>,
    pub released: Option<String>,
    pub rating: Option<f64>,
    pub votecount: Option<i64>,
    pub length_minutes: Option<i64>,
    pub description: Option<String>,
    pub image_path: Option<String>,
    pub titles: Option<String>,
    pub developers: Option<String>,
    pub staff: Option<String>,
    pub cover_focal_x: Option<i64>,
    pub cover_focal_y: Option<i64>,
    pub ambience_path: Option<String>,
    pub ambience_mode: String,
    pub ambience_plate: bool,
    pub ambience_focal_x: i64,
    pub ambience_focal_y: i64,
}

pub fn read_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MetaRow> {
    Ok(MetaRow {
        game_id: row.get("game_id")?,
        image_id: row.get("image_id")?,
        alias: row.get("alias")?,
        released: row.get("released")?,
        rating: row.get("rating")?,
        votecount: row.get("votecount")?,
        length_minutes: row.get("length_minutes")?,
        description: row.get("description")?,
        image_path: row.get("image_path")?,
        titles: row.get("titles")?,
        developers: row.get("developers")?,
        staff: row.get("staff")?,
        cover_focal_x: row.get("cover_focal_x")?,
        cover_focal_y: row.get("cover_focal_y")?,
        ambience_path: row.get("ambience_path")?,
        ambience_mode: row.get("ambience_mode")?,
        ambience_plate: row.get("ambience_plate")?,
        ambience_focal_x: row.get("ambience_focal_x")?,
        ambience_focal_y: row.get("ambience_focal_y")?,
    })
}

pub fn to_api(row: &MetaRow) -> Result<GameMetadata> {
    Ok(GameMetadata {
        game_id: row.game_id.clone(),
        image_id: row.image_id.clone(),
        alias: row.alias.clone(),
        released: row.released.clone(),
        rating: row.rating,
        votecount: row.votecount,
        length_minutes: row.length_minutes,
        description: row.description.clone(),
        image_path: row.image_path.clone(),
        titles: from_json::<Vec<LocalizedTitle>>(&row.titles)?,
        tags: None,
        developers: from_json::<Vec<String>>(&row.developers)?,
        staff: from_json::<Vec<MetadataStaff>>(&row.staff)?,
        cover_focal_x: row.cover_focal_x,
        cover_focal_y: row.cover_focal_y,
        ambience_path: row.ambience_path.clone(),
        ambience_mode: row.ambience_mode.clone(),
        ambience_plate: row.ambience_plate,
        ambience_focal_x: row.ambience_focal_x,
        ambience_focal_y: row.ambience_focal_y,
    })
}

fn from_json<T: serde::de::DeserializeOwned>(raw: &Option<String>) -> Result<Option<T>> {
    match raw {
        Some(text) => Ok(Some(serde_json::from_str(text)?)),
        None => Ok(None),
    }
}

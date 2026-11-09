use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Deserializer, Serialize};

pub fn double_option<'de, D, T>(d: D) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(d).map(Some)
}

pub use crate::generated::characters::GameCharacter;
pub use crate::generated::metadata::{GameMetadata, MetadataPatch};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub vndb_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Wishlist,
    Playing,
    Finished,
    Dropped,
    Stalled,
}

impl Status {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "wishlist" => Some(Self::Wishlist),
            "playing" => Some(Self::Playing),
            "finished" => Some(Self::Finished),
            "dropped" => Some(Self::Dropped),
            "stalled" => Some(Self::Stalled),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wishlist => "wishlist",
            Self::Playing => "playing",
            Self::Finished => "finished",
            Self::Dropped => "dropped",
            Self::Stalled => "stalled",
        }
    }
}

impl FromSql for Status {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let raw = value.as_str()?;
        Self::parse(raw).ok_or_else(|| {
            FromSqlError::other(std::io::Error::other(format!("库里出现未知 status：{raw}")))
        })
    }
}

impl ToSql for Status {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MagpieMode {
    #[default]
    Off,
    Full,
    Window,
}

impl MagpieMode {
    pub fn from_db(raw: i64) -> Self {
        match raw {
            1 => Self::Full,
            2 => Self::Window,
            _ => Self::Off,
        }
    }

    pub fn to_db(self) -> i64 {
        match self {
            Self::Off => 0,
            Self::Full => 1,
            Self::Window => 2,
        }
    }

    pub fn hotkey_action(self) -> Option<i32> {
        match self {
            Self::Off => None,
            Self::Full => Some(0),
            Self::Window => Some(1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGameState {
    pub game_id: String,
    pub status: Status,
    pub user_rating: Option<f64>,
    pub notes: Option<String>,
    pub default_installation_id: Option<String>,
    pub play_seconds: i64,
    pub play_count: i64,
    pub last_played_at: Option<i64>,
    pub foreground_seconds: i64,
    pub alive_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub id: String,
    pub game_id: Option<String>,
    pub dir_path: String,
    pub entry_path: Option<String>,
    pub exes: Option<Vec<String>>,
    pub size_bytes: Option<i64>,
    pub le_launch: bool,
    pub magpie: MagpieMode,
    pub launch_args: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameListItem {
    pub id: String,
    pub name: String,
    pub vndb_id: Option<String>,
    pub alias: Option<String>,
    pub created_at: i64,
    pub status: Status,
    pub user_rating: Option<f64>,
    pub vndb_rating: Option<f64>,
    pub installation_count: i64,
    pub can_launch: bool,
    pub cover_path: Option<String>,
    pub cover_focal_x: Option<i64>,
    pub cover_focal_y: Option<i64>,
    pub va_names: Option<String>,
    pub va_original_names: Option<String>,
    pub tag_names: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct GamePatch {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub vndb_id: Option<Option<String>>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct StatePatch {
    pub status: Option<Status>,
    #[serde(default, deserialize_with = "double_option")]
    pub user_rating: Option<Option<f64>>,
    #[serde(default, deserialize_with = "double_option")]
    pub notes: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub default_installation_id: Option<Option<String>>,
    pub play_seconds: Option<i64>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct InstallationPatch {
    #[serde(default, deserialize_with = "double_option")]
    pub game_id: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub entry_path: Option<Option<String>>,
    pub le_launch: Option<bool>,
    pub magpie: Option<MagpieMode>,
    #[serde(default, deserialize_with = "double_option")]
    pub launch_args: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaySession {
    pub id: String,
    pub game_id: String,
    pub installation_id: Option<String>,
    pub started_at: i64,
    pub last_seen_at: i64,
    pub ended_at: Option<i64>,
    pub foreground_seconds: i64,
    pub alive_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GamePlayLog {
    pub count: i64,
    pub first_at: Option<i64>,
    pub last_at: Option<i64>,
    pub total_seconds: i64,
    pub sessions: Vec<PlaySession>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedTitle {
    pub lang: String,
    pub title: String,
    pub latin: Option<String>,
    pub official: Option<bool>,
    pub main: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetadataTag {
    pub id: Option<String>,
    pub name: String,
    pub category: Option<String>,
    pub rating: Option<f64>,
    pub vn_count: Option<i64>,
    pub spoiler: Option<i64>,
    pub lie: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetadataStaff {
    pub role: Option<String>,
    pub name: Option<String>,
    pub original: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterTrait {
    pub id: Option<String>,
    pub name: Option<String>,
    pub group_name: Option<String>,
    pub spoiler: Option<i64>,
    pub lie: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterVa {
    pub staff_id: String,
    pub name: Option<String>,
    pub original: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CharacterHit {
    pub game_id: String,
    pub game_title: String,
    pub character_id: String,
    pub name: Option<String>,
    pub original: Option<String>,
    pub role: Option<String>,
    pub gender: Option<Vec<String>>,
    pub traits: Option<Vec<CharacterTrait>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewGame {
    pub name: String,
    pub vndb_id: Option<String>,
    pub metadata: Option<MetadataPatch>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewInstallation {
    pub game_id: String,
    pub dir_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallationInput {
    pub id: Option<String>,
    pub dir_path: String,
    pub entry_path: Option<String>,
    #[serde(default)]
    pub le_launch: bool,
    #[serde(default)]
    pub magpie: MagpieMode,
    pub launch_args: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskEvent {
    pub phase: String,
    #[serde(default)]
    pub ts: i64,
    pub done: Option<i64>,
    pub total: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub run_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    #[serde(default)]
    pub seq: u64,
}

#[cfg(test)]
mod tests {
    use super::MetadataPatch;
    use crate::generated::metadata::{is_set, METADATA_FIELDS};

    #[test]
    fn patch_tells_missing_apart_from_null() {
        let missing: MetadataPatch = serde_json::from_str(r"{}").unwrap();
        let nulled: MetadataPatch = serde_json::from_str(r#"{"ambience_path": null}"#).unwrap();
        let set: MetadataPatch =
            serde_json::from_str(r#"{"ambience_path": "D:/a.png"}"#).unwrap();

        assert_eq!(missing.ambience_path, None);
        assert_eq!(nulled.ambience_path, Some(None));
        assert_eq!(set.ambience_path.as_ref(), Some(&Some("D:/a.png".to_string())));
    }

    #[test]
    fn 补全报告由规格表驱动且不遗漏字段() {
        assert_eq!(METADATA_FIELDS.len(), 20);
        assert_eq!(METADATA_FIELDS.iter().filter(|f| f.in_patch).count(), 18);
        assert_eq!(METADATA_FIELDS.iter().filter(|f| f.label.is_some()).count(), 11);

        let patch: MetadataPatch = serde_json::from_str(
            r#"{"alias":"A","released":"2020-01-01","rating":8.5,"votecount":10,
                "length_minutes":120,"description":"D","image_path":"p.png",
                "titles":[{"lang":"ja","title":"T"}],"tags":[{"name":"t"}],
                "developers":["d"],"staff":[{"name":"s"}],
                "cover_focal_x":30,"cover_focal_y":40,"ambience_path":"a.png",
                "ambience_mode":"soft","ambience_plate":true,
                "ambience_focal_x":50,"ambience_focal_y":22}"#,
        )
        .unwrap();

        assert_eq!(
            patch.filled_labels(),
            vec![
                "别名", "发售日", "评分", "评分人数", "时长", "简介", "封面", "标题", "标签",
                "开发商", "制作组"
            ]
        );
        for f in METADATA_FIELDS.iter().filter(|f| f.in_patch) {
            assert!(is_set(&patch, f.key), "{} 未被 is_set 覆盖", f.key);
        }
    }

    #[test]
    fn 本地视觉字段不进补全报告() {
        let base: MetadataPatch = serde_json::from_str(r#"{"alias":"A"}"#).unwrap();
        let with_local: MetadataPatch = serde_json::from_str(
            r#"{"alias":"A","cover_focal_x":30,"ambience_mode":"soft","ambience_plate":true}"#,
        )
        .unwrap();
        assert_eq!(with_local.filled_labels(), base.filled_labels());
        assert_eq!(with_local.filled_labels(), vec!["别名"]);
    }
}

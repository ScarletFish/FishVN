use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

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
    pub installation_count: i64,
    pub can_launch: bool,
    pub cover_path: Option<String>,
    pub cover_focal_x: Option<i64>,
    pub cover_focal_y: Option<i64>,
    pub va_names: Option<String>,
    pub tag_names: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct GamePatch {
    pub name: Option<String>,
    pub vndb_id: Option<Option<String>>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct StatePatch {
    pub status: Option<Status>,
    pub user_rating: Option<Option<f64>>,
    pub notes: Option<Option<String>>,
    pub default_installation_id: Option<Option<String>>,
    pub play_seconds: Option<i64>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct InstallationPatch {
    pub game_id: Option<Option<String>>,
    pub entry_path: Option<Option<String>>,
    pub le_launch: Option<bool>,
    pub magpie: Option<MagpieMode>,
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

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameMetadata {
    pub game_id: String,
    pub alias: Option<String>,
    pub titles: Option<Vec<LocalizedTitle>>,
    pub rating: Option<f64>,
    pub votecount: Option<i64>,
    pub length_minutes: Option<i64>,
    pub released: Option<String>,
    pub description: Option<String>,
    pub image_id: Option<String>,
    pub image_path: Option<String>,
    pub cover_focal_x: Option<i64>,
    pub cover_focal_y: Option<i64>,
    pub tags: Option<Vec<MetadataTag>>,
    pub developers: Option<Vec<String>>,
    pub staff: Option<Vec<MetadataStaff>>,
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

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct MetadataPatch {
    pub alias: Option<Option<String>>,
    pub titles: Option<Option<Vec<LocalizedTitle>>>,
    pub rating: Option<Option<f64>>,
    pub votecount: Option<Option<i64>>,
    pub length_minutes: Option<Option<i64>>,
    pub released: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub image_path: Option<Option<String>>,
    pub cover_focal_x: Option<Option<i64>>,
    pub cover_focal_y: Option<Option<i64>>,
    pub tags: Option<Option<Vec<MetadataTag>>>,
    pub developers: Option<Option<Vec<String>>>,
    pub staff: Option<Option<Vec<MetadataStaff>>>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameCharacter {
    pub id: String,
    pub name: Option<String>,
    pub original: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub description: Option<String>,
    pub gender: Option<Vec<String>>,
    pub age: Option<i64>,
    pub birthday: Option<String>,
    pub blood_type: Option<String>,
    pub height: Option<i64>,
    pub weight: Option<i64>,
    pub bust: Option<i64>,
    pub waist: Option<i64>,
    pub hips: Option<i64>,
    pub cup: Option<String>,
    pub role: Option<String>,
    pub image_id: Option<String>,
    pub image_path: Option<String>,
    pub sort_order: i64,
    pub traits: Option<Vec<CharacterTrait>>,
    pub va: Option<Vec<CharacterVa>>,
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
    pub done: Option<i64>,
    pub total: Option<i64>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub run_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
}

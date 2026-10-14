use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

/// games 表行：作品身份
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    /// VNDB 编号 'v14'；None = 未匹配
    pub vndb_id: Option<String>,
    /// unix 秒
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Wishlist,
    Playing,
    Finished,
    Dropped,
}

impl Status {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "wishlist" => Some(Self::Wishlist),
            "playing" => Some(Self::Playing),
            "finished" => Some(Self::Finished),
            "dropped" => Some(Self::Dropped),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wishlist => "wishlist",
            Self::Playing => "playing",
            Self::Finished => "finished",
            Self::Dropped => "dropped",
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

/// user_game_state 表行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGameState {
    pub game_id: String,
    pub status: Status,
    pub user_rating: Option<f64>,
    pub notes: Option<String>,
    /// 启动这份作品时的默认安装
    pub default_installation_id: Option<String>,
    /// unix 秒
    pub last_played_at: Option<i64>,
    pub play_count: i64,
}

/// installations 表行：一个本地目录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub id: String,
    /// 所属作品；None = 待认领
    pub game_id: Option<String>,
    pub dir_path: String,
    /// 入口 exe，相对 dir_path
    pub entry_path: Option<String>,
    /// 候选入口清单
    pub exes: Option<Vec<String>>,
    pub size_bytes: Option<i64>,
}

/// 列表投影：作品 + 状态 + 本地安装概况，卡片一次渲染所需
#[derive(Debug, Clone, Serialize)]
pub struct GameListItem {
    pub id: String,
    pub name: String,
    pub vndb_id: Option<String>,
    pub created_at: i64,
    pub status: Status,
    pub user_rating: Option<f64>,
    pub last_played_at: Option<i64>,
    pub play_count: i64,
    /// 已登记的本地目录数
    pub installation_count: i64,
    /// 至少有一个目录已指定入口
    pub can_launch: bool,
    pub cover_path: Option<String>,
}

/// PATCH 的可空字段用双层 Option：外层 None = 没给，内层 None = 显式置空
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
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct InstallationPatch {
    pub game_id: Option<Option<String>>,
    pub entry_path: Option<Option<String>>,
}

/// 作品正式名在 games.name，这里不重复存
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameMetadata {
    pub game_id: String,
    pub alias: Option<String>,
    pub titles: Option<Vec<LocalizedTitle>>,
    pub rating: Option<f64>,
    pub votecount: Option<i64>,
    pub released: Option<String>,
    pub description: Option<String>,
    /// VNDB 侧的 cv 编号，只用来判断封面要不要重下
    pub image_id: Option<String>,
    pub image_path: Option<String>,
    pub tags: Option<Vec<MetadataTag>>,
    pub developers: Option<Vec<String>>,
}

/// 一部作品可能一条中文标题都没有，靠 lang 挑不出想要的那条，只能看 official / main
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedTitle {
    pub lang: String,
    pub title: String,
    pub latin: Option<String>,
    pub official: Option<bool>,
    pub main: Option<bool>,
}

/// 用户手打的标签没有排序键与剧透等级，同一列里两种都可能有
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetadataTag {
    /// VNDB 标签编号；用户手打或本地自建的标签没有
    pub id: Option<String>,
    pub name: String,
    pub category: Option<String>,
    pub rating: Option<f64>,
    pub vn_count: Option<i64>,
    pub spoiler: Option<i64>,
    pub lie: Option<bool>,
}

/// PATCH 的可空字段用双层 Option：外层 None = 没给，内层 None = 显式置空。
/// 字段留空表示「不动」，写空字符串与置 null 是两回事。
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct MetadataPatch {
    pub alias: Option<Option<String>>,
    pub titles: Option<Option<Vec<LocalizedTitle>>>,
    pub rating: Option<Option<f64>>,
    pub votecount: Option<Option<i64>>,
    pub released: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub image_path: Option<Option<String>>,
    /// 整列替换，不按标签逐个合并
    pub tags: Option<Option<Vec<MetadataTag>>>,
    pub developers: Option<Option<Vec<String>>>,
}

/// characters 表行：角色属于作品。VNDB 上同一角色在不同作品里是不同条目
/// （数据详略与立绘各不相同），所以 c 编号按作品各存一行，不建共享本体
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameCharacter {
    /// VNDB c 编号，或本地自建的 `local-<uuid>`
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
    /// VNDB 的出场定位：main / primary / side / appears
    pub role: Option<String>,
    pub image_id: Option<String>,
    pub image_path: Option<String>,
    /// 由写入位置决定（见 characters::replace_all），读取端据此还原顺序
    pub sort_order: i64,
    pub traits: Option<Vec<CharacterTrait>>,
    pub va: Option<Vec<CharacterVa>>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterTrait {
    /// VNDB trait 编号（i 前缀的一组数字）
    pub id: Option<String>,
    pub name: String,
    pub group_name: Option<String>,
    pub spoiler: Option<i64>,
    pub lie: Option<bool>,
}

/// 角色 × 声优。写入时只用 staff_id / name / original 建 staffs 行，
/// note 是版本标记（如 Realta Nua），同一对角色与声优可带多条不同 note
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
    /// 给 Some 但一个字段都没填时，upsert 报「没有可更新的字段」而不是当没给
    pub metadata: Option<MetadataPatch>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewInstallation {
    pub game_id: String,
    pub dir_path: String,
}

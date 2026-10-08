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

#[derive(Debug, Clone, Deserialize)]
pub struct NewGame {
    pub name: String,
    pub vndb_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewInstallation {
    pub game_id: String,
    pub dir_path: String,
}

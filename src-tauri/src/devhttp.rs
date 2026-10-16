//! 调试期的本地 HTTP 通道，让浏览器调真实后端。
//!
//! 路由是 commands.rs 的镜像，加命令时这里要同步改。
//!
//! 只绑 127.0.0.1，且要设 FISHVN_DEV_HTTP 才开：本机任何进程都能读写用户的库

use std::sync::{Arc, Mutex};

use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::error::{AppError, Result};
use crate::types::GameCharacter;
use crate::vndb::RateLimiter;
use crate::{characters, commands, games, installations, metadata, vndb};

/// 与 vite 的 4545 分开：撞上会自己请求自己
pub const PORT: u16 = 4546;

struct Fail(StatusCode, String);

impl axum::response::IntoResponse for Fail {
    fn into_response(self) -> axum::response::Response {
        axum::response::IntoResponse::into_response((self.0, self.1))
    }
}

impl From<AppError> for Fail {
    fn from(e: AppError) -> Self {
        Fail(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }
}

type Reply = std::result::Result<Json<Value>, Fail>;

#[derive(Clone)]
pub struct Dev {
    pub conn: Arc<Mutex<Connection>>,
    pub rate: Arc<RateLimiter>,
    pub app: tauri::AppHandle,
}

/// 字段用命令原样的 snake_case：camelCase 到 snake 的转换只有 Tauri 侧有
#[derive(Default, serde::Deserialize)]
pub struct Args {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub input: Option<Value>,
    #[serde(default)]
    pub patch: Option<Value>,
    #[serde(default)]
    pub game_id: Option<String>,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub vndb_id: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub characters: Option<Value>,
}

impl Args {
    fn need<'a>(&'a self, slot: Option<&'a String>, name: &str) -> Result<&'a str> {
        slot.map(|s| s.as_str())
            .ok_or_else(|| AppError::BadRequest(format!("缺少 {name}")))
    }

    pub fn id(&self) -> Result<&str> {
        self.need(self.id.as_ref(), "id")
    }

    pub fn keyword(&self) -> Result<&str> {
        self.need(self.keyword.as_ref(), "keyword")
    }

    pub fn vndb_id(&self) -> Result<&str> {
        self.need(self.vndb_id.as_ref(), "vndb_id")
    }

    pub fn parse<T: DeserializeOwned>(&self, slot: Option<&Value>, name: &str) -> Result<T> {
        let raw = slot.ok_or_else(|| AppError::BadRequest(format!("缺少 {name}")))?;
        serde_json::from_value(raw.clone()).map_err(AppError::from)
    }

    pub fn path(&self) -> Result<&str> {
        self.need(self.path.as_ref(), "path")
    }
}

fn finish<T: Serialize>(r: Result<T>) -> Reply {
    r.map(|v| Json(serde_json::to_value(v).unwrap_or(Value::Null)))
        .map_err(Fail::from)
}

fn run<T: Serialize>(dev: &Dev, f: impl FnOnce(&Connection) -> Result<T>) -> Reply {
    let guard = dev
        .conn
        .lock()
        .map_err(|_| Fail::from(AppError::Io(std::io::Error::other("数据库锁已毒化"))))?;
    finish(f(&guard))
}

macro_rules! handler {
    ($name:ident, |$conn:ident, $a:ident| $body:expr) => {
        async fn $name(AxumState(dev): AxumState<Dev>, Json($a): Json<Args>) -> Reply {
            run(&dev, |$conn| -> Result<_> { $body })
        }
    };
}

handler!(list_games, |c, _a| games::list_games(c));
handler!(get_game, |c, a| games::get_game(c, a.id()?));
handler!(create_game, |c, a| games::create_game(
    c,
    &a.parse(a.input.as_ref(), "input")?
));
handler!(update_game, |c, a| {
    games::update_game(c, a.id()?, &a.parse(a.patch.as_ref(), "patch")?)
});
handler!(delete_game, |c, a| games::delete_game(c, a.id()?));
handler!(get_game_state, |c, a| games::get_game_state(c, a.id()?));
handler!(update_game_state, |c, a| {
    games::update_game_state(c, a.id()?, &a.parse(a.patch.as_ref(), "patch")?)
});
handler!(list_installations, |c, a| {
    installations::list_installations(c, a.game_id.as_deref())
});
handler!(get_installation, |c, a| installations::get_installation(
    c,
    a.id()?
));
handler!(create_installation, |c, a| {
    installations::create_installation(c, &a.parse(a.input.as_ref(), "input")?)
});
handler!(update_installation, |c, a| {
    installations::update_installation(c, a.id()?, &a.parse(a.patch.as_ref(), "patch")?)
});
handler!(delete_installation, |c, a| {
    installations::delete_installation(c, a.id()?)
});
handler!(get_game_metadata, |c, a| metadata::get(c, a.id()?));
handler!(update_game_metadata, |c, a| {
    metadata::upsert(c, a.id()?, &a.parse(a.patch.as_ref(), "patch")?)
});
handler!(list_game_characters, |c, a| characters::list(c, a.id()?));
handler!(save_game_characters, |c, a| {
    let list: Vec<GameCharacter> = a.parse(a.characters.as_ref(), "characters")?;
    characters::replace_all(c, a.id()?, &list)
});

async fn search_vndb(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let keyword = match a.keyword().and_then(vndb::validate_keyword) {
        Ok(k) => k,
        Err(e) => return Err(Fail(StatusCode::BAD_REQUEST, e.to_string())),
    };
    dev.rate.acquire().await;
    finish(vndb::search(vndb::http(), &keyword).await)
}

async fn fetch_vn_metadata(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    let vndb_id = match a.vndb_id().and_then(vndb::validate_vn_id) {
        Ok(v) => v,
        Err(e) => return Err(Fail(StatusCode::BAD_REQUEST, e.to_string())),
    };
    dev.rate.acquire().await;
    dev.rate.acquire().await;
    let dir = crate::data_dir(&dev.app)
        .map_err(|e| Fail(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    finish(vndb::fetch_sync(vndb::http(), &vndb_id, &dir).await)
}

async fn allow_local_file(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    finish(
        a.path()
            .and_then(|p| commands::allow_local_file(dev.app.clone(), p.to_owned())),
    )
}

async fn thumbnail(AxumState(dev): AxumState<Dev>, Json(a): Json<Args>) -> Reply {
    finish(
        a.path()
            .and_then(|p| commands::thumbnail(dev.app.clone(), p.to_owned())),
    )
}

pub fn router(dev: Dev) -> Router {
    Router::new()
        .route("/list_games", post(list_games))
        .route("/get_game", post(get_game))
        .route("/create_game", post(create_game))
        .route("/update_game", post(update_game))
        .route("/delete_game", post(delete_game))
        .route("/get_game_state", post(get_game_state))
        .route("/update_game_state", post(update_game_state))
        .route("/list_installations", post(list_installations))
        .route("/get_installation", post(get_installation))
        .route("/create_installation", post(create_installation))
        .route("/update_installation", post(update_installation))
        .route("/delete_installation", post(delete_installation))
        .route("/get_game_metadata", post(get_game_metadata))
        .route("/update_game_metadata", post(update_game_metadata))
        .route("/list_game_characters", post(list_game_characters))
        .route("/save_game_characters", post(save_game_characters))
        .route("/search_vndb", post(search_vndb))
        .route("/fetch_vn_metadata", post(fetch_vn_metadata))
        .route("/allow_local_file", post(allow_local_file))
        .route("/thumbnail", post(thumbnail))
        .with_state(dev)
}

pub const COMMANDS: &[&str] = &[
    "list_games",
    "get_game",
    "create_game",
    "update_game",
    "delete_game",
    "get_game_state",
    "update_game_state",
    "list_installations",
    "get_installation",
    "create_installation",
    "update_installation",
    "delete_installation",
    "get_game_metadata",
    "update_game_metadata",
    "list_game_characters",
    "save_game_characters",
    "search_vndb",
    "fetch_vn_metadata",
    "allow_local_file",
    "thumbnail",
];

//! VNDB Kana API v2 客户端，只做网络 I/O，不碰数据库。
//!
//! 响应形状的实测依据见 `docs/dev/vndb-api.md`。

use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{AppError, Result};

const API_BASE: &str = "https://api.vndb.org/kana";

/// 搜索阶段：只够判断是哪部作品，不取 description 与 tags
const SEARCH_FIELDS: &str = "id,title,alttitle,released,olang,image{id,thumbnail,thumbnail_dims}";

/// 元数据阶段：详情页真正渲染的字段。
/// 分级标记 sexual / violence 刻意不取：默认全下载，过滤留给设置项，
/// 别当成漏做过滤补上
const META_FIELDS: &str = "id,title,alttitle,olang,released,devstatus,rating,votecount,description,titles{lang,title,latin,official,main},image{id,url,dims},tags{id,name,category,rating,spoiler,lie,vn_count},developers{id,name,original,type}";

const SEARCH_RESULTS: usize = 25;

/// 刻意与 VNDB「单请求超 3 秒被服务端中断」对齐：主动放弃的错误可判断，
/// 被服务端掐断只剩一个语义不明的连接错误
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// 图片走另一个客户端。图片响应体比 API 的 JSON 大得多，
/// 3 秒那条中断线是按 API 定的
const IMAGE_TIMEOUT: Duration = Duration::from_secs(8);

/// 服务端额度是 200 次 / 5 分钟且不暴露余量，客户端取更保守的口径
const RATE_LIMIT_MAX: usize = 20;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(5 * 60);

pub fn http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("HTTP 客户端建不起来")
    })
}

pub fn image_http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(IMAGE_TIMEOUT)
            .build()
            .expect("图片客户端建不起来")
    })
}

/// 窗口内的历史请求时刻满员时睡到最早那次出窗，不忙等
pub struct RateLimiter {
    max: usize,
    window: Duration,
    history: tokio::sync::Mutex<VecDeque<Instant>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::new_with(RATE_LIMIT_MAX, RATE_LIMIT_WINDOW)
    }

    pub fn new_with(max: usize, window: Duration) -> Self {
        Self {
            max,
            window,
            history: tokio::sync::Mutex::new(VecDeque::with_capacity(max)),
        }
    }

    pub async fn acquire(&self) {
        loop {
            let wait = {
                let mut history = self.history.lock().await;
                let now = Instant::now();
                while history
                    .front()
                    .is_some_and(|t| now.duration_since(*t) >= self.window)
                {
                    history.pop_front();
                }
                if history.len() < self.max {
                    history.push_back(now);
                    return;
                }
                let since_oldest = history.front().map_or(Duration::ZERO, Instant::elapsed);
                // 睡 0 会立刻重入本轮循环变成忙等，出窗边界上要保证至少睡 1ms
                self.window
                    .saturating_sub(since_oldest)
                    .max(Duration::from_millis(1))
            };
            tokio::time::sleep(wait).await;
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
pub struct VnEnvelope<T> {
    pub results: Vec<T>,
    pub more: bool,
    pub count: Option<i64>,
}

/// 搜索档只回 thumbnail，url / dims 恒为 None
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnImage {
    pub id: Option<String>,
    pub url: Option<String>,
    pub thumbnail: Option<String>,
    pub dims: Option<Vec<i64>>,
    pub thumbnail_dims: Option<Vec<i64>>,
    /// VNDB 响应里没有这个键，由 fetch_vn 落盘后回填
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnSearchHit {
    pub id: String,
    pub title: String,
    pub alttitle: Option<String>,
    pub released: Option<String>,
    pub olang: Option<String>,
    pub image: Option<VnImage>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnTitle {
    pub lang: String,
    pub title: String,
    pub latin: Option<String>,
    pub official: Option<bool>,
    pub main: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnTag {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub rating: Option<f64>,
    pub spoiler: Option<i64>,
    pub lie: Option<bool>,
    pub vn_count: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnDeveloper {
    pub id: String,
    pub name: Option<String>,
    pub original: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
}

/// VNDB 的 null 率很高（rating 238/400 为 null、alttitle 140/400 为 null、
/// image 罕见但确有 null），所以除 id 与 title 外一律 Option
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnMetadata {
    pub id: String,
    pub title: String,
    pub alttitle: Option<String>,
    pub olang: Option<String>,
    pub released: Option<String>,
    pub devstatus: Option<i64>,
    pub rating: Option<f64>,
    pub votecount: Option<i64>,
    pub description: Option<String>,
    pub titles: Option<Vec<VnTitle>>,
    pub image: Option<VnImage>,
    pub tags: Option<Vec<VnTag>>,
    pub developers: Option<Vec<VnDeveloper>>,
}

pub fn validate_keyword(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("搜索关键词不能为空".into()));
    }
    Ok(trimmed.to_owned())
}

/// VNDB 编号形如 v14：v 前缀 + 数字
pub fn validate_vn_id(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    let digits = trimmed.strip_prefix('v').unwrap_or_default();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(AppError::BadRequest("vndb_id 必须是 v+数字".into()));
    }
    Ok(trimmed.to_owned())
}

pub async fn search(http: &reqwest::Client, keyword: &str) -> Result<Vec<VnSearchHit>> {
    // sort 必带：不带就按 id 返回，最相关的排不到前面。
    // search 过滤器只能写字符串 —— 传数组不报错，而是静默失效成无条件结果
    let body = json!({
        "filters": ["search", "=", keyword],
        "fields": SEARCH_FIELDS,
        "sort": "searchrank",
        "results": SEARCH_RESULTS,
    });
    let env: VnEnvelope<VnSearchHit> = post(http, body).await?;
    Ok(env.results)
}

pub async fn fetch_vn(
    http: &reqwest::Client,
    vndb_id: &str,
    data_dir: &std::path::Path,
) -> Result<VnMetadata> {
    let body = json!({
        "filters": ["id", "=", vndb_id],
        "fields": META_FIELDS,
        "results": 1,
    });
    let env: VnEnvelope<VnMetadata> = post(http, body).await?;
    let mut vn = env
        .results
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("VNDB 没有这条记录：{vndb_id}")))?;
    if let Some(img) = vn.image.as_mut() {
        img.local_path = fetch_image(image_http(), img, data_dir).await;
    }
    Ok(vn)
}

/// 下 url 不下 thumbnail：thumbnail 宽度不足以覆盖 2× 屏，2× 屏要 554 宽
///
/// 下载失败只记日志不返回错误 —— 图挂了不该让整次元数据同步失败
async fn fetch_image(
    http: &reqwest::Client,
    img: &mut VnImage,
    data_dir: &std::path::Path,
) -> Option<String> {
    let (Some(url), Some(id)) = (img.url.as_deref(), img.id.as_deref()) else {
        return None;
    };
    // cv 编号是 VNDB 给每张图的永久身份，换图才换号，拿它当文件名即可判断重下
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return None;
    }

    let dir = data_dir.join("covers").join("vndb");
    let dst = dir.join(format!("{id}.jpg"));
    if dst.is_file() {
        return Some(dst.to_string_lossy().into_owned());
    }
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }

    let resp = match http.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("封面下载失败 {url}：{e}");
            return None;
        }
    };
    if !resp.status().is_success() {
        eprintln!("封面下载失败 {url}：HTTP {}", resp.status());
        return None;
    }
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("封面读取失败 {url}：{e}");
            return None;
        }
    };
    if let Err(e) = std::fs::write(&dst, &bytes) {
        eprintln!("封面落盘失败 {}：{e}", dst.display());
        return None;
    }
    Some(dst.to_string_lossy().into_owned())
}

async fn post<T: for<'de> Deserialize<'de>>(
    http: &reqwest::Client,
    body: Value,
) -> Result<VnEnvelope<T>> {
    let resp = http
        .post(format!("{API_BASE}/vn"))
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Upstream(format!("VNDB 请求失败：{e}")))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| AppError::Upstream(format!("VNDB 响应读不出：{e}")))?;

    match status.as_u16() {
        200 => serde_json::from_str(&text)
            .map_err(|e| AppError::Upstream(format!("VNDB 响应不是预期 JSON：{e}"))),
        // 400 是我们请求写错了，body 里是 VNDB 指出的字段名，要带上报错现场
        400 => Err(AppError::BadRequest(format!(
            "VNDB 拒绝请求（400）：{text}"
        ))),
        429 => Err(AppError::RateLimited),
        _ => Err(AppError::Upstream(format!("VNDB 返回 {status}"))),
    }
}

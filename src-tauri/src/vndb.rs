use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{AppError, Result};
use crate::tasks::Progress;
use crate::types::{CharacterTrait, CharacterVa, GameCharacter};

const API_BASE: &str = "https://api.vndb.org/kana";

const SEARCH_FIELDS: &str = "id,title,alttitle,released,olang,image{id,thumbnail,thumbnail_dims}";

const META_FIELDS: &str = "id,title,alttitle,olang,released,devstatus,rating,votecount,description,titles{lang,title,latin,official,main},image{id,url,dims},tags{id,name,category,rating,spoiler,lie,vn_count},developers{id,name,original,type},va{character{id},staff{id,name,original},note}";

const CHAR_FIELDS: &str = "id,name,original,aliases,description,gender,age,birthday,blood_type,height,weight,bust,waist,hips,cup,image{id,url,dims},traits{id,name,group_name,spoiler,lie},vns{role,spoiler}";

const CHAR_RESULTS: usize = 100;

const SEARCH_RESULTS: usize = 25;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

const IMAGE_TIMEOUT: Duration = Duration::from_secs(8);

const RATE_LIMIT_MAX: usize = 100;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(5 * 60);

const IMAGE_CONCURRENCY: usize = 8;

const IMAGE_RETRIES: usize = 3;

const IMAGE_BACKOFF_MS: u64 = 300;

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
                match take_slot(&mut history, self.max, self.window, Instant::now()) {
                    Ok(()) => return,
                    Err(wait) => wait,
                }
            };
            tokio::time::sleep(wait).await;
        }
    }
}

const MIN_WAIT: Duration = Duration::from_millis(1);

pub fn take_slot(
    history: &mut VecDeque<Instant>,
    max: usize,
    window: Duration,
    now: Instant,
) -> std::result::Result<(), Duration> {
    while history
        .front()
        .is_some_and(|t| now.duration_since(*t) >= window)
    {
        history.pop_front();
    }
    if history.len() < max {
        history.push_back(now);
        return Ok(());
    }
    let since_oldest = history.front().map_or(Duration::ZERO, |t| now.duration_since(*t));
    Err(window.saturating_sub(since_oldest).max(MIN_WAIT))
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnImage {
    pub id: Option<String>,
    pub url: Option<String>,
    pub thumbnail: Option<String>,
    pub dims: Option<Vec<i64>>,
    pub thumbnail_dims: Option<Vec<i64>>,
    #[serde(skip_deserializing)]
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
    pub va: Option<Vec<VnVa>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnVa {
    pub note: Option<String>,
    pub staff: Option<VnVaStaff>,
    pub character: Option<VnVaCharacter>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnVaStaff {
    pub id: String,
    pub name: Option<String>,
    pub original: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnVaCharacter {
    pub id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct VnSync {
    #[serde(flatten)]
    pub vn: VnMetadata,
    pub characters: Vec<GameCharacter>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CharRow {
    id: String,
    name: Option<String>,
    original: Option<String>,
    aliases: Option<Vec<String>>,
    description: Option<String>,
    gender: Option<Vec<String>>,
    age: Option<i64>,
    birthday: Option<Vec<Option<i64>>>,
    blood_type: Option<String>,
    height: Option<i64>,
    weight: Option<i64>,
    bust: Option<i64>,
    waist: Option<i64>,
    hips: Option<i64>,
    cup: Option<String>,
    image: Option<CharImage>,
    traits: Option<Vec<CharacterTrait>>,
    vns: Option<Vec<CharVn>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CharImage {
    id: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct CharVn {
    pub role: Option<String>,
}

const SYNCED_ROLES: [&str; 2] = ["main", "primary"];

pub fn role_of(vns: Option<&[CharVn]>) -> Option<&str> {
    vns.and_then(|list| {
        list.iter()
            .find_map(|v| v.role.as_deref().filter(|role| SYNCED_ROLES.contains(role)))
    })
}

pub fn validate_keyword(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("搜索关键词不能为空".into()));
    }
    Ok(trimmed.to_owned())
}

pub fn validate_vn_id(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    let digits = trimmed.strip_prefix('v').unwrap_or_default();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(AppError::BadRequest("vndb_id 必须是 v+数字".into()));
    }
    Ok(trimmed.to_owned())
}

pub async fn search(http: &reqwest::Client, keyword: &str) -> Result<Vec<VnSearchHit>> {
    let body = json!({
        "filters": ["search", "=", keyword],
        "fields": SEARCH_FIELDS,
        "sort": "searchrank",
        "results": SEARCH_RESULTS,
    });
    let env: VnEnvelope<VnSearchHit> = post(http, "vn", body).await?;
    Ok(env.results)
}

pub async fn fetch_sync(
    http: &reqwest::Client,
    vndb_id: &str,
    data_dir: &std::path::Path,
    progress: &dyn Progress,
) -> Result<VnSync> {
    if progress.cancelled() {
        return Err(AppError::Cancelled);
    }
    progress.report("meta", None, None);
    let (vn, rows) = tokio::join!(
        fetch_vn(http, vndb_id, data_dir),
        fetch_character_rows(http, vndb_id)
    );
    let vn = vn?;
    let rows = rows?;
    if progress.cancelled() {
        return Err(AppError::Cancelled);
    }
    progress.report("characters", None, None);
    let characters = map_characters(rows, vn.va.as_deref().unwrap_or_default(), data_dir, progress).await?;
    Ok(VnSync { vn, characters })
}

async fn fetch_vn(
    http: &reqwest::Client,
    vndb_id: &str,
    data_dir: &std::path::Path,
) -> Result<VnMetadata> {
    let body = json!({
        "filters": ["id", "=", vndb_id],
        "fields": META_FIELDS,
        "results": 1,
    });
    let env: VnEnvelope<VnMetadata> = post(http, "vn", body).await?;
    let mut vn = env
        .results
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("VNDB 没有这条记录：{vndb_id}")))?;
    if let Some(img) = vn.image.as_mut() {
        img.local_path = fetch_image(
            image_http(),
            img.url.as_deref(),
            img.id.as_deref(),
            data_dir,
        )
        .await;
    }
    Ok(vn)
}

async fn fetch_character_rows(http: &reqwest::Client, vndb_id: &str) -> Result<Vec<CharRow>> {
    let body = json!({
        "filters": ["vn", "=", ["id", "=", vndb_id]],
        "fields": CHAR_FIELDS,
        "results": CHAR_RESULTS,
    });
    let env: VnEnvelope<CharRow> = post(http, "character", body).await?;
    Ok(env.results)
}

async fn map_characters(
    rows: Vec<CharRow>,
    va: &[VnVa],
    data_dir: &std::path::Path,
    progress: &dyn Progress,
) -> Result<Vec<GameCharacter>> {
    let va_by_character = index_va(va);
    let mut out = Vec::with_capacity(rows.len());
    let mut jobs = Vec::new();
    for row in rows {
        let Some(role) = role_of(row.vns.as_deref()).map(str::to_owned) else {
            continue;
        };
        if let Some((Some(url), Some(id))) = row.image.as_ref().map(|i| (&i.url, &i.id)) {
            jobs.push((out.len(), url.clone(), id.clone()));
        }
        let entry = va_by_character.get(&row.id).map(Vec::as_slice);
        out.push(to_game_character(row, role, entry));
    }
    download_character_images(out, jobs, data_dir, progress).await
}

type VaIndex = std::collections::HashMap<String, Vec<CharacterVa>>;

fn index_va(va: &[VnVa]) -> VaIndex {
    let mut index: VaIndex = std::collections::HashMap::new();
    for v in va {
        let Some(c) = v.character.as_ref().filter(|c| !c.id.is_empty()) else {
            continue;
        };
        let Some(staff) = v.staff.as_ref().filter(|s| !s.id.is_empty()) else {
            continue;
        };
        index.entry(c.id.clone()).or_default().push(CharacterVa {
            staff_id: staff.id.clone(),
            name: staff.name.clone(),
            original: staff.original.clone(),
            note: v.note.clone(),
        });
    }
    index
}

fn to_game_character(row: CharRow, role: String, va: Option<&[CharacterVa]>) -> GameCharacter {
    GameCharacter {
        id: row.id,
        name: row.name,
        original: row.original,
        aliases: row.aliases,
        description: row.description,
        gender: dedupe(row.gender.unwrap_or_default()),
        age: row.age,
        birthday: format_birthday(row.birthday),
        blood_type: row.blood_type,
        height: row.height,
        weight: row.weight,
        bust: row.bust,
        waist: row.waist,
        hips: row.hips,
        cup: row.cup,
        role: Some(role),
        image_id: row.image.as_ref().and_then(|i| i.id.clone()),
        image_path: None,
        sort_order: 0,
        traits: row.traits,
        va: va.map(<[CharacterVa]>::to_vec),
    }
}

fn format_birthday(parts: Option<Vec<Option<i64>>>) -> Option<String> {
    let joined: Vec<String> = parts
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .map(|n| n.to_string())
        .collect();
    (!joined.is_empty()).then(|| joined.join("-"))
}

fn dedupe(values: Vec<String>) -> Option<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    let out: Vec<String> = values
        .into_iter()
        .filter(|v| seen.insert(v.clone()))
        .collect();
    (!out.is_empty()).then_some(out)
}

async fn download_character_images(
    mut chars: Vec<GameCharacter>,
    jobs: Vec<(usize, String, String)>,
    data_dir: &std::path::Path,
    progress: &dyn Progress,
) -> Result<Vec<GameCharacter>> {
    let data_dir = data_dir.to_path_buf();
    let total = jobs.len() as i64;
    let mut done = 0_i64;
    let mut pending = jobs.into_iter();
    let mut set = tokio::task::JoinSet::new();
    if total > 0 {
        progress.report("images", Some(0), Some(total));
    }
    for _ in 0..IMAGE_CONCURRENCY {
        spawn_next_image(&mut set, &mut pending, &data_dir);
    }
    while let Some(finished) = set.join_next().await {
        if let Ok((idx, path)) = finished {
            chars[idx].image_path = path;
        }
        done += 1;
        progress.report("images", Some(done), Some(total));
        if progress.cancelled() {
            set.abort_all();
            return Err(AppError::Cancelled);
        }
        spawn_next_image(&mut set, &mut pending, &data_dir);
    }
    Ok(chars)
}

fn spawn_next_image(
    set: &mut tokio::task::JoinSet<(usize, Option<String>)>,
    pending: &mut std::vec::IntoIter<(usize, String, String)>,
    data_dir: &std::path::Path,
) {
    let Some((idx, url, id)) = pending.next() else {
        return;
    };
    let data_dir = data_dir.to_path_buf();
    set.spawn(async move {
        let path = fetch_image(image_http(), Some(&url), Some(&id), &data_dir).await;
        (idx, path)
    });
}

enum ImageError {
    Status(u16),
    Transport(String),
}

async fn fetch_image(
    http: &reqwest::Client,
    url: Option<&str>,
    id: Option<&str>,
    data_dir: &std::path::Path,
) -> Option<String> {
    let (Some(url), Some(id)) = (url, id) else {
        return None;
    };
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

    let mut last_err = String::new();
    for attempt in 1..=IMAGE_RETRIES {
        match download_image(http, url).await {
            Ok(bytes) => {
                if std::fs::write(&dst, &bytes).is_err() {
                    return None;
                }
                return Some(dst.to_string_lossy().into_owned());
            }
            Err(ImageError::Status(code)) => {
                eprintln!("图片下载失败 {url}：HTTP {code}");
                return None;
            }
            Err(ImageError::Transport(e)) => {
                last_err = e;
                if attempt < IMAGE_RETRIES {
                    tokio::time::sleep(Duration::from_millis(IMAGE_BACKOFF_MS * attempt as u64)).await;
                }
            }
        }
    }
    eprintln!("图片下载失败 {url}（重试 {IMAGE_RETRIES} 次）：{last_err}");
    None
}

async fn download_image(http: &reqwest::Client, url: &str) -> std::result::Result<Vec<u8>, ImageError> {
    let resp = match http.get(url).send().await {
        Ok(r) => r,
        Err(e) => return Err(ImageError::Transport(format!("{e}"))),
    };
    if !resp.status().is_success() {
        return Err(ImageError::Status(resp.status().as_u16()));
    }
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => return Err(ImageError::Transport(format!("{e}"))),
    };
    Ok(bytes.to_vec())
}

async fn post<T: for<'de> Deserialize<'de>>(
    http: &reqwest::Client,
    endpoint: &str,
    body: Value,
) -> Result<VnEnvelope<T>> {
    let resp = http
        .post(format!("{API_BASE}/{endpoint}"))
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
        400 => Err(AppError::BadRequest(format!(
            "VNDB 拒绝请求（400）：{text}"
        ))),
        429 => Err(AppError::RateLimited),
        _ => Err(AppError::Upstream(format!("VNDB 返回 {status}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, gender: Vec<&str>, roles: Vec<&str>) -> CharRow {
        CharRow {
            id: id.to_owned(),
            gender: Some(gender.into_iter().map(str::to_owned).collect()),
            vns: Some(
                roles
                    .into_iter()
                    .map(|r| CharVn {
                        role: Some(r.to_owned()),
                    })
                    .collect(),
            ),
            ..CharRow::default()
        }
    }

    #[test]
    fn gender按release重复要去掉() {
        let c = to_game_character(
            row("c1", vec!["f", "f", "m"], vec!["main"]),
            "main".into(),
            None,
        );
        assert_eq!(c.gender, Some(vec!["f".to_owned(), "m".to_owned()]));
    }

    #[test]
    fn 没有性别的角色留空而不是空数组() {
        let c = to_game_character(CharRow::default(), "main".into(), None);
        assert_eq!(c.gender, None);
    }

    #[test]
    fn birthday拍成月减日() {
        assert_eq!(
            format_birthday(Some(vec![Some(12), Some(15)])),
            Some("12-15".to_owned())
        );
    }

    #[test]
    fn birthday只有月时只留月() {
        assert_eq!(
            format_birthday(Some(vec![Some(12), None])),
            Some("12".to_owned())
        );
    }

    #[test]
    fn birthday只有日時只留日() {
        assert_eq!(
            format_birthday(Some(vec![None, Some(15)])),
            Some("15".to_owned())
        );
    }

    #[test]
    fn birthday两个分量都空时该字段留空() {
        assert_eq!(format_birthday(Some(vec![None, None])), None);
    }

    #[test]
    fn birthday整体缺失时该字段留空() {
        assert_eq!(format_birthday(None), None);
        assert_eq!(format_birthday(Some(vec![])), None);
    }

    #[test]
    fn 多条release取入库档位() {
        let r = row("c1", vec![], vec!["main", "side"]);
        assert_eq!(role_of(r.vns.as_deref()), Some("main"));
    }

    #[test]
    fn 主要角色的两档都入库() {
        for role in ["main", "primary"] {
            let r = row("c1", vec![], vec![role]);
            assert_eq!(role_of(r.vns.as_deref()), Some(role));
        }
    }

    #[test]
    fn 配角与登场不取() {
        for role in ["side", "appears"] {
            let r = row("c1", vec![], vec![role]);
            assert_eq!(role_of(r.vns.as_deref()), None);
        }
        assert_eq!(role_of(None), None);
        assert_eq!(role_of(Some(&[])), None);
    }

    #[test]
    fn 多条release里首条是配角时往后找主要档() {
        let r = row("c1", vec![], vec!["side", "primary"]);
        assert_eq!(role_of(r.vns.as_deref()), Some("primary"));
    }

    #[test]
    fn va按角色分组且保留各版本note() {
        let va = vec![
            VnVa {
                note: Some("Realta Nua".into()),
                staff: Some(VnVaStaff {
                    id: "s1".into(),
                    name: Some("彩音".into()),
                    original: None,
                }),
                character: Some(VnVaCharacter { id: "c1".into() }),
            },
            VnVa {
                note: Some("KO版".into()),
                staff: Some(VnVaStaff {
                    id: "s2".into(),
                    name: None,
                    original: None,
                }),
                character: Some(VnVaCharacter { id: "c1".into() }),
            },
        ];
        let index = index_va(&va);
        let c = to_game_character(
            row("c1", vec![], vec!["main"]),
            "main".into(),
            index.get("c1").map(Vec::as_slice),
        );
        let got = c.va.unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].note, Some("Realta Nua".into()));
        assert_eq!(got[1].staff_id, "s2");
    }

    #[test]
    fn 没有对应va的角色va留空() {
        let c = to_game_character(row("c1", vec![], vec!["primary"]), "primary".into(), None);
        assert_eq!(c.va, None);
        assert_eq!(c.sort_order, 0);
    }

    #[test]
    fn 缺声优的va条目不留空编号() {
        let va = vec![
            VnVa {
                note: None,
                staff: None,
                character: Some(VnVaCharacter { id: "c1".into() }),
            },
            VnVa {
                note: None,
                staff: Some(VnVaStaff::default()),
                character: Some(VnVaCharacter { id: "c2".into() }),
            },
        ];
        assert!(index_va(&va).is_empty());
    }
}

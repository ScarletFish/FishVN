//! VNDB Kana API v2 客户端，只做网络 I/O，不碰数据库。
//!
//! 响应形状的实测依据见 `docs/dev/vndb-api.md`。

use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{AppError, Result};
use crate::types::{CharacterTrait, CharacterVa, GameCharacter};

const API_BASE: &str = "https://api.vndb.org/kana";

/// 搜索阶段：只够判断是哪部作品，不取 description 与 tags
const SEARCH_FIELDS: &str = "id,title,alttitle,released,olang,image{id,thumbnail,thumbnail_dims}";

/// 元数据阶段：详情页真正渲染的字段。
/// 分级标记 sexual / violence 刻意不取：默认全下载，过滤留给设置项，
/// 别当成漏做过滤补上
/// va 只列已配音的角色，且 /character 上没有声优 —— CV 只能从这里取
const META_FIELDS: &str = "id,title,alttitle,olang,released,devstatus,rating,votecount,description,titles{lang,title,latin,official,main},image{id,url,dims},tags{id,name,category,rating,spoiler,lie,vn_count},developers{id,name,original,type},va{character{id},staff{id,name,original},note}";

/// traits 的 name / group_name 继承自 /trait：fields 里写出 traits{id,name,group_name}
/// 才带回名字
const CHAR_FIELDS: &str = "id,name,original,aliases,description,gender,age,birthday,blood_type,height,weight,bust,waist,hips,cup,image{id,url,dims},traits{id,name,group_name,spoiler,lie},vns{role,spoiler}";

/// /character 的 results 有上限，超过要另翻页
const CHAR_RESULTS: usize = 100;

const SEARCH_RESULTS: usize = 25;

/// 刻意与 VNDB「单请求超 3 秒被服务端中断」对齐：主动放弃的错误可判断，
/// 被服务端掐断只剩一个语义不明的连接错误
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// 图片走另一个客户端。图片响应体比 API 的 JSON 大得多，
/// 3 秒那条中断线是按 API 定的
const IMAGE_TIMEOUT: Duration = Duration::from_secs(8);

/// 服务端额度是 200 次 / 5 分钟，余量无从查询，客户端按一半取留出裕量
const RATE_LIMIT_MAX: usize = 100;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(5 * 60);

/// 立绘走 t.vndb.org 这个 CDN，与 api.vndb.org 的限流额度是两套，
/// 并发数只受本机带宽与 CDN 礼貌限速约束
const IMAGE_CONCURRENCY: usize = 8;

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

/// 名额到手给 Ok；满了给 Err(还要等多久)
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

/// 搜索档只回 thumbnail，url / dims 恒为 None
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
    pub va: Option<Vec<VnVa>>,
}

/// /vn 的 va 元素。note 是版本标记（如 Realta Nua），同一角色不同版本各有条，
/// 按原样保留，合并会丢掉「哪个版本换了声优」的信息
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

/// VNDB 没有一次请求能给全两者，所以这是两次请求的合并结果；
/// 角色集由前端逐个确认后走 save_game_characters 落库
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
    /// `[月, 日]`。这个形状只有实测能确认：`/kana/schema` 对它不给类型声明，
    /// 与 `released` 同性质。照直觉写成标量会让整个 `/character` 响应反序列化失败，
    /// 整次同步报「响应不是预期 JSON」
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

/// side/appears 里混着无立绘的配角，收进来只是在卡条里
/// 多一排占位块，还要为它们多下一批图
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
    let env: VnEnvelope<VnSearchHit> = post(http, "vn", body).await?;
    Ok(env.results)
}

/// 作品元数据与角色集任一失败整体报错：只填上一半会让调用方
/// 以为同步已经完成，用户看到的却是空角色列表
pub async fn fetch_sync(
    http: &reqwest::Client,
    vndb_id: &str,
    data_dir: &std::path::Path,
) -> Result<VnSync> {
    let (vn, rows) = tokio::join!(
        fetch_vn(http, vndb_id, data_dir),
        fetch_character_rows(http, vndb_id)
    );
    let vn = vn?;
    let rows = rows?;
    let characters = map_characters(rows, vn.va.as_deref().unwrap_or_default(), data_dir).await;
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

/// vn 过滤器只吃这个嵌套形状，写平成 ["id","=",v] 是 400
async fn fetch_character_rows(http: &reqwest::Client, vndb_id: &str) -> Result<Vec<CharRow>> {
    let body = json!({
        "filters": ["vn", "=", ["id", "=", vndb_id]],
        "fields": CHAR_FIELDS,
        "results": CHAR_RESULTS,
    });
    let env: VnEnvelope<CharRow> = post(http, "character", body).await?;
    Ok(env.results)
}

/// /vn 的 va 只列已配音的角色，角色行统一由 /character 产出：在 va 里出现、
/// 却没进 /character 的角色，在本作品里就按没有立绘与资料处理
async fn map_characters(
    rows: Vec<CharRow>,
    va: &[VnVa],
    data_dir: &std::path::Path,
) -> Vec<GameCharacter> {
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
    download_character_images(out, jobs, data_dir).await
}

type VaIndex = std::collections::HashMap<String, Vec<CharacterVa>>;

fn index_va(va: &[VnVa]) -> VaIndex {
    let mut index: VaIndex = std::collections::HashMap::new();
    for v in va {
        let Some(c) = v.character.as_ref().filter(|c| !c.id.is_empty()) else {
            continue;
        };
        // 缺声优的条目直接跳过：落库只会多出一位无名声优
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

/// `[月, 日]` 拍平成 `月-日`；只公布到月或只公布到日时存已知的那个分量，
/// 两边都没有则该字段留空
fn format_birthday(parts: Option<Vec<Option<i64>>>) -> Option<String> {
    let joined: Vec<String> = parts
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .map(|n| n.to_string())
        .collect();
    (!joined.is_empty()).then(|| joined.join("-"))
}

/// gender 按 release 逐条返回，同一角色多版本会出重复值
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
) -> Vec<GameCharacter> {
    let data_dir = data_dir.to_path_buf();
    let mut pending = jobs.into_iter();
    let mut set = tokio::task::JoinSet::new();
    for _ in 0..IMAGE_CONCURRENCY {
        spawn_next_image(&mut set, &mut pending, &data_dir);
    }
    while let Some(done) = set.join_next().await {
        if let Ok((idx, path)) = done {
            chars[idx].image_path = path;
        }
        spawn_next_image(&mut set, &mut pending, &data_dir);
    }
    chars
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

/// 下 url 不下 thumbnail：thumbnail 宽度不足以覆盖 2× 屏，2× 屏要 554 宽
///
/// 下载失败只记日志不返回错误 —— 图挂了不该让整次元数据同步失败
async fn fetch_image(
    http: &reqwest::Client,
    url: Option<&str>,
    id: Option<&str>,
    data_dir: &std::path::Path,
) -> Option<String> {
    let (Some(url), Some(id)) = (url, id) else {
        return None;
    };
    // 图编号（VN 封面 cv / 角色 ch）是 VNDB 给每张图的永久身份，换图才换号，
    // 拿它当文件名即可判断重下
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
            eprintln!("图片下载失败 {url}：{e}");
            return None;
        }
    };
    if !resp.status().is_success() {
        eprintln!("图片下载失败 {url}：HTTP {}", resp.status());
        return None;
    }
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("图片读取失败 {url}：{e}");
            return None;
        }
    };
    if let Err(e) = std::fs::write(&dst, &bytes) {
        eprintln!("图片落盘失败 {}：{e}", dst.display());
        return None;
    }
    Some(dst.to_string_lossy().into_owned())
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
        // 400 是我们请求写错了，body 里是 VNDB 指出的字段名，要带上报错现场
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

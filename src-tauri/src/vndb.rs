use serde_json::json;
use tauri_plugin_log::log;

use crate::error::{AppError, Result};
use crate::tasks::Progress;
use crate::types::Status;

mod characters;
mod client;
mod dto;
mod images;
mod titles;

pub use characters::role_in;
pub use titles::{chain, ulist_chain};
pub use client::{http, take_slot, RateLimiter};
pub use dto::{
    CharVn, ImageJob, ImageKind, UlistEntry, UlistLabel, UlistVn, VnDeveloper, VnEnvelope, VnImage,
    VnMetadata, VnSearchHit, VnSync, VnTag, VnTitle, VnVa, VnVaCharacter, VnVaStaff,
};
pub(crate) use images::{download_images, IMAGE_CONCURRENCY};

pub(crate) fn image_client() -> &'static reqwest::Client {
    client::image_http()
}
use dto::{AuthInfo, CharRow, LabelList};

const SEARCH_FIELDS: &str = "id,title,alttitle,released,olang,image{id,thumbnail,thumbnail_dims}";

const META_FIELDS: &str = "id,title,alttitle,olang,released,devstatus,rating,votecount,length_minutes,description,titles{lang,title,latin,official,main},image{id,url,dims,thumbnail},tags{id,name,category,rating,spoiler,lie,vn_count},developers{id,name,original,type},staff{role,name,original,note},va{character{id},staff{id,name,original},note}";

const CHAR_FIELDS: &str = "id,name,original,aliases,description,gender,age,birthday,blood_type,height,weight,bust,waist,hips,cup,image{id,url,dims},traits{id,name,group_name,spoiler,lie},vns{id,role,spoiler}";

const CHAR_RESULTS: usize = 100;
const CHAR_MAX_PAGES: usize = 12;

const CHAR_GROUP: usize = 10;

pub const BATCH_SIZE: usize = 20;

const SEARCH_RESULTS: usize = 25;

const ULIST_FIELDS: &str = "vn.title,vn.alttitle,vn.olang,vn.titles{lang,title,official},labels{id,label},vote,lastmod";

const ULIST_RESULTS: usize = 100;

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
    let env: VnEnvelope<VnSearchHit> = client::post(http, "vn", body, None).await?;
    Ok(env.results)
}

pub async fn fetch_sync(
    http: &reqwest::Client,
    game_id: &str,
    vndb_id: &str,
    progress: &dyn Progress,
) -> Result<(VnSync, Vec<ImageJob>)> {
    if progress.cancelled() {
        return Err(AppError::Cancelled);
    }
    progress.report("meta", None, None);
    let meta_start = std::time::Instant::now();
    let (vn, rows) = tokio::select! {
        res = async { tokio::join!(fetch_vn(http, vndb_id), fetch_character_rows(http, vndb_id)) } => res,
        _ = crate::tasks::wait_cancel(progress) => {
            return Err(AppError::Cancelled);
        }
    };
    let meta_ms = meta_start.elapsed().as_millis();
    let vn = vn?;
    let rows = rows?;
    if progress.cancelled() {
        return Err(AppError::Cancelled);
    }
    progress.report_note("characters", &format!("{} 位", rows.len()));
    let chars_start = std::time::Instant::now();
    let chars_expected = rows.len();
    let scope = crate::settings::current().character_scope;
    let (characters, images) =
        characters::map_characters(rows, vn.va.as_deref().unwrap_or_default(), vndb_id, scope);
    let jobs = sync_jobs(game_id, &vn, &images);
    log::debug!(
        target: "backfill",
        "请求分解：元数据 {meta_ms}ms 角色 {}ms（{chars_expected} 位）",
        chars_start.elapsed().as_millis()
    );
    Ok((VnSync { vn, characters }, jobs))
}

fn cover_job(game_id: &str, vn: &VnMetadata) -> Option<ImageJob> {
    let img = vn.image.as_ref()?;
    let (Some(url), Some(id)) = (img.url.as_deref(), img.id.as_deref()) else {
        return None;
    };
    Some(ImageJob {
        game_id: game_id.to_owned(),
        kind: ImageKind::Cover,
        image_id: id.to_owned(),
        url: url.to_owned(),
        thumb: img.thumbnail.clone(),
    })
}

fn sync_jobs(game_id: &str, vn: &VnMetadata, images: &[characters::CharacterImage]) -> Vec<ImageJob> {
    let mut jobs: Vec<ImageJob> = cover_job(game_id, vn).into_iter().collect();
    for image in images {
        jobs.push(ImageJob {
            game_id: game_id.to_owned(),
            kind: ImageKind::Character {
                character_id: image.character_id.clone(),
            },
            image_id: image.image_id.clone(),
            url: image.url.clone(),
            thumb: None,
        });
    }
    jobs
}

#[derive(Debug, Clone)]
pub struct BatchItem {
    pub game_id: String,
    pub vndb_id: String,
}

#[derive(Debug, Clone)]
pub struct BatchSync {
    pub game_id: String,
    pub vndb_id: String,
    pub sync: VnSync,
    pub jobs: Vec<ImageJob>,
}

pub async fn fetch_batch(
    http: &reqwest::Client,
    rate: &RateLimiter,
    items: &[BatchItem],
    progress: &dyn Progress,
) -> Result<(Vec<BatchSync>, Vec<String>)> {
    if items.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    progress.report("meta", None, None);
    let started = std::time::Instant::now();
    let ids: Vec<String> = items.iter().map(|item| item.vndb_id.clone()).collect();

    rate.acquire().await;
    let vn_future = async {
        let begun = std::time::Instant::now();
        let result = fetch_vns_batch(http, &ids).await;
        (result, begun.elapsed().as_millis())
    };
    rate.acquire().await;
    let char_future = async {
        let begun = std::time::Instant::now();
        let result = fetch_characters_batch(http, rate, &ids).await;
        (result, begun.elapsed().as_millis())
    };

    let ((vn_result, vn_ms), (char_result, char_ms)) = tokio::join!(vn_future, char_future);
    let found: std::collections::HashMap<String, VnMetadata> = vn_result?
        .into_iter()
        .map(|vn| (vn.id.clone(), vn))
        .collect();
    let rows = char_result?;
    let (synced, missing) = assemble(items, found, rows);
    log::debug!(
        target: "backfill",
        "批量取数：{} 部（缺 {}）总 {}ms（vn {vn_ms}ms 与角色 {char_ms}ms 并发）",
        synced.len(),
        missing.len(),
        started.elapsed().as_millis()
    );
    Ok((synced, missing))
}

fn assemble(
    items: &[BatchItem],
    mut found: std::collections::HashMap<String, VnMetadata>,
    rows: Vec<dto::CharRow>,
) -> (Vec<BatchSync>, Vec<String>) {
    let mut by_vn: std::collections::HashMap<&str, Vec<usize>> = std::collections::HashMap::new();
    for (idx, row) in rows.iter().enumerate() {
        for vn in row.vns.iter().flatten() {
            if let Some(id) = vn.id.as_deref() {
                by_vn.entry(id).or_default().push(idx);
            }
        }
    }
    for bucket in by_vn.values_mut() {
        bucket.sort_unstable();
        bucket.dedup();
    }

    let scope = crate::settings::current().character_scope;
    let mut out = Vec::with_capacity(items.len());
    let mut missing = Vec::new();
    for item in items {
        let Some(vn) = found.remove(&item.vndb_id) else {
            missing.push(item.vndb_id.clone());
            continue;
        };
        let picked: Vec<dto::CharRow> = by_vn
            .get(item.vndb_id.as_str())
            .map(|idx| idx.iter().map(|i| rows[*i].clone()).collect())
            .unwrap_or_default();
        let (characters, images) = characters::map_characters(
            picked,
            vn.va.as_deref().unwrap_or_default(),
            &item.vndb_id,
            scope,
        );
        log::debug!(
            target: "backfill",
            "组装：{} {} 位角色 {} 张图",
            item.vndb_id,
            characters.len(),
            images.len()
        );
        let jobs = sync_jobs(&item.game_id, &vn, &images);
        out.push(BatchSync {
            game_id: item.game_id.clone(),
            vndb_id: item.vndb_id.clone(),
            sync: VnSync { vn, characters },
            jobs,
        });
    }
    (out, missing)
}

fn or_filters(ids: &[String]) -> serde_json::Value {
    let mut filters = Vec::with_capacity(ids.len() + 1);
    filters.push(json!("or"));
    for id in ids {
        filters.push(json!(["id", "=", id]));
    }
    serde_json::Value::Array(filters)
}

async fn fetch_vns_batch(http: &reqwest::Client, ids: &[String]) -> Result<Vec<VnMetadata>> {
    let body = json!({
        "filters": or_filters(ids),
        "fields": META_FIELDS,
        "results": ids.len().min(CHAR_RESULTS),
    });
    let env: VnEnvelope<VnMetadata> = client::post_batch(http, "vn", body).await?;
    Ok(env.results)
}

async fn fetch_characters_batch(
    http: &reqwest::Client,
    rate: &RateLimiter,
    ids: &[String],
) -> Result<Vec<dto::CharRow>> {
    let groups: Vec<_> = ids
        .chunks(CHAR_GROUP)
        .map(|group| fetch_character_group(http, rate, group))
        .collect();
    let mut fetched = Vec::with_capacity(groups.len());
    for group in futures_util::future::join_all(groups).await {
        fetched.push(group?);
    }
    Ok(merge_rows(fetched))
}

fn merge_rows(groups: Vec<Vec<dto::CharRow>>) -> Vec<dto::CharRow> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for row in groups.into_iter().flatten() {
        if seen.insert(row.id.clone()) {
            out.push(row);
        }
    }
    out
}

async fn fetch_character_group(
    http: &reqwest::Client,
    rate: &RateLimiter,
    ids: &[String],
) -> Result<Vec<dto::CharRow>> {
    let filters = json!(["vn", "=", or_filters(ids)]);
    let mut out = Vec::new();
    for page in 1..=CHAR_MAX_PAGES {
        if page > 1 {
            rate.acquire().await;
        }
        let body = json!({
            "filters": filters,
            "fields": CHAR_FIELDS,
            "results": CHAR_RESULTS,
            "page": page,
        });
        let env: VnEnvelope<dto::CharRow> = client::post_batch(http, "character", body).await?;
        let more = env.more;
        out.extend(env.results);
        if !more {
            return Ok(out);
        }
    }
    log::warn!(target: "backfill", "角色行超过 {CHAR_MAX_PAGES} 页，只取前 {CHAR_MAX_PAGES} 页");
    Ok(out)
}

pub(crate) fn fill_image_paths(sync: &mut VnSync, results: &[(ImageJob, Option<String>)]) {
    for (job, path) in results {
        let Some(path) = path else { continue };
        match &job.kind {
            ImageKind::Cover => {
                if let Some(img) = sync.vn.image.as_mut() {
                    img.local_path = Some(path.clone());
                }
            }
            ImageKind::Character { character_id } => {
                if let Some(character) = sync.characters.iter_mut().find(|c| &c.id == character_id)
                {
                    character.image_path = Some(path.clone());
                }
            }
        }
    }
}

async fn fetch_vn(http: &reqwest::Client, vndb_id: &str) -> Result<VnMetadata> {
    let body = json!({
        "filters": ["id", "=", vndb_id],
        "fields": META_FIELDS,
        "results": 1,
    });
    let env: VnEnvelope<VnMetadata> = client::post(http, "vn", body, None).await?;
    let vn = env
        .results
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("VNDB 没有这条记录：{vndb_id}")))?;
    Ok(vn)
}

async fn fetch_character_rows(http: &reqwest::Client, vndb_id: &str) -> Result<Vec<CharRow>> {
    let body = json!({
        "filters": ["vn", "=", ["id", "=", vndb_id]],
        "fields": CHAR_FIELDS,
        "results": CHAR_RESULTS,
    });
    let env: VnEnvelope<CharRow> = client::post(http, "character", body, None).await?;
    Ok(env.results)
}

pub async fn test_token(http: &reqwest::Client, token: &str) -> Result<bool> {
    let body = json!({
        "fields": "id",
        "results": 1,
    });
    match client::post::<serde_json::Value>(http, "ulist", body, Some(token)).await {
        Ok(_) => Ok(true),
        Err(AppError::BadRequest(_)) => Ok(false),
        Err(other) => Err(other),
    }
}

pub async fn fetch_ulist(
    http: &reqwest::Client,
    token: &str,
    rate: &RateLimiter,
    on_page: impl Fn(usize, Option<i64>),
) -> Result<Vec<UlistEntry>> {
    let mut entries = Vec::new();
    let mut page = 1;
    loop {
        rate.acquire().await;
        let body = json!({
            "fields": ULIST_FIELDS,
            "results": ULIST_RESULTS,
            "page": page,
        });
        let env: VnEnvelope<UlistEntry> = client::post(http, "ulist", body, Some(token)).await?;
        let more = env.more;
        let total = env.count;
        if env.results.is_empty() {
            break;
        }
        entries.extend(env.results);
        on_page(entries.len(), total);
        if !more {
            break;
        }
        page += 1;
    }
    Ok(entries)
}

pub fn status_of_label(name: &str) -> Option<Status> {
    match name.to_ascii_lowercase().as_str() {
        "playing" => Some(Status::Playing),
        "finished" => Some(Status::Finished),
        "stalled" => Some(Status::Stalled),
        "dropped" => Some(Status::Dropped),
        "wishlist" | "planning" | "plan to play" => Some(Status::Wishlist),
        _ => None,
    }
}

fn status_zh(status: Status) -> &'static str {
    match status {
        Status::Wishlist => "计划中",
        Status::Playing => "游玩中",
        Status::Finished => "已完成",
        Status::Stalled => "搁置",
        Status::Dropped => "已弃坑",
    }
}

pub struct LabelMap {
    pairs: Vec<(i64, Status)>,
}

impl LabelMap {
    pub fn id_for(&self, status: Status) -> Option<i64> {
        self.pairs
            .iter()
            .find(|(_, s)| *s == status)
            .map(|(id, _)| *id)
    }

    fn other_ids(&self, status: Status) -> Vec<i64> {
        self.pairs
            .iter()
            .filter(|(_, s)| *s != status)
            .map(|(id, _)| *id)
            .collect()
    }
}

pub async fn fetch_label_map(http: &reqwest::Client, token: &str) -> Result<LabelMap> {
    let list: LabelList = client::get(http, "ulist_labels", Some(token)).await?;
    let pairs: Vec<(i64, Status)> = list
        .labels
        .iter()
        .filter_map(|l| status_of_label(&l.label).map(|s| (l.id, s)))
        .collect();
    if pairs.is_empty() {
        return Err(AppError::BadRequest(
            "VNDB 没有返回可用的状态标签，检查 Token 是否带 listread 权限".into(),
        ));
    }
    Ok(LabelMap { pairs })
}

pub async fn has_listwrite(http: &reqwest::Client, token: &str) -> Result<bool> {
    let info: AuthInfo = client::get(http, "authinfo", Some(token)).await?;
    Ok(info
        .permissions
        .iter()
        .any(|p| p.eq_ignore_ascii_case("listwrite")))
}

pub async fn set_ulist(
    http: &reqwest::Client,
    token: &str,
    vndb_id: &str,
    labels: &LabelMap,
    status: Option<Status>,
    rating: Option<Option<f64>>,
) -> Result<()> {
    let mut body = serde_json::Map::new();
    if let Some(status) = status {
        let id = labels.id_for(status).ok_or_else(|| {
            AppError::BadRequest(format!(
                "VNDB 上找不到「{}」对应的状态标签，这次没有写入",
                status_zh(status)
            ))
        })?;
        body.insert("labels_set".into(), json!([id]));
        let others = labels.other_ids(status);
        if !others.is_empty() {
            body.insert("labels_unset".into(), json!(others));
        }
    }
    if let Some(rating) = rating {
        body.insert(
            "vote".into(),
            match rating {
                Some(v) => json!(v.round().clamp(10.0, 100.0) as i64),
                None => serde_json::Value::Null,
            },
        );
    }
    if body.is_empty() {
        return Ok(());
    }
    client::patch(
        http,
        &format!("ulist/{vndb_id}"),
        serde_json::Value::Object(body),
        token,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: &str) -> VnMetadata {
        VnMetadata {
            id: id.to_owned(),
            title: format!("作品 {id}"),
            ..Default::default()
        }
    }

    fn char_row(id: &str, roles: &[(&str, &str)]) -> dto::CharRow {
        dto::CharRow {
            id: id.to_owned(),
            name: Some(format!("角色 {id}")),
            vns: Some(
                roles
                    .iter()
                    .map(|(vn, role)| dto::CharVn {
                        id: Some((*vn).to_owned()),
                        role: Some((*role).to_owned()),
                    })
                    .collect(),
            ),
            ..Default::default()
        }
    }

    fn found(vns: Vec<VnMetadata>) -> std::collections::HashMap<String, VnMetadata> {
        vns.into_iter().map(|vn| (vn.id.clone(), vn)).collect()
    }

    fn char_scalar_fields() -> Vec<&'static str> {
        let mut depth = 0usize;
        let mut start = 0usize;
        let mut out = Vec::new();
        for (i, ch) in CHAR_FIELDS.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                ',' if depth == 0 => {
                    let part = &CHAR_FIELDS[start..i];
                    if !part.is_empty() && !part.contains('{') {
                        out.push(part);
                    }
                    start = i + 1;
                }
                _ => {}
            }
        }
        let tail = &CHAR_FIELDS[start..];
        if !tail.is_empty() && !tail.contains('{') {
            out.push(tail);
        }
        out
    }

    #[test]
    fn 角色请求字段与规格里的上游标量字段一致() {
        use crate::generated::characters::CHARACTER_FIELDS as SPECS;

        let nested = ["role", "image_id", "image_path", "sort_order"];
        let mut expected = SPECS
            .iter()
            .filter(|f| f.in_columns && !nested.contains(&f.key))
            .map(|f| f.key)
            .collect::<Vec<_>>();
        expected.sort_unstable();

        let mut actual = char_scalar_fields();
        actual.sort_unstable();
        assert_eq!(actual, expected);

        for group in [
            "image{id,url,dims}",
            "traits{id,name,group_name,spoiler,lie}",
            "vns{id,role,spoiler}",
        ] {
            assert!(CHAR_FIELDS.contains(group), "CHAR_FIELDS 漏了 {group}");
        }
    }

    #[test]
    fn ulist字段串覆盖实抓响应的标题字段() {
        const FIXTURE: &str = include_str!("../tests/fixtures/ulist.json");
        let payload: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
        let vn = &payload["results"][0]["vn"];
        for (key, field) in [
            ("title", "vn.title"),
            ("alttitle", "vn.alttitle"),
            ("olang", "vn.olang"),
            ("titles", "vn.titles{lang,title,official}"),
        ] {
            assert!(ULIST_FIELDS.contains(field), "ULIST_FIELDS 漏了 {field}");
            assert!(vn.get(key).is_some(), "实抓响应里没有 {key}");
        }
        for key in ["lang", "title", "official"] {
            assert!(vn["titles"][0].get(key).is_some(), "titles 缺 {key}");
        }
        for field in ["labels{id,label}", "vote", "lastmod"] {
            assert!(ULIST_FIELDS.contains(field), "ULIST_FIELDS 漏了 {field}");
        }
    }

    #[test]
    fn 缺失的作品单独列出且不影响其余条目() {
        let items = vec![
            BatchItem {
                game_id: "g1".into(),
                vndb_id: "v1".into(),
            },
            BatchItem {
                game_id: "g2".into(),
                vndb_id: "v404".into(),
            },
            BatchItem {
                game_id: "g3".into(),
                vndb_id: "v3".into(),
            },
        ];
        let (synced, missing) = assemble(&items, found(vec![meta("v1"), meta("v3")]), Vec::new());

        assert_eq!(
            synced
                .iter()
                .map(|s| s.game_id.as_str())
                .collect::<Vec<_>>(),
            ["g1", "g3"]
        );
        assert_eq!(missing, vec!["v404".to_owned()]);
    }

    #[test]
    fn 角色只进本作品是主要档的那一部() {
        let items = vec![
            BatchItem {
                game_id: "g1".into(),
                vndb_id: "v1".into(),
            },
            BatchItem {
                game_id: "g2".into(),
                vndb_id: "v2".into(),
            },
        ];
        let rows = vec![char_row("c1", &[("v1", "side"), ("v2", "main")])];

        let (synced, _) = assemble(&items, found(vec![meta("v1"), meta("v2")]), rows);

        assert!(synced[0].sync.characters.is_empty());
        assert_eq!(synced[1].sync.characters.len(), 1);
        assert_eq!(
            synced[1].sync.characters[0].role.as_deref(),
            Some("main")
        );
    }

    #[test]
    fn 一部作品的多个角色互不串台() {
        let items = vec![
            BatchItem {
                game_id: "g1".into(),
                vndb_id: "v1".into(),
            },
            BatchItem {
                game_id: "g2".into(),
                vndb_id: "v2".into(),
            },
        ];
        let rows = vec![
            char_row("c1", &[("v1", "main")]),
            char_row("c2", &[("v2", "main")]),
            char_row("c3", &[("v1", "primary"), ("v2", "side")]),
        ];

        let (synced, _) = assemble(&items, found(vec![meta("v1"), meta("v2")]), rows);

        let first: Vec<&str> = synced[0]
            .sync
            .characters
            .iter()
            .map(|c| c.id.as_str())
            .collect();
        let second: Vec<&str> = synced[1]
            .sync
            .characters
            .iter()
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(first, ["c1", "c3"]);
        assert_eq!(second, ["c2"]);
    }

    #[test]
    fn 同一角色在同一作品有多条版本记录时只算一次() {
        let items = vec![BatchItem {
            game_id: "g1".into(),
            vndb_id: "v1".into(),
        }];
        let rows = vec![char_row("c1", &[("v1", "side"), ("v1", "main")])];

        let (synced, _) = assemble(&items, found(vec![meta("v1")]), rows);

        assert_eq!(synced[0].sync.characters.len(), 1);
        assert_eq!(
            synced[0].sync.characters[0].role.as_deref(),
            Some("main")
        );
    }

    #[test]
    fn 多个分组重复返回的角色只保留一条() {
        let shared = char_row("c1", &[("v1", "main"), ("v2", "main")]);
        let merged = merge_rows(vec![vec![shared.clone()], vec![shared]]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id, "c1");
    }

    #[test]
    fn 分组边界上共享的角色不会在同一作品里出现两次() {
        let items = vec![
            BatchItem {
                game_id: "g1".into(),
                vndb_id: "v1".into(),
            },
            BatchItem {
                game_id: "g2".into(),
                vndb_id: "v2".into(),
            },
        ];
        let shared = char_row("c1", &[("v1", "main"), ("v2", "main")]);
        let rows = merge_rows(vec![vec![shared.clone()], vec![shared]]);

        let (synced, _) = assemble(&items, found(vec![meta("v1"), meta("v2")]), rows);

        for entry in &synced {
            let ids: Vec<&str> = entry
                .sync
                .characters
                .iter()
                .map(|c| c.id.as_str())
                .collect();
            assert_eq!(ids, ["c1"]);
        }
    }

    #[test]
    fn 图片任务带上作品与角色归属() {
        let items = vec![BatchItem {
            game_id: "g1".into(),
            vndb_id: "v1".into(),
        }];
        let mut vn = meta("v1");
        vn.image = Some(VnImage {
            id: Some("cv1".into()),
            url: Some("https://t.vndb.org/cv/01/1.jpg".into()),
            thumbnail: Some("https://t.vndb.org/cv.t/01/1.jpg".into()),
            ..Default::default()
        });
        let mut row = char_row("c1", &[("v1", "main")]);
        row.image = Some(dto::CharImage {
            id: Some("ch1".into()),
            url: Some("https://t.vndb.org/ch/01/1.jpg".into()),
        });

        let (synced, _) = assemble(&items, found(vec![vn]), vec![row]);

        let jobs = &synced[0].jobs;
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].game_id, "g1");
        assert!(matches!(jobs[0].kind, ImageKind::Cover));
        assert_eq!(jobs[0].image_id, "cv1");
        assert!(matches!(
            jobs[1].kind,
            ImageKind::Character { ref character_id } if character_id == "c1"
        ));
        assert_eq!(jobs[1].image_id, "ch1");
    }

    #[test]
    fn 没有封面信息的作品不下封面() {
        let items = vec![BatchItem {
            game_id: "g1".into(),
            vndb_id: "v1".into(),
        }];
        let (synced, _) = assemble(&items, found(vec![meta("v1")]), Vec::new());
        assert!(synced[0].jobs.is_empty());
    }
}

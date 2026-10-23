use serde_json::json;

use crate::error::{AppError, Result};
use crate::tasks::Progress;

mod characters;
mod client;
mod dto;
mod images;

pub use characters::role_of;
pub use client::{http, take_slot, RateLimiter};
pub use dto::{
    CharVn, UlistEntry, UlistLabel, UlistVn, VnDeveloper, VnEnvelope, VnImage, VnMetadata, VnSearchHit,
    VnSync, VnTag, VnTitle, VnVa, VnVaCharacter, VnVaStaff,
};

use dto::CharRow;

const SEARCH_FIELDS: &str = "id,title,alttitle,released,olang,image{id,thumbnail,thumbnail_dims}";

const META_FIELDS: &str = "id,title,alttitle,olang,released,devstatus,rating,votecount,length_minutes,description,titles{lang,title,latin,official,main},image{id,url,dims},tags{id,name,category,rating,spoiler,lie,vn_count},developers{id,name,original,type},staff{role,name,original,note},va{character{id},staff{id,name,original},note}";

const CHAR_FIELDS: &str = "id,name,original,aliases,description,gender,age,birthday,blood_type,height,weight,bust,waist,hips,cup,image{id,url,dims},traits{id,name,group_name,spoiler,lie},vns{role,spoiler}";

const CHAR_RESULTS: usize = 100;

const SEARCH_RESULTS: usize = 25;

const ULIST_FIELDS: &str = "vn.id,labels{id,label},vote,lastmod";

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
    vndb_id: &str,
    data_dir: &std::path::Path,
    progress: &dyn Progress,
) -> Result<VnSync> {
    if progress.cancelled() {
        return Err(AppError::Cancelled);
    }
    progress.report("meta", None, None);
    let (vn, rows) = tokio::join!(fetch_vn(http, vndb_id), fetch_character_rows(http, vndb_id));
    let mut vn = vn?;
    let rows = rows?;
    if progress.cancelled() {
        return Err(AppError::Cancelled);
    }
    if let Some(img) = vn.image.as_mut() {
        if img.url.is_some() && img.id.is_some() {
            progress.report("cover", None, None);
            if progress.cancelled() {
                return Err(AppError::Cancelled);
            }
            img.local_path = images::fetch_image(
                client::image_http(),
                img.url.as_deref(),
                img.id.as_deref(),
                data_dir,
            )
            .await;
            if img.local_path.is_none() {
                progress.report_error("cover", "封面下载失败，可稍后在详情页重新获取");
            }
        }
    }
    if progress.cancelled() {
        return Err(AppError::Cancelled);
    }
    progress.report("characters", None, None);
    let characters = characters::map_characters(
        rows,
        vn.va.as_deref().unwrap_or_default(),
        data_dir,
        progress,
    )
    .await?;
    Ok(VnSync { vn, characters })
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

pub async fn fetch_ulist(http: &reqwest::Client, token: &str) -> Result<Vec<UlistEntry>> {
    let mut entries = Vec::new();
    let mut page = 1;
    loop {
        let body = json!({
            "fields": ULIST_FIELDS,
            "results": ULIST_RESULTS,
            "page": page,
        });
        let env: VnEnvelope<UlistEntry> = client::post(http, "ulist", body, Some(token)).await?;
        let more = env.more;
        if env.results.is_empty() {
            break;
        }
        entries.extend(env.results);
        if !more {
            break;
        }
        page += 1;
    }
    Ok(entries)
}

use serde_json::json;

use crate::error::{AppError, Result};
use crate::tasks::Progress;
use crate::types::Status;

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
use dto::{AuthInfo, CharRow, LabelList};

const SEARCH_FIELDS: &str = "id,title,alttitle,released,olang,image{id,thumbnail,thumbnail_dims}";

const META_FIELDS: &str = "id,title,alttitle,olang,released,devstatus,rating,votecount,length_minutes,description,titles{lang,title,latin,official,main},image{id,url,dims},tags{id,name,category,rating,spoiler,lie,vn_count},developers{id,name,original,type},staff{role,name,original,note},va{character{id},staff{id,name,original},note}";

const CHAR_FIELDS: &str = "id,name,original,aliases,description,gender,age,birthday,blood_type,height,weight,bust,waist,hips,cup,image{id,url,dims},traits{id,name,group_name,spoiler,lie},vns{role,spoiler}";

const CHAR_RESULTS: usize = 100;

const SEARCH_RESULTS: usize = 25;

const ULIST_FIELDS: &str = "vn.title,labels{id,label},vote,lastmod";

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
    let (vn, rows) = tokio::select! {
        res = async { tokio::join!(fetch_vn(http, vndb_id), fetch_character_rows(http, vndb_id)) } => res,
        _ = crate::tasks::wait_cancel(progress) => {
            return Err(AppError::Cancelled);
        }
    };
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
            let fetched = tokio::select! {
                res = images::fetch_image(
                    client::image_http(),
                    img.url.as_deref(),
                    img.id.as_deref(),
                    data_dir,
                ) => res,
                _ = crate::tasks::wait_cancel(progress) => {
                    return Err(AppError::Cancelled);
                }
            };
            img.local_path = match fetched {
                Ok(local) => local,
                Err(e) => {
                    let msg = match e {
                        images::ImageError::Local => {
                            "封面保存失败（错误码：local）：数据目录不可写".to_owned()
                        }
                        _ => format!("封面下载失败，请稍后重试（错误码：{}）", e.code()),
                    };
                    progress.report_error("cover", &msg);
                    None
                }
            };
        }
    }
    if progress.cancelled() {
        return Err(AppError::Cancelled);
    }
    progress.report_note("characters", &format!("{} 位", rows.len()));
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

use std::time::Duration;

use super::client::image_http;
use crate::error::{AppError, Result};
use crate::tasks::Progress;
use crate::types::GameCharacter;

const IMAGE_CONCURRENCY: usize = 8;

const IMAGE_RETRIES: usize = 3;

const IMAGE_BACKOFF_MS: u64 = 300;

pub(super) async fn download_character_images(
    mut chars: Vec<GameCharacter>,
    jobs: Vec<(usize, String, String)>,
    data_dir: &std::path::Path,
    progress: &dyn Progress,
) -> Result<Vec<GameCharacter>> {
    let data_dir = data_dir.to_path_buf();
    let total = jobs.len() as i64;
    let mut done = 0_i64;
    let mut failed = 0_i64;
    let mut pending = jobs.into_iter();
    let mut set = tokio::task::JoinSet::new();
    if total > 0 {
        progress.report("images", Some(0), Some(total));
    }
    for _ in 0..IMAGE_CONCURRENCY {
        spawn_next_image(&mut set, &mut pending, &data_dir);
    }
    loop {
        let joined = tokio::select! {
            joined = set.join_next() => joined,
            _ = crate::tasks::wait_cancel(progress) => {
                set.abort_all();
                return Err(AppError::Cancelled);
            }
        };
        let Some(finished) = joined else { break };
        match finished {
            Ok((idx, path)) => {
                let missing = path.is_none();
                chars[idx].image_path = path;
                if missing {
                    failed += 1;
                }
            }
            Err(_) => failed += 1,
        }
        done += 1;
        progress.report("images", Some(done), Some(total));
        spawn_next_image(&mut set, &mut pending, &data_dir);
    }
    if failed > 0 {
        progress.report_error("images", &format!("{failed} 张角色立绘下载失败，已跳过"));
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

pub(super) async fn fetch_image(
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
                tauri_plugin_log::log::warn!("图片下载失败 {url}：HTTP {code}");
                return None;
            }
            Err(ImageError::Transport(e)) => {
                last_err = e;
                if attempt < IMAGE_RETRIES {
                    tokio::time::sleep(Duration::from_millis(IMAGE_BACKOFF_MS * attempt as u64))
                        .await;
                }
            }
        }
    }
    tauri_plugin_log::log::warn!("图片下载失败 {url}（重试 {IMAGE_RETRIES} 次）：{last_err}");
    None
}

async fn download_image(
    http: &reqwest::Client,
    url: &str,
) -> std::result::Result<Vec<u8>, ImageError> {
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

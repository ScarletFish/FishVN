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
        let path = fetch_image(image_http(), Some(&url), None, Some(&id), &data_dir)
            .await
            .ok()
            .flatten();
        (idx, path)
    });
}

pub(super) enum ImageError {
    Http(u16),
    Timeout,
    Network,
    Local,
}

impl ImageError {
    pub(super) fn code(&self) -> String {
        match self {
            Self::Http(code) => code.to_string(),
            Self::Timeout => "timeout".to_owned(),
            Self::Network => "network".to_owned(),
            Self::Local => "local".to_owned(),
        }
    }
}

fn download_candidates(url: &str, thumb: Option<&str>) -> Vec<String> {
    let mut targets = Vec::with_capacity(2);
    if let Some(thumb) = thumb {
        if thumb != url {
            targets.push(thumb.to_owned());
        }
    }
    targets.push(url.to_owned());
    targets
}

pub(super) async fn fetch_image(
    http: &reqwest::Client,
    url: Option<&str>,
    thumb: Option<&str>,
    id: Option<&str>,
    data_dir: &std::path::Path,
) -> std::result::Result<Option<String>, ImageError> {
    let (Some(url), Some(id)) = (url, id) else {
        return Ok(None);
    };
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Ok(None);
    }

    let dir = data_dir.join("covers").join("vndb");
    let dst = dir.join(format!("{id}.jpg"));
    if dst.is_file() {
        return Ok(Some(dst.to_string_lossy().into_owned()));
    }
    if std::fs::create_dir_all(&dir).is_err() {
        return Err(ImageError::Local);
    }

    let targets = download_candidates(url, thumb);

    let mut last_err = None;
    for target in &targets {
        for attempt in 1..=IMAGE_RETRIES {
            let started = std::time::Instant::now();
            match download_image(http, target).await {
                Ok(bytes) => {
                    tauri_plugin_log::log::debug!(
                        target: "backfill",
                        "图片：第 {attempt} 次 {} KB {}ms {target}",
                        bytes.len() / 1024,
                        started.elapsed().as_millis()
                    );
                    if std::fs::write(&dst, &bytes).is_err() {
                        return Err(ImageError::Local);
                    }
                    return Ok(Some(dst.to_string_lossy().into_owned()));
                }
                Err(e) => {
                    tauri_plugin_log::log::debug!(
                        target: "backfill",
                        "图片失败：第 {attempt} 次 {} {}ms {target}",
                        e.code(),
                        started.elapsed().as_millis()
                    );
                    let http_code = match &e {
                        ImageError::Http(code) => Some(*code),
                        _ => None,
                    };
                    match http_code {
                        Some(404) => {
                            last_err = Some(e);
                            break;
                        }
                        Some(code) => {
                            tauri_plugin_log::log::warn!("图片下载失败 {target}：HTTP {code}");
                            return Err(e);
                        }
                        None => {
                            if attempt < IMAGE_RETRIES {
                                tokio::time::sleep(Duration::from_millis(
                                    IMAGE_BACKOFF_MS * attempt as u64,
                                ))
                                .await;
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
            }
        }
    }
    let e = last_err.unwrap_or(ImageError::Network);
    tauri_plugin_log::log::warn!("图片下载失败 {url}（重试 {IMAGE_RETRIES} 次）：{}", e.code());
    Err(e)
}

async fn download_image(
    http: &reqwest::Client,
    url: &str,
) -> std::result::Result<Vec<u8>, ImageError> {
    let resp = match http.get(url).send().await {
        Ok(r) => r,
        Err(e) => return Err(transport_error(&e)),
    };
    if !resp.status().is_success() {
        return Err(ImageError::Http(resp.status().as_u16()));
    }
    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => return Err(transport_error(&e)),
    };
    Ok(bytes.to_vec())
}

fn transport_error(e: &reqwest::Error) -> ImageError {
    if e.is_timeout() {
        ImageError::Timeout
    } else {
        ImageError::Network
    }
}

#[cfg(test)]
mod tests {
    use super::download_candidates;

    #[test]
    fn thumbnail_comes_first() {
        let targets = download_candidates(
            "https://t.vndb.org/cv/76/93476.jpg",
            Some("https://t.vndb.org/cv.t/76/93476.jpg"),
        );
        assert_eq!(
            targets,
            vec![
                "https://t.vndb.org/cv.t/76/93476.jpg",
                "https://t.vndb.org/cv/76/93476.jpg"
            ]
        );
    }

    #[test]
    fn no_thumbnail_falls_back_to_original() {
        let targets = download_candidates("https://t.vndb.org/ch/16/38016.jpg", None);
        assert_eq!(targets, vec!["https://t.vndb.org/ch/16/38016.jpg"]);
    }

    #[test]
    fn same_url_is_not_fetched_twice() {
        let targets = download_candidates(
            "https://t.vndb.org/cv/76/93476.jpg",
            Some("https://t.vndb.org/cv/76/93476.jpg"),
        );
        assert_eq!(targets, vec!["https://t.vndb.org/cv/76/93476.jpg"]);
    }
}

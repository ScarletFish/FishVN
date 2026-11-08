use std::time::Duration;

use super::dto::ImageJob;
use crate::error::Result;
use crate::tasks::Progress;

const IMAGE_RETRIES: usize = 3;

const IMAGE_BACKOFF_MS: u64 = 300;

pub(crate) const IMAGE_CONCURRENCY: usize = 16;

pub(crate) async fn download_images(
    http: &reqwest::Client,
    jobs: Vec<ImageJob>,
    concurrency: usize,
    data_dir: &std::path::Path,
    progress: &dyn Progress,
) -> Result<Vec<(ImageJob, Option<String>)>> {
    let total = jobs.len() as i64;
    if total == 0 {
        return Ok(Vec::new());
    }
    progress.report("images", Some(0), Some(total));
    let data_dir = data_dir.to_path_buf();
    let mut pending = jobs.into_iter();
    let mut set = tokio::task::JoinSet::new();
    let width = concurrency.max(1).min(total as usize);
    for _ in 0..width {
        spawn_image(&mut set, &mut pending, &data_dir, http);
    }
    let mut done = 0_i64;
    let mut out = Vec::with_capacity(total as usize);
    loop {
        let joined = tokio::select! {
            joined = set.join_next() => joined,
            _ = crate::tasks::wait_cancel(progress) => {
                set.abort_all();
                return Ok(out);
            }
        };
        let Some(finished) = joined else { break };
        if let Ok((job, path)) = finished {
            out.push((job, path));
        }
        done += 1;
        progress.report("images", Some(done), Some(total));
        spawn_image(&mut set, &mut pending, &data_dir, http);
    }
    Ok(out)
}

fn spawn_image(
    set: &mut tokio::task::JoinSet<(ImageJob, Option<String>)>,
    pending: &mut std::vec::IntoIter<ImageJob>,
    data_dir: &std::path::Path,
    http: &reqwest::Client,
) {
    let Some(job) = pending.next() else {
        return;
    };
    let data_dir = data_dir.to_path_buf();
    let http = http.clone();
    set.spawn(async move {
        let path = fetch_image(
            &http,
            Some(&job.url),
            job.thumb.as_deref(),
            Some(&job.image_id),
            &data_dir,
        )
        .await
        .ok()
        .flatten();
        (job, path)
    });
}

#[derive(Debug)]
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
    targets.push(url.to_owned());
    if let Some(thumb) = thumb {
        if thumb != url {
            targets.push(thumb.to_owned());
        }
    }
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
                        Some(code) if code >= 500 => {
                            if attempt < IMAGE_RETRIES {
                                tokio::time::sleep(Duration::from_millis(
                                    IMAGE_BACKOFF_MS * attempt as u64,
                                ))
                                .await;
                                continue;
                            }
                            tauri_plugin_log::log::warn!("图片下载失败 {target}：HTTP {code}");
                            return Err(ImageError::Http(code));
                        }
                        Some(code) => {
                            tauri_plugin_log::log::warn!("图片下载失败 {target}：HTTP {code}");
                            return Err(ImageError::Http(code));
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
    use super::*;
    use crate::tasks::Progress;
    use crate::vndb::ImageKind;
    use axum::http::StatusCode;
    use axum::routing::get;
    use axum::Router;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct Silent;

    impl Progress for Silent {
        fn report(&self, _phase: &str, _done: Option<i64>, _total: Option<i64>) {}
        fn report_error(&self, _phase: &str, _message: &str) {}
        fn cancelled(&self) -> bool {
            false
        }
    }

    fn client() -> reqwest::Client {
        reqwest::Client::builder().no_proxy().build().unwrap()
    }

    async fn serve(app: Router) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }

    fn job(base: &str, id: &str) -> ImageJob {
        ImageJob {
            game_id: "g1".to_owned(),
            kind: ImageKind::Cover,
            image_id: id.to_owned(),
            url: format!("{base}/{id}.jpg"),
            thumb: None,
        }
    }

    #[tokio::test]
    async fn 图片池并发下载而不是逐张等待() {
        let live = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let (live_in, peak_in) = (live.clone(), peak.clone());
        let app = Router::new().route(
            "/{name}",
            get(move || {
                let (live, peak) = (live_in.clone(), peak_in.clone());
                async move {
                    let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    live.fetch_sub(1, Ordering::SeqCst);
                    vec![9u8; 16]
                }
            }),
        );
        let base = serve(app).await;
        let dir = tempfile::tempdir().unwrap();
        let jobs: Vec<ImageJob> = (0..16).map(|i| job(&base, &format!("i{i}"))).collect();

        let out = download_images(&client(), jobs, 16, dir.path(), &Silent)
            .await
            .unwrap();

        assert_eq!(out.len(), 16);
        assert!(out.iter().all(|(_, path)| path.is_some()));
        let peak = peak.load(Ordering::SeqCst);
        assert!(peak >= 8, "16 张应同时在途，实际峰值 {peak}");
    }

    #[tokio::test]
    async fn 封面优先取原图且不请求缩略图() {
        let thumb_hits = Arc::new(AtomicUsize::new(0));
        let counter = thumb_hits.clone();
        let app = Router::new()
            .route(
                "/cv.t01.jpg",
                get(move || {
                    let counter = counter.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        vec![9u8; 8]
                    }
                }),
            )
            .route("/cv01.jpg", get(|| async { vec![7u8; 24] }));
        let base = serve(app).await;
        let dir = tempfile::tempdir().unwrap();

        let path = fetch_image(
            &client(),
            Some(&format!("{base}/cv01.jpg")),
            Some(&format!("{base}/cv.t01.jpg")),
            Some("cv1"),
            dir.path(),
        )
        .await
        .unwrap()
        .expect("原图应直接落盘");

        assert_eq!(std::fs::read(path).unwrap(), vec![7u8; 24]);
        assert_eq!(thumb_hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn 原图404时回落缩略图() {
        let app = Router::new()
            .route("/cv01.jpg", get(|| async { StatusCode::NOT_FOUND }))
            .route("/cv.t01.jpg", get(|| async { vec![9u8; 8] }));
        let base = serve(app).await;
        let dir = tempfile::tempdir().unwrap();

        let path = fetch_image(
            &client(),
            Some(&format!("{base}/cv01.jpg")),
            Some(&format!("{base}/cv.t01.jpg")),
            Some("cv1"),
            dir.path(),
        )
        .await
        .unwrap()
        .expect("回落缩略图后应落盘");

        assert_eq!(std::fs::read(path).unwrap(), vec![9u8; 8]);
    }

    #[tokio::test]
    async fn 服务端临时错误会重试到成功() {
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = hits.clone();
        let app = Router::new().route(
            "/ch01.jpg",
            get(move || {
                let counter = counter.clone();
                async move {
                    let n = counter.fetch_add(1, Ordering::SeqCst);
                    if n < 2 {
                        (StatusCode::BAD_GATEWAY, Vec::new())
                    } else {
                        (StatusCode::OK, vec![1u8; 8])
                    }
                }
            }),
        );
        let base = serve(app).await;
        let dir = tempfile::tempdir().unwrap();

        let path = fetch_image(
            &client(),
            Some(&format!("{base}/ch01.jpg")),
            None,
            Some("ch1"),
            dir.path(),
        )
        .await
        .unwrap()
        .expect("第三次应成功");

        assert_eq!(hits.load(Ordering::SeqCst), 3);
        assert_eq!(std::fs::read(path).unwrap(), vec![1u8; 8]);
    }

    #[tokio::test]
    async fn 本地已有同名文件时直接复用不再发请求() {
        let app = Router::new()
            .route("/cv.t01.jpg", get(|| async { StatusCode::NOT_FOUND }))
            .route("/cv01.jpg", get(|| async { vec![3u8; 12] }));
        let base = serve(app).await;
        let dir = tempfile::tempdir().unwrap();

        let first = fetch_image(
            &client(),
            Some(&format!("{base}/cv01.jpg")),
            Some(&format!("{base}/cv.t01.jpg")),
            Some("cv9"),
            dir.path(),
        )
        .await
        .unwrap();
        assert!(first.is_some());

        let again = fetch_image(
            &client(),
            Some("http://127.0.0.1:1/never.jpg"),
            None,
            Some("cv9"),
            dir.path(),
        )
        .await
        .unwrap();
        assert_eq!(again, first, "本地已有同名文件时直接复用，不再发请求");
    }

    #[test]
    fn 原图优先缩略图兜底() {
        let targets = download_candidates(
            "https://t.vndb.org/cv/76/93476.jpg",
            Some("https://t.vndb.org/cv.t/76/93476.jpg"),
        );
        assert_eq!(
            targets,
            vec![
                "https://t.vndb.org/cv/76/93476.jpg",
                "https://t.vndb.org/cv.t/76/93476.jpg"
            ]
        );
    }

    #[test]
    fn 没有缩略图时只取原图() {
        let targets = download_candidates("https://t.vndb.org/ch/16/38016.jpg", None);
        assert_eq!(targets, vec!["https://t.vndb.org/ch/16/38016.jpg"]);
    }

    #[test]
    fn 缩略图同址时不重复请求() {
        let targets = download_candidates(
            "https://t.vndb.org/cv/76/93476.jpg",
            Some("https://t.vndb.org/cv/76/93476.jpg"),
        );
        assert_eq!(targets, vec!["https://t.vndb.org/cv/76/93476.jpg"]);
    }
}

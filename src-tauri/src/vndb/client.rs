use std::collections::VecDeque;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::Value;

use super::dto::VnEnvelope;
use crate::error::{AppError, Result};

const API_BASE: &str = "https://api.vndb.org/kana";

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

const IMAGE_TIMEOUT: Duration = Duration::from_secs(8);

const RATE_LIMIT_MAX: usize = 100;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(5 * 60);

const MIN_WAIT: Duration = Duration::from_millis(1);

pub fn http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("HTTP 客户端建不起来")
    })
}

pub(super) fn image_http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .read_timeout(IMAGE_TIMEOUT)
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
    let since_oldest = history
        .front()
        .map_or(Duration::ZERO, |t| now.duration_since(*t));
    Err(window.saturating_sub(since_oldest).max(MIN_WAIT))
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) async fn post<T: for<'de> Deserialize<'de>>(
    http: &reqwest::Client,
    endpoint: &str,
    body: Value,
    token: Option<&str>,
) -> Result<VnEnvelope<T>> {
    let mut builder = http
        .post(format!("{API_BASE}/{endpoint}"))
        .json(&body);
    if let Some(token) = token {
        builder = builder.header("Authorization", format!("token {token}"));
    }
    let (status, text) = send(builder).await?;
    decode(status, &text)
}

pub(super) async fn get<T: for<'de> Deserialize<'de>>(
    http: &reqwest::Client,
    endpoint: &str,
    token: Option<&str>,
) -> Result<T> {
    let mut builder = http.get(format!("{API_BASE}/{endpoint}"));
    if let Some(token) = token {
        builder = builder.header("Authorization", format!("token {token}"));
    }
    let (status, text) = send(builder).await?;
    decode(status, &text)
}

pub(super) async fn patch(
    http: &reqwest::Client,
    endpoint: &str,
    body: Value,
    token: &str,
) -> Result<()> {
    let builder = http
        .patch(format!("{API_BASE}/{endpoint}"))
        .header("Authorization", format!("token {token}"))
        .json(&body);
    let (status, text) = send(builder).await?;
    check_write(status, &text)
}

async fn send(builder: reqwest::RequestBuilder) -> Result<(u16, String)> {
    let resp = builder.send().await.map_err(|e| {
        if e.is_timeout() {
            AppError::Upstream(format!("VNDB 请求超时：{e}"))
        } else {
            AppError::Upstream(format!("VNDB 请求失败：{e}"))
        }
    })?;

    let status = resp.status().as_u16();
    let text = resp.text().await.map_err(|e| {
        if e.is_timeout() {
            AppError::Upstream(format!("VNDB 响应超时：{e}"))
        } else {
            AppError::Upstream(format!("VNDB 响应读不出：{e}"))
        }
    })?;
    Ok((status, text))
}

fn decode<T: for<'de> Deserialize<'de>>(status: u16, text: &str) -> Result<T> {
    match status {
        200 => serde_json::from_str(text)
            .map_err(|e| AppError::Upstream(format!("VNDB 响应不是预期 JSON：{e}"))),
        400 => Err(AppError::BadRequest(format!(
            "VNDB 拒绝请求（400）：{text}"
        ))),
        401 => Err(AppError::BadRequest("VNDB 鉴权失败（401）：token 可能无效".into())),
        429 => Err(AppError::RateLimited),
        _ => Err(AppError::Upstream(format!("VNDB 返回 {status}"))),
    }
}

fn check_write(status: u16, text: &str) -> Result<()> {
    match status {
        200 | 204 => Ok(()),
        400 => Err(AppError::BadRequest(format!(
            "VNDB 拒绝写入（400）：{text}"
        ))),
        401 => Err(AppError::BadRequest(
            "VNDB 鉴权失败（401）：token 可能无效，或没有勾选 listwrite 权限".into(),
        )),
        403 => Err(AppError::BadRequest(
            "VNDB 拒绝写入（403）：token 缺少 listwrite 权限".into(),
        )),
        429 => Err(AppError::RateLimited),
        _ => Err(AppError::Upstream(format!("VNDB 返回 {status}"))),
    }
}

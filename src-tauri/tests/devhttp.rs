//! devhttp 的路由是命令层的镜像，两边手写维护。加命令时漏了路由，
//! 编译照过，浏览器里只看到一个看不出原因的 404
//!
//! file_response 的反例：content-type 写死或按错扩展名、字节被改写、
//! 跳过扩展名校验、跳过文件存在校验，对应用例分别变红

#![cfg(debug_assertions)]

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::http::StatusCode;
use fishvn::tasks::EventLog;
use fishvn::types::TaskEvent;
use futures_util::StreamExt;

fn push(log: &Mutex<EventLog>, phase: &str, done: Option<i64>, total: Option<i64>) {
    log.lock().unwrap().push(
        "vndb-sync",
        TaskEvent {
            phase: phase.to_owned(),
            ts: 0,
            done,
            total,
            detail: None,
            error: None,
            run_id: 0,
            game_id: None,
            seq: 0,
        },
    );
}

/// SSE 流不结束，读首帧即可，两秒等不到就是路由或尾随坏了
async fn first_chunk(resp: axum::response::Response) -> String {
    let mut stream = resp.into_body().into_data_stream();
    let chunk = tokio::time::timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("SSE 首帧超时")
        .expect("流提前结束")
        .expect("帧读不出");
    String::from_utf8_lossy(&chunk).into_owned()
}

#[tokio::test]
async fn sse补读缓冲里的历史事件() {
    let log = Arc::new(Mutex::new(EventLog::default()));
    push(&log, "meta", None, None);
    push(&log, "images", Some(3), Some(20));

    let resp = fishvn::devhttp::events_response(&log, 0);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/event-stream"
    );
    let text = first_chunk(resp).await;
    assert!(text.contains("id: 1"), "首帧缺 id：{text}");
    assert!(text.contains("event: vndb-sync"), "首帧缺事件名：{text}");
    assert!(text.contains("\"phase\":\"meta\""), "首帧缺数据：{text}");
    assert!(text.contains("\"seq\":1"), "首帧数据缺序号：{text}");
}

#[tokio::test]
async fn sse游标只补读之后的事件() {
    let log = Arc::new(Mutex::new(EventLog::default()));
    push(&log, "meta", None, None);
    push(&log, "images", Some(1), Some(20));

    let text = first_chunk(fishvn::devhttp::events_response(&log, 1)).await;
    assert!(text.contains("id: 2"), "游标没生效：{text}");
    assert!(!text.contains("id: 1"), "补读了游标前的事件：{text}");
    assert!(text.contains("\"phase\":\"images\""), "补读的不是当次事件：{text}");
}

#[tokio::test]
async fn sse尾随连接后到达的事件() {
    let log = Arc::new(Mutex::new(EventLog::default()));
    let fed = log.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        push(&fed, "images", Some(7), Some(20));
    });

    let text = first_chunk(fishvn::devhttp::events_response(&log, 0)).await;
    assert!(text.contains("id: 1"), "尾随首帧缺 id：{text}");
    assert!(
        text.contains("\"phase\":\"images\""),
        "没接到连接后才推送的事件：{text}"
    );
}

/// 从 commands.rs 里抠出 #[tauri::command] 标注的函数名
fn declared_commands() -> BTreeSet<String> {
    let src = include_str!("../src/commands.rs");
    let mut out = BTreeSet::new();
    let mut pending = false;
    for line in src.lines() {
        let line = line.trim();
        if line == "#[tauri::command]" {
            pending = true;
            continue;
        }
        if !pending {
            continue;
        }
        let rest = line
            .strip_prefix("pub fn ")
            .or_else(|| line.strip_prefix("pub async fn "));
        if let Some(rest) = rest {
            if let Some(name) = rest.split('(').next() {
                out.insert(name.to_owned());
            }
        }
        pending = false;
    }
    assert!(!out.is_empty(), "commands.rs 里没找到命令，解析写坏了");
    out
}

/// 浏览器会话的前端异常直接看 devtools，前端日志命令不经 devhttp
const DEVHTTP_EXEMPT: &[&str] = &["log_frontend", "resolve_app_quit"];

#[test]
fn 路由表覆盖全部命令() {
    let declared = declared_commands();
    let routed: BTreeSet<String> = fishvn::devhttp::COMMANDS
        .iter()
        .map(|s| (*s).to_owned())
        .collect();

    let missing: Vec<_> = declared
        .difference(&routed)
        .filter(|name| !DEVHTTP_EXEMPT.contains(&name.as_str()))
        .collect();
    assert!(missing.is_empty(), "命令没进 devhttp 路由：{missing:?}");

    let extra: Vec<_> = routed.difference(&declared).collect();
    assert!(
        extra.is_empty(),
        "devhttp 里有 commands.rs 没有的命令：{extra:?}"
    );
}

#[tokio::test]
async fn 文件路由按扩展名回传字节与类型() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = vec![7u8, 11, 13, 17];
    let path = dir.path().join("cover.png");
    std::fs::write(&path, &bytes).unwrap();

    let resp = fishvn::devhttp::file_response(path.to_str().unwrap());
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("content-type").unwrap(), "image/png");
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), bytes);
}

#[test]
fn 文件路由对非图片扩展名返回400() {
    let resp = fishvn::devhttp::file_response("C:\\pics\\notes.txt");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn 文件路由对不存在的图片返回404() {
    let resp = fishvn::devhttp::file_response("C:\\pics\\missing.png");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[test]
fn 镜像命令按前端字段名取参() {
    let args: fishvn::devhttp::Args = serde_json::from_value(serde_json::json!({
        "src": "D:\\pics\\cover.jpg",
        "path": "D:\\other\\file.jpg",
        "patch": {"theme": "dark"},
        "merge": {"keep": "v1"}
    }))
    .unwrap();

    assert_eq!(args.src().unwrap(), "D:\\pics\\cover.jpg");
    assert_eq!(args.path().unwrap(), "D:\\other\\file.jpg");
    assert_eq!(args.parse::<serde_json::Value>(args.patch.as_ref(), "patch").unwrap()["theme"], "dark");
    assert_eq!(args.parse::<serde_json::Value>(args.merge.as_ref(), "merge").unwrap()["keep"], "v1");
}

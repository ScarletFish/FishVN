use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::types::TaskEvent;

pub const LOG_CAP: usize = 2048;

pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize)]
pub struct LoggedEvent {
    pub seq: u64,
    pub name: String,
    pub event: TaskEvent,
}

#[derive(Debug, Default)]
pub struct EventLog {
    next: u64,
    entries: VecDeque<LoggedEvent>,
}

impl EventLog {
    pub fn push(&mut self, name: &str, mut event: TaskEvent) -> LoggedEvent {
        self.next += 1;
        event.seq = self.next;
        let logged = LoggedEvent {
            seq: self.next,
            name: name.to_owned(),
            event,
        };
        if self.entries.len() == LOG_CAP {
            self.entries.pop_front();
        }
        self.entries.push_back(logged.clone());
        logged
    }

    pub fn since(&self, after: u64) -> Vec<LoggedEvent> {
        self.entries
            .iter()
            .filter(|entry| entry.seq > after)
            .cloned()
            .collect()
    }
}

pub trait Progress: Sync {
    fn report(&self, phase: &str, done: Option<i64>, total: Option<i64>);
    fn report_item(&self, phase: &str, done: Option<i64>, total: Option<i64>, _game_id: &str) {
        self.report(phase, done, total);
    }
    fn report_error(&self, phase: &str, message: &str);
    fn report_note(&self, phase: &str, _detail: &str) {
        self.report(phase, None, None);
    }
    fn cancelled(&self) -> bool;
}

pub(crate) async fn wait_cancel(progress: &dyn Progress) {
    while !progress.cancelled() {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub struct Task {
    name: String,
    app: AppHandle,
    log: Arc<Mutex<EventLog>>,
    cancel: Arc<AtomicBool>,
    run_id: u64,
    current: Mutex<Option<String>>,
}

impl Task {
    pub fn new(
        name: &str,
        app: AppHandle,
        log: Arc<Mutex<EventLog>>,
        cancel: Arc<AtomicBool>,
        run_id: u64,
    ) -> Self {
        Self {
            name: name.to_owned(),
            app,
            log,
            cancel,
            run_id,
            current: Mutex::new(None),
        }
    }

    pub fn enter(&self, game_id: &str) {
        match self.current.lock() {
            Ok(mut current) => *current = Some(game_id.to_owned()),
            Err(poison) => *poison.into_inner() = Some(game_id.to_owned()),
        }
    }

    pub fn leave(&self) {
        match self.current.lock() {
            Ok(mut current) => *current = None,
            Err(poison) => *poison.into_inner() = None,
        }
    }

    fn current_id(&self) -> Option<String> {
        match self.current.lock() {
            Ok(current) => current.clone(),
            Err(poison) => poison.into_inner().clone(),
        }
    }

    fn push(
        &self,
        phase: &str,
        done: Option<i64>,
        total: Option<i64>,
        detail: Option<String>,
        error: Option<String>,
        game_id: Option<&str>,
    ) {
        let game_id = game_id.map(str::to_owned).or_else(|| self.current_id());
        let logged = match self.log.lock() {
            Ok(mut log) => log.push(
                &self.name,
                TaskEvent {
                    phase: phase.to_owned(),
                    ts: now_ms(),
                    done,
                    total,
                    detail,
                    error,
                    game_id,
                    run_id: self.run_id,
                    seq: 0,
                },
            ),
            Err(_) => return,
        };
        let _ = self.app.emit(&self.name, logged.event);
    }
}

impl Progress for Task {
    fn report(&self, phase: &str, done: Option<i64>, total: Option<i64>) {
        self.push(phase, done, total, None, None, None);
    }

    fn report_item(&self, phase: &str, done: Option<i64>, total: Option<i64>, game_id: &str) {
        self.push(phase, done, total, None, None, Some(game_id));
    }

    fn report_error(&self, phase: &str, message: &str) {
        tauri_plugin_log::log::warn!("任务 {} 在 {phase} 阶段报告：{message}", self.name);
        self.push(phase, None, None, None, Some(message.to_owned()), None);
    }

    fn report_note(&self, phase: &str, detail: &str) {
        self.push(phase, None, None, Some(detail.to_owned()), None, None);
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Default)]
pub struct Tasks {
    flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl Tasks {
    pub fn register(&self, name: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.flags
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .insert(name.to_owned(), flag.clone());
        flag
    }

    pub fn cancel(&self, name: &str) -> bool {
        let flag = self
            .flags
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .get(name)
            .cloned();
        match flag {
            Some(flag) => {
                flag.store(true, Ordering::Relaxed);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(phase: &str) -> TaskEvent {
        TaskEvent {
            phase: phase.to_owned(),
            ts: 0,
            done: None,
            total: None,
            detail: None,
            error: None,
            run_id: 0,
            game_id: None,
            seq: 0,
        }
    }

    #[test]
    fn 推送的事件自带与日志一致的序号() {
        let mut log = EventLog::default();
        let first = log.push("t", event("a"));
        let second = log.push("t", event("b"));

        assert_eq!(first.event.seq, first.seq);
        assert_eq!(second.event.seq, second.seq);
        assert_eq!(log.since(first.seq)[0].event.seq, second.seq);
    }

    #[test]
    fn seq单调递增且since按游标补读() {
        let mut log = EventLog::default();
        let first = log.push("t", event("a"));
        let second = log.push("t", event("b"));

        assert_eq!(second.seq, first.seq + 1);
        let missed = log.since(first.seq);
        assert_eq!(missed.len(), 1);
        assert_eq!(missed[0].seq, second.seq);
        assert!(log.since(second.seq).is_empty());
    }

    #[test]
    fn 超出容量从最旧的开始丢() {
        let mut log = EventLog::default();
        let mut first = 0;
        for i in 0..LOG_CAP + 5 {
            let entry = log.push("t", event("x"));
            if i == 0 {
                first = entry.seq;
            }
        }

        let kept = log.since(0);
        assert_eq!(kept.len(), LOG_CAP);
        assert_eq!(kept[0].seq, first + 5);
    }

    #[test]
    fn 取消信号只作用在当次运行() {
        let tasks = Tasks::default();
        let flag = tasks.register("vndb-sync");

        assert!(!flag.load(Ordering::Relaxed));
        assert!(tasks.cancel("vndb-sync"));
        assert!(flag.load(Ordering::Relaxed));
        assert!(!tasks.cancel("没在跑的任务"));

        let fresh = tasks.register("vndb-sync");
        assert!(!fresh.load(Ordering::Relaxed));
    }
}

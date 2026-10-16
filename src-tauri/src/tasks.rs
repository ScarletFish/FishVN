use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::types::TaskEvent;

pub const LOG_CAP: usize = 512;

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
    pub fn push(&mut self, name: &str, event: TaskEvent) -> LoggedEvent {
        self.next += 1;
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
    fn cancelled(&self) -> bool;
}

pub struct Task {
    name: String,
    app: AppHandle,
    log: Arc<Mutex<EventLog>>,
    cancel: Arc<AtomicBool>,
}

impl Task {
    pub fn new(
        name: &str,
        app: AppHandle,
        log: Arc<Mutex<EventLog>>,
        cancel: Arc<AtomicBool>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            app,
            log,
            cancel,
        }
    }
}

impl Progress for Task {
    fn report(&self, phase: &str, done: Option<i64>, total: Option<i64>) {
        let logged = match self.log.lock() {
            Ok(mut log) => log.push(
                &self.name,
                TaskEvent {
                    phase: phase.to_owned(),
                    done,
                    total,
                },
            ),
            Err(_) => return,
        };
        let _ = self.app.emit(&self.name, logged.event);
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
            done: None,
            total: None,
        }
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

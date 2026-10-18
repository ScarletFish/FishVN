use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension};
use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob, JobObjectBasicAccountingInformation,
    QueryInformationJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use crate::db::lock;
use crate::error::{AppError, Result};
use crate::games;
use crate::installations;
use crate::settings::{self, TimeMetric};
use crate::types::{Installation, PlaySession, Status};
use crate::{new_id, now};

const TICK: Duration = Duration::from_millis(1000);
const SETTLE_TICKS: u32 = 3;
const HEARTBEAT_TICKS: u32 = 30;

const COLUMNS: &str = "id, game_id, installation_id, started_at, last_seen_at, ended_at,
                       foreground_seconds, COALESCE(ended_at, last_seen_at) - started_at AS alive_seconds";

struct Job(HANDLE);

unsafe impl Send for Job {}

impl Job {
    fn create() -> Result<Self> {
        let handle = unsafe { CreateJobObjectW(None, PCWSTR::null()) }
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(Self(handle))
    }

    fn attach(&self, pid: u32) -> Result<()> {
        let process = unsafe { OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid) }
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        let assigned = unsafe { AssignProcessToJobObject(self.0, process) };
        let _ = unsafe { CloseHandle(process) };
        assigned.map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))
    }

    fn active_processes(&self) -> u32 {
        let mut info = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        let queried = unsafe {
            QueryInformationJobObject(
                Some(self.0),
                JobObjectBasicAccountingInformation,
                std::ptr::from_mut(&mut info).cast(),
                std::mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                None,
            )
        };
        match queried {
            Ok(()) => info.ActiveProcesses,
            Err(_) => 0,
        }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

pub fn launch(shared: Arc<Mutex<Connection>>, game_id: &str) -> Result<PlaySession> {
    let conn = lock(&shared)?;
    games::get_game(&conn, game_id)?;
    if active(&conn, game_id)?.is_some() {
        return Err(AppError::BadRequest("这部作品已经在游玩中".into()));
    }
    let (installation, dir, entry) = resolve(&conn, game_id)?;

    let mut child = command_for(&entry)
        .current_dir(&dir)
        .spawn()
        .map_err(|e| AppError::BadRequest(format!("启动失败：{e}")))?;
    let job = Job::create()?;
    if let Err(e) = job.attach(child.id()) {
        let _ = child.kill();
        return Err(e);
    }

    let started_at = now();
    let session = PlaySession {
        id: new_id(),
        game_id: game_id.to_owned(),
        installation_id: Some(installation.id),
        started_at,
        last_seen_at: started_at,
        ended_at: None,
        foreground_seconds: 0,
        alive_seconds: 0,
    };
    conn.execute(
        "INSERT INTO play_sessions
           (id, game_id, installation_id, started_at, last_seen_at, ended_at, foreground_seconds)
         VALUES (?, ?, ?, ?, ?, NULL, 0)",
        params![
            &session.id,
            &session.game_id,
            &session.installation_id,
            started_at,
            started_at
        ],
    )?;
    drop(conn);

    let watch_id = session.id.clone();
    let watch_game = game_id.to_owned();
    thread::spawn(move || watch(job, shared, watch_id, watch_game, started_at));
    Ok(session)
}

pub fn sweep(conn: &Connection) -> Result<()> {
    let orphans: Vec<(String, i64, i64, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT game_id, started_at, last_seen_at, foreground_seconds
             FROM play_sessions WHERE ended_at IS NULL",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        rows.collect::<rusqlite::Result<_>>()?
    };
    conn.execute(
        "UPDATE play_sessions SET ended_at = last_seen_at WHERE ended_at IS NULL",
        [],
    )?;
    for (game_id, started_at, last_seen_at, foreground) in orphans {
        let seconds = match settings::current().time_metric {
            TimeMetric::Foreground => foreground,
            TimeMetric::Alive => last_seen_at - started_at,
        };
        add_play_time(conn, &game_id, seconds)?;
    }
    Ok(())
}

pub fn list_active(conn: &Connection) -> Result<Vec<PlaySession>> {
    let sql = format!("SELECT {COLUMNS} FROM play_sessions WHERE ended_at IS NULL ORDER BY started_at");
    collect(conn, &sql, params![])
}

fn watch(
    job: Job,
    shared: Arc<Mutex<Connection>>,
    session_id: String,
    game_id: String,
    started_at: i64,
) {
    let mut foreground = 0i64;
    let mut alive_at = started_at;
    let mut idle = 0u32;
    let mut ticks = 0u32;
    let mut in_front = false;
    let mut confirmed = false;

    loop {
        thread::sleep(TICK);
        if job.active_processes() == 0 {
            idle += 1;
            if idle >= SETTLE_TICKS {
                break;
            }
            continue;
        }
        idle = 0;
        ticks += 1;
        alive_at = now();
        let front = foreground_in_job(job.0);
        if front {
            foreground += 1;
        }
        if front != in_front || ticks.is_multiple_of(HEARTBEAT_TICKS) {
            in_front = front;
            store(&shared, &session_id, alive_at, foreground, None);
        }
        if !confirmed && ticks >= SETTLE_TICKS {
            confirmed = true;
            mark_playing(&shared, &game_id);
        }
    }
    store(&shared, &session_id, alive_at, foreground, Some(alive_at));
    if let Ok(conn) = shared.lock() {
        let seconds = match settings::current().time_metric {
            TimeMetric::Foreground => foreground,
            TimeMetric::Alive => alive_at - started_at,
        };
        let _ = add_play_time(&conn, &game_id, seconds);
    }
}

fn store(
    shared: &Arc<Mutex<Connection>>,
    id: &str,
    alive_at: i64,
    foreground: i64,
    ended_at: Option<i64>,
) {
    let Ok(conn) = shared.lock() else {
        return;
    };
    let _ = conn.execute(
        "UPDATE play_sessions
            SET last_seen_at = ?, foreground_seconds = ?, ended_at = COALESCE(?, ended_at)
          WHERE id = ?",
        params![alive_at, foreground, ended_at, id],
    );
}

fn add_play_time(conn: &Connection, game_id: &str, seconds: i64) -> Result<()> {
    if seconds <= 0 {
        return Ok(());
    }
    conn.execute(
        "UPDATE user_game_state SET play_seconds = play_seconds + ? WHERE game_id = ?",
        params![seconds, game_id],
    )?;
    Ok(())
}

fn mark_playing(shared: &Arc<Mutex<Connection>>, game_id: &str) {
    let Ok(conn) = shared.lock() else {
        return;
    };
    let _ = conn.execute(
        "UPDATE user_game_state SET status = ? WHERE game_id = ? AND status = ?",
        params![Status::Playing.as_str(), game_id, Status::Wishlist.as_str()],
    );
}

fn foreground_in_job(job: HANDLE) -> bool {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return false;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return false;
    }
    let Ok(process) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else {
        return false;
    };
    let mut inside = BOOL(0);
    let checked = unsafe { IsProcessInJob(process, Some(job), &mut inside) };
    let _ = unsafe { CloseHandle(process) };
    checked.is_ok() && inside.as_bool()
}

fn resolve(conn: &Connection, game_id: &str) -> Result<(Installation, PathBuf, PathBuf)> {
    let state = games::get_game_state(conn, game_id)?;
    let launchable: Vec<Installation> = installations::list_installations(conn, Some(game_id))?
        .into_iter()
        .filter(|item| item.entry_path.is_some())
        .collect();

    let picked = state
        .default_installation_id
        .as_ref()
        .and_then(|id| launchable.iter().find(|item| &item.id == id))
        .or_else(|| launchable.first())
        .ok_or_else(|| AppError::BadRequest("这部作品还没有指定启动文件".into()))?
        .clone();

    let dir = PathBuf::from(&picked.dir_path);
    let entry = dir.join(picked.entry_path.as_deref().unwrap_or_default());
    if !entry.is_file() {
        return Err(AppError::BadRequest(format!(
            "启动文件不存在：{}",
            entry.display()
        )));
    }
    Ok((picked, dir, entry))
}

fn command_for(entry: &Path) -> Command {
    let ext = entry
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match ext.as_str() {
        "bat" | "cmd" => {
            let mut command = Command::new("cmd.exe");
            command.arg("/C").arg(entry);
            command
        }
        "lnk" => {
            let mut command = Command::new("cmd.exe");
            command.arg("/C").arg("start").arg("").arg(entry);
            command
        }
        _ => Command::new(entry),
    }
}

fn active(conn: &Connection, game_id: &str) -> Result<Option<PlaySession>> {
    let sql = format!("SELECT {COLUMNS} FROM play_sessions WHERE game_id = ? AND ended_at IS NULL");
    Ok(conn
        .query_row(&sql, params![game_id], read_row)
        .optional()?)
}

fn collect(conn: &Connection, sql: &str, args: impl rusqlite::Params) -> Result<Vec<PlaySession>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(args, read_row)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn read_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlaySession> {
    Ok(PlaySession {
        id: row.get("id")?,
        game_id: row.get("game_id")?,
        installation_id: row.get("installation_id")?,
        started_at: row.get("started_at")?,
        last_seen_at: row.get("last_seen_at")?,
        ended_at: row.get("ended_at")?,
        foreground_seconds: row.get("foreground_seconds")?,
        alive_seconds: row.get("alive_seconds")?,
    })
}

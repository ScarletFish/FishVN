use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use rusqlite::{params, Connection, OptionalExtension};
use windows::core::{BOOL, PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, DuplicateHandle, HANDLE, LPARAM, WPARAM, DUPLICATE_SAME_ACCESS,
};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob, JobObjectBasicAccountingInformation,
    QueryInformationJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_QUOTA, PROCESS_TERMINATE, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GetForegroundWindow, GetWindowThreadProcessId, HWND_MESSAGE, PostMessageW,
    WM_HOTKEY,
};

use crate::db::lock;
use crate::error::{AppError, Result};
use crate::games;
use crate::installations;
use crate::settings::{self, AppConfig, TimeMetric};
use crate::types::{GamePlayLog, Installation, MagpieMode, PlaySession, Status};
use crate::{new_id, now};
use tauri::{AppHandle, Emitter};
use tauri_plugin_log::log;

const TICK: Duration = Duration::from_millis(1000);
const SETTLE_TICKS: u32 = 3;
const HEARTBEAT_TICKS: u32 = 30;

const MAGPIE_READY_TIMEOUT: Duration = Duration::from_secs(5);
const MAGPIE_WINDOW_TIMEOUT: Duration = Duration::from_secs(60);
const MAGPIE_SETTLE: Duration = Duration::from_secs(2);
const MAGPIE_POLL: Duration = Duration::from_millis(500);
const MAGPIE_HOTKEY_CLASS: &str = "Magpie_Hotkey";

const COLUMNS: &str = "id, game_id, installation_id, started_at, last_seen_at, ended_at,
                       foreground_seconds, COALESCE(ended_at, last_seen_at) - started_at AS alive_seconds";

#[derive(Clone, serde::Serialize)]
struct SessionEnded {
    seconds: i64,
}

struct Job(HANDLE);

unsafe impl Send for Job {}

struct SendHandle(HANDLE);

unsafe impl Send for SendHandle {}

impl Drop for SendHandle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

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

    fn duplicate(&self) -> Result<HANDLE> {
        let mut dup = HANDLE::default();
        unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                self.0,
                GetCurrentProcess(),
                &mut dup,
                0,
                false,
                DUPLICATE_SAME_ACCESS,
            )
        }
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
        Ok(dup)
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

pub fn launch(shared: Arc<Mutex<Connection>>, game_id: &str, app: Option<AppHandle>) -> Result<PlaySession> {
    let conn = lock(&shared)?;
    games::get_game(&conn, game_id)?;
    if active(&conn, game_id)?.is_some() {
        return Err(AppError::BadRequest("这个游戏已经在游玩中".into()));
    }
    let (installation, dir, entry) = resolve(&conn, game_id)?;
    let config = settings::current();
    let magpie_mode = installation.magpie;
    let magpie_path = if magpie_mode != MagpieMode::Off {
        Some(resolve_tool(&config.magpie_path, "Magpie")?)
    } else {
        None
    };

    let mut command = build_launch_command(&installation, &entry, &config)?;
    command.current_dir(&dir);
    let mut child = command
        .spawn()
        .map_err(|e| AppError::BadRequest(format!("启动失败：{e}")))?;
    let job = Job::create()?;
    if let Err(e) = job.attach(child.id()) {
        let _ = child.kill();
        return Err(e);
    }
    let magpie_job = magpie_path
        .as_ref()
        .and_then(|_| job.duplicate().ok())
        .map(SendHandle);

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
    thread::spawn(move || watch(job, shared, watch_id, watch_game, started_at, app));
    if let (Some(path), Some(magpie_job)) = (magpie_path, magpie_job) {
        thread::spawn(move || trigger_magpie(&path, magpie_job, magpie_mode));
    }
    log::info!(target: "session", "游玩开始：{game_id} 会话 {}", session.id);
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
    if !orphans.is_empty() {
        log::info!(target: "session", "启动补账：{} 个未收尾的会话", orphans.len());
    }
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

pub fn play_log(conn: &Connection, game_id: &str, metric: TimeMetric) -> Result<GamePlayLog> {
    let sql = format!(
        "SELECT {COLUMNS} FROM play_sessions WHERE game_id = ? ORDER BY started_at, id"
    );
    let sessions = collect(conn, &sql, params![game_id])?;
    let total_seconds = sessions
        .iter()
        .map(|session| match metric {
            TimeMetric::Foreground => session.foreground_seconds,
            TimeMetric::Alive => session.alive_seconds,
        })
        .sum();
    Ok(GamePlayLog {
        count: sessions.len() as i64,
        first_at: sessions.first().map(|session| session.started_at),
        last_at: sessions.last().map(|session| session.started_at),
        total_seconds,
        sessions,
    })
}

fn watch(
    job: Job,
    shared: Arc<Mutex<Connection>>,
    session_id: String,
    game_id: String,
    started_at: i64,
    app: Option<AppHandle>,
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
        let front = foreground_is_game(job.0);
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
        let (seconds, metric) = match settings::current().time_metric {
            TimeMetric::Foreground => (foreground, "前台"),
            TimeMetric::Alive => (alive_at - started_at, "存活"),
        };
        log::info!(target: "session", "游玩结束：{game_id} {seconds} s（{metric}）");
        if let Err(e) = add_play_time(&conn, &game_id, seconds) {
            log::warn!(
                target: "session",
                "游玩时长没记上：{game_id} {seconds} s：[{}] {e}",
                e.kind()
            );
        }
        if seconds > 0 {
            if let Some(app) = &app {
                let _ = app.emit("session-ended", SessionEnded { seconds });
            }
        }
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
        "UPDATE user_game_state SET status = ?, state_modified_at = ? WHERE game_id = ? AND status = ?",
        params![
            Status::Playing.as_str(),
            crate::now(),
            game_id,
            Status::Wishlist.as_str()
        ],
    );
}

fn foreground_pid() -> Option<u32> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return None;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    (pid != 0).then_some(pid)
}

fn pid_in_job(pid: u32, job: HANDLE) -> bool {
    let Ok(process) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else {
        return false;
    };
    let mut inside = BOOL(0);
    let checked = unsafe { IsProcessInJob(process, Some(job), &mut inside) };
    let _ = unsafe { CloseHandle(process) };
    checked.is_ok() && inside.as_bool()
}

fn is_magpie_process(pid: u32) -> bool {
    let Ok(process) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else {
        return false;
    };
    let mut buf = vec![0u16; 260];
    let mut len = buf.len() as u32;
    let named = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
    };
    let _ = unsafe { CloseHandle(process) };
    if named.is_err() {
        return false;
    }
    let path = String::from_utf16_lossy(&buf[..len as usize]);
    Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("Magpie.exe"))
}

fn foreground_is_game(job: HANDLE) -> bool {
    foreground_pid().is_some_and(|pid| pid_in_job(pid, job) || is_magpie_process(pid))
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
        .ok_or_else(|| AppError::BadRequest("这个游戏还没有指定启动文件".into()))?
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

fn build_launch_command(
    installation: &Installation,
    entry: &Path,
    config: &AppConfig,
) -> Result<Command> {
    if installation.le_launch {
        if !entry
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
        {
            return Err(AppError::BadRequest(
                "转区启动仅支持 exe 入口，bat / lnk 请改用直接启动".into(),
            ));
        }
        let le_path = resolve_tool(&config.le_path, "Locale Emulator")?;
        let mut command = Command::new(&le_path);
        command.arg(entry);
        return Ok(command);
    }
    let mut command = command_for(entry);
    if let Some(raw) = installation
        .launch_args
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        command.args(split_command_line(raw));
    }
    Ok(command)
}

fn resolve_tool(path: &Option<String>, name: &str) -> Result<PathBuf> {
    let raw = path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            AppError::BadRequest(format!("尚未在「设置 → 工具」中指定 {name} 路径"))
        })?;
    let tool = PathBuf::from(raw);
    if !tool.is_file() {
        return Err(AppError::BadRequest(format!(
            "{name} 路径不存在：{}",
            tool.display()
        )));
    }
    Ok(tool)
}

fn split_command_line(raw: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut open = false;
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if quoted && chars.peek() == Some(&'"') {
                    chars.next();
                    current.push('"');
                } else {
                    quoted = !quoted;
                }
                open = true;
            }
            c if c.is_whitespace() && !quoted => {
                if open {
                    out.push(std::mem::take(&mut current));
                    open = false;
                }
            }
            c => {
                current.push(c);
                open = true;
            }
        }
    }
    if open {
        out.push(current);
    }
    out
}

fn trigger_magpie(path: &Path, job: SendHandle, mode: MagpieMode) {
    let Some(action) = mode.hotkey_action() else {
        return;
    };
    if !ensure_magpie_running(path) {
        return;
    }
    if !wait_foreground_game(job.0, MAGPIE_WINDOW_TIMEOUT) {
        return;
    }
    thread::sleep(MAGPIE_SETTLE);
    if !foreground_is_game(job.0) {
        return;
    }
    let _ = post_magpie_hotkey(action);
}

fn ensure_magpie_running(path: &Path) -> bool {
    if find_magpie_hotkey_window().is_some() {
        return true;
    }
    if Command::new(path).spawn().is_err() {
        return false;
    }
    let deadline = Instant::now() + MAGPIE_READY_TIMEOUT;
    while Instant::now() < deadline {
        thread::sleep(Duration::from_millis(200));
        if find_magpie_hotkey_window().is_some() {
            return true;
        }
    }
    false
}

fn find_magpie_hotkey_window() -> Option<windows::Win32::Foundation::HWND> {
    let class: Vec<u16> = MAGPIE_HOTKEY_CLASS
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        FindWindowExW(
            Some(HWND_MESSAGE),
            None,
            PCWSTR(class.as_ptr()),
            PCWSTR::null(),
        )
    }
    .ok()
    .filter(|hwnd| !hwnd.0.is_null())
}

fn post_magpie_hotkey(action: i32) -> bool {
    let Some(hwnd) = find_magpie_hotkey_window() else {
        return false;
    };
    unsafe { PostMessageW(Some(hwnd), WM_HOTKEY, WPARAM(action as usize), LPARAM(0)) }.is_ok()
}

fn wait_foreground_game(job: HANDLE, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if foreground_is_game(job) {
            return true;
        }
        let now = Instant::now();
        if now >= deadline {
            return false;
        }
        thread::sleep(MAGPIE_POLL.min(deadline - now));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 按空白拆参数() {
        assert_eq!(
            split_command_line("-window -w 1280"),
            vec!["-window", "-w", "1280"]
        );
    }

    #[test]
    fn 引号里的空白不分隔() {
        assert_eq!(
            split_command_line("-path \"D:\\My Games\\One\""),
            vec!["-path", "D:\\My Games\\One"]
        );
    }

    #[test]
    fn 空串与纯空白没有参数() {
        assert!(split_command_line("").is_empty());
        assert!(split_command_line("   \t ").is_empty());
    }

    #[test]
    fn 引号包着的空串是独立参数() {
        assert_eq!(split_command_line("a \"\" b"), vec!["a", "", "b"]);
    }

    #[test]
    fn 相邻引号拼进同一参数() {
        assert_eq!(split_command_line("\"a\"\"b\"c"), vec!["a\"bc"]);
    }

    #[test]
    fn 引号内外的字符连成一个参数() {
        assert_eq!(split_command_line("pre\"in mid\"post"), vec!["prein midpost"]);
    }

    #[test]
    fn 游玩记录按口径汇总并保留每条会话() {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::ensure_schema(&conn).unwrap();
        let id = games::create_game(
            &conn,
            &crate::types::NewGame {
                name: "甲".into(),
                vndb_id: None,
                metadata: None,
            },
        )
        .unwrap()
        .id;
        conn.execute(
            "INSERT INTO play_sessions (id, game_id, started_at, last_seen_at, ended_at, foreground_seconds)
             VALUES ('s1', ?1, 100, 3600, 3600, 1200)",
            params![id],
        )
        .unwrap();

        let log = play_log(&conn, &id, TimeMetric::Foreground).unwrap();
        assert_eq!(log.count, 1);
        assert_eq!(log.total_seconds, 1200);
        assert_eq!(log.first_at, Some(100));
        assert_eq!(log.last_at, Some(100));
        assert_eq!(log.sessions[0].alive_seconds, 3500);
        assert_eq!(play_log(&conn, &id, TimeMetric::Alive).unwrap().total_seconds, 3500);
        assert_eq!(play_log(&conn, "无此作品", TimeMetric::Foreground).unwrap().count, 0);
    }
}

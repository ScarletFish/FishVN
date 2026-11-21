use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
use windows::core::PWSTR;

use crate::error::{AppError, Result};

const DISCOVER_TIMEOUT: Duration = Duration::from_secs(60);
const DISCOVER_POLL: Duration = Duration::from_millis(300);

pub fn resolve_app_id(entry: &Path) -> Result<String> {
    find_app_id(entry).ok_or_else(|| {
        AppError::BadRequest("这个启动文件不在 Steam 游戏目录里，无法用 Steam 方式启动".into())
    })
}

pub fn probe_app_id(entry: &Path) -> Option<String> {
    find_app_id(entry)
}

fn find_app_id(entry: &Path) -> Option<String> {
    let steam_dirs = steamlocate::locate_all().ok()?;
    let target = path_key(entry);

    for steam_dir in steam_dirs {
        let Ok(libraries) = steam_dir.libraries() else {
            continue;
        };
        for library in libraries.flatten() {
            for app in library.apps().flatten() {
                let app_dir = library.resolve_app_dir(&app);
                if path_within(&target, &path_key(&app_dir)) {
                    return Some(app.app_id.to_string());
                }
            }
        }
    }
    None
}

pub fn wait_for_launched_pid(dir: &Path, before: &HashSet<u32>) -> Result<u32> {
    let target = path_key(dir);
    let deadline = Instant::now() + DISCOVER_TIMEOUT;
    loop {
        if let Some(pid) = foreground_pid() {
            if before.contains(&pid)
                && process_image(pid).is_some_and(|image| path_within(&path_key(&image), &target))
            {
                return Ok(pid);
            }
        }
        for (pid, image) in running_processes() {
            if before.contains(&pid) || !path_within(&path_key(&image), &target) {
                continue;
            }
            return Ok(pid);
        }
        if Instant::now() >= deadline {
            return Err(AppError::BadRequest(
                "Steam 已启动，但没在游戏目录里找到运行中的游戏进程".into(),
            ));
        }
        std::thread::sleep(DISCOVER_POLL);
    }
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

pub fn pids_in_dir(dir: &Path) -> HashSet<u32> {
    let target = path_key(dir);
    running_processes()
        .into_iter()
        .filter(|(_, image)| path_within(&path_key(image), &target))
        .map(|(pid, _)| pid)
        .collect()
}

fn running_processes() -> Vec<(u32, PathBuf)> {
    let mut out = Vec::new();
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return out;
        };
        if snapshot.is_invalid() {
            return out;
        }
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if let Some(image) = process_image(entry.th32ProcessID) {
                    out.push((entry.th32ProcessID, image));
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    out
}

fn process_image(pid: u32) -> Option<PathBuf> {
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return None;
        };
        if handle.is_invalid() {
            return None;
        }
        let mut buf = vec![0u16; 1024];
        let mut len = buf.len() as u32;
        let named = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        named.ok()?;
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        (!path.is_empty()).then(|| PathBuf::from(path))
    }
}

pub fn path_within(path: &str, base: &str) -> bool {
    let path = path.trim_end_matches(['\\', '/']);
    let base = base.trim_end_matches(['\\', '/']);
    if path.eq_ignore_ascii_case(base) {
        return true;
    }
    let prefix = format!("{base}\\");
    path.len() > prefix.len() && path[..prefix.len()].eq_ignore_ascii_case(&prefix)
}

fn path_key(path: &Path) -> String {
    let normalized = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    normalized
        .to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 路径归属按目录边界判定且忽略大小写() {
        assert!(path_within(
            "d:\\steam\\steamapps\\common\\game\\bin\\game.exe",
            "D:\\Steam\\steamapps\\common\\Game"
        ));
        assert!(path_within(
            "d:\\steam\\steamapps\\common\\game",
            "D:\\Steam\\steamapps\\common\\Game"
        ));
    }

    #[test]
    fn 相邻同名前缀目录不算归属() {
        assert!(!path_within(
            "d:\\steam\\steamapps\\common\\game2\\game.exe",
            "d:\\steam\\steamapps\\common\\game"
        ));
        assert!(!path_within(
            "d:\\other\\game\\game.exe",
            "d:\\steam\\steamapps\\common\\game"
        ));
    }
}

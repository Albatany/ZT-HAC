//! Dynamic App Restrictor: a lightweight poller that terminates restricted
//! binaries until the UI fires an unlock. Rules map name -> unlocked.

use serde::Serialize;
use std::{collections::HashMap, sync::{Arc, Mutex}, thread, time::Duration};
use sysinfo::{Process, System};
use tauri::{AppHandle, Emitter};

pub type Rules = Arc<Mutex<HashMap<String, bool>>>;

#[derive(Serialize)]
pub struct ProcInfo { pub pid: u32, pub name: String, pub memory_kb: u64 }

#[derive(Serialize)]
pub struct Restriction { pub name: String, pub unlocked: bool }

/// Lowercases the name; on Linux the kernel truncates `comm` to 15 chars.
pub fn normalize(name: &str) -> Result<String, String> {
    let n = name.trim().to_lowercase();
    if n.is_empty() {
        return Err("process name is empty".into());
    }
    #[cfg(target_os = "linux")]
    let n: String = n.chars().take(15).collect();
    Ok(n)
}

pub fn list_processes() -> Vec<ProcInfo> {
    let mut sys = System::new();
    sys.refresh_processes();
    let mut v: Vec<ProcInfo> = sys
        .processes()
        .iter()
        .map(|(pid, p)| ProcInfo { pid: pid.as_u32(), name: p.name().to_string(), memory_kb: p.memory() / 1024 })
        .collect();
    v.sort_by(|a, b| b.memory_kb.cmp(&a.memory_kb));
    v.truncate(150);
    v
}

#[cfg(target_os = "windows")]
fn kill(p: &Process) -> bool {
    use winapi::um::{handleapi::CloseHandle, processthreadsapi::{OpenProcess, TerminateProcess}, winnt::PROCESS_TERMINATE};
    unsafe {
        let h = OpenProcess(PROCESS_TERMINATE, 0, p.pid().as_u32());
        if h.is_null() {
            return false;
        }
        let ok = TerminateProcess(h, 1) != 0;
        CloseHandle(h);
        ok
    }
}

#[cfg(not(target_os = "windows"))]
fn kill(p: &Process) -> bool {
    p.kill() // SIGKILL via sysinfo (reads /proc on Linux)
}

pub fn spawn_monitor(app: AppHandle, rules: Rules) {
    thread::spawn(move || {
        let mut sys = System::new();
        loop {
            thread::sleep(Duration::from_millis(800));
            let active: Vec<String> = rules.lock().unwrap().iter().filter(|(_, u)| !**u).map(|(k, _)| k.clone()).collect();
            if active.is_empty() {
                continue;
            }
            sys.refresh_processes();
            for p in sys.processes().values() {
                let name = p.name().to_lowercase();
                if active.contains(&name) && kill(p) {
                    let _ = app.emit("app-blocked", name);
                }
            }
        }
    });
}

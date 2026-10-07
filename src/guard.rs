//! Environment Guard: watches sensitive files and seals them (via the vault)
//! once every known editor/IDE process has closed. The password lives only in
//! this process's memory while the guard is armed.

use crate::vault;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread, time::Duration};
use sysinfo::System;
use tauri::{AppHandle, Emitter};

const EDITORS: &[&str] = &[
    "code", "code-oss", "codium", "nvim", "vim", "nano", "emacs", "gedit", "kate", "zed",
    "sublime_text", "idea", "pycharm", "webstorm", "notepad.exe", "notepad++.exe",
    "code.exe", "sublime_text.exe", "idea64.exe",
];

pub struct GuardRuntime {
    _watcher: RecommendedWatcher,
    stop: Arc<AtomicBool>,
    pub paths: Vec<String>,
}

impl Drop for GuardRuntime {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

pub fn arm(app: AppHandle, paths: Vec<String>, password: String) -> Result<GuardRuntime, String> {
    if password.chars().count() < 8 {
        return Err("password must be at least 8 characters".into());
    }
    let targets: Vec<PathBuf> = paths.iter().filter(|p| !p.trim().is_empty()).map(PathBuf::from).collect();
    if targets.is_empty() {
        return Err("add at least one file to guard".into());
    }

    // File watcher: parent directories are watched so events survive the file being sealed.
    let watched = targets.clone();
    let app_w = app.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(ev) = res {
            for p in ev.paths.iter().filter(|p| watched.contains(p)) {
                let _ = app_w.emit("guard-event", format!("{:?}: {}", ev.kind, p.display()));
            }
        }
    })
    .map_err(|e| e.to_string())?;
    for t in &targets {
        let dir = t.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
        watcher.watch(dir, RecursiveMode::NonRecursive).map_err(|e| e.to_string())?;
    }

    // Editor monitor: seal on the open -> closed transition.
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let tg = targets.clone();
    thread::spawn(move || {
        let mut sys = System::new();
        let mut was_open = false;
        while !flag.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(2));
            sys.refresh_processes();
            let open = sys.processes().values().any(|p| {
                let n = p.name().to_lowercase();
                EDITORS.contains(&n.as_str())
            });
            if was_open && !open {
                for t in tg.iter().filter(|t| t.is_file()) {
                    let msg = match vault::encrypt_path(&t.to_string_lossy(), &password) {
                        Ok(_) => format!("Sealed {}", t.display()),
                        Err(e) => format!("Failed to seal {}: {e}", t.display()),
                    };
                    let _ = app.emit("guard-locked", msg);
                }
            }
            was_open = open;
        }
    });

    Ok(GuardRuntime { _watcher: watcher, stop, paths })
}

//! ZT-HAC core: shared state and the Tauri IPC command surface.

mod guard;
mod network;
mod restrictor;
mod vault;

use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[derive(Default)]
struct AppState {
    rules: restrictor::Rules,
    ports: Mutex<Vec<network::PortRule>>,
    guard: Mutex<Option<guard::GuardRuntime>>,
}

#[derive(Serialize)]
struct GuardStatus { armed: bool, paths: Vec<String> }

fn guard_status_of(g: &Option<guard::GuardRuntime>) -> GuardStatus {
    GuardStatus { armed: g.is_some(), paths: g.as_ref().map(|r| r.paths.clone()).unwrap_or_default() }
}

// ---- Dynamic App Restrictor -------------------------------------------------
#[tauri::command]
async fn list_processes() -> Result<Vec<restrictor::ProcInfo>, String> {
    Ok(restrictor::list_processes())
}

#[tauri::command]
fn list_restrictions(state: State<AppState>) -> Vec<restrictor::Restriction> {
    let mut v: Vec<_> = state.rules.lock().unwrap().iter()
        .map(|(n, u)| restrictor::Restriction { name: n.clone(), unlocked: *u }).collect();
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

#[tauri::command]
fn restrict_app(name: String, state: State<AppState>) -> Result<(), String> {
    state.rules.lock().unwrap().insert(restrictor::normalize(&name)?, false);
    Ok(())
}

#[tauri::command]
fn unlock_app(name: String, state: State<AppState>) -> Result<(), String> {
    state.rules.lock().unwrap().insert(restrictor::normalize(&name)?, true);
    Ok(())
}

#[tauri::command]
fn relock_app(name: String, state: State<AppState>) -> Result<(), String> {
    state.rules.lock().unwrap().insert(restrictor::normalize(&name)?, false);
    Ok(())
}

#[tauri::command]
fn remove_restriction(name: String, state: State<AppState>) -> Result<(), String> {
    state.rules.lock().unwrap().remove(&restrictor::normalize(&name)?);
    Ok(())
}

// ---- One-Click Folder Vault -------------------------------------------------
#[tauri::command]
async fn encrypt_vault(path: String, password: String) -> Result<vault::VaultReport, String> {
    tauri::async_runtime::spawn_blocking(move || vault::encrypt_path(&path, &password))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn decrypt_vault(path: String, password: String) -> Result<vault::VaultReport, String> {
    tauri::async_runtime::spawn_blocking(move || vault::decrypt_path(&path, &password))
        .await.map_err(|e| e.to_string())?
}

// ---- Automated Environment Guard --------------------------------------------
#[tauri::command]
fn guard_arm(app: AppHandle, paths: Vec<String>, password: String, state: State<AppState>) -> Result<GuardStatus, String> {
    let rt = guard::arm(app, paths, password)?;
    let mut g = state.guard.lock().unwrap();
    *g = Some(rt); // dropping a previous runtime stops its threads
    Ok(guard_status_of(&g))
}

#[tauri::command]
fn guard_disarm(state: State<AppState>) -> GuardStatus {
    let mut g = state.guard.lock().unwrap();
    *g = None;
    guard_status_of(&g)
}

#[tauri::command]
fn guard_status(state: State<AppState>) -> GuardStatus {
    guard_status_of(&state.guard.lock().unwrap())
}

// ---- Network Port Stopper ---------------------------------------------------
#[tauri::command]
fn list_port_rules(state: State<AppState>) -> Vec<network::PortRule> {
    state.ports.lock().unwrap().clone()
}

#[tauri::command]
async fn block_port(port: u16, protocol: String, state: State<'_, AppState>) -> Result<Vec<network::PortRule>, String> {
    network::block(port, &protocol)?;
    let mut v = state.ports.lock().unwrap();
    let rule = network::PortRule { port, protocol };
    if !v.contains(&rule) {
        v.push(rule);
    }
    Ok(v.clone())
}

#[tauri::command]
async fn unblock_port(port: u16, protocol: String, state: State<'_, AppState>) -> Result<Vec<network::PortRule>, String> {
    network::unblock(port, &protocol)?;
    let mut v = state.ports.lock().unwrap();
    v.retain(|r| !(r.port == port && r.protocol == protocol));
    Ok(v.clone())
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            let rules = app.state::<AppState>().rules.clone();
            restrictor::spawn_monitor(app.handle().clone(), rules);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_processes, list_restrictions, restrict_app, unlock_app, relock_app, remove_restriction,
            encrypt_vault, decrypt_vault,
            guard_arm, guard_disarm, guard_status,
            list_port_rules, block_port, unblock_port,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ZT-HAC");
}

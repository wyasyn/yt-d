use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::models::{now_millis, HistoryItem};
use crate::state::AppState;

fn history_file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("cannot resolve app data dir: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("history.json"))
}

pub fn load(app: &AppHandle) -> Vec<HistoryItem> {
    history_file(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn persist(app: &AppHandle, items: &[HistoryItem]) {
    if let Ok(path) = history_file(app) {
        if let Ok(json) = serde_json::to_string_pretty(items) {
            let _ = std::fs::write(path, json);
        }
    }
}

/// Snapshot a finished (done or failed) download into persistent history.
pub fn record(app: &AppHandle, download_id: u64) {
    let state = app.state::<AppState>();
    let entry = {
        let downloads = state.downloads.lock().unwrap();
        match downloads.get(&download_id) {
            Some(e) => e.clone(),
            None => return,
        }
    };
    let item = HistoryItem {
        id: entry.created_at, // unique enough and stable across sessions
        url: entry.url,
        title: entry.title,
        thumbnail: entry.thumbnail,
        quality: entry.quality,
        file_path: entry.file_path,
        total_bytes: entry.total_bytes,
        status: entry.status,
        finished_at: now_millis(),
    };
    let mut history = state.history.lock().unwrap();
    history.insert(0, item);
    history.truncate(500);
    persist(app, &history);
}

#[tauri::command]
pub fn get_history(state: tauri::State<'_, AppState>) -> Vec<HistoryItem> {
    state.history.lock().unwrap().clone()
}

#[tauri::command]
pub fn remove_history_item(app: AppHandle, state: tauri::State<'_, AppState>, id: u64) {
    let mut history = state.history.lock().unwrap();
    history.retain(|h| h.id != id);
    persist(&app, &history);
}

#[tauri::command]
pub fn clear_history(app: AppHandle, state: tauri::State<'_, AppState>) {
    let mut history = state.history.lock().unwrap();
    history.clear();
    persist(&app, &history);
}

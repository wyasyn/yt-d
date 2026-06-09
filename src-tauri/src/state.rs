use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use crate::engine::EnginePaths;
use crate::models::{DownloadEntry, HistoryItem, Settings};

#[derive(Default)]
pub struct AppState {
    pub settings: Mutex<Settings>,
    pub engine: Mutex<EnginePaths>,
    /// Active + recent downloads for this session, keyed by id.
    pub downloads: Mutex<HashMap<u64, DownloadEntry>>,
    /// Running yt-dlp child processes (stdout/stderr already taken by reader threads).
    pub children: Mutex<HashMap<u64, std::process::Child>>,
    pub history: Mutex<Vec<HistoryItem>>,
    pub next_id: AtomicU64,
}

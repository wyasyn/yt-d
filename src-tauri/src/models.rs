use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// Custom download directory; None = OS Downloads folder.
    pub download_dir: Option<String>,
    pub default_quality: String,
    pub max_concurrent: usize,
    pub notifications: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_dir: None,
            default_quality: "best".into(),
            max_concurrent: 3,
            notifications: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Queued,
    Downloading,
    Paused,
    Merging,
    Done,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub url: String,
    /// "best" | "2160" | "1080" | "720" | "480" | "audio-mp3" | "audio-m4a"
    pub quality: String,
    pub playlist: bool,
    pub title: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadEntry {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub quality: String,
    pub playlist: bool,
    pub status: Status,
    pub percent: f64,
    pub speed: Option<f64>,
    pub eta: Option<f64>,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    /// "Item 3 of 12" when downloading a playlist.
    pub playlist_progress: Option<String>,
    pub file_path: Option<String>,
    pub error: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub quality: String,
    pub file_path: Option<String>,
    pub total_bytes: Option<u64>,
    pub status: Status,
    pub finished_at: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub uploader: Option<String>,
    pub duration: Option<f64>,
    pub is_playlist: bool,
    pub entry_count: Option<usize>,
    pub extractor: Option<String>,
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

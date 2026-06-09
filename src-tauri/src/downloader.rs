use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::atomic::Ordering;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::models::{now_millis, DownloadEntry, DownloadRequest, MediaInfo, Status};
use crate::state::AppState;
use crate::{history, settings};

const FILE_MARKER: &str = "GRABIT_FILE::";

/// Async command for one-shot engine calls; hides the console window on Windows.
pub fn engine_command(program: &str) -> tokio::process::Command {
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut cmd = tokio::process::Command::new(program);
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Blocking variant for long-lived downloads. Uses std::process so child
/// handles can live in (and be dropped from) managed state without a runtime.
fn engine_command_std(program: &str) -> std::process::Command {
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut cmd = std::process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

fn ytdlp_path(app: &AppHandle) -> Result<String, String> {
    let state = app.state::<AppState>();
    let engine = state.engine.lock().unwrap();
    engine
        .ytdlp
        .clone()
        .ok_or_else(|| "yt-dlp is not installed yet".to_string())
}

fn emit_snapshot(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut list: Vec<DownloadEntry> = state.downloads.lock().unwrap().values().cloned().collect();
    list.sort_by_key(|e| std::cmp::Reverse(e.created_at));
    let _ = app.emit("downloads-changed", list);
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    id: u64,
    status: Status,
    percent: f64,
    speed: Option<f64>,
    eta: Option<f64>,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
    playlist_progress: Option<String>,
}

#[tauri::command]
pub async fn fetch_metadata(app: AppHandle, url: String) -> Result<MediaInfo, String> {
    let ytdlp = ytdlp_path(&app)?;
    let output = engine_command(&ytdlp)
        .args(["-J", "--flat-playlist", "--no-warnings", "--", &url])
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("failed to run yt-dlp: {e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let msg = err
            .lines()
            .rev()
            .find(|l| l.contains("ERROR"))
            .unwrap_or("could not read this URL")
            .trim();
        return Err(msg.replace("ERROR:", "").trim().to_string());
    }
    let info: Value = serde_json::from_slice(&output.stdout)
        .map_err(|_| "unexpected response from yt-dlp".to_string())?;

    let is_playlist = info.get("_type").and_then(Value::as_str) == Some("playlist");
    let thumbnail = info
        .get("thumbnail")
        .and_then(Value::as_str)
        .map(String::from)
        .or_else(|| {
            info.get("thumbnails")
                .and_then(Value::as_array)
                .and_then(|t| t.last())
                .and_then(|t| t.get("url"))
                .and_then(Value::as_str)
                .map(String::from)
        })
        .or_else(|| {
            // Flat playlists often carry thumbnails only on entries.
            info.get("entries")
                .and_then(Value::as_array)
                .and_then(|e| e.first())
                .and_then(|e| {
                    e.get("thumbnails")
                        .and_then(Value::as_array)
                        .and_then(|t| t.last())
                        .and_then(|t| t.get("url"))
                        .and_then(Value::as_str)
                        .map(String::from)
                })
        });

    Ok(MediaInfo {
        url,
        title: info
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Untitled")
            .to_string(),
        thumbnail,
        uploader: info
            .get("uploader")
            .or_else(|| info.get("channel"))
            .or_else(|| info.get("uploader_id"))
            .and_then(Value::as_str)
            .map(String::from),
        duration: info.get("duration").and_then(Value::as_f64),
        is_playlist,
        entry_count: info
            .get("entries")
            .and_then(Value::as_array)
            .map(|e| e.len()),
        extractor: info
            .get("extractor_key")
            .and_then(Value::as_str)
            .map(String::from),
    })
}

fn build_args(app: &AppHandle, entry: &DownloadEntry) -> Result<Vec<String>, String> {
    let state = app.state::<AppState>();
    let (ffmpeg_dir, app_settings) = {
        let engine = state.engine.lock().unwrap();
        (engine.ffmpeg_dir.clone(), state.settings.lock().unwrap().clone())
    };
    let dest = settings::download_dir(app, &app_settings)?;
    std::fs::create_dir_all(&dest).map_err(|e| format!("cannot create download folder: {e}"))?;

    let mut args: Vec<String> = vec![
        "--no-warnings".into(),
        "--newline".into(),
        "--progress".into(),
        "--progress-template".into(),
        "%(progress)j".into(),
        "--no-mtime".into(),
        "--continue".into(),
        "--print".into(),
        format!("after_move:{FILE_MARKER}%(filepath)s"),
        "--no-simulate".into(),
        // --print implies quiet; re-enable [Merger]/playlist lines for stage detection
        "--no-quiet".into(),
        "-o".into(),
        dest.join("%(title)s [%(id)s].%(ext)s")
            .to_string_lossy()
            .into_owned(),
    ];
    if let Some(dir) = ffmpeg_dir {
        args.push("--ffmpeg-location".into());
        args.push(dir);
    }
    args.push(if entry.playlist { "--yes-playlist" } else { "--no-playlist" }.into());

    match entry.quality.as_str() {
        "audio-mp3" => args.extend(["-x".into(), "--audio-format".into(), "mp3".into()]),
        "audio-m4a" => args.extend(["-x".into(), "--audio-format".into(), "m4a".into()]),
        "best" => args.extend(["-f".into(), "bv*+ba/b".into()]),
        height => args.extend([
            "-f".into(),
            format!("bv*[height<={height}]+ba/b[height<={height}]"),
        ]),
    }

    args.push("--".into());
    args.push(entry.url.clone());
    Ok(args)
}

fn parse_progress_line(line: &str, entry: &mut DownloadEntry) -> bool {
    let Ok(p) = serde_json::from_str::<Value>(line) else {
        return false;
    };
    let status = p.get("status").and_then(Value::as_str).unwrap_or("");
    let downloaded = p.get("downloaded_bytes").and_then(Value::as_u64);
    let total = p
        .get("total_bytes")
        .and_then(Value::as_u64)
        .or_else(|| p.get("total_bytes_estimate").and_then(Value::as_f64).map(|f| f as u64));
    entry.downloaded_bytes = downloaded.or(entry.downloaded_bytes);
    entry.total_bytes = total.or(entry.total_bytes);
    entry.speed = p.get("speed").and_then(Value::as_f64);
    entry.eta = p.get("eta").and_then(Value::as_f64);
    if let (Some(d), Some(t)) = (downloaded, total) {
        if t > 0 {
            entry.percent = (d as f64 / t as f64 * 100.0).min(100.0);
        }
    }
    if status == "finished" {
        // This fragment finished; postprocessing/merging may follow.
        entry.percent = 100.0;
        entry.speed = None;
        entry.eta = None;
    }
    true
}

fn spawn_job(app: &AppHandle, id: u64) -> Result<(), String> {
    let state = app.state::<AppState>();
    let entry = {
        let mut downloads = state.downloads.lock().unwrap();
        let entry = downloads.get_mut(&id).ok_or("download not found")?;
        entry.status = Status::Downloading;
        entry.speed = None;
        entry.eta = None;
        entry.error = None;
        entry.clone()
    };
    let ytdlp = ytdlp_path(app)?;
    let args = build_args(app, &entry)?;

    let mut child = engine_command_std(&ytdlp)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start yt-dlp: {e}"))?;

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;
    state.children.lock().unwrap().insert(id, child);

    let app = app.clone();
    std::thread::spawn(move || {
        let stderr_thread = std::thread::spawn(move || {
            let mut last_error = None;
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if line.contains("ERROR") {
                    last_error = Some(line.replace("ERROR:", "").trim().to_string());
                }
            }
            last_error
        });

        let mut last_emit = std::time::Instant::now();
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            let state = app.state::<AppState>();
            let event = {
                let mut downloads = state.downloads.lock().unwrap();
                let Some(entry) = downloads.get_mut(&id) else { break };
                if let Some(path) = line.strip_prefix(FILE_MARKER) {
                    entry.file_path = Some(path.to_string());
                } else if line.starts_with('{') {
                    if entry.status == Status::Downloading {
                        parse_progress_line(&line, entry);
                    }
                } else if line.starts_with("[download] Downloading item ") {
                    entry.playlist_progress =
                        Some(line.trim_start_matches("[download] Downloading ").to_string());
                    entry.percent = 0.0;
                } else if line.starts_with("[Merger]")
                    || line.starts_with("[ExtractAudio]")
                    || line.starts_with("[VideoConvertor]")
                {
                    if entry.status == Status::Downloading {
                        entry.status = Status::Merging;
                    }
                }
                ProgressEvent {
                    id,
                    status: entry.status,
                    percent: entry.percent,
                    speed: entry.speed,
                    eta: entry.eta,
                    downloaded_bytes: entry.downloaded_bytes,
                    total_bytes: entry.total_bytes,
                    playlist_progress: entry.playlist_progress.clone(),
                }
            };
            // Throttle UI events; status changes always matter, raw progress at ~10/s max.
            if last_emit.elapsed().as_millis() >= 100 || event.status != Status::Downloading {
                let _ = app.emit("download-progress", event);
                last_emit = std::time::Instant::now();
            }
        }

        let exit_ok = {
            let state = app.state::<AppState>();
            let child = state.children.lock().unwrap().remove(&id);
            match child {
                Some(mut child) => child.wait().map(|s| s.success()).unwrap_or(false),
                None => false,
            }
        };
        let last_error = stderr_thread.join().ok().flatten();

        let (final_status, notify) = {
            let state = app.state::<AppState>();
            let mut downloads = state.downloads.lock().unwrap();
            match downloads.get_mut(&id) {
                Some(entry) => {
                    let notify;
                    if entry.status == Status::Paused || entry.status == Status::Cancelled {
                        // User-initiated stop; keep that status.
                        notify = None;
                    } else if exit_ok {
                        entry.status = Status::Done;
                        entry.percent = 100.0;
                        entry.speed = None;
                        entry.eta = None;
                        notify = Some((true, entry.title.clone()));
                    } else {
                        entry.status = Status::Error;
                        entry.error =
                            Some(last_error.unwrap_or_else(|| "download failed".to_string()));
                        notify = Some((false, entry.title.clone()));
                    }
                    (entry.status, notify)
                }
                None => (Status::Cancelled, None),
            }
        };

        if matches!(final_status, Status::Done | Status::Error) {
            history::record(&app, id);
        }
        if let Some((success, title)) = notify {
            let enabled = app
                .state::<AppState>()
                .settings
                .lock()
                .unwrap()
                .notifications;
            if enabled {
                let _ = app
                    .notification()
                    .builder()
                    .title(if success { "Download complete" } else { "Download failed" })
                    .body(title)
                    .show();
            }
        }
        emit_snapshot(&app);
        pump_queue(&app);
    });
    Ok(())
}

/// Start queued downloads while there are free slots.
fn pump_queue(app: &AppHandle) {
    let state = app.state::<AppState>();
    let to_start: Vec<u64> = {
        let downloads = state.downloads.lock().unwrap();
        let max = state.settings.lock().unwrap().max_concurrent.max(1);
        let active = downloads
            .values()
            .filter(|e| matches!(e.status, Status::Downloading | Status::Merging))
            .count();
        if active >= max {
            return;
        }
        let mut queued: Vec<&DownloadEntry> = downloads
            .values()
            .filter(|e| e.status == Status::Queued)
            .collect();
        queued.sort_by_key(|e| e.created_at);
        queued.iter().take(max - active).map(|e| e.id).collect()
    };
    for id in to_start {
        if let Err(err) = spawn_job(app, id) {
            let state = app.state::<AppState>();
            if let Some(entry) = state.downloads.lock().unwrap().get_mut(&id) {
                entry.status = Status::Error;
                entry.error = Some(err);
            }
            history::record(app, id);
        }
    }
    emit_snapshot(app);
}

fn kill_child(app: &AppHandle, id: u64) {
    let state = app.state::<AppState>();
    let mut children = state.children.lock().unwrap();
    if let Some(child) = children.get_mut(&id) {
        let _ = child.kill();
    }
}

/// Stop every running yt-dlp process; called when the app exits.
pub fn kill_all(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut children = state.children.lock().unwrap();
    for (_, child) in children.iter_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
    children.clear();
}

#[tauri::command]
pub fn start_download(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    request: DownloadRequest,
) -> Result<u64, String> {
    let id = state.next_id.fetch_add(1, Ordering::SeqCst);
    let entry = DownloadEntry {
        id,
        url: request.url.clone(),
        title: request.title.unwrap_or_else(|| request.url.clone()),
        thumbnail: request.thumbnail,
        quality: request.quality,
        playlist: request.playlist,
        status: Status::Queued,
        percent: 0.0,
        speed: None,
        eta: None,
        downloaded_bytes: None,
        total_bytes: None,
        playlist_progress: None,
        file_path: None,
        error: None,
        created_at: now_millis(),
    };
    state.downloads.lock().unwrap().insert(id, entry);
    pump_queue(&app);
    Ok(id)
}

#[tauri::command]
pub fn pause_download(app: AppHandle, state: tauri::State<'_, AppState>, id: u64) {
    {
        let mut downloads = state.downloads.lock().unwrap();
        if let Some(entry) = downloads.get_mut(&id) {
            match entry.status {
                Status::Downloading | Status::Merging => {
                    entry.status = Status::Paused;
                    entry.speed = None;
                    entry.eta = None;
                }
                Status::Queued => entry.status = Status::Paused,
                _ => return,
            }
        }
    }
    kill_child(&app, id);
    emit_snapshot(&app);
}

#[tauri::command]
pub fn resume_download(app: AppHandle, state: tauri::State<'_, AppState>, id: u64) {
    {
        let mut downloads = state.downloads.lock().unwrap();
        match downloads.get_mut(&id) {
            Some(entry) if matches!(entry.status, Status::Paused | Status::Error) => {
                entry.status = Status::Queued;
                entry.error = None;
            }
            _ => return,
        }
    }
    pump_queue(&app);
}

#[tauri::command]
pub fn cancel_download(app: AppHandle, state: tauri::State<'_, AppState>, id: u64) {
    let partial = {
        let mut downloads = state.downloads.lock().unwrap();
        let finished = matches!(
            downloads.get(&id).map(|e| e.status),
            Some(Status::Done | Status::Error | Status::Cancelled)
        );
        if finished {
            downloads.remove(&id);
            None
        } else if let Some(entry) = downloads.get_mut(&id) {
            entry.status = Status::Cancelled;
            entry.speed = None;
            entry.eta = None;
            entry.file_path.clone().map(|p| format!("{p}.part"))
        } else {
            None
        }
    };
    kill_child(&app, id);
    if let Some(part) = partial {
        let _ = std::fs::remove_file(part);
    }
    emit_snapshot(&app);
}

#[tauri::command]
pub fn get_queue(state: tauri::State<'_, AppState>) -> Vec<DownloadEntry> {
    let mut list: Vec<DownloadEntry> = state.downloads.lock().unwrap().values().cloned().collect();
    list.sort_by_key(|e| std::cmp::Reverse(e.created_at));
    list
}

#[tauri::command]
pub fn clear_finished(app: AppHandle, state: tauri::State<'_, AppState>) {
    state
        .downloads
        .lock()
        .unwrap()
        .retain(|_, e| !matches!(e.status, Status::Done | Status::Error | Status::Cancelled));
    emit_snapshot(&app);
}

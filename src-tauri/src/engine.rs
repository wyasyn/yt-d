use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnginePaths {
    pub ytdlp: Option<String>,
    pub ffmpeg_dir: Option<String>,
    pub ready: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SetupProgress {
    component: String, // "yt-dlp" | "ffmpeg"
    phase: String,     // "downloading" | "extracting" | "done" | "error"
    percent: f64,
    message: Option<String>,
}

fn emit_setup(app: &AppHandle, component: &str, phase: &str, percent: f64, message: Option<String>) {
    let _ = app.emit(
        "engine-setup-progress",
        SetupProgress {
            component: component.into(),
            phase: phase.into(),
            percent,
            message,
        },
    );
}

fn bin_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("bin");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    let name = exe(name);
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(&name))
            .find(|p| p.is_file())
    })
}

/// Locate yt-dlp and ffmpeg: prefer our app-data copies, fall back to system PATH.
pub fn detect(app: &AppHandle) -> EnginePaths {
    let mut paths = EnginePaths::default();
    if let Ok(dir) = bin_dir(app) {
        let local_ytdlp = dir.join(exe("yt-dlp"));
        if local_ytdlp.is_file() {
            paths.ytdlp = Some(local_ytdlp.to_string_lossy().into_owned());
        }
        if dir.join(exe("ffmpeg")).is_file() {
            paths.ffmpeg_dir = Some(dir.to_string_lossy().into_owned());
        }
    }
    if paths.ytdlp.is_none() {
        paths.ytdlp = find_on_path("yt-dlp").map(|p| p.to_string_lossy().into_owned());
    }
    if paths.ffmpeg_dir.is_none() {
        paths.ffmpeg_dir = find_on_path("ffmpeg")
            .and_then(|p| p.parent().map(|d| d.to_string_lossy().into_owned()));
    }
    paths.ready = paths.ytdlp.is_some() && paths.ffmpeg_dir.is_some();
    paths
}

async fn download_file(
    app: &AppHandle,
    component: &str,
    url: &str,
    dest: &Path,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("GrabIt/0.1")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("network error: {e}"))?
        .error_for_status()
        .map_err(|e| format!("download failed: {e}"))?;
    let total = resp.content_length().unwrap_or(0);
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        if total > 0 {
            emit_setup(
                app,
                component,
                "downloading",
                downloaded as f64 / total as f64 * 100.0,
                None,
            );
        }
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| e.to_string())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn ytdlp_url() -> &'static str {
    if cfg!(windows) {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    }
}

/// Extract just ffmpeg/ffprobe out of the downloaded archive into bin/.
fn extract_ffmpeg(archive: &Path, dir: &Path) -> Result<(), String> {
    let wanted = [exe("ffmpeg"), exe("ffprobe")];
    let is_zip = archive.extension().is_some_and(|e| e == "zip");
    if is_zip {
        let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();
            if let Some(base) = name.rsplit('/').next() {
                if wanted.iter().any(|w| w == base) {
                    let dest = dir.join(base);
                    let mut out = std::fs::File::create(&dest).map_err(|e| e.to_string())?;
                    std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
                    make_executable(&dest)?;
                }
            }
        }
    } else {
        let file = std::fs::File::open(archive).map_err(|e| e.to_string())?;
        let decompressed = xz2::read::XzDecoder::new(file);
        let mut tar = tar::Archive::new(decompressed);
        for entry in tar.entries().map_err(|e| e.to_string())? {
            let mut entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path().map_err(|e| e.to_string())?.into_owned();
            if let Some(base) = path.file_name().and_then(|n| n.to_str()) {
                if wanted.iter().any(|w| w == base) {
                    let dest = dir.join(base);
                    entry.unpack(&dest).map_err(|e| e.to_string())?;
                    make_executable(&dest)?;
                }
            }
        }
    }
    Ok(())
}

async fn install_ffmpeg(app: &AppHandle, dir: &Path) -> Result<(), String> {
    let (url, archive_name) = if cfg!(windows) {
        (
            "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip",
            "ffmpeg.zip",
        )
    } else if cfg!(target_os = "macos") {
        ("https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip", "ffmpeg.zip")
    } else {
        (
            "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-linux64-gpl.tar.xz",
            "ffmpeg.tar.xz",
        )
    };
    let archive = dir.join(archive_name);
    download_file(app, "ffmpeg", url, &archive).await?;
    emit_setup(app, "ffmpeg", "extracting", 100.0, None);
    extract_ffmpeg(&archive, dir)?;
    let _ = std::fs::remove_file(&archive);
    // macOS evermeet builds ship ffmpeg only; ffprobe is optional for our usage.
    if !dir.join(exe("ffmpeg")).is_file() {
        return Err("ffmpeg binary not found in downloaded archive".into());
    }
    Ok(())
}

/// Download whichever of yt-dlp/ffmpeg is missing into the app-data bin dir.
#[tauri::command]
pub async fn setup_engine(app: AppHandle) -> Result<EnginePaths, String> {
    let dir = bin_dir(&app)?;
    let current = detect(&app);

    if current.ytdlp.is_none() {
        emit_setup(&app, "yt-dlp", "downloading", 0.0, None);
        let dest = dir.join(exe("yt-dlp"));
        download_file(&app, "yt-dlp", ytdlp_url(), &dest).await?;
        make_executable(&dest)?;
        emit_setup(&app, "yt-dlp", "done", 100.0, None);
    } else {
        emit_setup(&app, "yt-dlp", "done", 100.0, None);
    }

    if current.ffmpeg_dir.is_none() {
        emit_setup(&app, "ffmpeg", "downloading", 0.0, None);
        install_ffmpeg(&app, &dir).await?;
        emit_setup(&app, "ffmpeg", "done", 100.0, None);
    } else {
        emit_setup(&app, "ffmpeg", "done", 100.0, None);
    }

    let paths = detect(&app);
    if !paths.ready {
        return Err("engine setup did not produce working binaries".into());
    }
    let state = app.state::<AppState>();
    *state.engine.lock().unwrap() = paths.clone();
    Ok(paths)
}

#[tauri::command]
pub fn check_engine(app: AppHandle, state: tauri::State<'_, AppState>) -> EnginePaths {
    let paths = detect(&app);
    *state.engine.lock().unwrap() = paths.clone();
    paths
}

/// Update yt-dlp: self-update our copy, or re-download over a stale one.
#[tauri::command]
pub async fn update_engine(app: AppHandle) -> Result<String, String> {
    let ytdlp = {
        let state = app.state::<AppState>();
        let engine = state.engine.lock().unwrap();
        engine.ytdlp.clone().ok_or("yt-dlp not installed")?
    };
    let dir = bin_dir(&app)?;
    let is_ours = Path::new(&ytdlp).starts_with(&dir);
    if is_ours {
        // Re-download is more reliable than -U across platforms/builds.
        let dest = dir.join(exe("yt-dlp"));
        download_file(&app, "yt-dlp", ytdlp_url(), &dest).await?;
        make_executable(&dest)?;
        Ok("yt-dlp updated to the latest release".into())
    } else {
        let out = crate::downloader::engine_command(&ytdlp)
            .arg("--version")
            .output()
            .await
            .map_err(|e| e.to_string())?;
        let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok(format!(
            "Using system yt-dlp {version} — update it with your package manager"
        ))
    }
}

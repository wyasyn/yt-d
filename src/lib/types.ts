export type Status =
  | "queued"
  | "downloading"
  | "paused"
  | "merging"
  | "done"
  | "error"
  | "cancelled";

export type Quality =
  | "best"
  | "2160"
  | "1080"
  | "720"
  | "480"
  | "audio-mp3"
  | "audio-m4a";

export interface Settings {
  downloadDir: string | null;
  defaultQuality: Quality;
  maxConcurrent: number;
  notifications: boolean;
}

export interface EnginePaths {
  ytdlp: string | null;
  ffmpegDir: string | null;
  ready: boolean;
}

export interface EngineSetupProgress {
  component: "yt-dlp" | "ffmpeg";
  phase: "downloading" | "extracting" | "done" | "error";
  percent: number;
  message: string | null;
}

export interface MediaInfo {
  url: string;
  title: string;
  thumbnail: string | null;
  uploader: string | null;
  duration: number | null;
  isPlaylist: boolean;
  entryCount: number | null;
  extractor: string | null;
}

export interface DownloadEntry {
  id: number;
  url: string;
  title: string;
  thumbnail: string | null;
  quality: Quality;
  playlist: boolean;
  status: Status;
  percent: number;
  speed: number | null;
  eta: number | null;
  downloadedBytes: number | null;
  totalBytes: number | null;
  playlistProgress: string | null;
  filePath: string | null;
  error: string | null;
  createdAt: number;
}

export interface ProgressEvent {
  id: number;
  status: Status;
  percent: number;
  speed: number | null;
  eta: number | null;
  downloadedBytes: number | null;
  totalBytes: number | null;
  playlistProgress: string | null;
}

export interface HistoryItem {
  id: number;
  url: string;
  title: string;
  thumbnail: string | null;
  quality: Quality;
  filePath: string | null;
  totalBytes: number | null;
  status: Status;
  finishedAt: number;
}

export interface DownloadRequest {
  url: string;
  quality: Quality;
  playlist: boolean;
  title?: string;
  thumbnail?: string;
}

export const QUALITY_LABELS: Record<Quality, string> = {
  best: "Best quality",
  "2160": "4K (2160p)",
  "1080": "Full HD (1080p)",
  "720": "HD (720p)",
  "480": "SD (480p)",
  "audio-mp3": "Audio only (MP3)",
  "audio-m4a": "Audio only (M4A)",
};

export function formatBytes(bytes: number | null): string {
  if (bytes == null || bytes <= 0) return "";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${value.toFixed(value >= 100 || i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatSpeed(bytesPerSec: number | null): string {
  if (bytesPerSec == null || bytesPerSec <= 0) return "";
  return `${formatBytes(bytesPerSec)}/s`;
}

export function formatEta(seconds: number | null): string {
  if (seconds == null || seconds <= 0) return "";
  const s = Math.round(seconds);
  if (s < 60) return `${s}s left`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ${s % 60}s left`;
  return `${Math.floor(m / 60)}h ${m % 60}m left`;
}

export function formatDuration(seconds: number | null): string {
  if (seconds == null || seconds <= 0) return "";
  const s = Math.round(seconds);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const mm = h > 0 ? String(m).padStart(2, "0") : String(m);
  return `${h > 0 ? `${h}:` : ""}${mm}:${String(sec).padStart(2, "0")}`;
}

export function looksLikeUrl(text: string): boolean {
  return /^https?:\/\/\S+\.\S+/.test(text.trim());
}

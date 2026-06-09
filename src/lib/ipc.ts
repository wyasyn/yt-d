import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type {
  DownloadEntry,
  DownloadRequest,
  EnginePaths,
  EngineSetupProgress,
  HistoryItem,
  MediaInfo,
  ProgressEvent,
  Settings,
} from "./types";

export const checkEngine = () => invoke<EnginePaths>("check_engine");
export const setupEngine = () => invoke<EnginePaths>("setup_engine");
export const updateEngine = () => invoke<string>("update_engine");

export const fetchMetadata = (url: string) =>
  invoke<MediaInfo>("fetch_metadata", { url });

export const startDownload = (request: DownloadRequest) =>
  invoke<number>("start_download", { request });
export const pauseDownload = (id: number) => invoke("pause_download", { id });
export const resumeDownload = (id: number) => invoke("resume_download", { id });
export const cancelDownload = (id: number) => invoke("cancel_download", { id });
export const getQueue = () => invoke<DownloadEntry[]>("get_queue");
export const clearFinished = () => invoke("clear_finished");

export const getHistory = () => invoke<HistoryItem[]>("get_history");
export const removeHistoryItem = (id: number) =>
  invoke("remove_history_item", { id });
export const clearHistory = () => invoke("clear_history");

export const getSettings = () => invoke<Settings>("get_settings");
export const setSettings = (settings: Settings) =>
  invoke("set_settings", { settings });
export const getDefaultDownloadDir = () =>
  invoke<string>("get_default_download_dir");

export const openFile = (path: string) =>
  openPath(path).catch((e) => alert(`Could not open file: ${e}`));
export const showInFolder = (path: string) =>
  revealItemInDir(path).catch((e) => alert(`Could not show in folder: ${e}`));

export async function readClipboardUrl(): Promise<string | null> {
  try {
    const text = await readText();
    return text && /^https?:\/\/\S+\.\S+/.test(text.trim()) ? text.trim() : null;
  } catch {
    return null;
  }
}

export const pickFolder = () =>
  openDialog({ directory: true, multiple: false }) as Promise<string | null>;

export const onDownloadsChanged = (
  cb: (entries: DownloadEntry[]) => void,
): Promise<UnlistenFn> =>
  listen<DownloadEntry[]>("downloads-changed", (e) => cb(e.payload));

export const onDownloadProgress = (
  cb: (p: ProgressEvent) => void,
): Promise<UnlistenFn> =>
  listen<ProgressEvent>("download-progress", (e) => cb(e.payload));

export const onEngineSetupProgress = (
  cb: (p: EngineSetupProgress) => void,
): Promise<UnlistenFn> =>
  listen<EngineSetupProgress>("engine-setup-progress", (e) => cb(e.payload));

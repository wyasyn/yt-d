import type { DownloadEntry } from "../lib/types";
import { formatBytes, formatEta, formatSpeed } from "../lib/types";
import {
  cancelDownload,
  openFile,
  pauseDownload,
  resumeDownload,
  showInFolder,
} from "../lib/ipc";
import {
  CheckIcon,
  FilmIcon,
  FolderIcon,
  PauseIcon,
  PlayIcon,
  RetryIcon,
  XIcon,
} from "./Icons";

const STATUS_LABELS: Record<string, string> = {
  queued: "Queued",
  paused: "Paused",
  merging: "Processing",
  done: "Done",
  error: "Failed",
  cancelled: "Cancelled",
};

export function DownloadItem({ entry }: { entry: DownloadEntry }) {
  const active = entry.status === "downloading";
  const showBar = active || entry.status === "paused" || entry.status === "merging";

  return (
    <div className="row">
      {entry.thumbnail ? (
        <img className="thumb" src={entry.thumbnail} alt="" />
      ) : (
        <div className="thumb thumb-placeholder">
          <FilmIcon size={18} />
        </div>
      )}

      <div className="info">
        <div className="title" title={entry.title}>
          {entry.title}
        </div>

        {showBar && (
          <div
            className={`bar${entry.status === "paused" ? " paused" : ""}${
              entry.status === "merging" ? " indeterminate" : ""
            }`}
          >
            <div style={{ width: `${entry.percent}%` }} />
          </div>
        )}

        {active && (
          <div className="stats">
            <span className="pct">{entry.percent.toFixed(0)}%</span>
            {entry.totalBytes ? (
              <span>
                {formatBytes(entry.downloadedBytes)} / {formatBytes(entry.totalBytes)}
              </span>
            ) : null}
            <span>{formatSpeed(entry.speed)}</span>
            <span>{formatEta(entry.eta)}</span>
            {entry.playlistProgress && <span>{entry.playlistProgress}</span>}
          </div>
        )}
        {entry.status === "paused" && (
          <div className="stats">
            <span className="pct">{entry.percent.toFixed(0)}%</span>
            {entry.playlistProgress && <span>{entry.playlistProgress}</span>}
          </div>
        )}
        {entry.status === "done" && (
          <div className="sub">{formatBytes(entry.totalBytes)}</div>
        )}
        {entry.status === "error" && (
          <div className="error-text" title={entry.error ?? ""}>
            {entry.error ?? "Download failed"}
          </div>
        )}
      </div>

      {!active && entry.status !== "paused" && entry.status !== "queued" && (
        <span className={`status-chip ${entry.status}`}>
          {entry.status === "done" ? (
            <>
              <CheckIcon size={11} /> Done
            </>
          ) : (
            STATUS_LABELS[entry.status]
          )}
        </span>
      )}
      {entry.status === "queued" && <span className="status-chip queued">Queued</span>}

      <div className="actions">
        {(active || entry.status === "merging") && (
          <button
            className="icon-btn"
            title="Pause"
            onClick={() => pauseDownload(entry.id)}
          >
            <PauseIcon />
          </button>
        )}
        {entry.status === "paused" && (
          <button
            className="icon-btn"
            title="Resume"
            onClick={() => resumeDownload(entry.id)}
          >
            <PlayIcon />
          </button>
        )}
        {entry.status === "error" && (
          <button
            className="icon-btn"
            title="Retry"
            onClick={() => resumeDownload(entry.id)}
          >
            <RetryIcon />
          </button>
        )}
        {entry.status === "done" && entry.filePath && (
          <>
            <button
              className="icon-btn"
              title="Open file"
              onClick={() => openFile(entry.filePath!)}
            >
              <PlayIcon />
            </button>
            <button
              className="icon-btn"
              title="Show in folder"
              onClick={() => showInFolder(entry.filePath!)}
            >
              <FolderIcon />
            </button>
          </>
        )}
        <button
          className="icon-btn"
          title={
            ["done", "error", "cancelled"].includes(entry.status)
              ? "Remove from list"
              : "Cancel"
          }
          onClick={() => cancelDownload(entry.id)}
        >
          <XIcon />
        </button>
      </div>
    </div>
  );
}

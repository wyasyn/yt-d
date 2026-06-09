import type { HistoryItem, Quality } from "../lib/types";
import { formatBytes, QUALITY_LABELS } from "../lib/types";
import {
  clearHistory,
  openFile,
  removeHistoryItem,
  showInFolder,
} from "../lib/ipc";
import {
  FilmIcon,
  FolderIcon,
  PlayIcon,
  RetryIcon,
  TrashIcon,
} from "./Icons";

interface Props {
  items: HistoryItem[];
  onChanged: () => void;
  onRedownload: (url: string, quality: Quality, title: string) => void;
}

export function HistoryList({ items, onChanged, onRedownload }: Props) {
  if (items.length === 0) {
    return (
      <div className="empty">
        <div className="big">🕘</div>
        <div>No downloads yet</div>
        <div className="hint">Everything you download will be remembered here.</div>
      </div>
    );
  }

  return (
    <>
      <div className="list-toolbar">
        <h2>
          {items.length} download{items.length === 1 ? "" : "s"}
        </h2>
        <button
          className="link-btn danger"
          onClick={async () => {
            await clearHistory();
            onChanged();
          }}
        >
          Clear all
        </button>
      </div>
      {items.map((item) => (
        <div className="row" key={item.id}>
          {item.thumbnail ? (
            <img className="thumb" src={item.thumbnail} alt="" />
          ) : (
            <div className="thumb thumb-placeholder">
              <FilmIcon size={18} />
            </div>
          )}
          <div className="info">
            <div className="title" title={item.title}>
              {item.title}
            </div>
            <div className="sub">
              {[
                new Date(item.finishedAt).toLocaleString(),
                QUALITY_LABELS[item.quality] ?? item.quality,
                formatBytes(item.totalBytes),
              ]
                .filter(Boolean)
                .join(" · ")}
            </div>
          </div>
          {item.status === "error" && (
            <span className="status-chip error">Failed</span>
          )}
          <div className="actions">
            {item.status === "done" && item.filePath && (
              <>
                <button
                  className="icon-btn"
                  title="Open file"
                  onClick={() => openFile(item.filePath!)}
                >
                  <PlayIcon />
                </button>
                <button
                  className="icon-btn"
                  title="Show in folder"
                  onClick={() => showInFolder(item.filePath!)}
                >
                  <FolderIcon />
                </button>
              </>
            )}
            <button
              className="icon-btn"
              title="Download again"
              onClick={() => onRedownload(item.url, item.quality, item.title)}
            >
              <RetryIcon />
            </button>
            <button
              className="icon-btn"
              title="Remove from history"
              onClick={async () => {
                await removeHistoryItem(item.id);
                onChanged();
              }}
            >
              <TrashIcon />
            </button>
          </div>
        </div>
      ))}
    </>
  );
}

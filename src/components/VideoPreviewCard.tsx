import { useState } from "react";
import type { MediaInfo, Quality } from "../lib/types";
import { formatDuration, QUALITY_LABELS } from "../lib/types";
import { DownloadIcon, FilmIcon, XIcon } from "./Icons";

interface Props {
  info: MediaInfo;
  defaultQuality: Quality;
  onDownload: (quality: Quality, playlist: boolean) => void;
  onDismiss: () => void;
}

export function VideoPreviewCard({ info, defaultQuality, onDownload, onDismiss }: Props) {
  const [quality, setQuality] = useState<Quality>(defaultQuality);
  const [wholePlaylist, setWholePlaylist] = useState(info.isPlaylist);

  return (
    <div className="preview">
      {info.thumbnail ? (
        <img className="thumb" src={info.thumbnail} alt="" />
      ) : (
        <div className="thumb thumb-placeholder">
          <FilmIcon />
        </div>
      )}
      <div className="meta">
        <div className="title" title={info.title}>
          {info.title}
        </div>
        <div className="sub">
          {[
            info.uploader,
            formatDuration(info.duration),
            info.extractor,
          ]
            .filter(Boolean)
            .join(" · ")}
          {info.isPlaylist && info.entryCount ? (
            <>
              {" "}
              <span className="badge">Playlist · {info.entryCount} videos</span>
            </>
          ) : null}
        </div>
        <div className="controls">
          <select
            className="quality"
            value={quality}
            onChange={(e) => setQuality(e.target.value as Quality)}
          >
            {Object.entries(QUALITY_LABELS).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
          {info.isPlaylist && (
            <label className="checkbox">
              <input
                type="checkbox"
                checked={wholePlaylist}
                onChange={(e) => setWholePlaylist(e.target.checked)}
              />
              Download all {info.entryCount ?? ""} videos
            </label>
          )}
          <button className="btn" onClick={() => onDownload(quality, wholePlaylist)}>
            <DownloadIcon /> Download
          </button>
        </div>
      </div>
      <button className="icon-btn" onClick={onDismiss} title="Dismiss">
        <XIcon />
      </button>
    </div>
  );
}

export function PreviewSkeleton() {
  return (
    <div className="preview">
      <div className="thumb skeleton" />
      <div className="meta">
        <div className="title skeleton" style={{ width: "60%" }}>
          &nbsp;
        </div>
        <div className="sub skeleton" style={{ width: "35%" }}>
          &nbsp;
        </div>
        <div className="controls">
          <div className="skeleton" style={{ width: 140, height: 32 }} />
          <div className="skeleton" style={{ width: 110, height: 32 }} />
        </div>
      </div>
    </div>
  );
}

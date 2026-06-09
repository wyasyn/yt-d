import { useEffect, useState } from "react";
import { onEngineSetupProgress, setupEngine } from "../lib/ipc";
import type { EngineSetupProgress } from "../lib/types";
import { DownloadIcon } from "./Icons";

interface Props {
  onReady: () => void;
}

type ComponentState = { percent: number; phase: string };

export function EngineSetupScreen({ onReady }: Props) {
  const [components, setComponents] = useState<Record<string, ComponentState>>({
    "yt-dlp": { percent: 0, phase: "waiting" },
    ffmpeg: { percent: 0, phase: "waiting" },
  });
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    onEngineSetupProgress((p: EngineSetupProgress) => {
      setComponents((prev) => ({
        ...prev,
        [p.component]: { percent: p.percent, phase: p.phase },
      }));
    }).then((u) => (unlisten = u));
    return () => unlisten?.();
  }, []);

  const start = async () => {
    setError(null);
    setRunning(true);
    try {
      await setupEngine();
      onReady();
    } catch (e) {
      setError(String(e));
      setRunning(false);
    }
  };

  return (
    <div className="setup">
      <div className="logo">
        <DownloadIcon size={30} />
      </div>
      <h1>Welcome to GrabIt</h1>
      <p>
        GrabIt needs two small open-source tools to download videos:{" "}
        <strong>yt-dlp</strong> (the downloader) and <strong>ffmpeg</strong>{" "}
        (the converter). This is a one-time setup.
      </p>

      {running &&
        Object.entries(components).map(([name, c]) => (
          <div className="component" key={name}>
            <div className="label">
              <span>{name}</span>
              <span>
                {c.phase === "done"
                  ? "Ready ✓"
                  : c.phase === "extracting"
                    ? "Extracting…"
                    : c.phase === "downloading"
                      ? `${Math.round(c.percent)}%`
                      : "Waiting…"}
              </span>
            </div>
            <div className={`bar${c.phase === "extracting" ? " indeterminate" : ""}`}>
              <div style={{ width: `${c.phase === "done" ? 100 : c.percent}%` }} />
            </div>
          </div>
        ))}

      {error && <div className="error-banner">{error}</div>}

      {!running && (
        <button className="btn" onClick={start}>
          {error ? "Try again" : "Set up GrabIt"}
        </button>
      )}
    </div>
  );
}

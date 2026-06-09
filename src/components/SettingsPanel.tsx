import { useEffect, useState } from "react";
import type { Quality, Settings } from "../lib/types";
import { QUALITY_LABELS } from "../lib/types";
import {
  getDefaultDownloadDir,
  pickFolder,
  setSettings as saveSettings,
  updateEngine,
} from "../lib/ipc";

interface Props {
  settings: Settings;
  onClose: (updated: Settings) => void;
}

export function SettingsPanel({ settings, onClose }: Props) {
  const [draft, setDraft] = useState<Settings>(settings);
  const [effectiveDir, setEffectiveDir] = useState("");
  const [engineMsg, setEngineMsg] = useState<string | null>(null);
  const [updating, setUpdating] = useState(false);

  useEffect(() => {
    getDefaultDownloadDir().then(setEffectiveDir).catch(() => {});
  }, [draft.downloadDir]);

  const close = async () => {
    await saveSettings(draft);
    onClose(draft);
  };

  const runUpdate = async () => {
    setUpdating(true);
    setEngineMsg(null);
    try {
      setEngineMsg(await updateEngine());
    } catch (e) {
      setEngineMsg(String(e));
    } finally {
      setUpdating(false);
    }
  };

  return (
    <div className="overlay" onClick={close}>
      <div className="panel" onClick={(e) => e.stopPropagation()}>
        <h2>Settings</h2>

        <div className="field-group">
          <label>Save downloads to</label>
          <div className="dir-row">
            <input type="text" readOnly value={effectiveDir} title={effectiveDir} />
            <button
              className="btn secondary"
              onClick={async () => {
                const dir = await pickFolder();
                if (dir) setDraft({ ...draft, downloadDir: dir });
              }}
            >
              Change…
            </button>
            {draft.downloadDir && (
              <button
                className="btn secondary"
                title="Use the system Downloads folder"
                onClick={() => setDraft({ ...draft, downloadDir: null })}
              >
                Reset
              </button>
            )}
          </div>
        </div>

        <div className="field-group">
          <label>Default quality</label>
          <select
            value={draft.defaultQuality}
            onChange={(e) =>
              setDraft({ ...draft, defaultQuality: e.target.value as Quality })
            }
          >
            {Object.entries(QUALITY_LABELS).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </div>

        <div className="field-group">
          <label>Simultaneous downloads</label>
          <input
            type="number"
            min={1}
            max={8}
            value={draft.maxConcurrent}
            onChange={(e) =>
              setDraft({
                ...draft,
                maxConcurrent: Math.max(1, Math.min(8, Number(e.target.value) || 1)),
              })
            }
          />
        </div>

        <div className="field-group">
          <label className="checkbox" style={{ color: "var(--text)" }}>
            <input
              type="checkbox"
              checked={draft.notifications}
              onChange={(e) => setDraft({ ...draft, notifications: e.target.checked })}
            />
            Notify me when a download finishes
          </label>
        </div>

        <div className="footer">
          <div>
            <button className="btn secondary" onClick={runUpdate} disabled={updating}>
              {updating ? "Updating…" : "Update downloader"}
            </button>
            {engineMsg && <div className="engine-note">{engineMsg}</div>}
          </div>
          <button className="btn" onClick={close}>
            Done
          </button>
        </div>
      </div>
    </div>
  );
}

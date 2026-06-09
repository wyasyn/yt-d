import { useCallback, useEffect, useState } from "react";
import type {
  DownloadEntry,
  HistoryItem,
  MediaInfo,
  Quality,
  Settings,
} from "./lib/types";
import {
  checkEngine,
  clearFinished,
  fetchMetadata,
  getHistory,
  getQueue,
  getSettings,
  onDownloadProgress,
  onDownloadsChanged,
  startDownload,
} from "./lib/ipc";
import { EngineSetupScreen } from "./components/EngineSetupScreen";
import { UrlBar } from "./components/UrlBar";
import { PreviewSkeleton, VideoPreviewCard } from "./components/VideoPreviewCard";
import { DownloadItem } from "./components/DownloadItem";
import { HistoryList } from "./components/HistoryList";
import { SettingsPanel } from "./components/SettingsPanel";
import { DownloadIcon, GearIcon } from "./components/Icons";

type Tab = "downloads" | "history";

export default function App() {
  const [engineReady, setEngineReady] = useState<boolean | null>(null);
  const [tab, setTab] = useState<Tab>("downloads");
  const [settings, setSettings] = useState<Settings | null>(null);
  const [showSettings, setShowSettings] = useState(false);

  const [fetching, setFetching] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [preview, setPreview] = useState<MediaInfo | null>(null);

  const [downloads, setDownloads] = useState<DownloadEntry[]>([]);
  const [history, setHistory] = useState<HistoryItem[]>([]);

  const refreshHistory = useCallback(() => {
    getHistory().then(setHistory).catch(() => {});
  }, []);

  useEffect(() => {
    checkEngine()
      .then((e) => setEngineReady(e.ready))
      .catch(() => setEngineReady(false));
    getSettings().then(setSettings).catch(() => {});
    getQueue().then(setDownloads).catch(() => {});
    refreshHistory();
  }, [refreshHistory]);

  useEffect(() => {
    const subs: Promise<() => void>[] = [
      onDownloadsChanged((entries) => {
        setDownloads(entries);
        refreshHistory();
      }),
      onDownloadProgress((p) => {
        setDownloads((prev) =>
          prev.map((e) => (e.id === p.id ? { ...e, ...p } : e)),
        );
      }),
    ];
    return () => {
      subs.forEach((s) => s.then((unlisten) => unlisten()));
    };
  }, [refreshHistory]);

  const fetchUrl = async (url: string) => {
    setFetching(true);
    setFetchError(null);
    setPreview(null);
    try {
      setPreview(await fetchMetadata(url));
    } catch (e) {
      setFetchError(String(e));
    } finally {
      setFetching(false);
    }
  };

  const download = async (quality: Quality, playlist: boolean) => {
    if (!preview) return;
    await startDownload({
      url: preview.url,
      quality,
      playlist,
      title: preview.title,
      thumbnail: preview.thumbnail ?? undefined,
    });
    setPreview(null);
    setTab("downloads");
  };

  const redownload = async (url: string, quality: Quality, title: string) => {
    await startDownload({ url, quality, playlist: false, title });
    setTab("downloads");
  };

  if (engineReady === null) {
    return <div className="app" />;
  }
  if (!engineReady) {
    return (
      <div className="app">
        <EngineSetupScreen onReady={() => setEngineReady(true)} />
      </div>
    );
  }

  const hasFinished = downloads.some((d) =>
    ["done", "error", "cancelled"].includes(d.status),
  );

  return (
    <div className="app">
      <header className="header">
        <div className="brand">
          <span className="logo">
            <DownloadIcon size={15} />
          </span>
          GrabIt
        </div>
        <nav className="tabs">
          <button
            className={tab === "downloads" ? "active" : ""}
            onClick={() => setTab("downloads")}
          >
            Downloads
          </button>
          <button
            className={tab === "history" ? "active" : ""}
            onClick={() => {
              refreshHistory();
              setTab("history");
            }}
          >
            History
          </button>
        </nav>
        <div className="spacer" />
        <button
          className="icon-btn"
          title="Settings"
          onClick={() => setShowSettings(true)}
        >
          <GearIcon />
        </button>
      </header>

      {tab === "downloads" && (
        <>
          <UrlBar busy={fetching} onSubmit={fetchUrl} />
          {fetchError && <div className="error-banner">{fetchError}</div>}
          {fetching && <PreviewSkeleton />}
          {preview && settings && (
            <VideoPreviewCard
              info={preview}
              defaultQuality={settings.defaultQuality}
              onDownload={download}
              onDismiss={() => setPreview(null)}
            />
          )}
        </>
      )}

      <main className="content">
        {tab === "downloads" ? (
          downloads.length === 0 ? (
            <div className="empty">
              <div className="big">⬇️</div>
              <div>Nothing downloading yet</div>
              <div className="hint">
                Paste a link above to grab a video from YouTube, X, Instagram,
                Facebook, TikTok and 1800+ other sites.
              </div>
            </div>
          ) : (
            <>
              {hasFinished && (
                <div className="list-toolbar">
                  <h2>Downloads</h2>
                  <button className="link-btn" onClick={() => clearFinished()}>
                    Clear finished
                  </button>
                </div>
              )}
              {downloads.map((entry) => (
                <DownloadItem key={entry.id} entry={entry} />
              ))}
            </>
          )
        ) : (
          <HistoryList
            items={history}
            onChanged={refreshHistory}
            onRedownload={redownload}
          />
        )}
      </main>

      {showSettings && settings && (
        <SettingsPanel
          settings={settings}
          onClose={(updated) => {
            setSettings(updated);
            setShowSettings(false);
          }}
        />
      )}
    </div>
  );
}

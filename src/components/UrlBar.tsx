import { useEffect, useRef, useState } from "react";
import { readClipboardUrl } from "../lib/ipc";
import { looksLikeUrl } from "../lib/types";

interface Props {
  busy: boolean;
  onSubmit: (url: string) => void;
}

export function UrlBar({ busy, onSubmit }: Props) {
  const [url, setUrl] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  // Offer a video URL already sitting in the clipboard on launch.
  useEffect(() => {
    readClipboardUrl().then((clip) => {
      if (clip) setUrl((current) => current || clip);
    });
    inputRef.current?.focus();
  }, []);

  const paste = async () => {
    const clip = await readClipboardUrl();
    if (clip) {
      setUrl(clip);
      onSubmit(clip);
    } else {
      inputRef.current?.focus();
    }
  };

  const submit = () => {
    const trimmed = url.trim();
    if (looksLikeUrl(trimmed) && !busy) onSubmit(trimmed);
  };

  return (
    <div className="url-bar">
      <div className="field">
        <input
          ref={inputRef}
          type="text"
          placeholder="Paste a video link — YouTube, X, Instagram, Facebook, TikTok…"
          value={url}
          spellCheck={false}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && submit()}
        />
        <button className="paste" onClick={paste} title="Paste from clipboard">
          Paste
        </button>
      </div>
      <button className="btn" onClick={submit} disabled={busy || !looksLikeUrl(url)}>
        {busy ? "Checking…" : "Fetch"}
      </button>
    </div>
  );
}

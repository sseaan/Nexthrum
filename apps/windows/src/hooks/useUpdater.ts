import { useCallback, useEffect, useRef, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { downloadProgress, initialUpdateStatus, type UpdateStatus } from "../utils/updateState";

const CHECK_DELAY_MS = 4000;

/**
 * Checks GitHub Releases for a newer signed build shortly after launch and
 * drives the download/install flow. Failures are surfaced as state, never thrown.
 */
export function useUpdater() {
  const [status, setStatus] = useState<UpdateStatus>(initialUpdateStatus);
  const pending = useRef<Update | null>(null);

  const checkForUpdates = useCallback(async () => {
    setStatus({ kind: "checking" });
    try {
      const update = await check();
      if (!update) {
        setStatus({ kind: "up-to-date" });
        return;
      }
      pending.current = update;
      setStatus({ kind: "available", version: update.version, notes: update.body });
    } catch (e) {
      // Offline or the release has no update manifest yet: stay quiet.
      console.warn("Update check failed:", e);
      setStatus({ kind: "idle" });
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    const update = pending.current;
    if (!update) return;

    setStatus({ kind: "downloading", version: update.version, progress: null });
    let downloaded = 0;
    let total: number | undefined;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength;
          setStatus({ kind: "downloading", version: update.version, progress: 0 });
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setStatus({
            kind: "downloading",
            version: update.version,
            progress: downloadProgress(downloaded, total),
          });
        } else if (event.event === "Finished") {
          setStatus({ kind: "ready", version: update.version });
        }
      });
    } catch (e) {
      setStatus({ kind: "error", message: String(e) });
    }
  }, []);

  const restart = useCallback(async () => {
    await relaunch();
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => void checkForUpdates(), CHECK_DELAY_MS);
    return () => clearTimeout(timer);
  }, [checkForUpdates]);

  return { status, checkForUpdates, downloadAndInstall, restart };
}

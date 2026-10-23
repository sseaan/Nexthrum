import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from "../store/useAppStore";
import { isTauriRuntime } from "../utils/tauriEnv";
import type { GotifyMessage } from "../types";

/**
 * Hook to listen for Tauri backend events:
 * - new-message: A new message arrived from Gotify
 * - unread-count-changed: The unread count was updated
 * - connection-state-changed / connection-error: stream status
 * - show-settings: User clicked "Settings" in tray menu
 */
export function useTauriEvents() {
  const [ready, setReady] = useState(false);
  const addMessage = useAppStore((s) => s.addMessage);
  const setUnreadCount = useAppStore((s) => s.setUnreadCount);
  const setCurrentView = useAppStore((s) => s.setCurrentView);
  const setConnectionState = useAppStore((s) => s.setConnectionState);
  const setConnectionError = useAppStore((s) => s.setConnectionError);

  useEffect(() => {
    // No IPC bridge outside the Tauri webview: stay silent instead of logging
    // listener failures in the browser preview.
    if (!isTauriRuntime()) return;
    let active = true;
    const unlisteners: UnlistenFn[] = [];

    const subscribe = async () => {
      const registered = await Promise.all([
        listen<GotifyMessage>("new-message", (event) => {
          const msg = event.payload;
          addMessage({
            id: msg.id,
            appid: msg.appid,
            message: msg.message,
            title: msg.title,
            priority: msg.priority,
            date: msg.date,
            read: msg.read,
            received_at: msg.received_at,
          });
        }),
        listen<number>("unread-count-changed", (event) => setUnreadCount(event.payload)),
        listen<string>("connection-state-changed", (event) => {
          const state = event.payload;
          if (state === "connecting" || state === "connected" || state === "disconnected") {
            setConnectionState(state);
          }
        }),
        listen<string>("connection-error", (event) => setConnectionError(event.payload)),
        listen<void>("show-settings", () => setCurrentView("settings")),
      ]);
      if (!active) {
        registered.forEach((unlisten) => unlisten());
        return;
      }
      unlisteners.push(...registered);
      setReady(true);
    };

    void subscribe().catch((error) => console.error("Failed to register Tauri listeners:", error));

    return () => {
      active = false;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [addMessage, setUnreadCount, setCurrentView, setConnectionState, setConnectionError]);

  return ready;
}

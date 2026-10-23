import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "./store/useAppStore";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { TitleBar } from "./components/common/TitleBar";
import { MessageWindow } from "./components/MessageWindow/MessageWindow";
import { SettingsWindow } from "./components/SettingsWindow/SettingsWindow";
import type { MessageSnapshot } from "./types";

function App() {
  const currentView = useAppStore((s) => s.currentView);
  const setSettings = useAppStore((s) => s.setSettings);
  const setMessages = useAppStore((s) => s.setMessages);
  const setUnreadCount = useAppStore((s) => s.setUnreadCount);
  const setAppNames = useAppStore((s) => s.setAppNames);
  const setSettingsLoaded = useAppStore((s) => s.setSettingsLoaded);

  const listenersReady = useTauriEvents();

  const loadApplicationNames = async () => {
    try {
      const applications = await invoke<{ id: number; name: string }[]>(
        "get_applications",
      );
      setAppNames(
        Object.fromEntries(applications.map((app) => [app.id, app.name])),
      );
    } catch (e) {
      // Restricted servers keep the `App <id>` fallback.
      console.warn("Could not resolve Gotify application names:", e);
    }
  };

  useEffect(() => {
    if (!listenersReady) return;
    let active = true;
    const loadInitialData = async () => {
      try {
        const revisionBeforeSnapshot = useAppStore.getState().backendRevision;
        const [settings, snapshot] = await Promise.all([
          invoke("get_settings"),
          invoke<MessageSnapshot>("get_message_snapshot"),
        ]);
        if (!active) return;
        setSettings(settings as Parameters<typeof setSettings>[0]);
        setSettingsLoaded(true);
        // Merge over events received while the snapshot request was in flight.
        const currentMessages = useAppStore.getState().messages;
        const currentIds = new Set(currentMessages.map((message) => message.id));
        setMessages([
          ...currentMessages,
          ...snapshot.messages.filter((message) => !currentIds.has(message.id)),
        ]);
        if (useAppStore.getState().backendRevision === revisionBeforeSnapshot) {
          setUnreadCount(snapshot.unread_count);
        }
        await loadApplicationNames();
      } catch (e) {
        console.error("Failed to load initial data:", e);
      }
    };

    void loadInitialData();
    return () => {
      active = false;
    };
  }, [listenersReady, setSettings, setMessages, setUnreadCount, setAppNames, setSettingsLoaded]);

  return (
    <div
      data-app-mounted
      className="flex h-full flex-col overflow-hidden bg-[var(--color-bg)] text-[var(--color-text)]"
    >
      <TitleBar />
      <div className="flex-1 overflow-hidden">
        {currentView === "settings" ? <SettingsWindow /> : <MessageWindow />}
      </div>
    </div>
  );
}

export default App;

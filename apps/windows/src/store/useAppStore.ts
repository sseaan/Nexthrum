import { create } from "zustand";
import type { AppSettings, MessageListItem, MessageSnapshot } from "../types";

function deriveApps(
  messages: MessageListItem[],
  appNames: Record<number, string>,
): { appid: number; appname: string }[] {
  const seen = new Map<number, { appid: number; appname: string }>();
  for (const message of messages) {
    if (!seen.has(message.appid)) {
      seen.set(message.appid, {
        appid: message.appid,
        appname: appNames[message.appid] ?? `App ${message.appid}`,
      });
    }
  }
  return Array.from(seen.values());
}

interface AppStore {
  // Settings
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;

  // Messages
  messages: MessageListItem[];
  setMessages: (messages: MessageListItem[]) => void;
  applyMessageSnapshot: (snapshot: MessageSnapshot) => void;
  addMessage: (message: MessageListItem) => void;

  // UI State
  currentView: "messages" | "settings";
  setCurrentView: (view: "messages" | "settings") => void;
  selectedAppId: number | null;
  setSelectedAppId: (appId: number | null) => void;
  unreadCount: number;
  backendRevision: number;
  setUnreadCount: (count: number) => void;
  connectionState: "disconnected" | "connecting" | "connected";
  setConnectionState: (state: "disconnected" | "connecting" | "connected") => void;

  // App filters
  apps: { appid: number; appname: string }[];
  /** Names resolved from the Gotify `/application` endpoint. */
  appNames: Record<number, string>;
  setAppNames: (names: Record<number, string>) => void;
  /** Display label for a source, falling back to `App <id>`. */
  appLabel: (appid: number) => string;
}

const defaultSettings: AppSettings = {
  server_url: "",
  client_token: "",
  auto_start: false,
  start_hidden: false,
  minimize_to_tray: true,
  history_retention_days: 7,
  do_not_disturb_start: "22:00",
  do_not_disturb_end: "08:00",
};

export const useAppStore = create<AppStore>((set, get) => ({
  settings: defaultSettings,
  setSettings: (settings) => set({ settings }),

  messages: [],
  setMessages: (messages) =>
    set((state) => ({
      messages,
      apps: deriveApps(messages, state.appNames),
    })),
  applyMessageSnapshot: (snapshot) => {
    set((state) => ({
      messages: snapshot.messages,
      apps: deriveApps(snapshot.messages, state.appNames),
      unreadCount: snapshot.unread_count,
      backendRevision: state.backendRevision + 1,
    }));
  },
  addMessage: (message) =>
    set((state) => {
      // Deduplicate: skip if message with this ID already exists
      if (state.messages.some((m) => m.id === message.id)) {
        return {};
      }
      // Unread count is backend-owned; the matching unread-count-changed event
      // carries the authoritative post-persistence snapshot.
      const newMessages = [message, ...state.messages];
      // Update apps list
      const appMap = new Map(state.apps.map((a) => [a.appid, a]));
      if (!appMap.has(message.appid)) {
        appMap.set(message.appid, {
          appid: message.appid,
          appname: state.appNames[message.appid] ?? `App ${message.appid}`,
        });
      }
      return {
        messages: newMessages,
        apps: Array.from(appMap.values()),
      };
    }),
  currentView: "messages",
  setCurrentView: (currentView) => set({ currentView }),
  selectedAppId: null,
  setSelectedAppId: (selectedAppId) => set({ selectedAppId }),
  unreadCount: 0,
  backendRevision: 0,
  setUnreadCount: (unreadCount) => set((state) => ({
    unreadCount,
    backendRevision: state.backendRevision + 1,
  })),
  connectionState: "disconnected",
  setConnectionState: (connectionState) => set({ connectionState }),

  apps: [],
  appNames: {},
  setAppNames: (appNames) =>
    set((state) => ({
      appNames,
      // Keep the filter labels in sync with resolved names.
      apps: state.apps.map((app) => ({
        ...app,
        appname: appNames[app.appid] ?? app.appname,
      })),
    })),
  appLabel: (appid) => get().appNames[appid] ?? `App ${appid}`,
}));

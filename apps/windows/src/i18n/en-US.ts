const en = {
  app: { title: "Nexthrum" },

  toolbar: {
    clearAll: "Clear All",
    settings: "⚙ Settings",
    unread: "{n} unread",
  },

  filter: {
    label: "Filter by source",
    allApps: "All Apps",
  },

  message: {
    empty: "No messages yet",
    emptyHint:
      "Configure your Gotify server in Settings to start receiving messages",
    delete: "Delete Message",
    markRead: "Mark as Read",
    noTitle: "No Title",
    read: "Read",
    unread: "Unread",
  },

  tray: {
    messageList: "Message List",
    settings: "Settings",
    exit: "Exit",
  },

  settings: {
    title: "Settings",
    back: "← Back",
    serverConnection: "Server Connection",
    serverUrl: "Server URL",
    clientToken: "Client Token",
    testConnection: "Test Connection",
    testing: "Testing...",
    fillRequired: "Please fill in both Server URL and Client Token first.",
    connected: "Connected! Gotify server version: {version}",
    general: "General",
    autoStart: "Start with Windows",
    startHidden: "Start hidden to tray",
    minimizeToTray: "Minimize to tray on close",
    retention: "Message retention (days)",
    dnd: "Do Not Disturb",
    dndHint:
      "Notifications will be suppressed during this time range (messages are still saved).",
    dndStart: "Start",
    dndEnd: "End",
    dndTo: "to",
    save: "Save Settings",
    saving: "Saving...",
    saved: "Settings saved successfully!",
  },

  clearAll: {
    title: "Clear All Messages",
    confirm:
      "Are you sure you want to clear all messages? This action cannot be undone.",
    syncServer: "Also delete all messages from the server",
    cancel: "Cancel",
    confirmButton: "Clear All",
  },

  connection: {
    connected: "Connected",
    disconnected: "Disconnected",
    connecting: "Connecting…",
  },

  language: {
    label: "语言 / Language",
    zhCN: "中文",
    enUS: "English",
  },
} as const;

export default en;

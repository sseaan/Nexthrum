import { useAppStore } from "./store/useAppStore";
import type { MessageListItem } from "./types";

/**
 * Dev-only fixture so the message list can be inspected in a browser preview,
 * where the Rust host (and therefore all real data) is missing.
 *
 * Activate with `?preview=demo`. Stripped from production builds because the
 * call site is guarded by `import.meta.env.DEV`.
 */
export function enableDevPreviewIfRequested(): void {
  const preview = new URLSearchParams(window.location.search).get("preview");
  if (preview !== "demo") return;

  const now = Date.now();
  const demo: MessageListItem[] = [
    {
      id: 1,
      appid: 2,
      title: "备份任务已完成",
      message:
        "夜间备份已完成：共处理 1,284 个文件，耗时 3 分 12 秒。详情请查看管理面板日志。",
      priority: 5,
      date: new Date(now - 12 * 60 * 1000).toISOString(),
      read: false,
      received_at: new Date(now - 12 * 60 * 1000).toISOString(),
    },
    {
      id: 2,
      appid: 7,
      title: "CI 构建通过",
      message: "nexthrum@test-f523aa0 在 windows-latest 上构建成功，用时 9 分 2 秒。",
      priority: 3,
      date: new Date(now - 3 * 60 * 60 * 1000).toISOString(),
      read: false,
      received_at: new Date(now - 3 * 60 * 60 * 1000).toISOString(),
    },
    {
      id: 3,
      appid: 2,
      title: "磁盘空间提醒",
      message: "/var/lib/docker 已使用 87%，建议清理未使用的镜像与构建缓存。",
      priority: 8,
      date: new Date(now - 26 * 60 * 60 * 1000).toISOString(),
      read: true,
      received_at: new Date(now - 26 * 60 * 60 * 1000).toISOString(),
    },
  ];

  const store = useAppStore.getState();
  store.setSettings({
    server_url: "https://gotify.example.com",
    client_token: "preview",
    auto_start: false,
    start_hidden: false,
    minimize_to_tray: true,
    history_retention_days: 7,
    do_not_disturb_start: "22:00",
    do_not_disturb_end: "08:00",
  });
  store.setSettingsLoaded(true);
  store.setAppNames({ 2: "Infra", 7: "CI" });
  store.setMessages(demo);
  store.setUnreadCount(2);
  store.setConnectionState("connected");
}

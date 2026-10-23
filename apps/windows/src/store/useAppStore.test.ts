import { beforeEach, describe, expect, it } from "vitest";
import { useAppStore } from "./useAppStore";
import type { MessageListItem, MessageSnapshot } from "../types";

function item(id: number, appid: number, read = false): MessageListItem {
  return {
    id,
    appid,
    message: `body ${id}`,
    title: `title ${id}`,
    priority: 0,
    date: "2026-10-23T02:05:00Z",
    read,
    received_at: "2026-10-23T02:05:00Z",
  };
}

function snapshot(messages: MessageListItem[], unread: number): MessageSnapshot {
  return { messages, unread_count: unread };
}

beforeEach(() => {
  useAppStore.setState({
    messages: [],
    apps: [],
    appNames: {},
    unreadCount: 0,
    backendRevision: 0,
  });
});

describe("applyMessageSnapshot", () => {
  it("replaces messages, derives sources and syncs the unread count", () => {
    useAppStore.getState().applyMessageSnapshot(snapshot([item(1, 7), item(2, 3)], 2));

    const state = useAppStore.getState();
    expect(state.messages).toHaveLength(2);
    expect(state.apps.map((app) => app.appid).sort()).toEqual([3, 7]);
    expect(state.unreadCount).toBe(2);
    expect(state.backendRevision).toBe(1);
  });

  it("uses resolved application names when deriving sources", () => {
    useAppStore.getState().setAppNames({ 7: "CI" });
    useAppStore.getState().applyMessageSnapshot(snapshot([item(1, 7)], 1));

    expect(useAppStore.getState().apps[0].appname).toBe("CI");
  });
});

describe("addMessage", () => {
  it("prepends new messages and grows the source list", () => {
    useAppStore.getState().addMessage(item(1, 5));
    useAppStore.getState().addMessage(item(2, 5));

    const state = useAppStore.getState();
    expect(state.messages.map((m) => m.id)).toEqual([2, 1]);
    expect(state.apps).toHaveLength(1);
    expect(state.apps[0].appid).toBe(5);
  });

  it("ignores duplicate ids", () => {
    useAppStore.getState().addMessage(item(1, 5));
    useAppStore.getState().addMessage(item(1, 5));

    expect(useAppStore.getState().messages).toHaveLength(1);
  });

  it("does not touch the unread count (backend owns it)", () => {
    useAppStore.setState({ unreadCount: 4 });
    useAppStore.getState().addMessage(item(1, 5));

    expect(useAppStore.getState().unreadCount).toBe(4);
  });
});

describe("setAppNames", () => {
  it("relabels already derived sources and powers appLabel", () => {
    useAppStore.getState().setMessages([item(1, 9)]);
    useAppStore.getState().setAppNames({ 9: "家庭服务器" });

    const state = useAppStore.getState();
    expect(state.apps[0].appname).toBe("家庭服务器");
    expect(state.appLabel(9)).toBe("家庭服务器");
  });

  it("falls back to the app id for unknown sources", () => {
    expect(useAppStore.getState().appLabel(42)).toBe("App 42");
  });
});

describe("connection state", () => {
  it("clears the error when the stream becomes connected", () => {
    useAppStore.getState().setConnectionError("TLS handshake failed");
    useAppStore.getState().setConnectionState("connecting");
    expect(useAppStore.getState().connectionError).toBe("TLS handshake failed");

    useAppStore.getState().setConnectionState("connected");
    expect(useAppStore.getState().connectionError).toBeNull();
  });
});

describe("search state", () => {
  it("keeps results while typing and restores the full list when cleared", () => {
    useAppStore.getState().setMessages([item(1, 5), item(2, 5)]);
    useAppStore.getState().setSearchQuery("dep");
    useAppStore.getState().setSearchResults([item(1, 5)]);
    expect(useAppStore.getState().searchResults).toHaveLength(1);

    useAppStore.getState().setSearchQuery("   ");
    expect(useAppStore.getState().searchResults).toBeNull();
  });
});

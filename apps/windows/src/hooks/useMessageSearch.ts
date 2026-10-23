import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/useAppStore";
import type { MessageSnapshot } from "../types";

const DEBOUNCE_MS = 250;

/**
 * Runs the backend search for the current query, cancelling (ignoring) stale
 * responses so fast typing cannot resolve out of order.
 */
export function useMessageSearch() {
  const searchQuery = useAppStore((s) => s.searchQuery);
  const setSearchResults = useAppStore((s) => s.setSearchResults);

  useEffect(() => {
    const trimmed = searchQuery.trim();
    if (trimmed === "") {
      setSearchResults(null);
      return;
    }

    let cancelled = false;
    const timer = setTimeout(async () => {
      try {
        const snapshot = await invoke<MessageSnapshot>("search_messages", {
          query: trimmed,
        });
        if (!cancelled) {
          setSearchResults(snapshot.messages);
        }
      } catch (e) {
        console.error("Search failed:", e);
        if (!cancelled) {
          setSearchResults([]);
        }
      }
    }, DEBOUNCE_MS);

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [searchQuery, setSearchResults]);
}

import { useAppStore } from "../../store/useAppStore";
import { useI18n } from "../../i18n";
import { MessageItem } from "./MessageItem";
import { EmptyState } from "../ui/EmptyState";
import { Button } from "../ui/Button";
import { InboxIcon, SearchIcon } from "../ui/icons";

export function MessageList() {
  const { t } = useI18n();
  const setCurrentView = useAppStore((s) => s.setCurrentView);
  const messages = useAppStore((s) => s.messages);
  const selectedAppId = useAppStore((s) => s.selectedAppId);
  const searchQuery = useAppStore((s) => s.searchQuery);
  const searchResults = useAppStore((s) => s.searchResults);

  const isSearching = searchQuery.trim() !== "";
  const base = isSearching ? (searchResults ?? []) : messages;
  const filteredMessages =
    !isSearching && selectedAppId !== null
      ? base.filter((m) => m.appid === selectedAppId)
      : base;

  if (isSearching && filteredMessages.length === 0) {
    return (
      <EmptyState
        icon={<SearchIcon size={20} />}
        title={`${t.search.noMatches} “${searchQuery.trim()}”`}
      />
    );
  }

  if (filteredMessages.length === 0) {
    return (
      <EmptyState
        icon={<InboxIcon size={20} />}
        title={t.message.empty}
        hint={t.message.emptyHint}
        action={
          <Button size="sm" variant="secondary" onClick={() => setCurrentView("settings")}>
            {t.message.openSettings}
          </Button>
        }
      />
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto w-full max-w-3xl space-y-2 px-4 pt-3 pb-4">
        {filteredMessages.map((msg) => (
          <MessageItem key={msg.id} message={msg} />
        ))}
      </div>
    </div>
  );
}

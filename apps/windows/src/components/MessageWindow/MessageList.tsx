import { useAppStore } from "../../store/useAppStore";
import { useI18n } from "../../i18n";
import { MessageItem } from "./MessageItem";

export function MessageList() {
  const { t } = useI18n();
  const messages = useAppStore((s) => s.messages);
  const selectedAppId = useAppStore((s) => s.selectedAppId);

  const filteredMessages =
    selectedAppId !== null
      ? messages.filter((m) => m.appid === selectedAppId)
      : messages;

  if (filteredMessages.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--color-text-secondary)]">
        <div className="text-center">
          <p className="text-4xl mb-2">📭</p>
          <p className="text-sm">{t.message.empty}</p>
          <p className="text-xs mt-1">{t.message.emptyHint}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      {filteredMessages.map((msg) => (
        <MessageItem key={msg.id} message={msg} />
      ))}
    </div>
  );
}

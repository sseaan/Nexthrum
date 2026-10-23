import { useAppStore } from "../../store/useAppStore";
import { AppFilter } from "./AppFilter";
import { Toolbar } from "./Toolbar";
import { MessageList } from "./MessageList";
import { SettingsWindow } from "../SettingsWindow/SettingsWindow";

export function MessageWindow() {
  const currentView = useAppStore((s) => s.currentView);

  if (currentView === "settings") {
    return <SettingsWindow />;
  }

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg)]">
      <Toolbar />
      <AppFilter />
      <div className="flex-1 overflow-hidden">
        <MessageList />
      </div>
    </div>
  );
}

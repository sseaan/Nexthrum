import { useState } from "react";
import { useAppStore } from "../../store/useAppStore";
import { AppFilter } from "./AppFilter";
import { MessageHeader } from "./MessageHeader";
import { MessageList } from "./MessageList";
import { SettingsWindow } from "../SettingsWindow/SettingsWindow";
import { OnboardingView } from "../OnboardingWindow/OnboardingView";
import { useMessageSearch } from "../../hooks/useMessageSearch";
import { useUpdater } from "../../hooks/useUpdater";
import { UpdateBanner } from "../common/UpdateBanner";
import { initialUpdateStatus } from "../../utils/updateState";

export function MessageWindow() {
  const currentView = useAppStore((s) => s.currentView);
  const settings = useAppStore((s) => s.settings);
  const settingsLoaded = useAppStore((s) => s.settingsLoaded);
  const [onboardingDismissed, setOnboardingDismissed] = useState(false);
  const searchQuery = useAppStore((s) => s.searchQuery);
  useMessageSearch();
  const { status: updateStatus, downloadAndInstall, restart } = useUpdater();
  const [updateDismissed, setUpdateDismissed] = useState(false);
  const visibleUpdate =
    updateDismissed && updateStatus.kind === "available" ? initialUpdateStatus : updateStatus;

  if (currentView === "settings") {
    return <SettingsWindow />;
  }

  // Avoid flashing the onboarding while the stored settings are still loading.
  if (!settingsLoaded) {
    return <div className="h-full bg-[var(--color-bg)]" aria-busy="true" />;
  }

  // A fresh install has no server configured yet: guide the user first.
  if (settings.server_url.trim() === "" && !onboardingDismissed) {
    return <OnboardingView onSkip={() => setOnboardingDismissed(true)} />;
  }

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg)]">
      <UpdateBanner
        status={visibleUpdate}
        onInstall={() => void downloadAndInstall()}
        onRestart={() => void restart()}
        onDismiss={() => setUpdateDismissed(true)}
      />
      <MessageHeader />
      {searchQuery.trim() === "" && <AppFilter />}
      <div className="flex-1 overflow-hidden">
        <MessageList />
      </div>
    </div>
  );
}

import { useState } from "react";
import { useAppStore } from "../../store/useAppStore";
import { AppFilter } from "./AppFilter";
import { MessageHeader } from "./MessageHeader";
import { MessageList } from "./MessageList";
import { SettingsWindow } from "../SettingsWindow/SettingsWindow";
import { OnboardingView } from "../OnboardingWindow/OnboardingView";

export function MessageWindow() {
  const currentView = useAppStore((s) => s.currentView);
  const settings = useAppStore((s) => s.settings);
  const settingsLoaded = useAppStore((s) => s.settingsLoaded);
  const [onboardingDismissed, setOnboardingDismissed] = useState(false);

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
      <MessageHeader />
      <AppFilter />
      <div className="flex-1 overflow-hidden">
        <MessageList />
      </div>
    </div>
  );
}

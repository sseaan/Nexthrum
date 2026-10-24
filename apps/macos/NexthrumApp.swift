import AppKit
import SwiftUI

@MainActor
final class AppLifecycleDelegate: NSObject, NSApplicationDelegate {
    /// Set once the launch window was dismissed, so the delayed second pass cannot
    /// close a window the user opened from the menu bar in the meantime.
    private var dismissedLaunchWindow = false

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }

    /// "Hide Dock icon" is an activation policy, so it has to be set before the
    /// first window is ordered front to avoid the Dock tile flashing on launch.
    func applicationWillFinishLaunching(_ notification: Notification) {
        guard AppPreferences.storedHidesDockIcon() else { return }
        NSApplication.shared.setActivationPolicy(.accessory)
    }

    /// SwiftUI's `WindowGroup` opens its window regardless of the activation
    /// policy; start menu-bar-only by dismissing it instead.
    func applicationDidFinishLaunching(_ notification: Notification) {
        guard AppPreferences.storedHidesDockIcon() else { return }
        dismissLaunchWindow()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
            guard let self, !self.dismissedLaunchWindow else { return }
            self.dismissLaunchWindow()
        }
    }

    private func dismissLaunchWindow() {
        for window in NSApplication.shared.windows where window.isVisible && window.canBecomeMain {
            window.close()
            dismissedLaunchWindow = true
        }
    }
}

@main
struct NexthrumApp: App {
    @NSApplicationDelegateAdaptor(AppLifecycleDelegate.self) private var appDelegate
    @StateObject private var model = AppModel()

    var body: some Scene {
        WindowGroup("Nexthrum", id: "messages") {
            ContentView()
                .environmentObject(model)
                .environmentObject(model.preferences)
                .frame(minWidth: 720, minHeight: 460)
        }
        .commands {
            CommandGroup(replacing: .appInfo) {
                Button("About Nexthrum") {
                    NSApp.orderFrontStandardAboutPanel()
                }
            }
            CommandGroup(after: .appInfo) {
                Button("Check for Updates…") {
                    model.updaterController.checkForUpdates(nil)
                }
                .disabled(!model.updaterController.updater.canCheckForUpdates)
            }
        }

        Settings {
            SettingsView()
                .environmentObject(model)
                .environmentObject(model.preferences)
        }

        MenuBarExtra {
            MenuBarPanel(model: model)
                .environmentObject(model)
        } label: {
            Label("Nexthrum", systemImage: "tray.full")
                .labelStyle(.iconOnly)
                .help(L10n.t("menubar_help", model.unreadCount))
        }
        .menuBarExtraStyle(.window)
    }
}

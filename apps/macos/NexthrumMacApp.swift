import AppKit
import SwiftUI

@MainActor
final class AppLifecycleDelegate: NSObject, NSApplicationDelegate {
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }
}

@main
struct NexthrumMacApp: App {
    @NSApplicationDelegateAdaptor(AppLifecycleDelegate.self) private var appDelegate
    @StateObject private var model = AppModel()

    var body: some Scene {
        WindowGroup("Nexthrum", id: "messages") {
            ContentView()
                .environmentObject(model)
                .frame(minWidth: 720, minHeight: 460)
        }
        .commands {
            CommandGroup(replacing: .appInfo) {
                Button("About Nexthrum") {
                    NSApp.orderFrontStandardAboutPanel()
                }
            }
        }

        Settings {
            SettingsView()
                .environmentObject(model)
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

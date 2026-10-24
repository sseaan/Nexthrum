import AppKit
import Combine
import Foundation

/// UI language choices for the macOS app.
///
/// Windows keeps its own locale in local storage (`nexthrum.locale`); the macOS
/// choice is a host preference and never travels through UniFFI.
enum AppLanguage: String, CaseIterable, Identifiable {
    case system
    case english = "en"
    case simplifiedChinese = "zh-Hans"

    var id: String { rawValue }

    /// `nil` means "follow the system language" (no `AppleLanguages` override).
    var localeIdentifier: String? {
        switch self {
        case .system:
            return nil
        case .english:
            return "en"
        case .simplifiedChinese:
            return "zh-Hans"
        }
    }

    /// Picker label, rendered in the language the app is currently running in.
    /// The concrete languages are self-describing so they stay recognisable.
    var displayName: String {
        switch self {
        case .system:
            return L10n.t("System Default")
        case .english:
            return "English"
        case .simplifiedChinese:
            return "简体中文"
        }
    }
}

/// macOS-only presentation preferences: Dock visibility and UI language.
///
/// Server URL, token, retention and DND live in the shared Rust core
/// (`BridgeSettings`); these two are host UX choices, so they are persisted in
/// `UserDefaults` and applied to AppKit directly.
@MainActor
final class AppPreferences: ObservableObject {
    static let languageKey = "preferences.language"
    static let appliedLanguageKey = "preferences.appliedLanguage"
    static let hideDockIconKey = "preferences.hideDockIcon"

    @Published private(set) var language: AppLanguage
    @Published private(set) var hideDockIcon: Bool
    /// True while the picker differs from the language this process started with.
    @Published private(set) var needsRelaunch = false

    private let defaults: UserDefaults
    /// Injected so the Dock behaviour can be asserted without a running AppKit app.
    private let applyDockPolicy: (Bool) -> Void

    init(
        defaults: UserDefaults = .standard,
        applyDockPolicy: ((Bool) -> Void)? = nil
    ) {
        self.defaults = defaults
        self.applyDockPolicy = applyDockPolicy ?? AppPreferences.applyDockVisibility
        let storedLanguage = defaults.string(forKey: Self.languageKey).flatMap(AppLanguage.init(rawValue:))
        language = storedLanguage ?? .system
        hideDockIcon = defaults.bool(forKey: Self.hideDockIconKey)
        // The stored selection is what this process launched with, so adopt it as
        // the applied language and (re)write `AppleLanguages` for the next launch.
        AppPreferences.applyLanguage(language, defaults: defaults)
        defaults.set(language.rawValue, forKey: Self.appliedLanguageKey)
        needsRelaunch = false
    }

    /// Everything that reads bundle strings (menus, menu bar label, notifications)
    /// resolves its language at process start, so a change needs a relaunch.
    func select(language newLanguage: AppLanguage) {
        guard newLanguage != language else { return }
        language = newLanguage
        defaults.set(newLanguage.rawValue, forKey: Self.languageKey)
        AppPreferences.applyLanguage(newLanguage, defaults: defaults)
        let applied = defaults.string(forKey: Self.appliedLanguageKey)
        needsRelaunch = applied != newLanguage.rawValue
    }

    /// Hiding the Dock icon switches the app to `.accessory`: no Dock tile and no
    /// ⌘-Tab entry, while the menu bar extra keeps the app reachable.
    func setHideDockIcon(_ hidden: Bool) {
        guard hidden != hideDockIcon else { return }
        hideDockIcon = hidden
        defaults.set(hidden, forKey: Self.hideDockIconKey)
        applyDockPolicy(hidden)
    }

    static func applyDockVisibility(hidden: Bool) {
        NSApplication.shared.setActivationPolicy(hidden ? .accessory : .regular)
        if !hidden {
            NSApplication.shared.activate(ignoringOtherApps: true)
        }
    }

    /// Read without instantiating the object: the app delegate needs this before
    /// the first SwiftUI scene exists.
    static func storedHidesDockIcon(defaults: UserDefaults = .standard) -> Bool {
        defaults.bool(forKey: hideDockIconKey)
    }

    /// Rewrite `AppleLanguages` in the app domain; `nil` restores system default.
    static func applyLanguage(_ language: AppLanguage, defaults: UserDefaults = .standard) {
        if let identifier = language.localeIdentifier {
            defaults.set([identifier], forKey: "AppleLanguages")
        } else {
            defaults.removeObject(forKey: "AppleLanguages")
        }
    }

    /// Relaunch so a language change is applied consistently.
    static func relaunch() {
        let configuration = NSWorkspace.OpenConfiguration()
        configuration.createsNewApplicationInstance = true
        NSWorkspace.shared.openApplication(at: Bundle.main.bundleURL, configuration: configuration) { _, _ in
            DispatchQueue.main.async {
                NSApplication.shared.terminate(nil)
            }
        }
    }
}

import AppKit
import Combine
import Foundation
import OSLog
import Security
import Sparkle
import SwiftUI

let logger = Logger(subsystem: "com.nexthrum.app", category: "connection")

@MainActor
final class AppModel: ObservableObject {
    @Published var serverURL = ""

    @Published var clientToken = ""

    @Published var historyRetentionDays = 7

    @Published var doNotDisturbStart = "22:00"

    @Published var doNotDisturbEnd = "08:00"

    @Published var autoStart = false

    @Published var launchAtLoginMessage: String?

    @Published var connectionStatus: ConnectionStatus = .notConfigured

    var connectionLabel: String { ConnectionStatusPresentation.label(for: connectionStatus) }
    /// Last stream failure reported by the shared client; shown next to the state.

    @Published var connectionError: String?
    /// Diagnostics surfaced in Settings so "nothing happens" becomes inspectable.

    @Published var eventsReceived = 0

    @Published var secondsSinceLastEvent: Double?
    /// appid -> display name, resolved from the Gotify `/application` endpoint.

    @Published var applicationNames: [Int64: String] = [:]

    @Published var validationMessage: String?

    @Published var connectionTestMessage: String?
    /// Whether the last connection test succeeded (drives the colour).

    @Published var connectionTestSucceeded = false

    @Published var isTestingConnection = false

    @Published var messages: [MacMessage] = []

    @Published var unreadCount = 0

    @Published var storageMessage: String?

    @Published var showsClearConfirmation = false

    @Published var pendingMessageID: Int64?
    /// Search text; an empty value means "show everything".
    @Published var searchQuery = ""
    /// Matching messages while a search is active, `nil` otherwise.
    @Published var searchResults: [MacMessage]?
    /// Debounce handle for search keystrokes.
    var searchTask: Task<Void, Never>?
    /// Incremented when a caller wants the window to show the Settings pane.
    /// Using state keeps navigation working on SDKs without `openSettings`.

    @Published var settingsRequestToken = 0
    /// True once settings were loaded from disk, so first-run onboarding only
    /// appears for genuinely unconfigured installs.

    @Published var hasStoredSettings = false
    /// Keychain is only touched when the token is actually needed.
    var credentialLoaded = false

    @Published var notificationPermissionStatus = "Checking…"

    /// Sparkle preferences mirrored into observable state so Settings can show
    /// them: automatic checks, whether a check may start right now, and the last
    /// check timestamp. Sparkle owns the persisted truth in UserDefaults.
    @Published var automaticallyChecksForUpdates = true

    @Published var canCheckForUpdates = true

    @Published var lastUpdateCheckDate: Date?

    @Published var notificationPermissionDenied = false

    /// Sparkle drives macOS updates: it checks the appcast, verifies the EdDSA
    /// signature and installs the new bundle. Starting it with the app is the
    /// documented setup for non-nib applications.
    let updaterController = SPUStandardUpdaterController(
        startingUpdater: true,
        updaterDelegate: nil,
        userDriverDelegate: nil
    )

    let notificationService = MacNotificationService()

    let launchAtLoginService = LaunchAtLoginService()

    /// Host-only UI preferences (Dock visibility, language); not part of
    /// `BridgeSettings` because the shared core has no say in them.
    let preferences = AppPreferences()

    let bridge = BridgeService()

    let credentialStore: CredentialStore = KeychainCredentialStore()

    var notificationStatusObservation: AnyCancellable?

    var subscriptions = Set<AnyCancellable>()

    var eventTimer: Timer?

    var lastEventAt: Date?

    var connectingSince: Date?

    init() {
        notificationService.onOpenMessage = { [weak self] id in
            self?.openNotificationMessage(id)
        }
        launchAtLoginService.$errorMessage
            .receive(on: RunLoop.main)
            .sink { [weak self] error in self?.launchAtLoginMessage = error }
            .store(in: &subscriptions)
        observeUpdater()
        notificationStatusObservation = notificationService.$authorizationStatus
            .combineLatest(notificationService.$authorizationDenied)
            .sink { [weak self] status, denied in
                self?.notificationPermissionStatus = status
                self?.notificationPermissionDenied = denied
            }
        do {
            try configureStorage()
            if let settings = try bridge.loadSettings() {
                hasStoredSettings = true
                serverURL = settings.serverUrl
                autoStart = settings.autoStart
                historyRetentionDays = Int(settings.historyRetentionDays)
                doNotDisturbStart = settings.doNotDisturbStart
                doNotDisturbEnd = settings.doNotDisturbEnd
                launchAtLoginService.refresh()
                let snapshot = try bridge.getMessageSnapshot()
                messages = snapshot.messages.map {
                    MacMessage(
                        id: $0.id,
                        appId: $0.appId,
                        title: $0.title,
                        body: $0.body,
                        date: $0.date,
                        priority: $0.priority,
                        read: $0.read
                    )
                }
                unreadCount = Int(snapshot.unreadCount)
                // Reading the Keychain can show a system prompt, so it is done
                // after the window is on screen instead of blocking first paint.
                Task { @MainActor [weak self] in
                    await Task.yield()
                    self?.connectIfConfigured()
                }
            }
        } catch {
            storageMessage = userMessage(for: error)
        }
    }

    /// Loads the client token from the Keychain on demand.
    ///
    /// Called after the window appears (or when Settings is opened) so the
    /// system authorization prompt never blocks the first launch.

    func currentBridgeSettings() -> BridgeSettings {
        BridgeSettings(
            serverUrl: serverURL,
            clientToken: clientToken,
            autoStart: autoStart,
            historyRetentionDays: UInt32(historyRetentionDays),
            doNotDisturbStart: doNotDisturbStart,
            doNotDisturbEnd: doNotDisturbEnd
        )
    }

    deinit {
        eventTimer?.invalidate()
        bridge.stopConnection()
    }

    func configureStorage() throws {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("Nexthrum", isDirectory: true)
        try FileManager.default.createDirectory(at: base, withIntermediateDirectories: true)
        try bridge.setStoragePaths(
            databasePath: base.appendingPathComponent("nexthrum_messages.db").path,
            settingsPath: base.appendingPathComponent("settings.json").path
        )
        if serverURL.isEmpty, let settings = try bridge.loadSettings() {
            serverURL = settings.serverUrl
            autoStart = settings.autoStart
            launchAtLoginService.refresh()
            historyRetentionDays = Int(settings.historyRetentionDays)
            doNotDisturbStart = settings.doNotDisturbStart
            doNotDisturbEnd = settings.doNotDisturbEnd
            if ensureCredentialLoaded() {
                try bridge.setSettings(settings: currentBridgeSettings())
            }
        }
    }

    /// Messages the list should render: search results when searching.
    var displayedMessages: [MacMessage] {
        searchResults ?? messages
    }

    var isSearching: Bool {
        !searchQuery.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    var isConfigured: Bool {
        !serverURL.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty &&
        !clientToken.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }
}


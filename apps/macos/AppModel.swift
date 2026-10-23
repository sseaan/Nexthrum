import AppKit
import Combine
import Foundation
import OSLog
import Security
import SwiftUI

private let logger = Logger(subsystem: "com.nexthrum.app", category: "connection")

@MainActor
final class AppModel: ObservableObject {
    @Published var serverURL = ""
    @Published var clientToken = ""
    @Published var historyRetentionDays = 7
    @Published var doNotDisturbStart = "22:00"
    @Published var doNotDisturbEnd = "08:00"
    @Published var autoStart = false
    @Published var launchAtLoginMessage: String?
    @Published private(set) var connectionStatus: ConnectionStatus = .notConfigured
    var connectionLabel: String { ConnectionStatusPresentation.label(for: connectionStatus) }
    /// Last stream failure reported by the shared client; shown next to the state.
    @Published var connectionError: String?
    /// Diagnostics surfaced in Settings so "nothing happens" becomes inspectable.
    @Published private(set) var eventsReceived = 0
    @Published private(set) var secondsSinceLastEvent: Double?
    /// appid -> display name, resolved from the Gotify `/application` endpoint.
    @Published private(set) var applicationNames: [Int64: String] = [:]
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
    /// True once settings were loaded from disk, so first-run onboarding only
    /// appears for genuinely unconfigured installs.
    @Published private(set) var hasStoredSettings = false
    /// Keychain is only touched when the token is actually needed.
    private(set) var credentialLoaded = false
    @Published var notificationPermissionStatus = "Checking…"
    @Published var notificationPermissionDenied = false

    let notificationService = MacNotificationService()
    let launchAtLoginService = LaunchAtLoginService()

    private let bridge = BridgeService()
    private let credentialStore: CredentialStore = KeychainCredentialStore()
    private var notificationStatusObservation: AnyCancellable?
    private var subscriptions = Set<AnyCancellable>()
    private var eventTimer: Timer?
    private var lastEventAt: Date?
    private var connectingSince: Date?

    init() {
        notificationService.onOpenMessage = { [weak self] id in
            self?.openNotificationMessage(id)
        }
        launchAtLoginService.$errorMessage
            .receive(on: RunLoop.main)
            .sink { [weak self] error in self?.launchAtLoginMessage = error }
            .store(in: &subscriptions)
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
    @discardableResult
    func ensureCredentialLoaded() -> Bool {
        guard !credentialLoaded else { return true }
        do {
            clientToken = try credentialStore.load() ?? ""
            credentialLoaded = true
        } catch {
            logger.error("Keychain read failed: \(userMessage(for: error), privacy: .public)")
            storageMessage = userMessage(for: error)
            return false
        }
        return true
    }

    /// Connects using saved settings once the credential is available.
    func connectIfConfigured() {
        guard hasStoredSettings else { return }
        guard ensureCredentialLoaded() else { return }
        guard !clientToken.isEmpty else {
            connectionStatus = .credentialsRequired
            return
        }
        do {
            try bridge.setSettings(settings: currentBridgeSettings())
            try bridge.startConnection()
            connectionStatus = .connecting
            logger.info("Starting Gotify stream for \(self.serverURL, privacy: .public)")
            startEventPolling()
            refreshApplicationNames()
        } catch {
            storageMessage = userMessage(for: error)
        }
    }

    private func currentBridgeSettings() -> BridgeSettings {
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

    func validateAndSave() {
        validationMessage = nil
        guard MacSettingsPolicy.isValid(
            retentionDays: historyRetentionDays,
            dndStart: doNotDisturbStart,
            dndEnd: doNotDisturbEnd
        ) else {
            validationMessage = L10n.t("Retention must be 1–365 days and DND times must use HH:MM format.")
            return
        }
        let settings = BridgeSettings(
            serverUrl: serverURL,
            clientToken: clientToken,
            autoStart: autoStart,
            historyRetentionDays: UInt32(historyRetentionDays),
            doNotDisturbStart: doNotDisturbStart,
            doNotDisturbEnd: doNotDisturbEnd
        )
        var oldSettingsSnapshot: BridgeSettings?
        var oldTokenSnapshot: String?
        var settingsCommitted = false
        do {
            bridge.stopConnection()
            try configureStorage()
            let oldSettings = try bridge.loadSettings()
            let oldToken = try credentialStore.load()
            oldSettingsSnapshot = oldSettings
            oldTokenSnapshot = oldToken
            try CredentialSettingsTransaction.commit(
                newCredential: clientToken,
                oldCredential: oldToken,
                saveCredential: { try credentialStore.save($0) },
                saveSettings: { try bridge.setSettings(settings: settings) },
                restoreCredential: { try credentialStore.restore($0) }
            )
            settingsCommitted = true
            credentialLoaded = true
            hasStoredSettings = true
            // Launch-at-login is applied asynchronously and reported separately:
            // a failure there must never block saving or the connection.
            launchAtLoginService.setEnabled(autoStart)
            try bridge.startConnection()
            validationMessage = nil
            connectionTestMessage = nil
            connectionError = nil
            connectionStatus = .connecting
            logger.info("Settings saved; starting Gotify stream")
            startEventPolling()
            refreshApplicationNames()
        } catch {
            if settingsCommitted {
                validationMessage = L10n.t("session_settings_saved_connection_failed", userMessage(for: error))
                connectionStatus = .savedConnectionFailed
                return
            }
            restoreSettings(oldSettingsSnapshot, token: oldTokenSnapshot)
            var recoveryError: String?
            if let oldSettingsSnapshot, let oldTokenSnapshot, !oldTokenSnapshot.isEmpty {
                var restoredSettings = oldSettingsSnapshot
                restoredSettings.clientToken = oldTokenSnapshot
                do {
                    try bridge.setSettings(settings: restoredSettings)
                    try bridge.startConnection()
                    startEventPolling()
                } catch {
                    recoveryError = userMessage(for: error)
                }
            }
            if let recoveryError {
                validationMessage = L10n.t("session_restored_previous_connection_failed", userMessage(for: error), recoveryError)
            } else {
                validationMessage = userMessage(for: error)
            }
            connectionStatus = .configurationNotSaved
        }
    }

    private func restoreSettings(_ settings: BridgeSettings?, token: String?) {
        guard let settings else { return }
        serverURL = settings.serverUrl
        historyRetentionDays = Int(settings.historyRetentionDays)
        doNotDisturbStart = settings.doNotDisturbStart
        doNotDisturbEnd = settings.doNotDisturbEnd
        clientToken = token ?? ""
    }

    private func configureStorage() throws {
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

    var pollingActive: Bool { eventTimer != nil }

    private func startEventPolling() {
        eventTimer?.invalidate()
        // `.common` keeps polling alive while menus/sheets track events;
        // a default-mode timer would pause and leave the UI on "Connecting…".
        let timer = Timer(timeInterval: 0.2, repeats: true) { [weak self] _ in
            Task { @MainActor [weak self] in self?.pollEvents() }
        }
        RunLoop.main.add(timer, forMode: .common)
        eventTimer = timer
        logger.notice("Event polling started")
    }

    private func pollEvents() {
        if let lastEventAt {
            secondsSinceLastEvent = Date().timeIntervalSince(lastEventAt)
        }
        if let connectingSince, Date().timeIntervalSince(connectingSince) > 15 {
            let message = L10n.t("connection_watchdog")
            logger.fault("Connection watchdog: \(message, privacy: .public)")
            connectionError = message
            connectionStatus = .disconnected
            self.connectingSince = nil
        }
        while let event = bridge.pollEvent() {
            eventsReceived += 1
            lastEventAt = Date()
            secondsSinceLastEvent = 0
            switch event.state {
            case .connecting:
                connectionStatus = .connecting
                if connectingSince == nil { connectingSince = Date() }
            case .connected:
                connectionStatus = .connected
                connectionError = nil
                connectingSince = nil
            case .disconnected:
                connectionStatus = .disconnected
                connectingSince = nil
            }
            logger.notice(
                "Bridge event: state=\(String(describing: event.state), privacy: .public) message=\(event.message != nil) error=\(event.error ?? "-", privacy: .public)"
            )
            if let error = event.error {
                if event.state == .disconnected {
                    // Transport/handshake failure: keep it next to the state label
                    // instead of the settings validation area.
                    logger.error("Stream failure: \(error, privacy: .public)")
                    connectionError = error
                } else {
                    logger.error("Message persistence failed: \(error, privacy: .public)")
                    validationMessage = error
                    connectionTestMessage = L10n.t("message_not_saved", error)
                }
            }
            if MacSettingsPolicy.shouldReplaceUnreadCount(
                hasMessage: event.message != nil,
                hasError: event.error != nil,
                isConnecting: event.state == .connecting
            ) {
                unreadCount = Int(event.unreadCount)
            }
            if let message = event.message {
                messages.removeAll { $0.id == message.id }
                messages.insert(MacMessage(
                    id: message.id,
                    appId: message.appId,
                    title: message.title,
                    body: message.body,
                    date: message.date,
                    priority: message.priority,
                    read: message.read
                ), at: 0)
                Task { @MainActor [weak self] in
                    await self?.notificationService.deliver(
                        messageID: message.id,
                        title: message.title,
                        body: message.body,
                        coreAllowsNotification: event.shouldNotify
                    )
                }
            }
        }
    }

    func openNotificationMessage(_ id: Int64) {
        pendingMessageID = id
        markRead(id)
        NSApp.activate(ignoringOtherApps: true)
    }

    func clearPendingMessageNavigation() {
        pendingMessageID = nil
    }

    /// Resolve source names once per connection. Failures keep the `App <id>`
    /// fallback so the filter bar still works on restricted servers.
    func refreshApplicationNames() {
        guard isConfigured else { return }
        Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                let applications = try await Task.detached { [bridge] in
                    try bridge.listApplications()
                }.value
                self.applicationNames = Dictionary(
                    uniqueKeysWithValues: applications.map { ($0.id, $0.name) }
                )
                logger.notice("Resolved \(applications.count) Gotify applications")
            } catch {
                logger.notice(
                    "Could not resolve Gotify application names: \(userMessage(for: error), privacy: .public)"
                )
            }
        }
    }

    /// Display name for a message source, falling back to the app id.
    func sourceName(for appId: Int64) -> String {
        applicationNames[appId] ?? "App \(appId)"
    }

    /// Recreate the stream after a failure without touching saved settings.
    func reconnect() {
        validationMessage = nil
        connectionError = nil
        do {
            bridge.stopConnection()
            try bridge.startConnection()
            connectionStatus = .connecting
            logger.info("Reconnecting Gotify stream")
            startEventPolling()
        } catch {
            let message = userMessage(for: error)
            logger.error("Reconnect failed: \(message, privacy: .public)")
            validationMessage = message
            connectionStatus = .disconnected
        }
    }

    func stopStreaming() {
        eventTimer?.invalidate()
        eventTimer = nil
        bridge.stopConnection()
        connectionStatus = .disconnected
    }

    func markRead(_ id: Int64) {
        do {
            unreadCount = Int(try bridge.markMessageRead(messageId: id))
            if let index = messages.firstIndex(where: { $0.id == id }) {
                let current = messages[index]
                messages[index] = MacMessage(
                    id: current.id,
                    appId: current.appId,
                    title: current.title,
                    body: current.body,
                    date: current.date,
                    priority: current.priority,
                    read: true
                )
            }
        } catch { storageMessage = userMessage(for: error) }
    }

    func deleteMessage(_ id: Int64) {
        do {
            unreadCount = Int(try bridge.deleteMessage(messageId: id))
            messages.removeAll { $0.id == id }
        } catch { storageMessage = userMessage(for: error) }
    }

    func clearMessages() {
        do {
            unreadCount = Int(try bridge.clearMessages())
            messages.removeAll()
        } catch { storageMessage = userMessage(for: error) }
    }

    /// Tests the Gotify HTTP endpoint only. Deliberately does **not** persist
    /// settings, start the stream or touch the login item.
    func testConnection() {
        guard isConfigured else {
            connectionTestMessage = L10n.t("Enter the server URL and client token first.")
            return
        }
        guard !isTestingConnection else { return }

        isTestingConnection = true
        connectionTestMessage = nil
        let url = serverURL
        let token = clientToken
        let retention = UInt32(historyRetentionDays)
        let dndStart = doNotDisturbStart
        let dndEnd = doNotDisturbEnd
        Task { @MainActor [weak self] in
            do {
                let result = try await Task.detached {
                    let service = BridgeService()
                    try service.setSettings(settings: BridgeSettings(
                        serverUrl: url,
                        clientToken: token,
                        autoStart: false,
                        historyRetentionDays: retention,
                        doNotDisturbStart: dndStart,
                        doNotDisturbEnd: dndEnd
                    ))
                    return try service.testConnection()
                }.value
                self?.connectionTestMessage = result
                self?.connectionTestSucceeded = true
                self?.isTestingConnection = false
            } catch {
                let message = userMessage(for: error)
                logger.error("Connection test failed: \(message, privacy: .public)")
                self?.connectionTestMessage = message
                self?.connectionTestSucceeded = false
                self?.isTestingConnection = false
            }
        }
    }

    var isConfigured: Bool {
        !serverURL.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty &&
        !clientToken.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }
}

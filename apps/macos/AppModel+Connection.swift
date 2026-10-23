//! AppModel: connection responsibilities.

import AppKit
import Foundation

extension AppModel {
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

    var pollingActive: Bool { eventTimer != nil }

    func startEventPolling() {
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

    func pollEvents() {
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
}

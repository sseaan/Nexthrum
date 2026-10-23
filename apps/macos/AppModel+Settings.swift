//! AppModel: settings responsibilities.

import AppKit
import Foundation

extension AppModel {
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

    func requestSettings() {
        settingsRequestToken += 1
    }

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
}

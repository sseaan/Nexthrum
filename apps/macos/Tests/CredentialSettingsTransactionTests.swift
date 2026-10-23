import XCTest

final class CredentialSettingsTransactionTests: XCTestCase {
    func testCredentialWriteFailureDoesNotAttemptSettingsCommit() {
        var settingsSaved = false
        XCTAssertThrowsError(try CredentialSettingsTransaction.commit(
            newCredential: "new-token",
            oldCredential: "old-token",
            saveCredential: { _ in throw TestError.credentialWriteFailed },
            saveSettings: { settingsSaved = true },
            restoreCredential: { _ in }
        ))
        XCTAssertFalse(settingsSaved)
    }

    func testRollbackFailureIsReportedAlongsideSettingsFailure() {
        XCTAssertThrowsError(try CredentialSettingsTransaction.commit(
            newCredential: "new-token",
            oldCredential: "old-token",
            saveCredential: { _ in },
            saveSettings: { throw TestError.settingsSaveFailed },
            restoreCredential: { _ in throw TestError.credentialRollbackFailed }
        )) { error in
            let description = error.localizedDescription
            XCTAssertTrue(description.contains("Settings save failed"))
            XCTAssertTrue(description.contains("Credential rollback"))
        }
    }

    func testSettingsFailureRestoresPreviousCredential() {
        var stored = "old-token"
        XCTAssertThrowsError(try CredentialSettingsTransaction.commit(
            newCredential: "new-token",
            oldCredential: stored,
            saveCredential: { stored = $0 },
            saveSettings: { throw TestError.settingsSaveFailed },
            restoreCredential: { stored = $0 ?? "" }
        ))
        XCTAssertEqual(stored, "old-token")
    }

    func testSuccessfulCommitKeepsNewCredential() throws {
        var stored = "old-token"
        var settingsSaved = false
        try CredentialSettingsTransaction.commit(
            newCredential: "new-token",
            oldCredential: stored,
            saveCredential: { stored = $0 },
            saveSettings: { settingsSaved = true },
            restoreCredential: { stored = $0 ?? "" }
        )
        XCTAssertEqual(stored, "new-token")
        XCTAssertTrue(settingsSaved)
    }

    private enum TestError: Error {
        case settingsSaveFailed
        case credentialWriteFailed
        case credentialRollbackFailed
    }
}

/// Guards against blocking the UI: every bridge call used by the settings flow
/// must return promptly, even when the server is unreachable.
final class BridgeCallResponsivenessTests: XCTestCase {
    func testBridgeSequenceReturnsPromptly() throws {
        let root = FileManager.default.temporaryDirectory
            .appendingPathComponent("nexthrum-bridge-test-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }

        let service = BridgeService()
        try service.setStoragePaths(
            databasePath: root.appendingPathComponent("messages.sqlite").path,
            settingsPath: root.appendingPathComponent("settings.json").path
        )
        // Port 1 is reserved and refuses immediately.
        try service.setSettings(settings: BridgeSettings(
            serverUrl: "http://127.0.0.1:1",
            clientToken: "token",
            autoStart: false,
            historyRetentionDays: 7,
            doNotDisturbStart: "22:00",
            doNotDisturbEnd: "08:00"
        ))

        let started = Date()
        try? service.startConnection()
        for _ in 0..<20 {
            _ = service.pollEvent()
            Thread.sleep(forTimeInterval: 0.05)
        }
        _ = try? service.testConnection()
        service.stopConnection()

        XCTAssertLessThan(
            Date().timeIntervalSince(started),
            10,
            "bridge calls must not block the UI thread"
        )
    }
}

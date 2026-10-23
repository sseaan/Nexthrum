import Foundation

enum CredentialSettingsTransaction {
    /// Restore the old credential if committing the non-secret settings fails.
    static func commit(
        newCredential: String,
        oldCredential: String?,
        saveCredential: (String) throws -> Void,
        saveSettings: () throws -> Void,
        restoreCredential: (String?) throws -> Void
    ) throws {
        try saveCredential(newCredential)
        do {
            try saveSettings()
        } catch {
            let settingsError = error
            do {
                try restoreCredential(oldCredential)
            } catch {
                throw NSError(
                    domain: "NexthrumCredentialTransaction",
                    code: 1,
                    userInfo: [NSLocalizedDescriptionKey: "Settings save failed and credential rollback also failed. Settings: \(settingsError.localizedDescription). Credential rollback: \(error.localizedDescription)"]
                )
            }
            throw settingsError
        }
    }
}

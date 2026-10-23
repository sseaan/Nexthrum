import Foundation

/// UniFFI turns `BridgeError` into `String(reflecting:)` for
/// `localizedDescription`, which surfaces raw case names such as
/// "NetworkUnavailable". Map every case to an actionable user-facing message.
extension BridgeError {
    var userMessage: String {
        switch self {
        case .MissingServerUrl:
            return L10n.t("Enter the Gotify server URL.")
        case .MissingClientToken:
            return L10n.t("Enter the Gotify client token.")
        case .InvalidServerUrl:
            return L10n.t("Server URL must start with http:// or https://.")
        case .InvalidDndTime:
            return L10n.t("Do-not-disturb times must use HH:MM.")
        case .InvalidReceivedAt:
            return L10n.t("The message timestamp is invalid.")
        case .ConnectionFailed:
            return L10n.t("Could not connect to the Gotify server.")
        case .Unauthorized:
            return L10n.t("The server rejected the client token. Check the token and try again.")
        case .ServerNotFound:
            return L10n.t("The Gotify API was not found at this URL. Check the server path.")
        case .ServerRejected:
            return L10n.t("The Gotify server rejected the request.")
        case .NetworkUnavailable:
            return L10n.t("Cannot reach the server. Check the URL and your network connection.")
        case .RequestTimeout:
            return L10n.t("The server did not respond in time. Try again.")
        case .TlsFailure:
            return L10n.t("TLS handshake failed. Check the server certificate.")
        case .InvalidResponse:
            return L10n.t("The server returned an unexpected response.")
        case .ConnectionAlreadyRunning:
            return L10n.t("The connection is already running.")
        case .StorageNotConfigured:
            return L10n.t("Message storage is not configured yet.")
        case .InvalidStoragePath:
            return L10n.t("The message storage location is invalid.")
        case .StorageFailed:
            return L10n.t("Saving or reading messages failed.")
        }
    }
}

/// Bridge-aware replacement for `error.localizedDescription`.
func userMessage(for error: Error) -> String {
    if let bridgeError = error as? BridgeError {
        return bridgeError.userMessage
    }
    return error.localizedDescription
}

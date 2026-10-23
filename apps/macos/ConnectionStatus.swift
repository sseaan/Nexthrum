import Foundation

/// Connection state as data; the display label is localized separately so
/// logic never compares user-visible strings.
enum ConnectionStatus: Equatable {
    case notConfigured
    case credentialsRequired
    case connecting
    case connected
    case disconnected
    case savedConnectionFailed
    case configurationNotSaved
}

enum ConnectionStatusPresentation {
    static func label(for status: ConnectionStatus) -> String {
        switch status {
        case .notConfigured: return L10n.t("Not configured")
        case .credentialsRequired: return L10n.t("Credentials required")
        case .connecting: return L10n.t("Connecting…")
        case .connected: return L10n.t("Connected")
        case .disconnected: return L10n.t("Disconnected")
        case .savedConnectionFailed: return L10n.t("Saved; connection failed")
        case .configurationNotSaved: return L10n.t("Configuration not saved")
        }
    }
}

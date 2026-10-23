import Foundation

/// Validation shared by the macOS settings form before committing a new snapshot.
enum MacSettingsPolicy {
    static func shouldReplaceUnreadCount(hasMessage: Bool, hasError: Bool, isConnecting: Bool) -> Bool {
        hasMessage || hasError || isConnecting
    }

    static func isValidTime(_ value: String) -> Bool {
        let parts = value.split(separator: ":", omittingEmptySubsequences: false)
        guard parts.count == 2,
              parts[0].count == 2,
              parts[1].count == 2,
              let hour = Int(parts[0]),
              let minute = Int(parts[1]) else { return false }
        return (0..<24).contains(hour) && (0..<60).contains(minute)
    }

    static func isValid(retentionDays: Int, dndStart: String, dndEnd: String) -> Bool {
        (1...365).contains(retentionDays) && isValidTime(dndStart) && isValidTime(dndEnd)
    }
}

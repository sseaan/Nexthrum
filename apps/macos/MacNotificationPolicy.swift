import Foundation

/// Authorization states used by the platform adapter's delivery policy.
enum MacNotificationAuthorization: Equatable {
    case notDetermined
    case denied
    case authorized
    case provisional
    case ephemeral
    case unavailable
}

enum MacNotificationPolicy {
    static func shouldDeliver(
        coreAllowsNotification: Bool,
        authorization: MacNotificationAuthorization
    ) -> Bool {
        guard coreAllowsNotification else { return false }
        switch authorization {
        case .authorized, .provisional, .ephemeral:
            return true
        case .notDetermined, .denied, .unavailable:
            return false
        }
    }

    static func messageID(from userInfo: [AnyHashable: Any]) -> Int64? {
        if let value = userInfo["messageID"] as? Int64 { return value }
        if let value = userInfo["messageID"] as? NSNumber { return value.int64Value }
        return nil
    }
}

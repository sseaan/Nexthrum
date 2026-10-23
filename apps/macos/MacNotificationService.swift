import Combine
import Foundation
import UserNotifications

@MainActor
final class MacNotificationService: NSObject, ObservableObject, UNUserNotificationCenterDelegate {
    @Published private(set) var authorizationStatus = "Checking…"
    @Published private(set) var authorizationDenied = false
    private(set) var authorization = MacNotificationAuthorization.notDetermined

    var onOpenMessage: ((Int64) -> Void)?

    private let center = UNUserNotificationCenter.current()

    override init() {
        super.init()
        center.delegate = self
        Task { await refreshAuthorizationStatus() }
    }

    func requestAuthorization() async {
        do {
            _ = try await center.requestAuthorization(options: [.alert, .sound, .badge])
            await refreshAuthorizationStatus()
        } catch {
            authorizationStatus = "Permission request failed: \(error.localizedDescription)"
        }
    }

    func refreshAuthorizationStatus() async {
        let settings = await center.notificationSettings()
        switch settings.authorizationStatus {
        case .notDetermined:
            authorization = .notDetermined
            authorizationStatus = "Not requested"
            authorizationDenied = false
        case .denied:
            authorization = .denied
            authorizationStatus = "Disabled in System Settings"
            authorizationDenied = true
        case .authorized:
            authorization = .authorized
            authorizationStatus = "Enabled"
            authorizationDenied = false
        case .provisional:
            authorization = .provisional
            authorizationStatus = "Enabled (provisional)"
            authorizationDenied = false
        case .ephemeral:
            authorization = .ephemeral
            authorizationStatus = "Enabled"
            authorizationDenied = false
        @unknown default:
            authorization = .unavailable
            authorizationStatus = "Unavailable"
            authorizationDenied = false
        }
    }

    func deliver(messageID: Int64, title: String, body: String, coreAllowsNotification: Bool) async {
        guard coreAllowsNotification else { return }
        await refreshAuthorizationStatus()
        guard MacNotificationPolicy.shouldDeliver(
            coreAllowsNotification: coreAllowsNotification,
            authorization: authorization
        ) else { return }

        let content = UNMutableNotificationContent()
        content.title = title.isEmpty ? "Gotify message" : title
        content.body = body
        content.sound = .default
        content.userInfo = ["messageID": messageID]

        let request = UNNotificationRequest(
            identifier: "nexthrum-message-\(messageID)-\(UUID().uuidString)",
            content: content,
            trigger: nil
        )
        do {
            try await center.add(request)
        } catch {
            authorizationStatus = "Could not send notification: \(error.localizedDescription)"
        }
    }

    nonisolated func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        let messageID = MacNotificationPolicy.messageID(
            from: response.notification.request.content.userInfo
        )
        Task { @MainActor [weak self] in
            if let messageID {
                self?.onOpenMessage?(messageID)
            }
        }
        completionHandler()
    }

    nonisolated func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .list, .sound])
    }
}

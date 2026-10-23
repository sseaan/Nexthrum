import Combine
import Foundation
import ServiceManagement

@MainActor
final class LaunchAtLoginService: ObservableObject {
    @Published private(set) var isEnabled = false
    @Published private(set) var errorMessage: String?
    @Published private(set) var statusDescription = "Not registered"

    init() {
        refresh()
    }

    func refresh() {
        apply(SMAppService.mainApp.status)
    }

    /// Register/unregister off the main thread: `SMAppService` can block for a
    /// noticeable time (and may wait on a system prompt), which used to freeze
    /// the settings form.
    func setEnabled(_ enabled: Bool) {
        errorMessage = nil
        let status = SMAppService.mainApp.status
        if enabled && (status == .enabled || status == .requiresApproval) {
            // Already registered; the system list may just lag behind.
            apply(status)
            watchForSettledStatus(remainingChecks: 8)
            return
        }
        if !enabled && status == .notRegistered {
            apply(status)
            return
        }

        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            var failure: String?
            do {
                if enabled {
                    try SMAppService.mainApp.register()
                } else {
                    try SMAppService.mainApp.unregister()
                }
            } catch {
                failure = error.localizedDescription
            }
            DispatchQueue.main.async {
                guard let self else { return }
                if let failure {
                    self.apply(SMAppService.mainApp.status)
                    self.errorMessage = Self.describe(failure: failure)
                } else {
                    // The system updates its login-item list asynchronously.
                    self.watchForSettledStatus(remainingChecks: 10)
                }
            }
        }
    }

    /// Re-read the status a few times so the UI reflects the (async) system state
    /// instead of showing a stale "Disabled" right after enabling.
    private func watchForSettledStatus(remainingChecks: Int) {
        apply(SMAppService.mainApp.status)
        guard remainingChecks > 0 else { return }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) { [weak self] in
            guard let self else { return }
            let status = SMAppService.mainApp.status
            self.apply(status)
            if status == .enabled || status == .notRegistered || status == .notFound {
                return
            }
            self.watchForSettledStatus(remainingChecks: remainingChecks - 1)
        }
    }

    private func apply(_ status: SMAppService.Status) {
        isEnabled = status == .enabled
        statusDescription = Self.describe(status: status)
    }

    static func describe(status: SMAppService.Status) -> String {
        LaunchAtLoginPresentation.describe(status: status)
    }

    static func describe(failure: String) -> String {
        LaunchAtLoginPresentation.describe(failure: failure)
    }
}

/// Pure presentation helpers (non-isolated so they are unit-testable).
enum LaunchAtLoginPresentation {
    static func describe(status: SMAppService.Status) -> String {
        switch status {
        case .enabled:
            return L10n.t("Enabled")
        case .requiresApproval:
            return L10n.t("Pending approval in System Settings → General → Login Items")
        case .notRegistered:
            return L10n.t("Disabled")
        case .notFound:
            return L10n.t("Unavailable; move Nexthrum into /Applications")
        @unknown default:
            return L10n.t("Unknown status")
        }
    }

    static func describe(failure: String) -> String {
        let inApplications = Bundle.main.bundlePath.hasPrefix("/Applications/")
        let hint = inApplications ? "" : L10n.t("move_into_applications_hint")
        return L10n.t("launch_at_login_failed", failure, hint)
    }
}

import Combine
import Foundation
import Sparkle

/**
 Sparkle bridge for the Settings pane.

 Sparkle keeps `automaticallyChecksForUpdates` and the last check date in
 `UserDefaults`, so the model mirrors them instead of owning them: writes go
 through the updater, KVO notifications come back into `@Published` state.
 */
extension AppModel {
    var updater: SPUUpdater { updaterController.updater }

    func observeUpdater() {
        updater.publisher(for: \.canCheckForUpdates)
            .receive(on: RunLoop.main)
            .sink { [weak self] value in self?.canCheckForUpdates = value }
            .store(in: &subscriptions)

        updater.publisher(for: \.automaticallyChecksForUpdates)
            .receive(on: RunLoop.main)
            .sink { [weak self] value in self?.automaticallyChecksForUpdates = value }
            .store(in: &subscriptions)

        updater.publisher(for: \.lastUpdateCheckDate)
            .receive(on: RunLoop.main)
            .sink { [weak self] value in self?.lastUpdateCheckDate = value }
            .store(in: &subscriptions)

        // Seed immediately: KVO only fires on later changes.
        automaticallyChecksForUpdates = updater.automaticallyChecksForUpdates
        canCheckForUpdates = updater.canCheckForUpdates
        lastUpdateCheckDate = updater.lastUpdateCheckDate
    }

    func setAutomaticallyChecksForUpdates(_ value: Bool) {
        guard updater.automaticallyChecksForUpdates != value else { return }
        updater.automaticallyChecksForUpdates = value
    }

    /// Same entry point as the "Check for Updates…" menu item.
    func checkForUpdates() {
        guard updater.canCheckForUpdates else { return }
        updaterController.checkForUpdates(nil)
    }
}

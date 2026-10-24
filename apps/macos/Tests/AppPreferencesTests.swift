import XCTest

@MainActor
final class AppPreferencesTests: XCTestCase {
    /// Each test gets its own defaults domain so `AppleLanguages` writes during
    /// the test never touch the real app preference store.
    private func makePreferences(
        policy: @escaping (Bool) -> Void = { _ in }
    ) -> (preferences: AppPreferences, defaults: UserDefaults, domain: String) {
        let suiteName = "com.nexthrum.app.tests.preferences.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suiteName)!
        addTeardownBlock {
            defaults.removePersistentDomain(forName: suiteName)
        }
        return (AppPreferences(defaults: defaults, applyDockPolicy: policy), defaults, suiteName)
    }

    /// Reads the suite's own domain: `object(forKey:)` would fall through to the
    /// global `AppleLanguages` of the machine running the tests.
    private func storedAppleLanguages(_ defaults: UserDefaults, _ domain: String) -> Any? {
        defaults.synchronize()
        return defaults.persistentDomain(forName: domain)?["AppleLanguages"]
    }

    func testDefaultPreferencesFollowTheSystemAndShowTheDockIcon() {
        var policies: [Bool] = []
        let (preferences, defaults, domain) = makePreferences(policy: { policies.append($0) })
        XCTAssertEqual(preferences.language, .system)
        XCTAssertNil(storedAppleLanguages(defaults, domain))
        XCTAssertFalse(preferences.hideDockIcon)
        XCTAssertFalse(preferences.needsRelaunch)
        XCTAssertTrue(policies.isEmpty, "launch must not move the Dock icon")
    }

    func testSelectingALanguageWritesAppleLanguagesAndAsksForRelaunch() {
        let (preferences, defaults, domain) = makePreferences()
        preferences.select(language: .simplifiedChinese)
        XCTAssertEqual(preferences.language, .simplifiedChinese)
        XCTAssertEqual(storedAppleLanguages(defaults, domain) as? [String], ["zh-Hans"])
        XCTAssertEqual(defaults.string(forKey: AppPreferences.languageKey), "zh-Hans")
        XCTAssertTrue(preferences.needsRelaunch)
    }

    func testReturningToTheRunningLanguageClearsTheRelaunchHint() {
        let (preferences, defaults, domain) = makePreferences()
        preferences.select(language: .english)
        preferences.select(language: .system)
        XCTAssertEqual(preferences.language, .system)
        XCTAssertFalse(preferences.needsRelaunch)
        XCTAssertNil(storedAppleLanguages(defaults, domain))
    }

    func testStoredLanguageIsAdoptedOnLaunchWithoutRelaunchHint() {
        let suiteName = "com.nexthrum.app.tests.preferences.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suiteName)!
        addTeardownBlock {
            defaults.removePersistentDomain(forName: suiteName)
        }
        defaults.set(AppLanguage.simplifiedChinese.rawValue, forKey: AppPreferences.languageKey)

        let preferences = AppPreferences(defaults: defaults, applyDockPolicy: { _ in })
        XCTAssertEqual(preferences.language, .simplifiedChinese)
        XCTAssertFalse(preferences.needsRelaunch)
        XCTAssertEqual(storedAppleLanguages(defaults, suiteName) as? [String], ["zh-Hans"])
    }

    func testHidingTheDockIconPersistsAndAppliesThePolicyOnce() {
        var policies: [Bool] = []
        let (preferences, defaults, _) = makePreferences(policy: { policies.append($0) })
        preferences.setHideDockIcon(true)
        preferences.setHideDockIcon(true)
        XCTAssertTrue(preferences.hideDockIcon)
        XCTAssertTrue(defaults.bool(forKey: AppPreferences.hideDockIconKey))
        XCTAssertTrue(AppPreferences.storedHidesDockIcon(defaults: defaults))
        XCTAssertEqual(policies, [true])

        preferences.setHideDockIcon(false)
        XCTAssertFalse(preferences.hideDockIcon)
        XCTAssertEqual(policies, [true, false])
    }

    func testLanguageIdentifiersMapToBundleLocalizations() {
        XCTAssertNil(AppLanguage.system.localeIdentifier)
        XCTAssertEqual(AppLanguage.english.localeIdentifier, "en")
        XCTAssertEqual(AppLanguage.simplifiedChinese.localeIdentifier, "zh-Hans")
        XCTAssertFalse(AppLanguage.allCases.contains { $0.displayName.isEmpty })
    }
}

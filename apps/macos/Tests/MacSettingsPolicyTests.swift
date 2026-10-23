import ServiceManagement
import XCTest

final class MacSettingsPolicyTests: XCTestCase {
    func testConnectionStateEventsDoNotEraseUnreadCount() {
        XCTAssertFalse(MacSettingsPolicy.shouldReplaceUnreadCount(
            hasMessage: false,
            hasError: false,
            isConnecting: false
        ))
        XCTAssertTrue(MacSettingsPolicy.shouldReplaceUnreadCount(
            hasMessage: true,
            hasError: false,
            isConnecting: false
        ))
        XCTAssertTrue(MacSettingsPolicy.shouldReplaceUnreadCount(
            hasMessage: false,
            hasError: false,
            isConnecting: true
        ))
    }

    func testAcceptsValidDndTimesAndRetention() {
        XCTAssertTrue(MacSettingsPolicy.isValid(
            retentionDays: 7,
            dndStart: "22:00",
            dndEnd: "08:00"
        ))
        XCTAssertTrue(MacSettingsPolicy.isValidTime("00:00"))
        XCTAssertTrue(MacSettingsPolicy.isValidTime("23:59"))
    }

    func testRejectsInvalidTimeAndRetentionValues() {
        XCTAssertFalse(MacSettingsPolicy.isValidTime("24:00"))
        XCTAssertFalse(MacSettingsPolicy.isValidTime("12:60"))
        XCTAssertFalse(MacSettingsPolicy.isValidTime("7:00"))
        XCTAssertFalse(MacSettingsPolicy.isValidTime("noon"))
        XCTAssertFalse(MacSettingsPolicy.isValid(retentionDays: 0, dndStart: "22:00", dndEnd: "08:00"))
        XCTAssertFalse(MacSettingsPolicy.isValid(retentionDays: 366, dndStart: "22:00", dndEnd: "08:00"))
    }
}

final class BridgeErrorMessageTests: XCTestCase {
    func testEveryErrorHasActionableMessage() {
        let errors: [BridgeError] = [
            .MissingServerUrl, .MissingClientToken, .InvalidServerUrl, .InvalidDndTime,
            .InvalidReceivedAt, .ConnectionFailed, .Unauthorized, .ServerNotFound,
            .ServerRejected, .NetworkUnavailable, .RequestTimeout, .TlsFailure,
            .InvalidResponse, .ConnectionAlreadyRunning, .StorageNotConfigured,
            .InvalidStoragePath, .StorageFailed,
        ]
        var messages: Set<String> = []
        for error in errors {
            let message = error.userMessage
            XCTAssertFalse(message.isEmpty)
            // Must not leak the raw UniFFI case name.
            XCTAssertFalse(message.contains(String(describing: error)))
            messages.insert(message)
        }
        XCTAssertEqual(messages.count, errors.count, "every case needs a distinct message")
    }

    func testUnauthorizedPointsAtTheToken() {
        XCTAssertTrue(BridgeError.Unauthorized.userMessage.localizedCaseInsensitiveContains("token"))
    }

    func testUserMessageFallsBackForNonBridgeErrors() {
        struct Other: LocalizedError {
            var errorDescription: String? { "plain failure" }
        }
        XCTAssertEqual(userMessage(for: Other()), "plain failure")
        XCTAssertEqual(userMessage(for: BridgeError.RequestTimeout), BridgeError.RequestTimeout.userMessage)
    }
}

final class LaunchAtLoginDescriptionTests: XCTestCase {
    func testStatusDescriptionsAreDistinctAndActionable() {
        let enabled = LaunchAtLoginPresentation.describe(status: .enabled)
        let approval = LaunchAtLoginPresentation.describe(status: .requiresApproval)
        let disabled = LaunchAtLoginPresentation.describe(status: .notRegistered)
        let missing = LaunchAtLoginPresentation.describe(status: .notFound)

        XCTAssertEqual(Set([enabled, approval, disabled, missing]).count, 4)
        XCTAssertTrue(approval.localizedCaseInsensitiveContains("approval"))
        XCTAssertTrue(missing.localizedCaseInsensitiveContains("/Applications"))
    }

    func testApplicationsHintIsOnlyShownForBundlesOutsideApplications() {
        XCTAssertTrue(LaunchAtLoginPresentation.needsApplicationsHint(
            bundlePath: "/Users/someone/Downloads/NexthrumMac.app"
        ))
        XCTAssertTrue(LaunchAtLoginPresentation.needsApplicationsHint(
            bundlePath: "/Volumes/Nexthrum/NexthrumMac.app"
        ))
        XCTAssertFalse(LaunchAtLoginPresentation.needsApplicationsHint(
            bundlePath: "/Applications/NexthrumMac.app"
        ))
    }

    // Note: the message text itself is not asserted here. XCTest runs without the
    // app's localisation resources (Bundle.main is the test runner), so the
    // formatted string would only contain raw keys. The decision logic that
    // matters — whether the /Applications hint applies — is covered above.
}

final class MessageDateFormattingTests: XCTestCase {
    private var calendar: Calendar {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(secondsFromGMT: 0)!
        return calendar
    }

    private let now = ISO8601DateFormatter().date(from: "2026-10-23T09:00:00Z")!
    private let timeZone = TimeZone(secondsFromGMT: 0)!
    private let locale = Locale(identifier: "en_US_POSIX")

    private func label(_ raw: String) -> String {
        MessageDateFormatting.listLabel(
            for: raw,
            now: now,
            calendar: calendar,
            locale: locale,
            timeZone: timeZone
        )
    }

    func testSameDayShowsTimeOnly() {
        XCTAssertEqual(label("2026-10-23T02:05:00Z"), "02:05")
    }

    func testSameYearShowsMonthDayAndTime() {
        XCTAssertEqual(label("2026-03-04T16:45:00Z"), "03-04 16:45")
    }

    func testOlderThanAYearShowsDateOnly() {
        XCTAssertEqual(label("2024-12-31T23:59:00Z"), "2024-12-31")
    }

    func testFractionalSecondsAndMissingTimeZoneAreParsed() {
        XCTAssertEqual(label("2026-10-23T02:05:30.123Z"), "02:05")
        XCTAssertEqual(label("2026-10-23T02:05:30"), "02:05")
    }

    func testOffsetTimestampsAreConvertedToTheLocalDay() {
        // 02:05+08:00 is the previous day in UTC, so the row shows a date.
        XCTAssertEqual(label("2026-10-23T02:05:30.123+08:00"), "10-22 18:05")
    }

    func testUnparseableValueFallsBackToTheRawValue() {
        XCTAssertEqual(MessageDateFormatting.listLabel(for: "not-a-date"), "not-a-date")
        XCTAssertEqual(MessageDateFormatting.listLabel(for: ""), "")
    }

    func testDetailedLabelIsUsableForTooltips() {
        let detailed = MessageDateFormatting.detailedLabel(
            for: "2026-10-23T02:05:00Z",
            locale: locale,
            timeZone: timeZone
        )
        XCTAssertTrue(detailed.contains("2026"))
        XCTAssertTrue(detailed.contains("2:05"))
    }
}

final class MessageSourceGroupingTests: XCTestCase {
    private func message(id: Int64, app: Int64, read: Bool) -> MacMessage {
        MacMessage(
            id: id,
            appId: app,
            title: "T\(id)",
            body: "B\(id)",
            date: "2026-10-23T02:05:00Z",
            priority: 0,
            read: read
        )
    }

    func testSourcesAreCountedAndSortedByUnreadThenVolume() {
        let messages = [
            message(id: 1, app: 7, read: false),
            message(id: 2, app: 7, read: false),
            message(id: 3, app: 3, read: true),
            message(id: 4, app: 3, read: true),
            message(id: 5, app: 3, read: false),
            message(id: 6, app: 9, read: true),
        ]
        let sources = MessageSourceGrouping.sources(from: messages) { "App \($0)" }

        XCTAssertEqual(sources.map(\.appId), [7, 3, 9])
        XCTAssertEqual(sources[0].unread, 2)
        XCTAssertEqual(sources[1].total, 3)
        XCTAssertEqual(sources[1].unread, 1)
        XCTAssertEqual(sources[2].unread, 0)
        XCTAssertEqual(sources[0].label, "App 7")

        let named = MessageSourceGrouping.sources(from: messages) { $0 == 7 ? "CI" : "Other" }
        XCTAssertEqual(named[0].label, "CI")
        XCTAssertEqual(named[1].label, "Other")
    }

    func testSingleSourceProducesOneEntrySoTheBarCanStayHidden() {
        let sources = MessageSourceGrouping.sources(from: [
            message(id: 1, app: 4, read: false),
            message(id: 2, app: 4, read: true),
        ])
        XCTAssertEqual(sources.count, 1)
        XCTAssertEqual(sources[0].total, 2)
    }

    func testFilterSelectsASingleSourceOrEverything() {
        let messages = [
            message(id: 1, app: 7, read: false),
            message(id: 2, app: 3, read: true),
        ]
        XCTAssertEqual(MessageSourceGrouping.filter(messages, selected: nil).count, 2)
        XCTAssertEqual(MessageSourceGrouping.filter(messages, selected: 3).map(\.id), [2])
        XCTAssertTrue(MessageSourceGrouping.filter(messages, selected: 42).isEmpty)
    }

    func testEmptyInputProducesNoSources() {
        XCTAssertTrue(MessageSourceGrouping.sources(from: []).isEmpty)
    }
}

final class LocalizationTests: XCTestCase {
    /// Reads the string catalog from the repository, so the test validates the
    /// source of truth rather than the compiled bundle of the test runner.
    private func catalog() throws -> [String: Any] {
        let testFile = URL(fileURLWithPath: #filePath)
        let catalogURL = testFile
            .deletingLastPathComponent()      // Tests/
            .deletingLastPathComponent()      // apps/macos/
            .appendingPathComponent("Resources/Localizable.xcstrings")
        let data = try Data(contentsOf: catalogURL)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        return (json?["strings"] as? [String: Any]) ?? [:]
    }

    private func translations(_ catalog: [String: Any], _ language: String) -> [String: String] {
        var result: [String: String] = [:]
        for (key, value) in catalog {
            let entry = value as? [String: Any]
            let localizations = entry?["localizations"] as? [String: Any]
            let unit = (localizations?[language] as? [String: Any])?["stringUnit"] as? [String: Any]
            if let text = unit?["value"] as? String {
                result[key] = text
            }
        }
        return result
    }

    func testEveryEnglishKeyHasAChineseTranslation() throws {
        let strings = try catalog()
        let english = Set(translations(strings, "en").keys)
        let chinese = Set(translations(strings, "zh-Hans").keys)

        XCTAssertFalse(english.isEmpty)
        XCTAssertEqual(
            english,
            chinese,
            "missing zh-Hans keys: \(english.subtracting(chinese).sorted())"
        )
    }

    func testLocalizedStringsAreActuallyTranslated() throws {
        let strings = try catalog()
        let chinese = translations(strings, "zh-Hans")
        XCTAssertEqual(chinese["Messages"], "消息")
        XCTAssertEqual(chinese["Connected"], "已连接")
        XCTAssertEqual(chinese["unread_badge"], "%lld 条未读")
    }

    func testConnectionStatusLabelsAreDistinctAndLocalized() {
        let statuses: [ConnectionStatus] = [
            .notConfigured, .credentialsRequired, .connecting, .connected,
            .disconnected, .savedConnectionFailed, .configurationNotSaved,
        ]
        let labels = statuses.map(ConnectionStatusPresentation.label(for:))
        XCTAssertEqual(Set(labels).count, statuses.count, "labels must be unique")
        XCTAssertFalse(labels.contains { $0.isEmpty })
    }
}

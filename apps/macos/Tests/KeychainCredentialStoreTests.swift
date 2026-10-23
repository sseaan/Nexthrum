import XCTest

/// Integration tests against the real login keychain.
///
/// They are compile-time opt-in because a CI runner or an unsigned test bundle
/// may be unable to access the keychain (or may prompt the user):
///
///     xcodebuild ... test \
///       SWIFT_ACTIVE_COMPILATION_CONDITIONS='$(inherited) NEXTHRUM_KEYCHAIN_TESTS'
///
/// Runtime environment variables are not forwarded to the `xctest` process, so a
/// compilation condition is used instead of `ProcessInfo`.
///
/// Each test uses a throwaway service/account pair, so the application's real
/// token item (`com.nexthrum.gotify` / `client-token`) is never read or written.
final class KeychainCredentialStoreTests: XCTestCase {
    private var store: KeychainCredentialStore!
    private let account = "integration-\(UUID().uuidString)"

    override func setUpWithError() throws {
#if !NEXTHRUM_KEYCHAIN_TESTS
        throw XCTSkip(
            "Build with SWIFT_ACTIVE_COMPILATION_CONDITIONS=NEXTHRUM_KEYCHAIN_TESTS to run login-keychain tests"
        )
#else
        store = KeychainCredentialStore(
            service: "com.nexthrum.gotify.tests",
            account: account
        )
        try store.restore(nil)
#endif
    }

    override func tearDownWithError() throws {
        guard store != nil else { return }
        try store.restore(nil)
    }

    func testRoundTripPersistsAndOverwritesTheToken() throws {
        XCTAssertNil(try store.load(), "a fresh test account must start empty")

        try store.save("first-token")
        XCTAssertEqual(try store.load(), "first-token")

        try store.save("second-token")
        XCTAssertEqual(try store.load(), "second-token", "save must update, not duplicate")
    }

    func testRestoreClearsTheStoredToken() throws {
        try store.save("secret")
        try store.restore(nil)
        XCTAssertNil(try store.load())
        // Clearing twice must stay idempotent.
        try store.restore(nil)
    }

    func testRestoreRollsBackToThePreviousToken() throws {
        try store.save("current")
        try store.restore("previous")
        XCTAssertEqual(try store.load(), "previous")
    }

    func testUnicodeTokensSurviveTheRoundTrip() throws {
        let token = "tøken-🔑-测试"
        try store.save(token)
        XCTAssertEqual(try store.load(), token)
    }

    func testApplicationTokenItemIsNeverModified() throws {
        // Reading via the default identifiers must not error out even when the
        // app never stored a token; the test only asserts isolation.
        let applicationStore = KeychainCredentialStore()
        let before = try? applicationStore.load()
        try store.save("test-only-token")
        let after = try? applicationStore.load()
        XCTAssertEqual(before, after, "integration tests must not touch the app token")
    }
}

import XCTest

final class MacNotificationPolicyTests: XCTestCase {
    func testCorePolicyMustAllowDelivery() {
        XCTAssertFalse(MacNotificationPolicy.shouldDeliver(
            coreAllowsNotification: false,
            authorization: .authorized
        ))
        XCTAssertTrue(MacNotificationPolicy.shouldDeliver(
            coreAllowsNotification: true,
            authorization: .authorized
        ))
    }

    func testOnlyGrantedAuthorizationStatesAllowDelivery() {
        XCTAssertTrue(MacNotificationPolicy.shouldDeliver(
            coreAllowsNotification: true,
            authorization: .provisional
        ))
        XCTAssertTrue(MacNotificationPolicy.shouldDeliver(
            coreAllowsNotification: true,
            authorization: .ephemeral
        ))

        for authorization in [
            MacNotificationAuthorization.notDetermined,
            .denied,
            .unavailable,
        ] {
            XCTAssertFalse(MacNotificationPolicy.shouldDeliver(
                coreAllowsNotification: true,
                authorization: authorization
            ))
        }
    }

    func testNotificationMessageIDParsing() {
        XCTAssertEqual(MacNotificationPolicy.messageID(from: ["messageID": Int64(42)]), 42)
        XCTAssertEqual(MacNotificationPolicy.messageID(from: ["messageID": NSNumber(value: 7)]), 7)
        XCTAssertNil(MacNotificationPolicy.messageID(from: [:]))
        XCTAssertNil(MacNotificationPolicy.messageID(from: ["messageID": "invalid"]))
    }
}

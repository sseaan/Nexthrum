import Foundation

/// One message as rendered by the SwiftUI client.
///
/// Kept in its own file so both the app and the unit-test target can compile it
/// without pulling in `AppModel` (which depends on AppKit and the FFI bridge).
struct MacMessage: Identifiable, Hashable {
    let id: Int64
    let appId: Int64
    let title: String
    let body: String
    let date: String
    let priority: Int64
    let read: Bool
}

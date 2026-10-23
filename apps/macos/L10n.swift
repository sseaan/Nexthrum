import Foundation

/// Thin wrapper over `String(localized:)` for strings that cannot use SwiftUI's
/// `LocalizedStringKey` (values stored in model state, or interpolated formats).
///
/// Keys live in `Resources/Localizable.xcstrings`; static SwiftUI literals keep
/// using the literal itself as the key.
enum L10n {
    static func t(_ key: String) -> String {
        String(localized: String.LocalizationValue(key))
    }

    static func t(_ key: String, _ arguments: CVarArg...) -> String {
        let format = String(localized: String.LocalizationValue(key))
        return String(format: format, locale: .current, arguments: arguments)
    }
}

import Foundation

/// Renders a Gotify timestamp for list rows.
///
/// Rules (see docs/design-spec.md):
/// - same day  -> `14:05`
/// - same year -> `10-22 14:05`
/// - older     -> `2024-10-22`
enum MessageDateFormatting {
    /// Parses the RFC3339 timestamps Gotify sends, including fractional seconds.
    static func parse(_ raw: String) -> Date? {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }

        let withFraction = ISO8601DateFormatter()
        withFraction.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        if let date = withFraction.date(from: trimmed) {
            return date
        }

        let plain = ISO8601DateFormatter()
        plain.formatOptions = [.withInternetDateTime]
        if let date = plain.date(from: trimmed) {
            return date
        }

        // Some servers omit the timezone designator.
        let fallback = DateFormatter()
        fallback.locale = Locale(identifier: "en_US_POSIX")
        fallback.timeZone = TimeZone(secondsFromGMT: 0)
        fallback.dateFormat = "yyyy-MM-dd'T'HH:mm:ss"
        return fallback.date(from: trimmed)
    }

    /// Compact label for a list row.
    static func listLabel(
        for raw: String,
        now: Date = Date(),
        calendar: Calendar = .current,
        locale: Locale = .current,
        timeZone: TimeZone = .current
    ) -> String {
        guard let date = parse(raw) else {
            // Fall back to the raw value without the time zone noise.
            return String(raw.prefix(16))
        }

        var calendar = calendar
        calendar.timeZone = timeZone

        if calendar.isDate(date, inSameDayAs: now) {
            return time(date, locale: locale, timeZone: timeZone)
        }

        let sameYear = calendar.component(.year, from: date) == calendar.component(.year, from: now)
        let formatter = DateFormatter()
        formatter.locale = locale
        formatter.timeZone = timeZone
        formatter.dateFormat = sameYear ? "MM-dd HH:mm" : "yyyy-MM-dd"
        return formatter.string(from: date)
    }

    /// Full timestamp for tooltips and accessibility labels.
    static func detailedLabel(
        for raw: String,
        locale: Locale = .current,
        timeZone: TimeZone = .current
    ) -> String {
        guard let date = parse(raw) else { return raw }
        let formatter = DateFormatter()
        formatter.locale = locale
        formatter.timeZone = timeZone
        formatter.dateStyle = .medium
        formatter.timeStyle = .medium
        return formatter.string(from: date)
    }

    private static func time(_ date: Date, locale: Locale, timeZone: TimeZone) -> String {
        let formatter = DateFormatter()
        formatter.locale = locale
        formatter.timeZone = timeZone
        formatter.dateFormat = "HH:mm"
        return formatter.string(from: date)
    }
}

import Foundation

/// One message source (Gotify application) shown in the filter bar.
struct MessageSource: Identifiable, Hashable {
    let appId: Int64
    let name: String
    let total: Int
    let unread: Int

    var id: Int64 { appId }

    /// Gotify streamed messages only carry `appid`; the host resolves real names
    /// from `/application` and falls back to `App <id>`.
    var label: String { name }
}

enum MessageSourceGrouping {
    /// Groups messages by source, most active first. Callers show the filter bar
    /// only when more than one source exists.
    static func sources(
        from messages: [MacMessage],
        name: (Int64) -> String = { "App \($0)" }
    ) -> [MessageSource] {
        var totals: [Int64: Int] = [:]
        var unread: [Int64: Int] = [:]
        for message in messages {
            totals[message.appId, default: 0] += 1
            if !message.read {
                unread[message.appId, default: 0] += 1
            }
        }
        return totals
            .map { appId, total in
                MessageSource(
                    appId: appId,
                    name: name(appId),
                    total: total,
                    unread: unread[appId] ?? 0
                )
            }
            .sorted { left, right in
                if left.unread != right.unread { return left.unread > right.unread }
                if left.total != right.total { return left.total > right.total }
                return left.appId < right.appId
            }
    }

    /// Applies the selected source filter; `nil` means "all sources".
    static func filter(_ messages: [MacMessage], selected: Int64?) -> [MacMessage] {
        guard let selected else { return messages }
        return messages.filter { $0.appId == selected }
    }
}

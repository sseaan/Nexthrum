//! AppModel: messages responsibilities.

import AppKit
import Foundation

extension AppModel {
    func openNotificationMessage(_ id: Int64) {
        pendingMessageID = id
        markRead(id)
        NSApp.activate(ignoringOtherApps: true)
    }

    func clearPendingMessageNavigation() {
        pendingMessageID = nil
    }

    /// Resolve source names once per connection. Failures keep the `App <id>`
    /// fallback so the filter bar still works on restricted servers.

    func markRead(_ id: Int64) {
        do {
            unreadCount = Int(try bridge.markMessageRead(messageId: id))
            if let index = messages.firstIndex(where: { $0.id == id }) {
                let current = messages[index]
                messages[index] = MacMessage(
                    id: current.id,
                    appId: current.appId,
                    title: current.title,
                    body: current.body,
                    date: current.date,
                    priority: current.priority,
                    read: true
                )
            }
        } catch { storageMessage = userMessage(for: error) }
    }

    func deleteMessage(_ id: Int64) {
        do {
            unreadCount = Int(try bridge.deleteMessage(messageId: id))
            messages.removeAll { $0.id == id }
        } catch { storageMessage = userMessage(for: error) }
    }

    func clearMessages() {
        do {
            unreadCount = Int(try bridge.clearMessages())
            messages.removeAll()
        } catch { storageMessage = userMessage(for: error) }
    }

    /// Tests the Gotify HTTP endpoint only. Deliberately does **not** persist
    /// settings, start the stream or touch the login item.
}

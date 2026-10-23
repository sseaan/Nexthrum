import AppKit
import SwiftUI

/// Compact menu bar popover: recent messages with quick actions.
///
/// Shown from `MenuBarExtra` with `.window` style so the user can read and act
/// on messages without opening the main window.
struct MenuBarPanel: View {
    @ObservedObject var model: AppModel
    @Environment(\.openWindow) private var openWindow

    private var recentMessages: [MacMessage] {
        Array(model.messages.prefix(3))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            header
            Divider()
            content
            Divider()
            footer
        }
        .frame(width: 320)
        // A notification click or a message action should surface the window.
        .onChange(of: model.pendingMessageID) { _, messageID in
            guard messageID != nil else { return }
            openWindow(id: "messages")
            NSApp.activate(ignoringOtherApps: true)
        }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack {
                Text("Nexthrum")
                    .font(.headline)
                Spacer()
                if model.unreadCount > 0 {
                    Text("\(model.unreadCount)")
                        .font(.caption.weight(.semibold))
                        .padding(.horizontal, 6)
                        .padding(.vertical, 1)
                        .background(Color.accentColor, in: Capsule())
                        .foregroundStyle(.white)
                        .accessibilityLabel("\(model.unreadCount) unread messages")
                }
            }
            Label(model.connectionLabel, systemImage: model.isConfigured ? "circle.dotted" : "wifi.slash")
                .font(.caption)
                .foregroundStyle(.secondary)
            if let error = model.connectionError {
                Text(error)
                    .font(.caption2)
                    .foregroundStyle(.red)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }

    @ViewBuilder
    private var content: some View {
        if recentMessages.isEmpty {
            Text(model.isConfigured
                 ? "No messages yet."
                 : "Configure a Gotify server in Settings.")
                .font(.callout)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(12)
        } else {
            ForEach(recentMessages) { message in
                MenuBarMessageRow(
                    message: message,
                    onOpen: {
                        open(mainWindowFor: message.id)
                    },
                    onMarkRead: { model.markRead(message.id) },
                    onDelete: { model.deleteMessage(message.id) }
                )
                if message.id != recentMessages.last?.id {
                    Divider().padding(.leading, 12)
                }
            }
        }
    }

    private var footer: some View {
        VStack(spacing: 0) {
            Button("Open Messages") {
                openWindow(id: "messages")
                NSApp.activate(ignoringOtherApps: true)
            }
            .buttonStyle(.plain)
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .frame(maxWidth: .infinity, alignment: .leading)

            Button("Settings…") {
                model.requestSettings()
                openWindow(id: "messages")
                NSApp.activate(ignoringOtherApps: true)
            }
            .buttonStyle(.plain)
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .frame(maxWidth: .infinity, alignment: .leading)

            Divider()

            Button("Quit Nexthrum") { NSApp.terminate(nil) }
                .buttonStyle(.plain)
                .padding(.horizontal, 12)
                .padding(.vertical, 6)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.vertical, 4)
    }

    private func open(mainWindowFor messageID: Int64) {
        model.markRead(messageID)
        openWindow(id: "messages")
        NSApp.activate(ignoringOtherApps: true)
        model.pendingMessageID = messageID
    }
}

private struct MenuBarMessageRow: View {
    let message: MacMessage
    let onOpen: () -> Void
    let onMarkRead: () -> Void
    let onDelete: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            HStack(spacing: 6) {
                if !message.read {
                    Circle()
                        .fill(Color.accentColor)
                        .frame(width: 6, height: 6)
                        .accessibilityHidden(true)
                }
                Text(message.title)
                    .font(.callout.weight(message.read ? .regular : .semibold))
                    .lineLimit(1)
                Spacer()
                Text(MessageDateFormatting.listLabel(for: message.date))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            Text(message.body)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(2)
        }
        .contentShape(Rectangle())
        .onTapGesture(perform: onOpen)
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .accessibilityElement(children: .combine)
        .accessibilityHint("Opens the message in the main window")
        .contextMenu {
            Button("Open in Window", action: onOpen)
            if !message.read {
                Button("Mark as Read", action: onMarkRead)
            }
            Divider()
            Button("Delete", role: .destructive, action: onDelete)
        }
    }
}

import AppKit
import SwiftUI

/// Sidebar destinations. Stored as real selection state so the detail pane
/// actually switches (a constant selection made the Settings entry inert).
enum SidebarItem: Hashable {
    case messages
    case settings
}

struct ContentView: View {
    @EnvironmentObject private var model: AppModel
    @State private var selection: SidebarItem = .messages
    /// The first-run guide can be dismissed to browse (empty) messages.
    @State private var onboardingDismissed = false
    /// `nil` means "all sources"; the filter bar only appears with 2+ sources.
    @State private var selectedSourceId: Int64?

    var body: some View {
        NavigationSplitView {
            List(selection: $selection) {
                Label("Messages", systemImage: "tray.full")
                    .tag(SidebarItem.messages)
                Label("Settings", systemImage: "gearshape")
                    .tag(SidebarItem.settings)
            }
            .navigationTitle("Nexthrum")
            .listStyle(.sidebar)
        } detail: {
            switch selection {
            case .messages:
                messagesPane
            case .settings:
                // The standalone Settings scene stays available from the menu bar;
                // embedding the view here keeps in-window navigation working too.
                SettingsView()
                    .navigationTitle("Settings")
            }
        }
    }

    // MARK: - Messages

    private var messagesPane: some View {
        Group {
            if !model.hasStoredSettings && !onboardingDismissed {
                OnboardingView(onSkip: { onboardingDismissed = true })
            } else {
                configuredMessagesPane
            }
        }
    }

    private var configuredMessagesPane: some View {
        VStack(spacing: 0) {
            headerBar
            Divider()
            if let problem = model.storageMessage {
                banner(problem)
            }
            if model.messages.isEmpty {
                emptyState
            } else {
                if sources.count > 1 {
                    MessageFilterBar(
                        sources: sources,
                        totalCount: model.messages.count,
                        totalUnread: model.unreadCount,
                        selectedAppId: $selectedSourceId
                    )
                    Divider()
                }
                cardList
            }
        }
        .background(Color(nsColor: .windowBackgroundColor))
        .onChange(of: model.settingsRequestToken) { _, _ in
            selection = .settings
        }
        .confirmationDialog(
            "Clear all local messages?",
            isPresented: $model.showsClearConfirmation,
            titleVisibility: .visible
        ) {
            Button("Clear Local Messages", role: .destructive) { model.clearMessages() }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("This only clears messages stored on this Mac. It does not delete messages from the Gotify server.")
        }
    }

    private var headerBar: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 10) {
                Text("Messages")
                    .font(.title2.weight(.semibold))

                if model.unreadCount > 0 {
                    Text(L10n.t("unread_badge", model.unreadCount))
                        .font(.caption.weight(.semibold))
                        .padding(.horizontal, 8)
                        .padding(.vertical, 2)
                        .background(Color.accentColor.opacity(0.16), in: Capsule())
                        .foregroundStyle(Color.accentColor)
                        .accessibilityLabel(L10n.t("unread_a11y", model.unreadCount))
                }

                Spacer()

                if model.isConfigured {
                    Button {
                        model.reconnect()
                    } label: {
                        Label("Reconnect", systemImage: "arrow.clockwise")
                    }
                    .labelStyle(.iconOnly)
                    .help("Reconnect")
                    .disabled(model.connectionStatus == .connecting)
                }
                Button {
                    model.requestSettings()
                } label: {
                    Label("Settings", systemImage: "gearshape")
                }
                .labelStyle(.iconOnly)
                .help("Settings…")
            }

            HStack(spacing: 6) {
                Circle()
                    .fill(connectionColor)
                    .frame(width: 7, height: 7)
                    .accessibilityHidden(true)
                Text(model.connectionLabel)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                Text(L10n.t("cached_count", model.messages.count))
                    .font(.subheadline)
                    .foregroundStyle(.tertiary)
                Spacer()
                if !model.messages.isEmpty {
                    Button("Clear All…") { model.showsClearConfirmation = true }
                        .buttonStyle(.link)
                        .font(.subheadline)
                }
            }

            if let error = model.connectionError {
                Text(error)
                    .font(.footnote)
                    .foregroundStyle(.red)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .background(.bar)
    }

    private var connectionColor: Color {
        switch model.connectionStatus {
        case .connected:
            return .green
        case .connecting:
            return .orange
        default:
            return .secondary
        }
    }

    private func banner(_ text: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: "exclamationmark.triangle.fill")
            Text(text)
        }
        .font(.footnote)
        .foregroundStyle(.orange)
        .padding(.horizontal, 16)
        .padding(.vertical, 6)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.orange.opacity(0.08))
    }

    private var emptyState: some View {
        ContentUnavailableView {
            Label("No Messages Yet", systemImage: "tray")
        } description: {
            Text(model.isConfigured
                 ? "Messages will appear here when the native Gotify connection is enabled."
                 : "Configure a Gotify server in Settings to get started.")
        } actions: {
            if !model.isConfigured {
                Button("Open Settings…") {
                    selection = .settings
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var sources: [MessageSource] {
        MessageSourceGrouping.sources(from: model.messages) { model.sourceName(for: $0) }
    }

    private var visibleMessages: [MacMessage] {
        MessageSourceGrouping.filter(model.messages, selected: selectedSourceId)
    }

    private var cardList: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 8) {
                    ForEach(visibleMessages) { message in
                        MessageCard(
                            message: message,
                            sourceName: model.sourceName(for: message.appId)
                        )
                            .contextMenu {
                                if !message.read {
                                    Button("Mark as Read") { model.markRead(message.id) }
                                }
                                Button("Delete", role: .destructive) {
                                    model.deleteMessage(message.id)
                                }
                            }
                            .onTapGesture { model.markRead(message.id) }
                            .id(message.id)
                    }
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 12)
            }
            .onChange(of: sources.map(\.appId)) { _, appIds in
                if let selectedSourceId, !appIds.contains(selectedSourceId) {
                    self.selectedSourceId = nil
                }
            }
            .onChange(of: model.pendingMessageID) { _, messageID in
                guard let messageID else { return }
                withAnimation { proxy.scrollTo(messageID, anchor: .center) }
                model.clearPendingMessageNavigation()
            }
        }
    }
}

import SwiftUI

/// Source filter chips shown above the message cards when more than one Gotify
/// application appears in the list.
struct MessageFilterBar: View {
    let sources: [MessageSource]
    let totalCount: Int
    let totalUnread: Int
    @Binding var selectedAppId: Int64?

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 6) {
                chip(
                    title: "All",
                    count: totalCount,
                    unread: totalUnread,
                    isSelected: selectedAppId == nil
                ) {
                    selectedAppId = nil
                }

                ForEach(sources) { source in
                    chip(
                        title: source.label,
                        count: source.total,
                        unread: source.unread,
                        isSelected: selectedAppId == source.appId
                    ) {
                        selectedAppId = source.appId
                    }
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)
        }
        .background(.bar)
        .accessibilityLabel("Filter messages by source")
    }

    private func chip(
        title: String,
        count: Int,
        unread: Int,
        isSelected: Bool,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack(spacing: 5) {
                Text(title)
                    .font(.caption.weight(isSelected ? .semibold : .regular))
                Text("\(count)")
                    .font(.caption2.monospacedDigit())
                    .opacity(0.75)
                if unread > 0 {
                    Text("\(unread)")
                        .font(.caption2.weight(.bold).monospacedDigit())
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(
                            Capsule().fill(isSelected ? Color.white.opacity(0.25) : Color.accentColor.opacity(0.18))
                        )
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .background(
                Capsule().fill(isSelected ? Color.accentColor : Color.primary.opacity(0.06))
            )
            .foregroundStyle(isSelected ? Color.white : Color.primary)
            .overlay(
                Capsule().strokeBorder(
                    isSelected ? Color.clear : Color.primary.opacity(0.08),
                    lineWidth: 1
                )
            )
        }
        .buttonStyle(.plain)
        .accessibilityLabel(
            unread > 0
                ? L10n.t("filter_chip_a11y", title, count, unread)
                : L10n.t("filter_chip_a11y_plain", title, count)
        )
        .accessibilityAddTraits(isSelected ? [.isSelected] : [])
    }
}

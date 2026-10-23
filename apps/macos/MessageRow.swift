import SwiftUI

/// A single message rendered as a card.
///
/// Unread state uses three redundant cues (accent bar, dot, font weight) so it
/// survives grayscale, and the card gets a tinted surface plus elevation on hover.
struct MessageCard: View {
    let message: MacMessage
    /// Resolved Gotify application name for the source badge.
    var sourceName: String = ""
    var now: Date = Date()

    @State private var isHovering = false

    private var tint: Color { Color.accentColor }
    private var isUnread: Bool { !message.read }

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            RoundedRectangle(cornerRadius: 2)
                .fill(isUnread ? tint : Color.clear)
                .frame(width: 3)
                .accessibilityHidden(true)

            VStack(alignment: .leading, spacing: 6) {
                header
                Text(message.body)
                    .font(.callout)
                    .foregroundStyle(isUnread ? .primary : .secondary)
                    .lineLimit(3)
                    .fixedSize(horizontal: false, vertical: true)
                if !badges.isEmpty {
                    badgeRow
                }
            }
        }
        .padding(.vertical, 10)
        .padding(.horizontal, 12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(cardBackground)
        .overlay(
            RoundedRectangle(cornerRadius: 10)
                .strokeBorder(isUnread ? tint.opacity(0.35) : Color.primary.opacity(0.08), lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .shadow(
            color: .black.opacity(isHovering ? 0.18 : 0.06),
            radius: isHovering ? 8 : 2,
            y: isHovering ? 3 : 1
        )
        .animation(.easeOut(duration: 0.12), value: isHovering)
        .onHover { isHovering = $0 }
        .contentShape(RoundedRectangle(cornerRadius: 10))
        .accessibilityElement(children: .combine)
        .accessibilityLabel(
            "\(isUnread ? "Unread" : "Read") message, \(title), \(MessageDateFormatting.detailedLabel(for: message.date))"
        )
    }

    private var title: String {
        message.title.isEmpty ? "No Title" : message.title
    }

    private var header: some View {
        HStack(alignment: .firstTextBaseline, spacing: 6) {
            if isUnread {
                Circle()
                    .fill(tint)
                    .frame(width: 6, height: 6)
                    .accessibilityHidden(true)
            }
            Text(title)
                .font(.headline)
                .fontWeight(isUnread ? .bold : .regular)
                .foregroundStyle(isUnread ? .primary : .secondary)
                .lineLimit(1)
            Spacer(minLength: 8)
            Text(MessageDateFormatting.listLabel(for: message.date, now: now))
                .font(.caption)
                .monospacedDigit()
                .foregroundStyle(.tertiary)
                .help(MessageDateFormatting.detailedLabel(for: message.date))
        }
    }

    private var badges: [String] {
        var badges = [sourceName.isEmpty ? "App \(message.appId)" : sourceName]
        if message.priority > 0 {
            badges.append("P\(message.priority)")
        }
        return badges
    }

    private var badgeRow: some View {
        HStack(spacing: 6) {
            ForEach(badges, id: \.self) { badge in
                Text(badge)
                    .font(.caption2.weight(.medium))
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(
                        Capsule().fill(isUnread ? tint.opacity(0.14) : Color.primary.opacity(0.06))
                    )
                    .foregroundStyle(isUnread ? tint : .secondary)
            }
        }
    }

    private var cardBackground: some View {
        RoundedRectangle(cornerRadius: 10)
            .fill(
                isUnread
                    ? AnyShapeStyle(tint.opacity(isHovering ? 0.12 : 0.08))
                    : AnyShapeStyle(Color.primary.opacity(isHovering ? 0.07 : 0.035))
            )
    }
}

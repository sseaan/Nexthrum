import Foundation

extension AppModel {
    /// Debounced search: cancels the previous request so fast typing cannot
    /// resolve out of order.
    func updateSearch(_ query: String) {
        searchQuery = query
        searchTask?.cancel()

        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            searchResults = nil
            return
        }

        searchTask = Task { @MainActor [weak self] in
            try? await Task.sleep(nanoseconds: 250_000_000)
            guard !Task.isCancelled, let self else { return }
            do {
                let snapshot = try await Task.detached { [bridge] in
                    try bridge.searchMessages(query: trimmed)
                }.value
                guard !Task.isCancelled, self.searchQuery == query else { return }
                self.searchResults = snapshot.messages.map { message in
                    MacMessage(
                        id: message.id,
                        appId: message.appId,
                        title: message.title,
                        body: message.body,
                        date: message.date,
                        priority: message.priority,
                        read: message.read
                    )
                }
            } catch {
                logger.error("Search failed: \(userMessage(for: error), privacy: .public)")
                self.searchResults = []
            }
        }
    }

    func clearSearch() {
        searchTask?.cancel()
        searchQuery = ""
        searchResults = nil
    }
}

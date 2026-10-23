import SwiftUI

/// First-run guide. Shown instead of the message list until a server and token
/// are configured, so a fresh install explains what to enter and where to find it.
struct OnboardingView: View {
    @EnvironmentObject private var model: AppModel
    @Environment(\.openSettings) private var openSettings
    var onSkip: () -> Void = {}

    @State private var showTokenField = false

    private var canSubmit: Bool {
        !model.serverURL.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            && !model.clientToken.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                header
                stepList
                form
                actions
            }
            .padding(24)
            .frame(maxWidth: 560, alignment: .leading)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 6) {
            Image(systemName: "tray.full.fill")
                .font(.system(size: 34))
                .foregroundStyle(Color.accentColor)
                .accessibilityHidden(true)
            Text("Welcome to Nexthrum")
                .font(.title2.weight(.semibold))
            Text("Connect your Gotify server to receive messages here. It only takes a minute.")
                .font(.callout)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
        }
    }

    private var stepList: some View {
        VStack(alignment: .leading, spacing: 10) {
            step(
                1,
                title: "Server URL",
                detail: "Use the address you open in a browser, for example https://gotify.example.com"
            )
            step(
                2,
                title: "Client token",
                detail: "In the Gotify web UI open Clients, create or copy a client token, then paste it below."
            )
            step(
                3,
                title: "Test, then save",
                detail: "Test the connection first; saving stores the token in the macOS Keychain."
            )
        }
        .padding(14)
        .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 10))
    }

    private func step(_ index: Int, title: String, detail: String) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Text("\(index)")
                .font(.caption.weight(.bold))
                .frame(width: 18, height: 18)
                .background(Color.accentColor.opacity(0.16), in: Circle())
                .foregroundStyle(Color.accentColor)
                .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.subheadline.weight(.semibold))
                Text(detail)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .accessibilityElement(children: .combine)
    }

    private var form: some View {
        VStack(alignment: .leading, spacing: 10) {
            TextField("Server URL", text: $model.serverURL, prompt: Text("https://gotify.example.com"))
                .textContentType(.URL)
                .textFieldStyle(.roundedBorder)

            if showTokenField {
                SecureField("Client Token", text: $model.clientToken, prompt: Text("Paste your client token"))
                    .textFieldStyle(.roundedBorder)
            } else {
                Button("Paste client token…") { showTokenField = true }
                    .buttonStyle(.link)
                    .font(.callout)
            }

            if let message = model.connectionTestMessage {
                Label(message, systemImage: model.connectionTestSucceeded ? "checkmark.circle.fill" : "xmark.circle.fill")
                    .font(.footnote)
                    .foregroundStyle(model.connectionTestSucceeded ? .green : .red)
                    .fixedSize(horizontal: false, vertical: true)
            }
            if let error = model.validationMessage {
                Text(error)
                    .font(.footnote)
                    .foregroundStyle(.red)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
    }

    private var actions: some View {
        HStack(spacing: 10) {
            Button(model.isTestingConnection ? "Testing…" : "Test Connection") {
                if !showTokenField { showTokenField = true }
                model.testConnection()
            }
            .disabled(!canSubmit || model.isTestingConnection)

            Button("Save and Connect") {
                model.validateAndSave()
            }
            .keyboardShortcut(.defaultAction)
            .disabled(!canSubmit)

            Spacer()

            Button("Skip for now") { onSkip() }
                .buttonStyle(.link)
                .font(.callout)

            Button("Advanced…") { openSettings() }
                .buttonStyle(.link)
                .font(.callout)
        }
    }
}

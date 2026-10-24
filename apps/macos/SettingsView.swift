import AppKit
import SwiftUI

struct SettingsView: View {
    @EnvironmentObject private var model: AppModel
    @EnvironmentObject private var preferences: AppPreferences

    var body: some View {
        Form {
            Section("Server Connection") {
                TextField("Server URL", text: $model.serverURL)
                    .textContentType(.URL)
                SecureField("Client Token", text: $model.clientToken)
                Button(model.isTestingConnection ? "Testing…" : "Test Connection") {
                    model.testConnection()
                }
                .disabled(!model.isConfigured || model.isTestingConnection)

                if let connectionTestMessage = model.connectionTestMessage {
                    Text(connectionTestMessage)
                        .foregroundStyle(model.connectionTestSucceeded ? .green : .red)
                }

            }

            Section("Message History") {
                Stepper(L10n.t("keep_messages_days", model.historyRetentionDays), value: $model.historyRetentionDays, in: 1...365)
            }

            Section("Do Not Disturb") {
                TextField("Start (HH:MM)", text: $model.doNotDisturbStart)
                    .monospacedDigit()
                TextField("End (HH:MM)", text: $model.doNotDisturbEnd)
                    .monospacedDigit()
                Text("Messages are still received and saved during these hours.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            Section("Notifications") {
                LabeledContent("Permission", value: model.notificationPermissionStatus)
                if model.notificationPermissionDenied {
                    Button("Open Notification Settings") {
                        if let url = URL(string: "x-apple.systempreferences:com.apple.Notifications-Settings.extension") {
                            NSWorkspace.shared.open(url)
                        }
                    }
                } else if model.notificationPermissionStatus == "Not requested" {
                    Button("Enable Notifications") {
                        Task { await model.notificationService.requestAuthorization() }
                    }
                }
                Text("Only messages allowed by your Do Not Disturb settings generate notifications.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            Section("Appearance") {
                Toggle("Show icon in Dock", isOn: Binding(
                    get: { !preferences.hideDockIcon },
                    set: { preferences.setHideDockIcon(!$0) }
                ))
                Text("When the Dock icon is hidden, Nexthrum keeps running in the menu bar.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)

                Picker("Language", selection: Binding(
                    get: { preferences.language },
                    set: { preferences.select(language: $0) }
                )) {
                    ForEach(AppLanguage.allCases) { language in
                        Text(language.displayName).tag(language)
                    }
                }
                if preferences.needsRelaunch {
                    VStack(alignment: .leading, spacing: 6) {
                        Text("Language changes take effect after Nexthrum restarts.")
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                        Button("Restart Now") { AppPreferences.relaunch() }
                    }
                }
            }

            Section("Startup") {
                Toggle("Launch Nexthrum at login", isOn: $model.autoStart)
                LabeledContent("Login item status", value: model.launchAtLoginService.statusDescription)
                if let error = model.launchAtLoginMessage {
                    Text(error).foregroundStyle(.red)
                }
                Text("macOS may require approval in System Settings.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            Section("Updates") {
                Toggle("Automatically check for updates", isOn: Binding(
                    get: { model.automaticallyChecksForUpdates },
                    set: { model.setAutomaticallyChecksForUpdates($0) }
                ))

                HStack {
                    Button(model.canCheckForUpdates ? "Check for Updates…" : "Checking…") {
                        model.checkForUpdates()
                    }
                    .disabled(!model.canCheckForUpdates)

                    Spacer()

                    if let lastCheck = model.lastUpdateCheckDate {
                        Text(lastCheck, format: .dateTime.year().month().day().hour().minute())
                            .font(.footnote)
                            .foregroundStyle(.secondary)
                    }
                }

                Text("Updates are delivered through GitHub Releases and verified with Sparkle's EdDSA signature.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            Section("Status") {
                Button("Save Settings") { model.validateAndSave() }
                    .disabled(!model.isConfigured)
                if let validationMessage = model.validationMessage {
                    Text(validationMessage).foregroundStyle(.red)
                }
                LabeledContent("Connection", value: model.connectionLabel)
                if let connectionError = model.connectionError {
                    Text(connectionError)
                        .font(.footnote)
                        .foregroundStyle(.red)
                        .fixedSize(horizontal: false, vertical: true)
                }
                HStack {
                    Button("Reconnect") { model.reconnect() }
                        .disabled(!model.isConfigured)
                    Button("Disconnect") {
                        model.stopStreaming()
                    }
                    .disabled(model.connectionStatus == .disconnected)
                }
                Text("Live connection events and messages are forwarded from the shared Rust client.")
                    .font(.footnote)
                    .foregroundStyle(.secondary)

                DisclosureGroup("Diagnostics") {
                    LabeledContent("Polling", value: model.pollingActive ? "Active" : "Stopped")
                    LabeledContent("Events received", value: "\(model.eventsReceived)")
                    LabeledContent(
                        "Since last event",
                        value: model.secondsSinceLastEvent.map { String(format: "%.1f s", $0) } ?? "—"
                    )
                    Text("If the state stays on Connecting, check Console for subsystem com.nexthrum.app, category connection.")
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .formStyle(.grouped)
        .padding()
        .frame(minWidth: 460, minHeight: 300)
        // The Keychain prompt appears here, on demand, instead of at launch.
        .task { model.ensureCredentialLoaded() }
    }
}

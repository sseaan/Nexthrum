# Nexthrum 架构设计

## 1. 架构目标与现状

目标是在 Windows 保持 Tauri/React 产品的同时，新增 SwiftUI 原生 macOS 客户端，并尽可能共享消息业务与 Gotify 实现。

目标分层已落地：两个平台前端都通过 Rust Core 处理消息业务，`apps/windows/src-tauri` 与 `apps/macos` 只是 host adapter。

- Windows：`apps/windows/src-tauri/src/lib.rs` 负责启动与状态，`commands.rs` 暴露 Tauri command，`websocket.rs` 仅做事件/通知转发，`credentials.rs` 管理 Windows 凭据；React 通过 invoke/events 通信。
- macOS：SwiftUI 经 UniFFI `BridgeService` 使用共享 Gotify client 与 SQLite repository，已具备设置、连接、消息管理、通知、菜单栏与启动项；凭据在 Keychain。
- 共享：`nexthrum-core`（用例/策略）、`nexthrum-gotify`（HTTP/WebSocket）、`nexthrum-storage`（SQLite/migration）、`nexthrum-ffi`（bridge）。

尚未完成的是发布工程，而非业务分层：Windows/macOS 正式签名与公证、macOS 实机验收。详见 `docs/release.md`。

## 2. 目标架构图

```text
Windows                                      macOS
React + Zustand                              SwiftUI
       │ Tauri invoke/events                    │ Swift bindings + AsyncStream
Tauri platform adapter                      macOS platform adapter
       └──────────────────┬────────────────────┘
                          Rust Nexthrum Core
                   ┌────────┼──────────┐
              Use cases  Gotify client  Repository
                           │            │
                       HTTP/WS       SQLite + migrations

平台 adapter：窗口、托盘/菜单栏、通知、启动项、数据目录、凭据安全存储
```

## 3. 模块职责

### Rust Core
- 领域类型：设置、Gotify 消息、本地消息、连接状态、错误和事件 DTO。
- 用例：消息接收与去重、列表/未读查询、标读、删除、清空均通过 `MessageRepository` port；变更用例统一返回操作后的未读快照。Tauri commands 与 UniFFI bridge 仅负责参数校验、调用用例及呈现错误/事件。
- 网络客户端：HTTP 连接测试、WebSocket 生命周期、消息解码、退避重连；不发系统通知或 Tauri event。
- Repository：SQLite 读写与 schema 迁移；应用数据目录由宿主注入。

### Rust Core 与适配器
- `gotify.rs` 当前封装 Gotify HTTP 测试连接、WebSocket URL 构造、payload 解码及无 Tauri 依赖的连接/读取/重连 client；通过类型化 `GotifyEvent` channel 输出消息和连接状态。
- 以上 Gotify client 已迁移至 `crates/nexthrum-gotify`，依赖 `nexthrum-core`；`apps/windows/src-tauri/src/websocket.rs` 仅是 Windows/Tauri adapter，负责 client 生命周期、事件转发、消息用例调用和系统通知。
- **rustls crypto provider**: `tokio-tungstenite` only enables rustls' `std` feature, so a host that never installs a provider panics on the first TLS handshake ("Could not automatically determine the process-level CryptoProvider"). `nexthrum-gotify::ensure_crypto_provider()` (ring, idempotent) is called before any HTTP/TLS use, so Windows, macOS and tests no longer depend on host setup. FFI 的 worker 另用 `catch_unwind` 把 panic 转成 `Disconnected` 事件，避免"客户端静默死亡导致界面永久 Connecting"。
- Gotify WebSocket handshakes are bounded by `WEBSOCKET_CONNECT_TIMEOUT` (10s) and every `Disconnected` event now carries a classified reason (HTTP status, TLS, DNS, timeout, server close). The reason flows through `BridgeEvent.error` to both UIs, so "connecting forever" is impossible and users see *why* the stream failed; macOS also logs transitions through `os_log` (`subsystem com.nexthrum.app`, category `connection`).
- Windows host owns the tray icon handle (`app.manage`) so the icon is not dropped, and reconciles the OS launch-at-login entry with the persisted `auto_start` preference on startup and on save; failures are reported as "settings saved, launch at login failed" instead of being swallowed.
- Windows UI applies backend snapshots through `useMessageSnapshot`: if a live event bumped `backendRevision` while a mutation was in flight, the snapshot is re-fetched (bounded) instead of overwriting newer state.
- Windows connection configuration updates are serialized by a host mutex. Settings persist first; the command then reconnects/ disconnects deterministically and reports a distinct “settings saved, connection restart failed” error. Explicit connect commands use the same serialization boundary.
- Gotify client 每轮建连前都发出 `Connecting`，握手成功发出 `Connected`，断开或连接失败发出 `Disconnected`；Tauri 将状态转发为 `connection-state-changed`，UniFFI 映射至稳定连接状态并附带权威未读快照。
- gotify 适配器继续提供 HTTP 测试连接与 WebSocket 重连；Tauri 命令不再直接构造 Gotify API 请求。

- `nexthrum-ffi` 与 Tauri 入站 adapter 共用 Core `receive_message` use case 和 SQLite repository；持久化/清理失败均转成 bridge 错误事件，不再在 FFI 层重复编排消息策略。
- Gotify 网络错误在 `nexthrum-gotify::GotifyError` 中分类（`InvalidUrl` / `Unauthorized` / `NotFound` / `HttpStatus` / `InvalidResponse` / `Network` / `Timeout` / `Tls`），并映射为 UniFFI 的扁平 `BridgeError`（`Unauthorized`、`ServerNotFound`、`ServerRejected`、`NetworkUnavailable`、`RequestTimeout`、`TlsFailure`、`InvalidResponse`）。Windows 侧 command 返回带分类的可读文本。
- UniFFI 生成的 `localizedDescription` 只暴露枚举名，因此 macOS 用 `BridgeErrorMessage.swift` 的 `userMessage(for:)` 映射为可操作文案（如令牌被拒提示检查 token、TLS 失败提示证书），并由 XCTest 保证每个枚举都有独立文案且不泄露原始 case 名。
### macOS adapter
- SwiftUI 消息列表已接入共享 storage bridge：显示未读数，支持标记已读、单条本地删除及带二次确认的本地清空；清空提示明确说明不删除服务器消息。
- `apps/macos/NexthrumMac.xcodeproj` 已建立共享 scheme；build phase 生成 Swift bindings 并构建 Rust dylib，Xcode target 编译 SwiftUI、嵌入 dylib，`scripts/build-macos-xcode.sh` 的 `xcodebuild build` 已通过并运行验证。目前仅 arm64，正式签名、公证、Universal binary 尚待完成。
- macOS `MacNotificationService` 使用 `UNUserNotificationCenter` 管理授权状态和发送通知。`MacNotificationPolicy` 独立于系统 API，强制检查 Core `should_notify` 和授权状态，并解析点击通知中的 message ID；点击后激活应用、定位消息并标记已读。Xcode unit tests 覆盖策略门控、授权分类与 ID 解析。
- macOS 凭据读取失败会作为设置加载错误呈现，不再静默当成“未配置”。`CredentialSettingsTransaction` 将 Keychain 写入与 Rust 设置保存串成事务；设置保存失败时回滚旧 token，回滚失败时错误明确包含两阶段失败信息，表单恢复旧设置并尝试恢复旧连接。
- Xcode 单元测试用注入的 in-memory closures 覆盖事务成功与设置保存失败时的旧凭据恢复；不读取或修改真实 Keychain。
- ### Windows 凭据与设置

- Windows host 新增 `credentials` 模块：client token 写入 Windows Credential Manager（target `Nexthrum/GotifyClientToken`），`settings.json` 只保存非敏感字段。
- 启动时读取设置会把旧版本遗留的明文 token 幂等迁移到凭据库并回写不含 token 的 JSON；迁移失败时保留明文设置继续运行，下次启动重试。
- 保存设置先写凭据再写 JSON；JSON 原子替换失败时回滚为旧凭据，避免两个存储不一致。清空 token 会删除凭据项。
- 凭据读取失败时保留服务器 URL 等非敏感设置并记录脱敏警告，用户可重新输入 token，不会因凭据故障丢弃配置。

macOS 保留 `KeychainCredentialStore` 与 `CredentialSettingsTransaction` 的同等语义。
- macOS `BridgeSettings` 持久化 auto-start intent；Settings 使用 `SMAppService.mainApp` 同步注册状态并区分 Enabled、Approval required、Disabled/Unavailable。启动项操作失败单独反馈，不回滚已保存的服务器配置或凭据。
- macOS `NexthrumMacApp` 提供 `MenuBarExtra` 入口：打开消息窗口、打开设置、明确退出；Dock 主窗口关闭后应用及 Gotify 后台连接继续运行。
- 通知点击通过共享 AppModel 状态重新打开并激活消息窗口，然后滚动至目标消息；标准 About 面板由 AppKit 提供。
- UI 仅从携带权威未读快照的事件刷新计数（初始 connecting、消息或错误）；connected/disconnected 状态事件无计数，不得将 badge 清零。
- Swift bridge 只使用稳定 DTO/用例接口，不接触内部 Rust 模型或 SQLite。

### UI
- 两端分别遵循本平台交互规范；不要求共享 UI 代码。
- Tauri `new-message` 只更新消息列表，不在前端自行递增未读数；未读数只接受后端携带的权威快照，避免事件到达顺序导致重复计数。标读、删除、清空操作完成后，Windows UI 通过原子消息快照同步消息列表和未读数。

## 4. 业务事件与操作

Gotify 的流式消息只带 `appid`，不含应用名，因此两个平台都会额外调用 `GET /application`（`X-Gotify-Key` 客户端令牌）解析来源名称，并在失败（403/404/离线）时回退为 `App <id>`：Windows `get_applications` command + Zustand `appNames`，macOS `BridgeService.listApplications()` + `AppModel.applicationNames`。

消息列表 DTO（`MessageListItem`）携带 `priority` 与 `received_at`，由 SQLite 查询直接读取，UniFFI `BridgeMessage` 与 TS `MessageListItem` 均不再使用占位默认值。macOS FFI 保留 `get_message_snapshot` 及消息操作接口，另有 `get_messages`、`unread_count` 查询供兼容调用；Windows Tauri 仅暴露原子 `get_message_snapshot` 查询，不再注册已无前端调用的 `get_messages` 和 `get_unread_count` 旧命令。所有状态变更更新最新未读状态；本地清空与服务器端删除严格分离。

Tauri 的 `mark_message_read`、`delete_message` 和 `clear_messages` 命令直接返回操作后读取的 `MessageSnapshot`（消息列表与未读数），消息变更命令由 host mutex 串行化，UI 不再另发快照查询；同时继续发出未读数事件，供全局 badge 同步。SQLite snapshot 自身在单一 read transaction 中读取列表和未读数。

## 5. 持久化与迁移

消息以 Gotify ID 幂等保存：首次落库触发通知意图；重连后的重复投递可更新服务器字段，但不得覆盖本地已读状态和首次收到时间，也不重复触发新消息通知。SQLite repository 在事务中判断首次插入，Core use case 通过 repository port 取得结果，两平台共用相同规则。

SQLite 以 migration 管理 schema；repository 提供明确 CRUD 与事务边界。`nexthrum-storage` 接收宿主注入的显式数据库/设置路径；`AppSettings` 对缺省 JSON 字段应用默认值。当前 SQLite `PRAGMA user_version` 迁移 v1 可兼容未版本化的既有 messages 表，并对早期 schema 做列对齐（补齐 `priority`/`extras`/`read`/`received_at`，以服务器日期回填），避免旧行在新版本下写入失败或被保留策略误删；测试覆盖未来版本拒绝、极旧 schema 升级、migration 幂等、损坏数据库、旧数据保留、CRUD、保留清理和设置兼容。Windows 路径保持 `%APPDATA%/Nexthrum` 下的既有文件名；macOS 使用 Application Support/Nexthrum。设置文件采用同目录临时文件后替换，BridgeService 仅在持久化成功后更新内存设置。

## 6. 平台行为

| 功能 | Windows | macOS |
|---|---|---|
| UI | Tauri + React | SwiftUI |
| 常驻入口 | 系统托盘 | MenuBarExtra/状态栏菜单 |
| 通知 | Windows 原生通知 | UserNotifications（先申请权限） |
| 开机启动 | Windows/Tauri 平台机制 | ServiceManagement，受系统版本约束 |
| 凭据 | Windows Credential Manager | Keychain |
| 数据路径 | AppData 策略 | Application Support |
| 关闭行为 | 可最小化至托盘的设置 | 遵循 macOS 窗口与菜单栏惯例 |

## 7. 关键质量属性

- Core 可脱离桌面框架运行测试。
- 所有 token 日志脱敏，网络错误可诊断且不吞掉服务器失败。
- 重连状态可观测；连接配置变更有确定的取消/重启语义。
- 数据库迁移和用户数据兼容可测试。
- 统一行为契约、平台独立 UX；签名、公证与权限在各平台 CI/发布中单独验证。

## 8. 迁移阶段

1. 风险收敛：凭据脱敏、连接状态、配置事务和错误反馈。
2. 业务解耦：抽出 use cases、Gotify client、repository，保留现有 Tauri API。
3. 数据治理：迁移框架、数据路径审计与兼容、测试基线。
4. Swift bridge：验证 UniFFI DTO 与异步事件，再稳定接口。
5. macOS vertical slice：SwiftUI 配置、连接、持久化、消息、通知、菜单栏。
6. 完整平台行为、双平台 CI、签名发布与迁移验证。

具体任务与验收见 `docs/refactor-plan.md`。

# Nexthrum 技术规范

## 1. 技术决策

### Windows
- Tauri 2、React 18、TypeScript、Zustand、Tailwind CSS v4、Rust。
- Tauri 命令层已是薄适配器：参数校验 + 调用 Rust Core 用例 + 事件/错误呈现。

### macOS
- 原生 UI：SwiftUI；自动更新使用内嵌的 Sparkle 2（EdDSA 签名 appcast）。
- 最低 macOS 14.0，仅 Apple Silicon（arm64）；`<key>CFBundleVersion</key>` 使用单调递增构建号。
- 共享业务：Rust Core；Swift 通过生成的 UniFFI bindings 调用（已验证 DTO、错误映射、异步事件轮询与取消）。
- 平台能力：Swift/Apple 框架实现菜单栏、窗口、UserNotifications、开机启动、Keychain 等。

### 共享基础设施
- Rust stable；serde DTO；Tokio 网络运行时；Gotify HTTP/WebSocket 客户端。
- SQLite 持久化，采用显式 schema migration；当前 SQLite 打开流程使用 `PRAGMA user_version` 的 v1 事务式迁移，可接管现有未版本化 messages 表。
- Core 不依赖 Tauri、React、SwiftUI、平台 UI 或具体平台路径。

nexthrum-core crate 已包含领域 DTO、消息策略与接收用例，并通过 `MessageRepository` 隔离存储。`nexthrum-gotify` 承担 Gotify HTTP/WebSocket API、消息解码及重连循环；`nexthrum-storage` 承担 SQLite migration、CRUD 与设置文件读写。上述共享 crate 均不依赖 Tauri 或宿主平台路径。Tauri host 保留平台生命周期、事件/通知转发及 Windows 数据路径解析。

## 2. 目录结构

```text
apps/
  windows/                   # Windows 客户端（Tauri + React）
    src/                     # React UI
    src-tauri/               # Tauri host adapter + commands
  macos/                     # SwiftUI Xcode 项目、XCTest
crates/
  nexthrum-core/             # 领域模型、用例、配置与事件
  nexthrum-gotify/           # Gotify HTTP/WebSocket、重连、解码
  nexthrum-storage/          # SQLite repository、schema migrations
  nexthrum-ffi/              # UniFFI bridge
tools/
  uniffi-bindgen/            # 本地 Swift bindings 生成器
scripts/                     # bindings 生成、macOS 构建校验
docs/  devlog/               # 规范文档与开发日志
```

当前 Cargo workspace 成员包括 `apps/windows/src-tauri`、`nexthrum-core`、`nexthrum-gotify`、`nexthrum-storage`、`nexthrum-ffi` 与本地 UniFFI 生成器。FFI bridge 由宿主提供显式 SQLite 路径；macOS 使用 Application Support/Nexthrum。入站消息通过 Core 策略后持久化，存储失败会返回错误，不会伪装成成功事件。

## 3. 分层约束

1. **Core/Use cases**：配置、连接状态、消息处理、DND、未读和保留期规则；输出类型化事件及明确错误。
2. **Adapters**：Gotify 网络、SQLite、系统通知/托盘/启动项。平台能力通过接口或宿主注入。
3. **UI**：Windows React 与 macOS SwiftUI 各自呈现平台界面；不得直接访问 SQLite。
4. **Bridge**：DTO 稳定、版本化；不暴露 Rust 内部实现、数据库连接或 Tauri 类型。

## 4. 数据与安全

- 消息表保留 Gotify 字段及本地收到时间、已读状态；主键冲突策略须明确（重复收到不能意外重置已读状态）。
- 数据目录由平台适配层提供：Windows 使用应用专属 Local/Roaming AppData 的明确策略；macOS 使用 Application Support。迁移前记录当前实际路径与文件名并提供兼容迁移。
- Token 优先使用平台凭据存储。macOS bridge 的 settings JSON 必须将 `client_token` 留空；Swift host 使用 Keychain 注入 token，仅在内存中传给 Gotify client，禁止写入日志或普通设置文件。Windows 迁移期保留既有设置兼容策略。
- WebSocket URL 使用安全 URL 构造和 token 编码；日志必须脱敏。
- HTTPS 校验证书；网络超时、HTTP 状态、鉴权错误分别建模。

## 5. 数据流

```text
Gotify ── WebSocket ── Gotify adapter ── Core use case
                                         ├── Repository(SQLite)
                                         ├── typed events ── Tauri events → React
                                         └── typed events ── Swift bridge → SwiftUI

UI 操作 → platform adapter/bridge → Core use case → Repository / Gotify API
```

Core 负责“发生什么”，平台层负责“如何展示通知/事件”。通知点击携带消息 ID，由宿主打开窗口并请求定位。

## 6. 测试与质量门槛

- Rust：fmt、clippy、单元测试、临时 SQLite repository/migration 测试。
- 领域测试：消息去重、已读/未读、DND（含跨午夜）、保留清理、设置兼容、重连状态与配置更新。
- Bridge：Rust 与 Swift DTO/事件映射测试；避免手工复制模型造成漂移。
- Windows：`npm --prefix apps/windows run typecheck`、`npm --prefix apps/windows test`（vitest：store 派生、快照竞态、时间格式）、前端构建、Windows target `cargo check`。
- macOS：Rust macOS target 构建、Swift 编译及原生集成测试。
- 发布：Windows 安装包/便携包；macOS arm64、最低 macOS 14.0，签名、公证、DMG/ZIP 策略待发布配置确定。

## 7. 依赖治理

锁定并审查 Rust、npm、Swift Package 依赖；升级前运行跨平台测试。Rust Core 不引入 UI/OS 框架依赖。具体版本以锁文件和平台 SDK 支持为准，不在架构文档中重复维护易过期的 patch 版本。

Windows 前端当前采用 Vite 8 + React 18 + Tailwind CSS v4：Tailwind v4 通过 `@tailwindcss/postcss` 参与 PostCSS，样式入口用 `@import "tailwindcss"` 与 `@source` 声明扫描范围，不再使用 `tailwind.config.js`；新增/调整源目录时必须同步更新 `src/styles/index.css` 的 `@source`，否则工具类不会被生成。

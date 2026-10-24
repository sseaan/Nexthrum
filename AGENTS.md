# AGENTS.md — Nexthrum 项目工作指引

## 1. 项目概述

Nexthrum 是连接 Gotify 服务器的桌面消息客户端，两个原生前端共用一套 Rust 业务内核：

- **Windows**：Tauri 2 + React 18 + TypeScript
- **macOS**：SwiftUI（arm64，最低 macOS 14.0），经 UniFFI 调用同一内核
- **共享内核**：Rust workspace（`crates/`），消息、存储、连接规则只在这里实现一次

## 2. 仓库结构

| 路径 | 职责 |
|------|------|
| `crates/nexthrum-core/` | 领域模型、用例（`message_service`）、DND/通知策略。**不得依赖 Tauri、UI、平台路径** |
| `crates/nexthrum-gotify/` | Gotify HTTP/WebSocket、URL 构造、解码、退避重连、取消 |
| `crates/nexthrum-storage/` | SQLite repository、schema migration、设置读写（路径由宿主注入） |
| `crates/nexthrum-ffi/` | UniFFI bridge（`BridgeService`、稳定 DTO、事件轮询） |
| `apps/windows/src-tauri/` | **仅 Windows host adapter**：窗口、托盘、通知、Tauri command、事件转发、Windows 凭据 |
| `apps/windows/src/` | Windows React UI（Zustand store、组件、i18n） |
| `apps/macos/` | SwiftUI app、Xcode 工程、XCTest、Keychain、内嵌 Sparkle（`Vendor/`） |
| `tools/uniffi-bindgen/` | 本地 Swift bindings 生成器（CI 依赖，勿删） |
| `scripts/` | bindings 生成、图标、macOS 构建/打包、签名身份 |
| `docs/ devlog/` | 标准文档与每日开发日志 |
| `.github/workflows/` | `ci.yml`（双平台检查）、`build.yml`（tag 触发的双平台发布）、`test-build.yml`（`test/**` 分支或手动触发的 Windows 预发布） |

## 3. 标准文档

| 文档 | 路径 | 内容 |
|------|------|------|
| 开发需求 | `docs/requirements.md` | 功能需求、用户故事、验收标准 |
| 技术规范 | `docs/tech-spec.md` | 技术决策、目录、分层约束、测试门槛 |
| 设计规范 | `docs/design-spec.md` | UI 布局、色彩、交互、平台差异 |
| 架构设计 | `docs/architecture.md` | 架构图、模块职责、事件/Tauri 协议、持久化 |
| 更新历史 | `docs/history.md` | 版本历史、已完成的重构与 UI 重做回顾、已归档文档去向 |
| 真机测试 | `docs/windows-testing.md` | Windows 验收清单、端口/SDK/空白窗口排查、测试版获取 |
| 发布手册 | `docs/release.md` | 版本同步、签名/公证、更新策略、发布检查清单 |
| 项目说明 | `README.md` / `README.zh.md` | 英文主文档与简体中文版（安装、构建、架构、许可） |
| 开发指南 | `docs/development-guide.md` | 环境搭建、开发流程、代码规范 |
| 开发日志 | `devlog/YYYY-MM-DD.md` | 完成事项 + 待办（先看最新一篇了解进度） |

## 4. 硬性架构约定

1. **业务只在 Core**：平台差异留在 adapter。新增消息规则先改 `nexthrum-core`，不要在 `apps/windows/src-tauri` 或 FFI 里重复编排。
2. **UI 不接触存储**：前端与 Swift 只能通过 Tauri command / UniFFI DTO 访问消息。
3. **后端拥有状态权威**：未读数与消息列表以 SQLite 快照为准；`new-message` 事件只插入列表，前端不得自行递增未读数。mutation command 直接返回 `MessageSnapshot`，不要“改完再查一次”。
4. **凭据不进 JSON**：Windows token 存 Windows Credential Manager（`apps/windows/src-tauri/src/credentials.rs`），macOS 存 Keychain；`settings.json` 只放非敏感字段。
5. **连接配置串行化**：`save_settings` / `connect_websocket` / `disconnect_websocket` 共用 `AppState.connection_update`，消息变更共用 `AppState.message_update`。
6. **设置与连接解耦**：设置持久化成功后即使重连失败也要保留新设置，并返回明确错误。
7. **迁移要向后兼容**：新增列必须走 migration，并考虑早期 schema 缺列（参考 `ensure_local_columns`）。
8. **macOS 签名与更新**：用 `scripts/create-local-signing-identity.sh` 建立稳定自签身份（不要退回 ad-hoc，否则钥匙串授权每次重建都重置）；macOS 自动更新走 Sparkle（`apps/macos/Vendor/Sparkle.framework`），`CFBundleVersion` 必须递增。
9. **样式基类必须进层**：`apps/windows/src/styles/index.css` 里所有可能与 Tailwind 工具类冲突的规则必须写在 `@layer base` 内——未分层 CSS 会无视优先级压过 `@layer utilities`，曾导致全站间距/边框/字号失效。不要重复 preflight 的 `*` 与表单 reset。
10. **不要用固定 offset 做视觉对齐**：字形留白随链接 SDK 变化，实测同一 `offset(x: -2)` 在 macOS 14.5 SDK 下对齐、在 macOS 26/27 SDK 下反向偏移。对齐问题要改布局结构，不补贴偏移量。
11. **macOS 发布必须用当前 SDK**：发布 job 跑在 `macos-26` runner（`macos-14` 已于 2026-11-02 起停止支持，且只带 macOS 14 SDK，产物在新系统上会退回旧界面）。校验产物 `Info.plist` 的 `DTSDKName`。
12. **Tauri 插件版本成对锁定**：`@tauri-apps/plugin-*` 写精确版本、对应 crate 用 `~` 锁同 minor，改动后跑 `npm --prefix apps/windows run check:tauri-versions`（CI 已接入）。
13. **发布说明只写四段**：简介 / 下载文件用途 / 系统要求 / 本版亮点；未签名与未公证提示写在「下载文件」段。模板见 `.github/workflows/build.yml` 的 `releaseBody`，每次发版同步文件名版本号。
14. **发布变更先读手册**：版本号需同时更新 `package.json`、`tauri.conf.json`、`Cargo.toml` 与 macOS `Info.plist`（CI 校验一致），macOS 还要递增 `CFBundleVersion`；签名/公证/更新与产物命名见 `docs/release.md`。
15. **发布产物固定**：`Nexthrum_<v>_x64-setup.exe`、`Nexthrum_<v>_x64-portable.zip`、`Nexthrum-<v>-arm64.dmg`、`Nexthrum-<v>.zip`、`latest.json`、`appcast.xml`；不要上传独立 `*.sig`。

## 5. 开发流程

1. **动手前**：读 `devlog/` 最新日志 + 相关标准文档；修改功能前先更新对应文档。
2. **开发中**：
   - 新 Tauri command：实现 → 在 `apps/windows/src-tauri/src/lib.rs` 的 `generate_handler!` 注册 → 更新 `docs/architecture.md` 事件/命令表
   - 新 Core 用例：写 `MessageRepository` port 方法 → 用例函数 → 契约测试
   - 新设置项：`AppSettings`（core）+ `defaultSettings`（TS）+ README/文档 + 表单控件；凭据类字段另走凭据模块
   - 前端组件：遵循 `docs/design-spec.md`，用 Zustand store，不直接拼后端状态
3. **提交前必须跑**：

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm --prefix apps/windows run typecheck
npm --prefix apps/windows test
```

Windows 相关改动还可运行 `npm --prefix apps/windows run build` 验证前端产物。

macOS 相关改动还要：

```bash
scripts/generate-swift-bindings.sh
swiftc -parse apps/macos/Generated/nexthrum_ffi.swift apps/macos/AppModel.swift
scripts/build-macos-xcode.sh          # arm64 app + @rpath + plist 校验
xcodebuild -project apps/macos/Nexthrum.xcodeproj -scheme Nexthrum \
  -destination 'platform=macOS,arch=arm64' CODE_SIGNING_ALLOWED=NO test
```

4. **收尾**：更新 `devlog/YYYY-MM-DD.md`（完成事项 + 待办）。

## 6. 代码风格

- **TypeScript**：严格模式、函数组件 + Hooks、Zustand；文案走 `apps/windows/src/i18n`，不硬编码 UI 字符串
- **Rust**：`rustfmt` 默认格式、`log` 分级日志、错误用 `String`/`BridgeError` 明确分类，不吞错
- **Swift**：SwiftUI 原生控件优先，业务判断留在 Core
- **CSS**：Tailwind 工具类优先，自定义颜色用 `apps/windows/src/styles/index.css` 的 CSS 变量

## 7. 数据位置

- Windows：SQLite `%APPDATA%/Nexthrum/nexthrum_messages.db`，设置 `nexthrum_data/settings.json`，token 在 Credential Manager
- macOS：Application Support/Nexthrum，token 在 Keychain
- 调试日志：`RUST_LOG=debug npm --prefix apps/windows run tauri dev`

## 8. Git 约定

```
<type>(<scope>): <subject>
```

`feat` / `fix` / `docs` / `refactor` / `test` / `chore`

**不要提交**：`target/`、`apps/windows/src-tauri/target/`、`apps/windows/dist/`、`apps/windows/node_modules/`、`apps/macos/Generated/`、`apps/windows/src-tauri/gen/schemas/`、`.DS_Store`、`*.db`、`.env*`。

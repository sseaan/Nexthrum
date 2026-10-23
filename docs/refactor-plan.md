# Nexthrum 架构评估与跨平台重构方案

## 1. 决策摘要

目标是在保留 Windows Tauri/React 应用的基础上，新增 **SwiftUI 原生 macOS 应用**。推荐将 Gotify 协议、消息用例和存储能力逐步抽为无 UI 依赖的 Rust Core；两个平台各自实现 UI 与 OS 集成。先解耦、补测试，再引入 Swift bridge（优先验证 UniFFI），不进行一次性全量重写。

## 2. 当前状态

分层重构已完成：DTO、Tauri commands、Core 消息用例、Gotify client、Storage、FFI bridge 均有独立 crate/模块；两个平台前端只保留 UI 与 OS 集成。Windows `websocket.rs` 保留 host 生命周期、Tauri 事件与通知 adapter；macOS 通过 UniFFI `BridgeService` 复用同一 client 与 repository。

已解决的风险（原基线）：

- Tauri `AppHandle` 不再渗入网络与业务：Gotify client 与 Core 用例不依赖 Tauri。
- 凭据暴露：WebSocket URL 清理基址 query/fragment 后附加编码 token；日志脱敏；token 移入 Credential Manager / Keychain，`settings.json` 只存非敏感字段。
- 并发语义：连接更新与消息变更各有 host mutex；设置先持久化再重连，失败返回“设置已保存”错误。
- 未读数/UI 缓存：后端权威快照；mutation command 直接返回 `MessageSnapshot`，前端不再自行漂移。
- 服务器删除失败：先校验 HTTP 结果，失败不清本地；错误分类为鉴权/网络/TLS/HTTP 状态/响应格式。
- SQLite migration：v1 事务式迁移 + 早期 schema 列对齐；覆盖旧数据保留与损坏库失败。
- 跨语言契约：UniFFI DTO 与错误分类稳定化，Swift 侧有文案映射与 XCTest；DTO 字段契约测试见下。

剩余风险：

- 发布工程未完成：Windows Authenticode 缺失，macOS 无签名/公证/DMG，更新策略未接入（见 `docs/release.md`）。
- macOS 通知授权、启动项审批、退出清理仅部分自动化覆盖，仍需实机验收。
- Keychain 集成测试为 opt-in，默认不在 CI 执行。

## 3. 目标结构

仓库为 Cargo workspace：根 `Cargo.toml` 管理 `apps/windows/src-tauri` 与 `crates/*`。平台前端只依赖 Core 用例，不接触 SQLite。

```text
apps/windows (React/Zustand) → Tauri adapter ─┐
                                              ├→ nexthrum-core 用例/策略
apps/macos (SwiftUI) → UniFFI bridge ─────────┘        │
                                                       ├→ nexthrum-storage（SQLite）
                                                       └→ nexthrum-gotify（HTTP/WS）

平台 host 负责：数据目录解析、凭据存储、通知、托盘/菜单栏、启动项。
```

Core 提供用例和类型化事件；不依赖 Tauri、Swift、React、平台路径和 UI 文案。Swift bridge 仅暴露稳定 DTO、用例与事件流，不暴露 SQLite 结构。UI 不直接操作数据库。

## 4. macOS 设计约束

- SwiftUI 原生窗口、设置与标准 macOS 控件；不移植无边框 Windows 标题栏。
- MenuBarExtra/状态栏菜单作为常驻入口，打开消息/设置并支持退出。
- UserNotifications 请求权限；点击通知打开消息并定位；拒绝授权不影响应用内查看。
- ServiceManagement 用于开机启动，按最低 macOS 版本选择 API。
- Application Support 存放本地数据，Keychain 存储 token。
- 关闭窗口、Dock 图标、菜单栏常驻模式遵循 macOS 生命周期；需产品确认常规 Dock 应用或菜单栏优先模式。
- Apple Silicon only (arm64); do not add Intel/x86_64 or universal build work.
- Current Xcode deployment target: macOS 14.0.
- Sandbox, signing and notarization remain release decisions.

## 5. 分阶段迁移

### 阶段 0：基线与安全（已完成）
- 脱敏凭据日志、正确编码 URL 参数、验证证书与错误状态。
- 明确设置保存原子性、配置清空时断开连接、连接状态事件。
- 审计现有数据路径/文件名，建立行为基线。

### 阶段 1：业务解耦（已完成）
- 将命令、状态和启动编排从 `lib.rs` 分开。
- 将消息用例从 WebSocket 循环分离；网络客户端仅输出类型化消息/状态事件。
- 以 repository、notification、event sink 等边界隔离平台能力。
- 保持现有 Tauri 命令和 React UI 尽量兼容。

### 阶段 2：持久化与测试（已完成）
- 引入 SQLite migration、事务边界和 repository 测试。
- 统一消息/未读状态变更后的快照或事件更新。
- 测试去重、DND 跨午夜、保留清理、重连配置变化、设置兼容和失败操作。
- 用本地 mock HTTP/WebSocket 服务器覆盖鉴权、HTTP 非 2xx、响应格式、连接关闭与重连取消（已落地 `crates/nexthrum-gotify/tests/mock_server.rs`）。
- 为旧数据路径迁移设计幂等、可恢复流程。

### 阶段 3：Swift bridge 验证（已完成）
- 以小型 spike 验证 UniFFI 的 Rust DTO、服务生命周期、异步事件/取消、错误映射和 macOS 构建。
- 如无法满足异步/构建要求，再评估薄 C ABI；避免手写大量 unsafe Swift 桥接。
- 固定版本化契约，并用测试防止 Rust、TS、Swift 模型漂移。

### 阶段 4：macOS vertical slice（已完成；通知/启动项/退出清理的实机验收待补）
- SwiftUI 连接设置、连接测试、实时收消息、SQLite 持久化和消息列表。
- 加入菜单栏入口、通知授权/投递与点击跳转。
- 验证断线恢复、休眠恢复、关闭窗口后台运行、权限拒绝及数据升级。
- 再补齐筛选、标读、删除、清空、DND、保留期、启动项和本地化。

### 阶段 5：发布治理（双平台发布与自动更新已完成；Apple 公证与 Windows Authenticode 待证书）
- Windows/macOS 分别构建与集成测试；Rust Core 在双平台 target 验证。
- 配置 Apple 签名、公证、DMG/ZIP；明确更新与版本策略。
- 现状：2.0.0 已发布 Windows 安装包/便携包与 macOS DMG/zip，双平台自动更新通道（`latest.json` / `appcast.xml`）已上线；macOS 使用稳定自签身份，公证与 Authenticode 仍缺证书。
- 确保 Windows 既有数据和配置兼容。

## 6. 方案比较

| 方案 | 优点 | 缺点/结论 |
|---|---|---|
| Tauri 同时用于 macOS | 最大程度复用 | 不满足 SwiftUI 原生要求，不推荐 |
| Swift 全量重写业务 | 直接原生 | 两套协议、存储、重连和业务规则长期分叉，不推荐 |
| Rust Core + 双平台原生 UI | 业务复用，原生体验 | 需投入分层、FFI 与测试建设；推荐 |

## 7. 验收标准

1. Core 脱离 Tauri 可测试，且 Windows/macOS target 可构建。
2. Core 对外无 Tauri/UI/平台路径类型；平台能力经 adapter 注入。
3. 关键业务行为由自动化测试覆盖，SQLite 有迁移测试。
4. SwiftUI 独立完成连接、收消息、持久化与展示，原生通知/菜单栏符合系统行为。
5. Windows 现有功能和数据兼容；错误不会静默丢失或误报成功。
6. 日志无凭据泄漏，双平台 CI 覆盖构建与核心测试。

## 8. 决策与待办

已决定：

- macOS 仅 Apple Silicon arm64，最低 macOS 14.0，不计划 Intel/universal。
- macOS 同时提供标准 Dock 窗口与 `MenuBarExtra` 菜单栏入口；关闭窗口后后台连接继续，退出为显式操作。
- 凭据：Windows 使用 Credential Manager（旧明文 token 幂等迁移），macOS 使用 Keychain；`settings.json` 不存 token。
- UniFFI 作为 Swift bridge，事件采用 start/poll/stop 轮询模型，worker 可取消并 join。
- Windows 发布格式为 NSIS 安装包 + 便携 zip。

待决定/待完成：

- Windows Authenticode 证书来源与 CI 导入方式。
- macOS 签名身份、公证凭据、DMG 打包与 Gatekeeper 验证的具体执行。
- 自动更新已接入：Windows 用 `tauri-plugin-updater`（`latest.json`），macOS 用 Sparkle（`appcast.xml`）。
- macOS 通知授权、启动项审批与退出清理的实机验收记录方式。

# Nexthrum

[English](README.md) · **简体中文**

跨平台的 [Gotify](https://gotify.net/) 桌面客户端：把你自建服务器上的推送消息变成系统原生通知。两个原生前端共用同一套 Rust 内核。

| 平台 | 界面 | 状态 |
|---|---|---|
| Windows 10 1809+ / 11 | Tauri 2 + React 18 + TypeScript | 已发布 |
| macOS 14+（Apple Silicon） | SwiftUI + Sparkle | 已发布 |

---

## 功能特性

- **实时消息流** — Gotify WebSocket 客户端支持自动重连、停止即取消、10 秒握手超时，界面不会一直卡在「正在连接」。
- **原生通知** — Windows toast 通知与 macOS `UserNotifications`，都遵循 Rust 内核统一的免打扰策略。
- **本地历史** — SQLite 存储，保留天数可配置（1–365 天，默认 7 天）。重复投递保留本地已读状态且不会重复通知。
- **卡片式消息列表** — 未读状态用三种冗余线索表达（竖条、圆点、字重），附带来源与优先级徽标，悬停有抬升效果。
- **搜索** — 对标题与正文做不区分大小写的匹配，带防抖；搜索期间未读数仍为全局值。
- **来源筛选** — 消息来自两个以上 Gotify 应用时自动出现；名称取自 `/application`，取不到时回退为 `App <id>`。
- **常驻入口** — Windows 托盘带未读红点与 tooltip；macOS 菜单栏面板显示最近三条消息并支持快捷操作。
- **安全的凭据存储** — 客户端令牌不会写入设置 JSON：Windows 用凭据管理器，macOS 用钥匙串，并支持旧明文令牌的幂等迁移。
- **自动更新** — 各平台独立的签名更新通道：Windows 使用 `tauri-plugin-updater`，macOS 使用 Sparkle（EdDSA 签名的 appcast）。
- **界面本地化** — 支持简体中文与英文；Windows 可在设置中切换，macOS 跟随系统语言。
- **开机启动** — Windows 自启动项与 macOS `SMAppService`，并在设置中显示系统真实的审批状态。

---

## 安装

从 [GitHub Releases](https://github.com/sseaan/Nexthrum/releases) 下载最新版本。

### Windows

| 文件 | 说明 |
|---|---|
| `Nexthrum_*_x64-setup.exe` | NSIS 安装包（当前用户安装，无需管理员） |
| `Nexthrum_portable.zip` | 便携版，解压即用 |

安装包**尚未做 Authenticode 签名**，首次运行会出现「Windows 已保护你的电脑」：选择 **更多信息 → 仍要运行**。

### macOS

| 文件 | 说明 |
|---|---|
| `Nexthrum-*_arm64.dmg` | 拖拽安装的应用包（Apple Silicon，macOS 14+） |

应用**未做公证**（没有付费 Apple 开发者账号），首次打开需要**右键 → 打开**，或者执行：

```bash
xattr -dr com.apple.quarantine /Applications/Nexthrum.app
```

由 Sparkle 安装的更新不带隔离标记，因此后续升级不会再被拦截。如果你装过 2.0 早期构建，请先删除旧的 `NexthrumMac.app` —— bundle id、设置和钥匙串条目都未改变。

---

## 快速开始

1. 启动 Nexthrum：窗口打开，托盘 / 菜单栏出现图标。
2. 打开**设置**，填写 **Gotify 服务器地址**（如 `https://gotify.example.com`）和**客户端令牌**（Gotify 网页端 → *Clients*）。
3. 点击**测试连接**验证 HTTP 接口，然后**保存** —— WebSocket 会自动开始连接。
4. 从 Gotify 发一条消息，它会出现在列表里；若不在免打扰时段，还会弹出系统通知。

---

## 从源码构建

### 环境要求

| 平台 | 依赖 |
|---|---|
| Windows | Node.js 20+、Rust stable、Visual Studio C++ 生成工具、WebView2 |
| macOS | Node.js 20+、Rust stable、Xcode 15.4+（arm64）、macOS 14 SDK |
| 通用 | Git；`cargo`、`npm` 在 `PATH` 中 |

### Windows 客户端

```bash
git clone https://github.com/sseaan/Nexthrum.git
cd Nexthrum

npm --prefix apps/windows ci
npm --prefix apps/windows run tauri dev     # 热更新开发模式
npm --prefix apps/windows run tauri build   # 生成 NSIS 安装包与便携版
```

### macOS 客户端

```bash
scripts/generate-swift-bindings.sh          # 生成 UniFFI Swift 绑定
scripts/build-macos-xcode.sh                # 构建并校验 Nexthrum.app
scripts/package-macos-dmg.sh                # 生成可本地分发的签名 DMG

# 单元测试
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project apps/macos/Nexthrum.xcodeproj -scheme Nexthrum \
  -destination 'platform=macOS,arch=arm64' CODE_SIGNING_ALLOWED=NO test
```

macOS 构建由 `scripts/create-local-signing-identity.sh` 创建的**稳定自签身份**签名，保证多次重建后代码签名（以及钥匙串授权）不变；它不能替代公证。

### 检查项

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm --prefix apps/windows run typecheck
npm --prefix apps/windows test
```

CI（`.github/workflows/ci.yml`）在 Windows 与 macOS arm64 上执行同样的检查，并额外包含 UniFFI 生成、Xcode 构建与 XCTest。

---

## 架构

```
apps/windows (React)  ──Tauri──┐
                               ├── nexthrum-core ── nexthrum-storage (SQLite)
apps/macos (SwiftUI) ──UniFFI──┘        │            nexthrum-gotify (HTTP/WS)
                                        └── 宿主适配层：托盘、通知、凭据、窗口生命周期
```

- `nexthrum-core` 承载全部业务规则：消息接收、去重、已读状态、免打扰、保留期。它不依赖 UI、Tauri 或平台路径。
- `nexthrum-gotify` 承载 HTTP API 与 WebSocket 客户端，包含重连退避、取消、类型化错误与 rustls crypto provider 安装。
- `nexthrum-storage` 承载 SQLite 迁移、查询与设置读写，路径由宿主注入。
- `nexthrum-ffi` 暴露稳定的 UniFFI 接口（`BridgeService`、DTO、类型化错误）给 Swift。
- 宿主适配层保持轻量：Tauri 命令与 SwiftUI 桥接只做参数校验、调用 Core 用例、呈现事件。

状态由后端拥有：未读数与消息列表始终来自原子快照，变更命令直接返回变更后的快照。

### 目录结构

```
apps/
  windows/            Windows 客户端（React + Vite）与 Tauri 宿主
  macos/              SwiftUI 应用、Xcode 工程、XCTest、Sparkle（Vendor/）
crates/
  nexthrum-core/      领域模型、用例、免打扰与通知策略
  nexthrum-gotify/    Gotify HTTP/WebSocket 客户端
  nexthrum-storage/   SQLite repository、迁移、设置
  nexthrum-ffi/       UniFFI 桥接
scripts/              绑定生成、图标、macOS 构建打包、签名身份
tools/uniffi-bindgen/ 本地 Swift 绑定生成器
docs/                 需求、架构、设计、技术规范、发布手册
devlog/               每日开发日志
```

### 技术栈

| 层 | Windows | macOS |
|---|---|---|
| 界面 | React 18 + TypeScript + Tailwind CSS v4 + Zustand | SwiftUI |
| 宿主 | Tauri 2（Rust） | AppKit 桥接 + UniFFI |
| 内核 | Rust workspace（`nexthrum-core`、`nexthrum-gotify`、`nexthrum-storage`） | 同一套 crate |
| 存储 | SQLite（rusqlite） | 同一 repository |
| 推送 | tokio-tungstenite + rustls | 同一客户端 |
| 通知 | `tauri-plugin-notification` | `UserNotifications` |
| 更新 | `tauri-plugin-updater`（`latest.json`） | Sparkle（`appcast.xml`） |
| 凭据 | Windows 凭据管理器 | 钥匙串 |

---

## 数据与隐私

- Nexthrum 只与你配置的 Gotify 服务器通信；没有遥测，也不经过第三方服务。
- 令牌保存在系统凭据库，永远不会写进 `settings.json`；日志做了脱敏。
- 本地数据位置：
  - Windows：`%APPDATA%\Nexthrum\nexthrum_messages.db` 与 `nexthrum_data\settings.json`
  - macOS：`~/Library/Application Support/Nexthrum/`
- 本地删除消息不会影响服务器，除非在清空对话框里勾选「同步删除服务器消息」。

---

## 文档

| 文档 | 内容 |
|---|---|
| [`docs/requirements.md`](docs/requirements.md) | 产品需求与验收标准 |
| [`docs/architecture.md`](docs/architecture.md) | 模块职责、事件、持久化、更新通道 |
| [`docs/tech-spec.md`](docs/tech-spec.md) | 分层约束、依赖、质量门槛 |
| [`docs/design-spec.md`](docs/design-spec.md) | 视觉语言、布局、可访问性规则 |
| [`docs/release.md`](docs/release.md) | 版本来源、签名、公证、更新、发布检查清单 |
| [`docs/development-guide.md`](docs/development-guide.md) | 环境搭建与开发流程 |
| [`AGENTS.md`](AGENTS.md) | 面向自动化代理的贡献约定 |

---

## 许可证

Copyright © 2026 Sean.

Nexthrum 是基于 **GNU Affero 通用公共许可证 v3.0**（AGPL-3.0）发布的自由软件，完整条款见 [LICENSE](LICENSE)。

你可以在该许可证条款下使用、研究、修改和再分发本项目。由于 AGPL-3.0 同样约束通过网络提供的软件，如果你把修改后的 Nexthrum 作为网络服务部署，必须向使用者提供对应源码。本项目**不提供任何担保**，详见许可证。

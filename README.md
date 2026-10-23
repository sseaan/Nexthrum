# Nexthrum

[🇨🇳 中文版](#nexthrum-中文)

A Windows desktop client for [Gotify](https://gotify.net/) — receive push messages via native Windows notifications.

Built with **Tauri 2.0 + React + TypeScript + Rust**.

---

## ✨ Features

- 🔗 **WebSocket real-time push** — connect to Gotify server, receive messages instantly
- 🔔 **Windows native notifications** — toast notifications via Windows Notification Center
- 📋 **Message history** — local SQLite storage with configurable retention (default 7 days)
- 🏷 **Filter by app** — filter messages by source application
- 📋 **Click to copy** — click any message to copy its content to clipboard
- 🧹 **Clear all** — clear local messages, with optional server sync
- 🌙 **Dark theme** — designed for low-distraction notification monitoring
- 📌 **System tray** — minimize to tray, always running in background
- 🚫 **Do Not Disturb** — suppress notifications during specified hours
- 🪟 **Frameless window** — clean custom title bar with drag support
- 🌍 **i18n** — English & 中文 supported, switch in Settings
- 🚀 **Auto-start** — optionally launch on Windows startup
- 📦 **Portable + Installer** — NSIS installer and portable .zip available

---

## 📸 Screenshots

*Coming soon*

---

## 📥 Installation

### Download

Get the latest version from [GitHub Releases](https://github.com/sseaan/Nexthrum/releases).

| File | Description |
|------|-------------|
| `Nexthrum_*_x64-setup.exe` | NSIS installer — install for current user |
| `Nexthrum_portable.zip` | Portable version — extract and run |

### System Requirements

- Windows 10 version 1809+ or Windows 11
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (pre-installed on Windows 11, auto-updated on Windows 10)

---

## 🚀 Usage

1. Launch Nexthrum (window opens, tray icon appears)
2. Click the tray icon or open the window from the tray menu
3. Go to **Settings** (⚙ icon in toolbar or tray menu → Settings)
4. Enter your **Gotify Server URL** and **Client Token**
5. Click **Test Connection** to verify
6. **Save Settings** — WebSocket connects automatically
7. Send a test message from your Gotify server — it appears in the list!

### Tray Menu

| Action | Description |
|--------|-------------|
| Left-click tray icon | Show/hide message window |
| Right-click → Message List | Show message window |
| Right-click → Settings | Open settings |
| Right-click → Exit | Quit application |

### Window Controls

| Button | Action |
|--------|--------|
| `—` | Minimize window |
| `✕` | Hide to tray (not quit — configurable in Settings) |
| Drag title bar | Move window |

---

## 🛠 Development

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) stable
- [Microsoft Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) (Windows)

### Setup

```bash
# Clone
git clone https://github.com/sseaan/Nexthrum.git
cd Nexthrum

# Install frontend dependencies
npm --prefix apps/windows install

# Start development (with hot reload)
npm --prefix apps/windows run tauri dev

# Build production
npm --prefix apps/windows run tauri build
```

### Project Structure

```
Nexthrum/
├── apps/
│   ├── windows/            # Windows client (Tauri + React)
│   │   ├── src/            # React frontend
│   │   │   ├── components/ # MessageWindow / SettingsWindow / common
│   │   │   ├── hooks/      # Tauri event listeners
│   │   │   ├── store/      # Zustand state
│   │   │   ├── i18n/       # Translations (zh-CN / en-US)
│   │   │   ├── types/      # TypeScript types
│   │   │   └── utils/      # Utilities
│   │   └── src-tauri/      # Tauri host adapter
│   │       └── src/
│   │           ├── lib.rs         # State, setup, command registration
│   │           ├── commands.rs    # Tauri commands
│   │           ├── websocket.rs   # Event adapter for the Gotify client
│   │           ├── credentials.rs # Windows Credential Manager
│   │           ├── storage.rs     # Storage path resolution
│   │           └── tray.rs        # System tray
│   └── macos/              # macOS client (SwiftUI + Xcode)
├── crates/                 # Shared Rust workspace
│   ├── nexthrum-core/      # Domain models, use cases, DND policy
│   ├── nexthrum-gotify/    # Gotify HTTP/WebSocket client
│   ├── nexthrum-storage/   # SQLite repository + migrations
│   └── nexthrum-ffi/       # UniFFI bridge for Swift
├── scripts/                # Bindings generation, macOS build checks
├── tools/uniffi-bindgen/   # Local Swift bindings generator
├── docs/ devlog/           # Documentation and development logs
└── .github/workflows/      # CI/CD
```

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri 2.0 |
| Frontend | React 18 + TypeScript |
| Styling | Tailwind CSS |
| State | Zustand |
| Backend | Rust |
| Storage | SQLite (rusqlite) |
| WebSocket | tokio-tungstenite (rustls) |
| Notifications | tauri-plugin-notification |
| CI/CD | GitHub Actions |

---

## 📄 License

MIT © [Sean](https://github.com/sseaan)

---

---

# Nexthrum 中文

[🇬🇧 English](#nexthrum)

一款 Windows 桌面 Gotify 客户端 — 通过 Windows 原生通知接收推送消息。

基于 **Tauri 2.0 + React + TypeScript + Rust** 构建。

---

## ✨ 功能特性

- 🔗 **WebSocket 实时推送** — 连接 Gotify 服务器，即时接收消息
- 🔔 **Windows 原生通知** — 通过 Windows 通知中心弹出提醒
- 📋 **消息历史记录** — 本地 SQLite 存储，可配置保留天数（默认 7 天）
- 🏷 **按应用筛选** — 按消息来源应用过滤列表
- 📋 **点击复制** — 单击消息即可复制内容到剪贴板
- 🧹 **一键清空** — 清空本地消息，可选同步删除服务器消息
- 🌙 **暗色主题** — 专为低干扰消息监控设计
- 📌 **系统托盘** — 最小化到托盘，后台持续运行
- 🚫 **免打扰模式** — 指定时段内不弹出通知
- 🪟 **无边框窗口** — 简洁自定义标题栏，支持拖拽移动
- 🌍 **多语言** — 支持中文 / English，设置中切换
- 🚀 **开机自启** — 可随 Windows 启动自动运行
- 📦 **安装包 + 便携版** — 提供 NSIS 安装包和 .zip 便携版

---

## 📸 截图

*即将推出*

---

## 📥 安装

### 下载

从 [GitHub Releases](https://github.com/sseaan/Nexthrum/releases) 获取最新版本。

| 文件 | 说明 |
|------|------|
| `Nexthrum_*_x64-setup.exe` | NSIS 安装包 — 为当前用户安装 |
| `Nexthrum_portable.zip` | 便携版 — 解压即用 |

### 系统要求

- Windows 10 版本 1809+ 或 Windows 11
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)（Windows 11 预装，Windows 10 自动更新）

---

## 🚀 使用说明

1. 启动 Nexthrum（窗口打开，托盘图标出现）
2. 点击托盘图标或从托盘菜单打开窗口
3. 进入**设置**（工具栏 ⚙ 按钮或托盘菜单 → 设置）
4. 填写 **Gotify 服务器地址**和**客户端令牌**
5. 点击**测试连接**验证
6. **保存设置** — WebSocket 自动连接
7. 从 Gotify 服务器发送一条测试消息 — 消息出现在列表中！

### 托盘菜单

| 操作 | 说明 |
|------|------|
| 左键单击托盘图标 | 显示/隐藏消息窗口 |
| 右键 → 消息列表 | 显示消息窗口 |
| 右键 → 设置 | 打开设置 |
| 右键 → 退出 | 退出应用 |

### 窗口控制

| 按钮 | 功能 |
|------|------|
| `—` | 最小化窗口 |
| `✕` | 隐藏到托盘（不退出，可在设置中修改） |
| 拖拽标题栏 | 移动窗口 |

---

## 🛠 开发

### 环境要求

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) stable
- [Microsoft Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)（Windows）

### 快速开始

```bash
# 克隆项目
git clone https://github.com/sseaan/Nexthrum.git
cd Nexthrum

# 安装前端依赖
npm --prefix apps/windows install

# 启动开发模式（热更新）
npm --prefix apps/windows run tauri dev

# 生产构建
npm --prefix apps/windows run tauri build
```

### 项目结构

```
Nexthrum/
├── apps/
│   ├── windows/            # Windows 客户端（Tauri + React）
│   │   ├── src/            # React 前端
│   │   │   ├── components/ # 消息窗口 / 设置窗口 / 公共组件
│   │   │   ├── hooks/      # Tauri 事件监听
│   │   │   ├── store/      # Zustand 状态管理
│   │   │   ├── i18n/       # 多语言翻译
│   │   │   ├── types/      # TypeScript 类型定义
│   │   │   └── utils/      # 工具函数
│   │   └── src-tauri/      # Tauri host adapter
│   │       └── src/
│   │           ├── lib.rs         # 状态、初始化、命令注册
│   │           ├── commands.rs    # Tauri 命令
│   │           ├── websocket.rs   # Gotify 客户端事件适配
│   │           ├── credentials.rs # Windows 凭据管理器
│   │           ├── storage.rs     # 存储路径解析
│   │           └── tray.rs        # 系统托盘
│   └── macos/              # macOS 客户端（SwiftUI + Xcode）
├── crates/                 # 共享 Rust workspace
│   ├── nexthrum-core/      # 领域模型、用例、DND 策略
│   ├── nexthrum-gotify/    # Gotify HTTP/WebSocket 客户端
│   ├── nexthrum-storage/   # SQLite repository + migrations
│   └── nexthrum-ffi/       # Swift 用 UniFFI bridge
├── scripts/                # bindings 生成、macOS 构建校验
├── tools/uniffi-bindgen/   # 本地 Swift bindings 生成器
├── docs/ devlog/           # 文档与开发日志
└── .github/workflows/      # CI/CD
```

### 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 前端 | React 18 + TypeScript |
| 样式 | Tailwind CSS |
| 状态管理 | Zustand |
| 后端 | Rust |
| 存储 | SQLite (rusqlite) |
| WebSocket | tokio-tungstenite (rustls) |
| 通知 | tauri-plugin-notification |
| 构建 | GitHub Actions |

---

## 📄 许可证

MIT © [Sean](https://github.com/sseaan)

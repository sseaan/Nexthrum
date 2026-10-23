# Nexthrum 开发指南

## 1. 项目现状与目标

Windows 当前由 React/TypeScript + Tauri 2/Rust 构成；新增目标是 SwiftUI 原生 macOS 客户端，并逐步抽取无 UI/平台依赖的 Rust Core。开始开发前查看 `devlog/` 最新记录及 `docs/requirements.md`、`docs/architecture.md`、`docs/tech-spec.md`、`docs/design-spec.md`。

目标架构与现状的差异见 `docs/refactor-plan.md`。不要假设目标目录、FFI 或 macOS 功能已经落地。

## 2. 仓库导航

```text
apps/windows/src/         Windows React UI
apps/windows/src-tauri/   Windows Tauri/Rust host adapter
apps/macos/               目标：SwiftUI app（建立后使用）
crates/                   目标：Rust Core/Gotify/Storage 等（渐进抽取）
docs/                     需求、架构、技术、设计和迁移方案
devlog/                   每日进展
.github/workflows/        CI 与发行
```

## 3. 开发环境

### Windows 当前应用
- Node.js 20 LTS、Rust stable、Visual Studio C++ Build Tools、WebView2。
- 安装依赖：`npm --prefix apps/windows ci`。
- 开发运行：`npm --prefix apps/windows run tauri dev`。
- 前端检查：`npm --prefix apps/windows run typecheck`；前端构建：`npm --prefix apps/windows run build`。
- 后端检查（从 workspace 根执行）：`cargo check --workspace`；完整门槛见第 5 节。

- 应用图标：母版为仓库根目录 `Nexthrum.png`（正方形，≥1024px），运行 `scripts/generate-icons.sh` 一次性重生成 Windows `apps/windows/src-tauri/icons/*` 与 `apps/macos/AppIcon.icns`；Xcode 通过 `CFBundleIconFile` 引用该 icns。
- 图标留白由 `scripts/icon_normalize.py` 归一：先按 alpha 裁剪母版透明边，再按平台重新留白（Windows 应用 92%、托盘 94%、macOS 80.5%）。**不要**把母版直接喂给 `tauri icon`，否则会叠加母版本身的留白。

### macOS 应用
- Apple Silicon（arm64）、macOS 14.0+、Xcode 及 Rust stable。
- 生成 Swift bindings：`scripts/generate-swift-bindings.sh`（需要 Rust；可选安装 `swiftformat` 进行格式化）。输出到 `apps/macos/Generated/`，生成 `.swift`、FFI header 和 modulemap。
- 构建并校验 app：`scripts/build-macos-xcode.sh`（校验 arm64、内嵌 dylib、`@rpath`、Info.plist）。
- 单元测试：`xcodebuild -project apps/macos/NexthrumMac.xcodeproj -scheme NexthrumMac -destination 'platform=macOS,arch=arm64' CODE_SIGNING_ALLOWED=NO test`。
- SwiftUI 通过稳定 Rust bridge 使用业务接口，不得绕过 Core 操作 SQLite。当前未配置正式签名与公证。

## 4. 代码边界

- 领域规则放 Rust Core，不放 React command handler、Swift View 或平台通知回调。
- Gotify 网络客户端不直接依赖 Tauri、SwiftUI 或系统通知。
- SQLite 通过 repository 访问，schema 变更必须有 migration 和兼容测试。
- 平台能力由 Windows/macOS adapter 实现；Core 通过事件/接口与宿主交互。
- DTO/API 变更同步更新架构文档和两端映射测试。
- UI 状态是缓存，持久化与业务状态以 Core 为准；操作失败不得乐观地永久改写状态。

## 5. 测试与检查

每次改动至少运行与改动范围相关的检查；功能完成前运行完整可用检查：

- Rust（workspace 根）：`cargo fmt --all --check`、`cargo test --workspace`、`cargo clippy --workspace --all-targets -- -D warnings`。
- CI：`.github/workflows/ci.yml` 在 Windows 与 macOS arm64 上执行上述检查及 TypeScript、UniFFI 生成、Xcode build/XCTest；本地提交前应至少跑通 Rust 与 TypeScript 部分。
- Windows 前端：`npm --prefix apps/windows run typecheck`、`npm --prefix apps/windows run build`。
- SQLite：临时数据库测试、迁移升级/回滚及旧用户数据兼容测试。
- Core：DND 跨午夜、重复消息、读状态、清理、连接状态和设置更新测试。
- macOS：Swift 编译及 XCTest；原生通知、菜单栏、启动项需在 macOS 环境集成验证。
- 跨平台发布前验证 Windows 与 macOS 构建，不以单平台成功代替。

## 6. 安全与数据变更

- 日志禁止输出 token、带 token 的 URL 或敏感消息内容。
- 服务器同步删除与本地删除分开实现，检查 HTTP 状态并向用户反馈。
- 更改数据目录、数据库文件名或设置格式前，审计既有安装实际路径，编写幂等迁移和备份/恢复策略。
- 新增设置要定义默认值、兼容缺省配置、平台映射和 UI 文案；凭据优先使用系统凭据存储。

## 7. 文档与开发日志

涉及需求、API、数据结构、平台行为或安装要求时，先更新对应标准文档，再实现。新增/修改 Tauri command 或 Core bridge 时更新 `docs/architecture.md` 的接口协议。完成后在当日 `devlog/YYYY-MM-DD.md` 记录完成事项、验证结果和遗留问题。

## 8. Git 提交

格式：`<type>(<scope>): <subject>`。常见 type：`feat`、`fix`、`docs`、`refactor`、`test`、`chore`。提交说明准确区分已经实现与规划中的工作。

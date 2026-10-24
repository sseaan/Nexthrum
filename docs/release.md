# Nexthrum 发布与签名手册

本文覆盖发布前必须完成的事项、当前尚未配置的部分，以及刻意做出的取舍。**没有付费证书的项目在 CI 中不会失败**，需要人工确认：Windows 无 Authenticode、macOS 无 Developer ID / 公证。

## 1. 版本号唯一来源

版本号出现在三处，发布前必须一致：

| 文件 | 字段 | 用途 |
|---|---|---|
| `apps/windows/package.json` | `version` | 基准版本（其余三处必须与之一致） |
| `apps/windows/package-lock.json` | `version` + `packages[""].version` | npm 锁文件 |
| `apps/windows/src-tauri/tauri.conf.json` | `version` | Windows 安装包/exe 版本 |
| `apps/windows/src-tauri/Cargo.toml` | `package.version` | crate 版本 |
| `apps/macos/Info.plist` | `CFBundleShortVersionString` / `CFBundleVersion` | macOS 版本与构建号 |

`ci.yml` 的 version consistency 步骤会校验 package.json / tauri.conf.json / Cargo.toml / Info.plist 四者一致，不一致直接失败。改版本后还需跑一次 `cargo check` 让 `Cargo.lock` 同步。

## 2. Windows 发布

**产物**

| 文件 | 说明 |
|---|---|
| `Nexthrum_<version>_x64-setup.exe` | NSIS 安装包（`installMode: currentUser`，无需管理员） |
| `Nexthrum_<version>_x64-portable.zip` | 便携版（与安装包同名规则） |
| `latest.json` | `tauri-action` 生成的更新清单，签名内嵌 |

**流程**

1. 同步三处版本号。
2. 本地验证：`npm --prefix apps/windows run build` 与 `cargo test --workspace`。
3. 打标签推送：`git tag vX.Y.Z && git push origin vX.Y.Z`。
4. `.github/workflows/build.yml` 在 `windows-latest` 上调用 `tauri-action`（`projectPath: apps/windows`）构建并创建 Release。
5. portable zip 步骤从 workspace 根 `target/<triple>/release/nexthrum.exe` 取二进制，命名为 `Nexthrum_<version>_x64-portable.zip`。
6. 上传完成后删除独立的 `*.sig` 资产：更新器从 `latest.json` 读取签名，单独的 `.sig` 文件对用户无意义。

**代码签名：未配置（维护者无证书，暂不计划）**

当前安装包未做 Authenticode 签名，用户首次运行会看到 SmartScreen「Windows 已保护你的电脑」提示。

- 自签名证书对公开发布**没有实际帮助**（其他人机器不信任该证书，警告依旧），因此不作为方案。
- 若将来购买 OV/EV 证书，需要在 `bundle.windows` 配置 `certificateThumbprint`、`digestAlgorithm`、`timestampUrl`，并在 CI secrets 中导入证书。
- 在此之前，发布说明必须写明：**未签名安装包**，选择「更多信息 → 仍要运行」。

## 3. macOS 发布

**现状**：仅 arm64、最低 macOS 14.0。维护者**没有付费 Apple Developer 账号**，因此使用「稳定自签身份 + 未公证 DMG + Sparkle 自动更新」；有账号时可按 3.2 的完整流程升级。

### 3.1 无开发者账号（当前路线）：稳定自签身份 + Sparkle

没有 Apple 证书不等于只能手动更新。当前采用：

**稳定自签名身份**（解决"每次重装钥匙串都重新授权"）

```bash
scripts/create-local-signing-identity.sh        # 一次性创建，私钥备份在 ~/.tauri/nexthrum-signing
scripts/package-macos-dmg.sh                    # 自动使用 "Nexthrum Local Signing" 签名

# 发布流水线里显式要求签名（默认不签名，便于本地快速构建）
CODE_SIGNING_ALLOWED=YES CODE_SIGN_IDENTITY="Nexthrum Local Signing" \
  scripts/build-macos-xcode.sh
```

发布流的 `generate_appcast` 用 `--ed-key-file` 直接读私钥文件，**不导入钥匙串**（runner 上导入会触发无法应答的授权弹窗）。

- 签名稳定 → designated requirement 不变 → Keychain「始终允许」与通知/登录项授权不再反复重置
- CI 通过 `MACOS_SIGNING_CERT_P12` / `MACOS_SIGNING_CERT_PASSWORD` 两个 secret 导入同一身份
- Gatekeeper 限制仍在：首次从 DMG 安装需右键打开或 `xattr -dr com.apple.quarantine`

**Sparkle 自动更新**（与 Apple 签名无关）

- `Sparkle.framework` 以二进制形式内嵌在 `apps/macos/Vendor/`（当前 2.10.0）
- `Info.plist`：`SUFeedURL` 指向 Release 的 `appcast.xml`、`SUPublicEDKey` 为 EdDSA 公钥、`SUEnableAutomaticChecks`
- `CFBundleVersion` 必须单调递增（当前 `2000`，每次发布递增）
- 更新包由 `generate_appcast` 用 EdDSA 私钥签名，客户端校验后才安装；私钥在登录钥匙串，导出备份为 `~/.tauri/sparkle-ed25519.key`，CI 用 `SPARKLE_EDDSA_PRIVATE_KEY`
- 本地已验证完整链路：appcast → 下载 → `EdDSA signature is correct for update` → 版本由 2000 升到 2001
- Release 产物：`Nexthrum-<version>-arm64.dmg`（人工安装，拖入 Applications）与 `Nexthrum-<version>.zip`（Sparkle 使用的应用包），清单为 `appcast.xml`

**从 `NexthrumMac.app` 迁移**：2.0 早期构建的 bundle 名为 `NexthrumMac.app`，现在统一为 `Nexthrum.app`。升级时先手动删除旧的 `NexthrumMac.app`，否则 `/Applications` 里会同时存在两份；`~/Library/Application Support/Nexthrum`、Keychain 条目与 bundle id（`com.nexthrum.app`）都不变，设置与凭据会沿用。

### 3.2 有开发者账号时的完整流程

1. **签名**：`DEVELOPMENT_TEAM=<TeamID>`、`CODE_SIGN_IDENTITY="Developer ID Application: ..."`，开启 hardened runtime。
2. **公证**：`xcrun notarytool submit <dmg> --keychain-profile <profile> --wait`。
3. **装订**：`xcrun stapler staple <dmg>`，再用 `spctl --assess --type execute --verbose=4` 验证返回 accepted。
4. **嵌套代码**：先签 `libnexthrum_ffi.dylib`，再签 app，保持 `@rpath` 引用。

### 3.3 macOS 测试

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer scripts/build-macos-xcode.sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project apps/macos/Nexthrum.xcodeproj -scheme Nexthrum \
  -destination 'platform=macOS,arch=arm64' CODE_SIGNING_ALLOWED=NO test
```

Keychain 集成测试默认跳过，需要编译期开关（运行期环境变量不会传给 `xctest` 进程）：

```bash
... test SWIFT_ACTIVE_COMPILATION_CONDITIONS='$(inherited) NEXTHRUM_KEYCHAIN_TESTS'
```

## 4. 更新策略

**已接入 `tauri-plugin-updater`（Windows）**，走 GitHub Releases：

- 清单：`https://github.com/sseaan/Nexthrum/releases/latest/download/latest.json`（`tauri-action` 在打 tag 时自动生成并上传）
- 签名：更新包用 minisign 私钥签名，应用用 `tauri.conf.json` 里的公钥校验；**签名与操作系统代码签名无关**，未做 Authenticode 也能安全更新
- 私钥位置：`~/.tauri/nexthrum-updater.key`（本机备份），CI 里为 `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 两个 secret
- 应用启动约 4 秒后检查一次；发现新版本显示非阻塞横幅，可「下载更新 → 立即重启」，也可「暂不更新」
- 失败（离线、清单缺失）静默回到 idle，不影响使用

**轮换密钥**：`npx tauri signer generate -w ~/.tauri/nexthrum-updater.key` → 更新 `tauri.conf.json` 的 `plugins.updater.pubkey` → 重新设置上面两个 secret。已发布版本的旧签名会失效，需要发一个新版本让用户过渡。

**macOS**：通过 Sparkle 自动更新（见 3.1），与 Windows 的 `tauri-plugin-updater` 各自使用独立清单与密钥。

## 5. 发布说明结构

`.github/workflows/build.yml` 的 `releaseBody` 是发布说明模板，**只保留四段**，其余内容（长段安装教程、双语长文、已知限制清单）不写进 Release：

| 段落 | 内容 |
|---|---|
| 简介 | 一句话定位：Windows/macOS 双前端 + 共享 Rust 内核；核心能力（实时消息、来源筛选、搜索、未读快照） |
| 下载文件 | 表格逐项说明每个产物的用途（安装包 / 便携版 / DMG / Sparkle zip / 更新清单），并用引用块写清未签名与未公证的首次运行提示 |
| 系统要求 | Windows 版本与 WebView2、macOS 版本与架构、服务器与令牌前提 |
| 本版亮点 | 面向用户的改动要点，按平台分组，不写内部实现细节 |

## 6. 发布前检查清单


- [ ] 三处版本号一致，`ci.yml` 双平台检查通过
- [ ] `cargo test --workspace`、`npm --prefix apps/windows run typecheck`、`npm --prefix apps/windows run build` 通过
- [ ] Windows：安装包可在干净环境安装/卸载，托盘、通知、开机启动、关闭到托盘行为正常
- [ ] Windows：凭证保存在 Windows Credential Manager，`settings.json` 不含 token
- [ ] macOS：`scripts/build-macos-xcode.sh` 通过（arm64、`@rpath`、plist、AppIcon 已入包）
- [ ] macOS：`xcodebuild test` 全部通过（Keychain 用例默认 skip，需要时用编译开关单独跑）
- [ ] macOS：`scripts/package-macos-dmg.sh` 产出 DMG，`codesign -dv` 显示 `Authority=Nexthrum Local Signing`
- [ ] macOS 实机：通知授权/拒绝、点击通知跳转、Launch at login 系统审批、退出清理连接
- [ ] macOS：Keychain 集成测试通过（`SWIFT_ACTIVE_COMPILATION_CONDITIONS='$(inherited) NEXTHRUM_KEYCHAIN_TESTS'`，见 `apps/macos/Tests/KeychainCredentialStoreTests.swift`）
- [ ] Release 资产齐全：`Nexthrum_<v>_x64-setup.exe`、`Nexthrum_<v>_x64-portable.zip`、`Nexthrum-<v>-arm64.dmg`、`Nexthrum-<v>.zip`、`latest.json`、`appcast.xml`，且没有 `*.sig`
- [ ] `latest.json` 含 `windows-x86_64` / `windows-x86_64-nsis` 条目与签名；`appcast.xml` 含 `sparkle:version` 与 `edSignature`
- [ ] 发布说明符合第 5 节四段结构（简介 / 下载文件 / 系统要求 / 本版亮点），未签名与未公证提示写在「下载文件」段

## 7. 测试版构建（不进入更新通道）

UI 改动需要在真实 Windows 机器上验收，但正式发布链路（tag → `build.yml` → `latest.json`/`appcast.xml`）不该为每次试验重打版本号。
`.github/workflows/test-build.yml` 专门解决这件事：

**触发方式**

| 方式 | 行为 |
|---|---|
| 推送到 `test/**` 分支 | 自动构建；改动涉及 `apps/windows/**`、`crates/**` 或该工作流本身时触发 |
| Actions → *Test build (Windows)* → Run workflow | 手动构建，可选是否生成预发布 |

**产物与命名**

| 文件 | 说明 |
|---|---|
| `Nexthrum_<version>_x64-test.<sha7>-setup.exe` | NSIS 安装包（复用正式版安装器配置） |
| `Nexthrum_<version>_x64-test.<sha7>-portable.zip` | 便携版，解压即用，推荐测试用 |

- 每次构建同时上传 Actions artifact（保留 14 天），并创建/更新**预发布** `test-<sha7>` 附加两个文件。
- 版本号保持 `package.json` 里的正式版本，**测试版不改版本号**；区分只看 tag 与文件名后缀。
- 预发布不会出现在 `releases/latest`，因此 `latest.json` 与 Sparkle `appcast.xml` 均不受影响，已安装的正式版**不会**被升级到测试版。
- 与正式包一样未做 Authenticode 签名，首次运行需在 SmartScreen 选择「更多信息 → 仍要运行」。

**验证清单**：`docs/windows-testing.md` 第 5 节（UI）与第 6 节（安装包：通知身份、自启动、凭据、卸载）。

**清理**：预发布可随时删除；`gh release list --limit 50` 查看，`gh release delete test-<sha7> --cleanup-tag --yes` 删除。

## 8. 相关文件

- `.github/workflows/ci.yml`：每次推送的双平台检查
- `.github/workflows/build.yml`：标签触发的双平台发布（Windows 包 + macOS DMG/zip/appcast）
- `.github/workflows/test-build.yml`：Windows 测试版构建（`test/**` 推送或手动触发，产出预发布）
- `apps/windows/src-tauri/tauri.conf.json`：bundle 与安装器配置
- `apps/macos/Nexthrum.xcconfig`：macOS 架构、部署目标与默认签名开关
- `apps/macos/Vendor/Sparkle.framework`：macOS 自动更新框架
- `scripts/build-macos-xcode.sh`：本地构建与产物校验
- `scripts/package-macos-dmg.sh`：稳定自签身份签名 + DMG 打包
- `apps/macos/Vendor/Sparkle.framework`：macOS 自动更新框架（2.10.0）
- `scripts/generate-icons.sh` / `scripts/icon_normalize.py`：图标重生成与留白归一

# Nexthrum 发布与签名手册

本文覆盖发布前必须完成的事项、当前尚未配置的部分，以及刻意做出的取舍。**尚未完成的项目在 CI 中不会失败**，需要人工确认。

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
| `Nexthrum_portable.zip` | 便携版 |

**流程**

1. 同步三处版本号。
2. 本地验证：`npm --prefix apps/windows run build` 与 `cargo test --workspace`。
3. 打标签推送：`git tag vX.Y.Z && git push origin vX.Y.Z`。
4. `.github/workflows/build.yml` 在 `windows-latest` 上调用 `tauri-action`（`projectPath: apps/windows`）构建并创建 Release。
5. portable zip 步骤从 workspace 根 `target/<triple>/release/nexthrum.exe` 取二进制。

**代码签名：未配置（维护者无证书，暂不计划）**

当前安装包未做 Authenticode 签名，用户首次运行会看到 SmartScreen「Windows 已保护你的电脑」提示。

- 自签名证书对公开发布**没有实际帮助**（其他人机器不信任该证书，警告依旧），因此不作为方案。
- 若将来购买 OV/EV 证书，需要在 `bundle.windows` 配置 `certificateThumbprint`、`digestAlgorithm`、`timestampUrl`，并在 CI secrets 中导入证书。
- 在此之前，发布说明必须写明：**未签名安装包**，选择「更多信息 → 仍要运行」。

## 3. macOS 发布

**现状**：仅 arm64、最低 macOS 14.0。维护者**没有付费 Apple Developer 账号**，因此默认走「ad-hoc 签名 + 未公证 DMG」路线；有账号时可按下面的完整流程升级。

### 3.1 无开发者账号（当前路线）

```bash
scripts/package-macos-dmg.sh            # 版本号默认取 package.json
```

脚本会：Release 构建 → `CODE_SIGN_IDENTITY=-`（ad-hoc 签名）→ 校验 app 与内嵌 `libnexthrum_ffi.dylib` 签名 → 打包 DMG（含 `/Applications` 快捷方式）。

能力与限制：

| 项目 | 状态 |
|---|---|
| ad-hoc 签名 | ✅ 有（`codesign -dv` 显示 `Signature=adhoc`） |
| Gatekeeper 首次打开 | ❌ 提示「无法验证开发者」 |
| 公证/stapling | ❌ 不可能（需要付费账号） |
| 收件人安装方式 | 右键 → 打开，或 `xattr -dr com.apple.quarantine /Applications/NexthrumMac.app` |

因此 **内部/自用分发可行，公开发布体验较差**，必须在发布说明中写明上述安装步骤。

**关于钥匙串重复授权**：ad-hoc 签名每次构建都会变化，macOS 会把新构建当成另一个应用，从而反复询问「允许访问钥匙串」。若希望「始终允许」长期有效，可在本机创建稳定的自签名身份：

```bash
scripts/create-local-signing-identity.sh
SIGN_IDENTITY="Nexthrum Local Signing" scripts/package-macos-dmg.sh
```

该身份同样不是 Developer ID，Gatekeeper 提示依旧存在，但签名与钥匙串 ACL 保持稳定。

### 3.2 有开发者账号时的完整流程

1. **签名**：`DEVELOPMENT_TEAM=<TeamID>`、`CODE_SIGN_IDENTITY="Developer ID Application: ..."`，开启 hardened runtime。
2. **公证**：`xcrun notarytool submit <dmg> --keychain-profile <profile> --wait`。
3. **装订**：`xcrun stapler staple <dmg>`，再用 `spctl --assess --type execute --verbose=4` 验证返回 accepted。
4. **嵌套代码**：先签 `libnexthrum_ffi.dylib`，再签 app，保持 `@rpath` 引用。

### 3.3 macOS 测试

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer scripts/build-macos-xcode.sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project apps/macos/NexthrumMac.xcodeproj -scheme NexthrumMac \
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

**macOS**：updater 目前只配置给 Windows；macOS 仍是手动更新（见 3.1）。

## 5. 发布前检查清单

- [ ] 三处版本号一致，`ci.yml` 双平台检查通过
- [ ] `cargo test --workspace`、`npm --prefix apps/windows run typecheck`、`npm --prefix apps/windows run build` 通过
- [ ] Windows：安装包可在干净环境安装/卸载，托盘、通知、开机启动、关闭到托盘行为正常
- [ ] Windows：凭证保存在 Windows Credential Manager，`settings.json` 不含 token
- [ ] macOS：`scripts/build-macos-xcode.sh` 通过（arm64、`@rpath`、plist、AppIcon 已入包）
- [ ] macOS：`xcodebuild test` 全部通过（Keychain 用例默认 skip，需要时用编译开关单独跑）
- [ ] macOS：`scripts/package-macos-dmg.sh` 产出 DMG，`codesign -dv` 显示 adhoc
- [ ] macOS 实机：通知授权/拒绝、点击通知跳转、Launch at login 系统审批、退出清理连接
- [ ] macOS：`NEXTHRUM_KEYCHAIN_TESTS=1` 下 Keychain 集成测试通过（见 `apps/macos/Tests/KeychainCredentialStoreTests.swift`）
- [ ] 发布说明包含未签名/未公证提示与系统要求

## 6. 相关文件

- `.github/workflows/ci.yml`：每次推送的双平台检查
- `.github/workflows/build.yml`：标签触发的 Windows 发布
- `apps/windows/src-tauri/tauri.conf.json`：bundle 与安装器配置
- `apps/macos/NexthrumMac.xcconfig`：macOS 架构、部署目标与签名开关
- `scripts/build-macos-xcode.sh`：本地构建与产物校验
- `scripts/package-macos-dmg.sh`：ad-hoc 签名 + DMG 打包
- `scripts/generate-icons.sh` / `scripts/icon_normalize.py`：图标重生成与留白归一

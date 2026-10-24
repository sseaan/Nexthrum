# 更新历史

本文件记录已经完成并归档的规划与决策，避免它们留在标准文档里误导后续开发。**当前该怎么做，看“现行文档”那一列；想知道为什么这么做，看这里。**

## 1. 版本历史

| 版本 | 日期 | 内容 |
|---|---|---|
| **2.0.1** | 2026-10-24 | Windows 界面重做：竖版窗口 400×680（最小 340×420）、类 macOS 扁平视觉、标题栏含 Logo 与最小化/最大化/关闭、主题切换独立按钮、消息行三档层级（标题/正文/来源徽标）、设置页分组；macOS 新增隐藏程序坞图标与界面语言、设置页「更新」分组（自动检查 + 检查更新 + 上次检查时间）；构建工具链迁移到 `macos-26` runner 与 macOS 26 SDK；Tauri 插件版本成对锁定并接入 CI 校验；vitest 升到 4.1.12 |
| 2.0.0 | 2026-10-23 | 首个跨平台版本：Windows（Tauri 2 + React）与 macOS（SwiftUI）共用 Rust 内核；卡片式列表、搜索、来源筛选、托盘未读红点、菜单栏面板、凭据入系统凭据库、双平台自动更新（`latest.json` / `appcast.xml`）、AGPL-3.0 |
| 1.0.1 | 2026-07-02 | 仅 Windows 的维护版本 |
| 1.0.0 | 2026-07-02 | 仅 Windows 的首个版本 |

## 2. 架构重构回顾（原 `docs/refactor-plan.md`）

**目标**：在保留 Windows Tauri/React 应用的前提下新增 SwiftUI 原生 macOS 应用。做法是把 Gotify 协议、消息用例与存储抽成无 UI 依赖的 Rust Core，两个平台各自只实现 UI 与系统集成——**不一次性全量重写**。

**方案比较**

| 方案 | 结论 |
|---|---|
| Tauri 同时用于 macOS | 复用度最高，但不满足 SwiftUI 原生要求 → 放弃 |
| Swift 全量重写业务 | 两套协议/存储/重连规则会长期分叉 → 放弃 |
| Rust Core + 双平台原生 UI | 业务复用 + 原生体验 → 采用 |

**阶段与结果**

| 阶段 | 内容 | 结果 |
|---|---|---|
| 0 | 基线与安全：日志脱敏、URL 参数编码、设置保存原子性、连接状态事件、数据路径审计 | 完成 |
| 1 | 业务解耦：命令/状态/启动编排从 `lib.rs` 拆出；消息用例与 WebSocket 循环分离；平台能力走边界接口 | 完成 |
| 2 | 持久化与测试：SQLite migration、事务边界、仓储测试；去重、DND 跨午夜、保留清理、重连配置变化；本地 mock HTTP/WebSocket | 完成 |
| 3 | Swift bridge 验证：UniFFI DTO、服务生命周期、异步事件/取消、错误映射 | 完成（UniFFI 胜出，未走 C ABI） |
| 4 | macOS vertical slice：连接设置、实时收消息、SQLite、列表、菜单栏、通知授权与点击跳转 | 完成；通知/启动项/退出清理的实机验收为长期跟踪项 |
| 5 | 发布治理：双平台构建、签名、DMG/ZIP、更新策略 | 完成（自签 + Sparkle；Apple 公证与 Windows Authenticode 仍缺证书） |

**当时识别并已解决的风险**

- Tauri `AppHandle` 渗入网络与业务 → Core 与 Gotify client 均不依赖 Tauri
- 凭据暴露 → token 移入凭据管理器/钥匙串，`settings.json` 只存非敏感字段，日志脱敏
- 并发语义 → 连接更新与消息变更各有 host mutex；设置先持久化再重连
- 未读数漂移 → 后端权威快照，mutation command 直接返回 `MessageSnapshot`
- 服务器删除失败被静默忽略 → 先校验 HTTP 结果，失败不清本地
- SQLite 迁移兼容 → 事务式 migration + 早期 schema 列对齐
- 跨语言契约漂移 → 稳定 DTO/错误分类 + 契约测试

**延续至今的约束**（已写入 `AGENTS.md` 与 `docs/tech-spec.md`）：业务只在 Core、UI 不接触存储、后端拥有状态权威、凭据不进 JSON、连接与消息分别串行化、迁移必须向后兼容。

## 3. Windows 界面重做回顾（原 `docs/windows-ui-redesign-plan.md`）

**触发问题**：截图显示内容贴边、标题栏缺少最大化与 Logo、设置入口找不到、视觉“丑陋且像通用模板”。

**根因（两条，都不是“样式没调好”）**

1. **CSS 层叠**：`styles/index.css` 里未分层（unlayered）的 `* { margin: 0; padding: 0 }`、`button {...}`、`input,select,textarea {...}` 压过了 Tailwind v4 的 `@layer utilities`——未分层样式无视优先级。结果全站 `px-*`/`py-*`/`space-y-*`/边框/字号工具类全部失效。
2. **链接 SDK**：macOS 发布构建跑在 `macos-14` runner 上（macOS 14.5 SDK），新系统上会退回旧界面设计，看起来“和以前不一样”。

**设计决策**

| 决策 | 取值 | 原因 |
|---|---|---|
| 窗口形态 | 竖版 400×680，最小 340×420 | 消息是单列时间流，宽屏浪费横向空间 |
| 视觉语言 | 类 macOS 扁平：白色内容面 + 1px 分割线 + 大留白，去卡片描边与阴影 | 避免通用模板感 |
| 标题栏 | 34px，Logo + 产品名 + 最小化/最大化-还原/关闭，双击最大化 | 无边框窗口必须自备三按钮 |
| 头部动作 | 重新连接（图标）、主题切换（图标）、设置（纯文字） | 齿轮图标会被误认为主题/系统设置 |
| 消息行 | 扁平行 + 分割线；未读用圆点 + 字重 + 强调色时间 | 不靠色块表达状态 |
| 行内层级 | 标题 13/600、正文 12 muted、来源与优先级 10px 徽标 | 让元信息不再像正文 |
| 主题 | 深色 / 白天 / 跟随系统，纯 Token 驱动 | 组件禁止硬编码颜色 |

**实施阶段**：布局止血与标题栏 → 主页面与设置入口 → 视觉组件与主题 → 验收工具链（`test-build.yml` 预发布）→ 层叠根因修复 → 竖版 + 扁平重做 → 窗口收紧与行层级微调。

**验收数据（可用于回归对比）**

- 800×600@125% 引导页图标由 `x=0` 变为 `x=165`（`max-w-xl` 居中 112 + `px-5` 25）
- 460×760、340×420 下均无横向溢出；头部标题与列表内容左边缘同为 `x=12`
- 2.0.1 发布构建 `DTSDKName=macosx26.5`，窗口 837×512@2x 下内容面 `rgb(255,255,255)`、侧边栏分界 287px

**留下的长期约束**：基样式必须写进 `@layer base`；视觉补偿不要用固定 offset（会随 SDK 变化）；发布用较新 runner 并通过 `DTSDKName` 校验；主题只走语义 Token；组件复用 `apps/windows/src/components/ui/`。

## 4. 已归档文档

| 原文件 | 归档原因 | 内容去向 |
|---|---|---|
| `docs/refactor-plan.md` | 迁移阶段全部完成，且描述的是“从单平台到双平台”的过渡状态 | 本文第 2 节；长期约束已进 `AGENTS.md`、`docs/tech-spec.md` |
| `docs/windows-ui-redesign-plan.md` | UI 重做已交付并验收，规划细节不再作为执行依据 | 本文第 3 节；现行视觉规则见 `docs/design-spec.md` |

## 5. 现行文档

| 想看什么 | 看哪里 |
|---|---|
| 产品需求与验收标准 | `docs/requirements.md` |
| 模块职责、事件、持久化、更新通道 | `docs/architecture.md` |
| 分层规则、依赖边界、质量门槛 | `docs/tech-spec.md` |
| 视觉与交互规范（含 Windows 头部按钮语义、主题、行内层级） | `docs/design-spec.md` |
| 发版、签名、更新、测试版 | `docs/release.md` |
| 环境搭建与开发流程、依赖版本约束 | `docs/development-guide.md` |
| Windows 真机测试清单 | `docs/windows-testing.md` |
| 面向自动化助手的工作约定 | `AGENTS.md` |
| 每日进展与遗留问题 | `devlog/` |

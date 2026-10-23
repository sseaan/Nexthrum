# Windows 端 UI 重构与视觉优化规划

> 状态：阶段 1–3 已实施（详见文末「实施进展」）  
> 适用范围：`apps/windows/src`、`apps/windows/src-tauri` 以及 Windows 端设计规范  
> 目标：在不改动已验证的 Rust 业务内核、WebSocket 和系统通知链路的前提下，修复 Windows 前端的布局、窗口壳层、视觉质量和设置入口问题。

## 1. 背景与问题结论

测试截图位于 `screenshot/`。当前后端消息持久化、WebSocket 连接和系统通知推送已经验证基本正常，因此本轮优先处理 Windows 前端表现层。

### 1.1 已观察到的问题

1. **布局错位**：标题、引导页文案和消息内容出现左侧贴边或裁切；输入框、卡片和操作区在不同窗口尺寸下缺少稳定的边距与收缩规则。
2. **标题栏不完整**：当前自定义标题栏只有最小化和关闭按钮，没有最大化/还原按钮；左侧没有 Logo 和产品名称；窗口装饰关闭后，系统原生标题栏提供的行为也需要由应用自行补齐。
3. **视觉层级不足**：标题栏、页面头部、输入区域、消息卡片和背景之间的层级不清楚；图标按钮难以识别；表单控件、按钮、间距和状态样式缺乏统一规范。
4. **设置入口不易发现**：代码中的设置页面和路由仍然存在，但消息页入口是无文字的小图标，视觉上容易被误认为刷新或主题按钮；引导页之外缺少明显的配置导航。
5. **品牌资产不完整**：`apps/windows/index.html` 仍引用 Vite 默认 favicon，标题栏也没有 Nexthrum 品牌标识。

### 1.2 代码现状证据

- `apps/windows/src/components/common/TitleBar.tsx`：仅实现最小化、关闭和拖动。
- `apps/windows/src-tauri/tauri.conf.json` 与 `src-tauri/src/lib.rs`：使用无边框窗口，`decorations: false`，因此最大化、Logo 和标题均需明确实现。
- `apps/windows/src/components/MessageWindow/MessageHeader.tsx`：设置按钮仍存在，但为纯图标且识别度不足。
- `apps/windows/src/components/SettingsWindow/SettingsWindow.tsx`：设置页面仍可用。
- `apps/windows/src/components/OnboardingWindow/OnboardingView.tsx`：首次引导页仍提供“高级设置”。
- `apps/windows/src/store/useAppStore.ts`：仍有 `currentView: "messages" | "settings"`。
- `apps/windows/index.html`：仍引用 `/vite.svg`。

因此，“设置界面消失”更准确地说是**入口可发现性和信息架构表现不合格**，不是后端设置能力丢失。

## 2. 重构目标与非目标

### 2.1 目标

- 消除左侧裁切、横向溢出和窗口尺寸变化导致的错位。
- 建立可适配 400×300、800×600、最大化及高 DPI 的页面布局。
- 完成带 Logo、软件名和三按钮的 Windows 自定义标题栏。
- 使消息页、设置页、引导页拥有一致的页面头部和控件规范。
- 让设置入口在消息页、托盘和引导页均清晰可见。
- 同时支持深色和白天/浅色视觉主题。
- 保留现有 Core、Storage、Gotify、通知和后端权威状态设计。
- 通过固定截图矩阵完成回归验收。

### 2.2 非目标

- 本轮不重新设计 Rust 消息业务规则。
- 本轮不修改通知推送协议和 DND 业务策略。
- 本轮不把存储或凭据逻辑移入前端。
- 本轮不追求 Windows 与 macOS 逐像素一致，只保持产品语义一致。

## 3. Windows 视觉方向

### 3.1 总体原则

- 消息优先：列表内容比装饰更重要。
- 结构清晰：标题栏、页面头部、筛选/搜索区、列表和状态提示必须有明显层级。
- 控件可发现：主要功能不只依赖不可识别的图标。
- Windows 友好：窗口按钮、拖动、焦点、键盘导航和高 DPI 行为符合桌面应用习惯。
- 主题一致：深色和白天模式使用同一套语义 Token，仅替换颜色和阴影表现。
- 可访问：状态不能只靠颜色表达；图标按钮必须有 tooltip、`aria-label` 和键盘焦点。

### 3.2 深色模式

深色模式延续现有蓝紫色基线，但进行层级整理：

| Token | 建议值 | 用途 |
|---|---|---|
| `--color-bg` | `#111827` | 主窗口背景 |
| `--color-surface` | `#18243d` | 标题栏、页面头部、表单分组 |
| `--color-surface-raised` | `#202f4d` | 消息卡片、弹出层 |
| `--color-surface-hover` | `#2a3d60` | 悬停和次级控件 |
| `--color-border` | `#344766` | 普通分隔线 |
| `--color-border-strong` | `#496184` | 聚焦或强调边框 |
| `--color-primary` | `#4f8df7` | 主按钮、未读状态、焦点 |
| `--color-primary-hover` | `#3678e8` | 主色悬停 |
| `--color-text` | `#edf3ff` | 主文本 |
| `--color-text-secondary` | `#aab8cf` | 次要文本 |
| `--color-text-muted` | `#71819b` | 辅助信息 |
| `--color-danger` | `#f87171` | 错误和关闭悬停 |
| `--color-success` | `#34d399` | 已连接和成功 |
| `--color-warning` | `#fbbf24` | 警告 |

深色模式不使用纯黑背景，也不使用大面积高饱和蓝色。主色只用于交互、未读和关键状态。

### 3.3 白天/浅色模式

白天模式不是简单地把深色背景替换成白色，而是要保持窗口层级、边界和品牌强调的一致性。建议默认跟随 Windows 系统主题，后续在设置中提供“跟随系统 / 深色 / 白天”选择。

| Token | 建议值 | 用途 |
|---|---|---|
| `--color-bg` | `#f3f6fb` | 主窗口背景，避免刺眼纯白 |
| `--color-surface` | `#ffffff` | 标题栏、页面头部和表单分组 |
| `--color-surface-raised` | `#ffffff` | 消息卡片和弹出层 |
| `--color-surface-hover` | `#eaf0f8` | 控件和卡片悬停背景 |
| `--color-border` | `#d7e0ec` | 普通分隔线和控件边框 |
| `--color-border-strong` | `#b7c6da` | 聚焦、选中和强调边框 |
| `--color-primary` | `#2563c7` | 主按钮、链接、未读状态 |
| `--color-primary-hover` | `#1d4fa3` | 主色悬停 |
| `--color-text` | `#172033` | 主文本 |
| `--color-text-secondary` | `#53627a` | 次要文本 |
| `--color-text-muted` | `#7b879a` | 辅助信息 |
| `--color-danger` | `#c93636` | 错误和关闭悬停 |
| `--color-success` | `#16804d` | 已连接和成功 |
| `--color-warning` | `#a86400` | 警告 |

白天模式具体要求：

- 页面背景使用冷灰蓝，不使用大面积纯白，降低长时间阅读的视觉疲劳。
- 标题栏与页面头部使用白色，依靠 1px 边框和轻微阴影区分层级。
- 消息卡片使用白色或近白色，边框必须足够清晰，不能只靠阴影区分。
- 主文本与背景保持至少 WCAG AA 对比度；次要文本不能过浅。
- 未读消息使用浅蓝底、蓝色竖条、圆点和加粗标题，不能只依赖浅色背景。
- 错误、成功、警告颜色在浅色背景上需要使用较深的文本色，并配合图标或标签。
- 输入框获得焦点时使用蓝色外环，而不只是改变边框颜色。
- 按钮悬停、按下和禁用状态必须可见，禁用状态不能仅使用低透明度。
- 弹窗和上下文菜单使用白色背景、明确边框和柔和阴影，避免与页面背景融为一体。
- 系统主题切换时不应改变布局尺寸，避免页面跳动。

## 4. 页面与组件规划

### 4.1 根布局

涉及：

- `apps/windows/src/App.tsx`
- `apps/windows/src/styles/index.css`
- `apps/windows/src/components/MessageWindow/*`
- `apps/windows/src/components/SettingsWindow/*`
- `apps/windows/src/components/OnboardingWindow/*`

计划：

1. 统一根节点高度和宽度计算。
2. 建立页面级 `min-width: 0`、`min-height: 0` 规则。
3. 建立统一内容容器，保证左右内边距和最大宽度。
4. 明确只有消息列表和设置正文区域滚动。
5. 排查固定宽度、flex 子项收缩、横向溢出、DPI 缩放和旧构建产物。
6. 验证 400×300、800×600、1200×900、最大化窗口和 125%/150% DPI。

验收：没有左侧裁切、横向滚动条或控件被挤出窗口。

### 4.2 自定义标题栏

涉及：

- `apps/windows/src/components/common/TitleBar.tsx`
- `apps/windows/src-tauri/src/lib.rs`
- `apps/windows/src-tauri/tauri.conf.json`

推荐保留自定义标题栏，但完整实现：

```text
[Logo] Nexthrum                                      [最小化][最大化][关闭]
```

工作项：

- 使用 Nexthrum Logo 和软件名。
- 增加最大化/还原按钮。
- 同步窗口最大化状态。
- 支持双击标题栏最大化/还原。
- 拖动区域与按钮区域严格分离。
- 按钮提供 hover、pressed、disabled 和关闭危险态。
- 增加 tooltip、`aria-label` 和键盘操作。
- 验证高 DPI、窗口恢复和最小化到托盘行为。

备选方案是恢复原生标题栏。如果自定义标题栏在 Windows 原生行为、高 DPI 或无障碍方面持续不稳定，应优先切换原生装饰，而不是继续堆叠补丁。

### 4.3 消息头部

涉及：`apps/windows/src/components/MessageWindow/MessageHeader.tsx`

目标结构：

```text
消息    [未读数]                    [刷新] [设置]
[搜索消息                                                    ]
[连接状态] · [缓存数量]                         [全部清空]
```

工作项：

- 设置按钮默认显示“设置”文字，窄窗口再退化为图标。
- 使用标准齿轮图标，不能使用难以辨认的太阳形图标。
- 搜索框、状态栏和操作区在小窗口下不互相挤压。
- 连接状态同时显示点、文本和错误提示。
- 统一头部背景、边框和内边距。

### 4.4 消息卡片

涉及：`MessageItem.tsx`、`MessageList.tsx`

工作项：

- 保持 10px 圆角、8px 间距和稳定内边距。
- 标题、时间、正文、来源和优先级使用明确网格或 flex 区域。
- 长标题和正文必须截断或换行，不得撑破窗口。
- 未读状态保留竖条、圆点和字重三重提示。
- 已读状态降低对比度但仍保持可读性。
- 浅色模式下使用清晰边框，不依赖阴影。

### 4.5 设置页

涉及：`SettingsWindow.tsx`、`i18n/en-US.ts`、`i18n/zh-CN.ts`

工作项：

- 设置页使用与主页面一致的标题栏和页面头部。
- 返回按钮使用明确的返回图标 + 文案。
- 分组采用卡片或清晰分隔线，避免所有字段堆在同一块区域。
- 服务器连接、通用行为、语言、免打扰分别成组。
- 保存按钮固定为主操作，测试连接为次操作。
- 成功、失败和“设置已保存但连接失败”必须有清晰反馈。
- 保留现有凭据不进入 JSON 的安全约束。

### 4.6 引导页

工作项：

- 保留三步引导，但调整为更稳定的纵向布局。
- 输入框、测试、保存、跳过和高级设置形成清楚的主次关系。
- “高级设置”使用次级按钮或链接按钮，但必须明显可发现。
- 首次引导和普通设置页共享输入框、按钮和状态提示组件。

## 5. 设置入口与视图架构

### 5.1 入口要求

设置至少可以从以下路径打开：

1. 消息页头部的“设置”按钮。
2. Windows 托盘菜单。
3. 首次引导页的“高级设置”。

消息页中的设置按钮不得仅依赖一个无文字图标；必须具备明确图形、tooltip、`aria-label` 和键盘焦点。

### 5.2 统一视图切换

当前 `App.tsx` 和 `MessageWindow.tsx` 都判断 `currentView === "settings"`，计划统一为：

```text
App
 ├── TitleBar
 └── MainView
     ├── MessageWindow
     └── SettingsWindow
```

避免消息页同时承担路由职责，为未来增加关于、更新和日志页面预留结构。

## 6. 实施阶段与任务分解

### Phase 0：基线确认

- 确认截图对应的前端构建版本。
- 记录当前窗口 CSS viewport、物理像素和 Windows DPI。
- 用最小窗口、默认窗口、最大化窗口分别复现问题。
- 检查 Tailwind 构建产物和运行时 CSS 是否一致。

交付物：问题复现记录和截图基线。

### Phase 1：布局止血与标题栏

- 修复根布局、内容容器和滚动边界。
- 消除横向溢出和左侧裁切。
- 完成 Logo、软件名和三按钮标题栏。
- 验证拖动、最大化、还原、最小化、关闭到托盘。

交付物：可稳定运行的窗口壳层和页面布局。

### Phase 2：主页面与设置入口

- 重构消息头部。
- 设置按钮改为明确可发现的入口。
- 重构消息卡片。
- 统一设置页和引导页的页面头部与按钮。
- 统一视图切换。

交付物：用户可以清晰浏览消息并进入设置。

### Phase 3：视觉系统和主题

- 抽取 `Button`、`IconButton`、`Card`、`TextInput`、`Section`、`StatusBadge` 等组件。
- 迁移颜色为语义 Token。
- 实现深色/白天/跟随系统主题。
- 补齐 hover、pressed、focus、disabled 和错误状态。
- 替换 Vite 默认 favicon 和品牌资产。

交付物：两套主题下结构一致、视觉层级稳定的 UI。

### Phase 4：回归验收

- 运行 TypeScript、前端测试和构建。
- 完成截图矩阵。
- 检查中文和英文布局。
- 检查高 DPI 和最大化行为。
- 更新开发日志和相关设计文档。

## 7. 验收矩阵

| 场景 | 尺寸/条件 | 重点检查 |
|---|---|---|
| 首次引导 | 800×600 | 边距、输入框、按钮主次关系 |
| 消息列表 | 800×600 | 标题栏、设置入口、消息卡片 |
| 长标题消息 | 800×600 | 换行、截断、时间位置 |
| 设置页 | 800×600 | 分组、返回、保存和测试连接 |
| 小窗口 | 400×300 | 不裁切、不横向溢出、按钮可用 |
| 最大化 | 全屏 | 内容最大宽度和留白 |
| 高 DPI | 125%/150% | 字体、标题栏和点击区域 |
| 深色模式 | 系统深色 | 对比度、状态层级 |
| 白天模式 | 系统浅色 | 边框、文字对比度、浅色状态 |
| 连接异常 | 800×600 | 错误横幅和重连入口 |
| 多来源消息 | 800×600 | 来源筛选和卡片布局 |
| 托盘入口 | 最小化/隐藏 | 打开消息、设置和退出 |

## 8. 验收标准

- [ ] 标题、输入框、卡片不会被左侧裁切。
- [ ] 不出现页面级横向滚动条。
- [ ] 标题栏有 Logo、Nexthrum、最小化、最大化/还原、关闭。
- [ ] 标题栏拖动和按钮点击互不冲突。
- [ ] 设置入口在消息页中一眼可见。
- [ ] 设置可从消息页、托盘和引导页进入。
- [ ] 深色模式和白天模式均有清晰的层级和足够的文字对比度。
- [ ] 消息已读/未读不只依赖颜色。
- [ ] 400×300、800×600、最大化和高 DPI 下布局稳定。
- [ ] 中文和英文文案不会导致主要控件溢出。
- [ ] 前端测试、类型检查和构建通过。
- [ ] 完成截图回归并更新开发日志。

## 9. 建议验证命令

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm --prefix apps/windows run typecheck
npm --prefix apps/windows test
npm --prefix apps/windows run build
```

## 10. 实施进展

### 已完成

**阶段 1：基础层**

- [x] `apps/windows/src/styles/index.css`：语义色 Token 重写（深色默认 + `[data-theme="light"]` 白天模式），新增 `--color-surface-raised`、`--color-primary-soft`、`--color-danger-soft`、`--color-success-soft`、`--color-text-muted`、`--color-overlay`、`--shadow-card/popover`
- [x] `apps/windows/src/theme/useTheme.ts`：`system / light / dark` 三态，`localStorage` 持久化（`nexthrum.theme`），`matchMedia` 跟随系统
- [x] `apps/windows/src/main.tsx`：首帧前 `initTheme()`，避免主题闪烁
- [x] 新增 `apps/windows/src/components/ui/`：`Button`、`IconButton`、`Input` + `Field`、`Section`、`Switch`、`SegmentedControl`、`PageHeader`、`EmptyState`、`icons`
- [x] `TitleBar.tsx`：Logo + 软件名 + 最小化/最大化-还原/关闭，`onResized` 同步最大化状态，双击标题栏切换，拖动区与按钮区分离
- [x] `capabilities/default.json`：补 `toggle-maximize`、`maximize`、`unmaximize`、`is-maximized` 权限
- [x] `App.tsx`：改为 `TitleBar + 单一视图出口`，`MessageWindow` 不再兼任设置页路由
- [x] 品牌资产：`index.html` 改用 `public/favicon.png`，移除 Vite 默认图标

**阶段 2：主页面**

- [x] `MessageHeader`：标题/未读胶囊/搜索/连接状态分行布局，设置入口带文字（<560px 退化为图标），错误横幅内提供「重新连接」
- [x] `MessageItem`：新 Token 卡片、未读三重提示、键盘可访问（Enter/Space）、来源标签 tooltip
- [x] `MessageList` + `EmptyState`：无消息（含「打开设置」按钮）与搜索无结果共用组件
- [x] `AppFilter`：胶囊筛选条统一 Token 与 `max-w-3xl` 容器

**阶段 3：设置页、引导页与更新横幅**

- [x] `SettingsWindow`：`PageHeader` + 返回按钮 + 分组卡片 + `Switch`/`SegmentedControl`，新增外观（主题 + 语言）分组
- [x] `OnboardingView`：`Field`/`Input`/`Button` 迁移，主次按钮分两行，窄窗口按钮换行不裁切
- [x] `UpdateBanner`：新 Token、图标与按钮，内容与 `max-w-3xl` 对齐
- [x] i18n：新增 `window.*`、`appearance.*`、`message.openSettings` 与设置项说明文案（en + zh-CN）

### 待完成

- [ ] Windows 实机验证：最大化/还原、双击标题栏、拖动、125%/150% DPI、400×300 极限尺寸
- [ ] 按第 7 节矩阵补齐深色/白天两套截图
- [ ] 如需系统主题自动切换之外的额外偏好（例如强制高对比度），再扩展设置项


# Windows 真机测试手册

> 适用：Windows 10 21H2+ / Windows 11，x64
> 目的：在真实 Windows 环境验证 UI 重构（窗口壳层、主题、布局、设置入口）与宿主能力（托盘、通知、凭据、自启动）
> 相关文档：`docs/windows-ui-redesign-plan.md`（改造规划与验收矩阵）、`docs/design-spec.md`（视觉规范）

## 1. 测试分层

| 层级 | 命令 | 能验证什么 | 不能验证什么 |
|---|---|---|---|
| 开发模式 | `npm --prefix apps/windows run tauri dev` | UI 布局、主题、标题栏按钮、DPI、交互 | 通知发送者身份、自启动注册表、安装包行为、自动更新 |
| 安装包 | `npm --prefix apps/windows run tauri build` 后运行 NSIS 安装器 | 上述全部 + 通知身份、凭据管理器、安装/卸载 | 自动更新（需要已发布版本与新版本对比） |

**结论：UI 问题在开发模式改，宿主行为必须在安装包里验。**

## 2. 环境准备

### 2.1 必需

```powershell
node -v      # >= 20 LTS
npm -v
rustc -V     # stable-x86_64-pc-windows-msvc
cargo -V
rustup show  # default host toolchain 必须是 msvc，不能是 gnu
```

缺失时：

```powershell
winget install OpenJS.NodeJS.LTS
winget install Microsoft.VisualStudio.2022.BuildTools   # 勾选「使用 C++ 的桌面开发」
rustup default stable-x86_64-pc-windows-msvc
```

WebView2 在 Win11 与 Win10 21H2+ 已内置；未装则 `winget install Microsoft.EdgeWebView2Runtime`。

### 2.2 取代码

```powershell
git clone <repo-url> Nexthrum
cd Nexthrum
git checkout test/windows-ui
npm --prefix apps/windows ci
```

不要把 macOS 上的 `node_modules`、`target/`、`dist/` 拷过来。

## 3. 启动与调试

### 3.1 开发模式

```powershell
$env:RUST_LOG="debug"            # 可选：输出后端分级日志
npm --prefix apps/windows run tauri dev
```

首轮会编译 Rust workspace，需要几分钟；之后是增量编译。终端保持打开，前后端日志都在这里。

前端 DevTools：窗口内 `Ctrl + Shift + I`（dev 构建启用）。用它检查：

- `document.documentElement.dataset.theme` 是否为 `dark` / `light`
- 是否出现 `scrollWidth > clientWidth`（横向溢出）
- 标题栏元素宽高、消息卡片内边距是否符合预期

### 3.2 安装包

```powershell
npm --prefix apps/windows run tauri build
```

产物：`apps/windows/src-tauri/target/release/bundle/nsis/Nexthrum_<version>_x64-setup.exe`。

安装到 `%LOCALAPPDATA%\Nexthrum`，安装后从开始菜单启动，按第 5 节验证宿主能力。

## 4. UI 验收清单（开发模式）

逐项勾选；发现异常时记录「场景 + 期望 + 实际 + 截图」。

### 4.1 标题栏

| # | 操作 | 期望 |
|---|---|---|
| 1 | 观察标题栏左侧 | 应用图标 + `Nexthrum` 文字 |
| 2 | 观察标题栏右侧 | 最小化、最大化、关闭三个按钮，顺序正确 |
| 3 | 点击最大化 | 窗口填满工作区，按钮图标变为「还原」 |
| 4 | 点击还原 | 回到原尺寸，图标变回「最大化」 |
| 5 | 双击标题栏空白区域 | 最大化 / 还原切换 |
| 6 | 最大化状态下再次双击 | 还原 |
| 7 | 按住标题栏空白区拖动 | 窗口跟随移动；松手后位置正确 |
| 8 | 按住窗口按钮拖动 | 窗口**不移动**，按钮正常触发 |
| 9 | 悬停关闭按钮 | 背景变危险色（红），图标变白 |
| 10 | 点击关闭（默认设置） | 窗口隐藏，托盘图标保留，连接不中断 |
| 11 | 托盘菜单 → 设置 / 消息列表 | 窗口重新显示并切到对应页面 |
| 12 | 最小化后点击任务栏图标 | 窗口恢复 |

> 若第 3–6 项图标或状态不同步，说明 `onResized` 同步失效；若第 7–9 项互相干扰，说明拖动区与按钮区未分离。

### 4.2 高 DPI

设置 → 系统 → 屏幕 → 缩放，依次切 **100% / 125% / 150%**：

| 检查点 | 期望 |
|---|---|
| 标题栏 | 视觉高度稳定，按钮点击区域不变形 |
| 中文文案 | 清晰不发虚，字体回退到 `Microsoft YaHei UI` |
| 1px 边框 | 视觉上仍为细线，不出现双线或消失 |
| 消息卡片 | 标题/正文/时间不错位，无半行裁切 |
| 设置页开关 | 拇指与轨道对齐，点击命中区域正常 |

切换缩放后建议关闭窗口再从托盘打开一次，排除 WebView2 缩放缓存。

### 4.3 极限尺寸

最小尺寸由代码固定为 400×300（`min_inner_size`），拉不到更小是正常行为。

| 尺寸 | 检查点 |
|---|---|
| 400×300 | 标题与未读胶囊不重叠；设置按钮退化为齿轮图标（阈值 560px）；搜索框不溢出；状态行「已连接 · 已缓存 N 条 · 全部清空」自动换行；消息卡片标题截断、时间仍在右侧 |
| 800×600（默认） | 设置按钮显示「设置」文字；列表左右留白一致；消息卡片间距均匀 |
| 最大化（1920/2560 宽） | 内容居中在 `max-w-3xl`（约 768px），标题栏与页面头部背景铺满，正文不被拉成超长行 |

### 4.4 主题

| # | 操作 | 期望 |
|---|---|---|
| 1 | 保持「跟随系统」，切换 Windows 深浅色 | 应用立即跟随，无需重启 |
| 2 | 设置 → 外观 → 深色 | 立即生效；重启后保持 |
| 3 | 设置 → 外观 → 白天 | 立即生效；重启后保持 |
| 4 | 设置 → 外观 → 跟随系统 | 回到跟随系统状态 |
| 5 | 白天模式：看消息卡片 | 边框可见，卡片与背景有明确层次 |
| 6 | 白天模式：看未读消息 | 浅蓝底 + 蓝色竖条 + 圆点 + 加粗标题，四重提示齐全 |
| 7 | 白天模式：看次要文字 | 仍可读，不发灰到看不清 |
| 8 | 白天模式：看错误提示 | 红字对比度足够，带警示图标 |
| 9 | 两种主题下切换页面 | 布局无位移、无高度跳动 |

### 4.5 设置入口

| # | 入口 | 期望 |
|---|---|---|
| 1 | 消息页头部「设置」按钮 | 宽窗口有文字 + 齿轮；窄窗口为齿轮 + tooltip |
| 2 | 托盘右键菜单 → 设置 | 打开设置页 |
| 3 | 首次引导页 →「高级设置…」 | 打开设置页 |
| 4 | 消息为空时的空状态 →「打开设置」 | 打开设置页 |
| 5 | 设置页 →「返回」 | 回到消息页，滚动位置合理 |

### 4.6 消息与列表

| # | 场景 | 期望 |
|---|---|---|
| 1 | 收到新消息（未读） | 列表顶部插入，未读数来自后端快照，不重复递增 |
| 2 | 点击未读卡片 | 复制内容到剪贴板，卡片转为已读，未读数下降 |
| 3 | 键盘 Tab 到卡片 + Enter/Space | 与点击行为一致，焦点环可见 |
| 4 | 右键卡片 | 上下文菜单（标记为已读 / 删除）位置跟随鼠标 |
| 5 | 搜索 | 命中结果正确；无命中显示查询词回显 |
| 6 | 多来源 | 出现胶囊筛选条；选中态高亮；来源消失后回到「全部应用」 |
| 7 | 长标题 + 长正文 | 标题单行截断，正文最多三行，时间不被挤走 |
| 8 | 全部清空 | 弹窗确认；勾选「同时删除服务器」行为正确；失败有明确报错 |

## 5. 安装包验收清单

| # | 场景 | 期望 |
|---|---|---|
| 1 | 首次安装启动 | 显示引导页，三步说明 + 内联表单 |
| 2 | 保存设置 | 令牌写入凭据管理器，`settings.json` 无明文 |
| 3 | 系统通知 | 发送者显示 `Nexthrum` 与正确图标 |
| 4 | 点击通知 | 窗口打开并定位到该消息 |
| 5 | 免打扰时段内收消息 | 无系统通知，消息仍入库并在列表中可见 |
| 6 | 开启「开机自启」并重新登录 | 静默启动（若同时开启「启动时隐藏」则无窗口） |
| 7 | 托盘红点与 tooltip | 未读数量与列表一致，读完后消失 |
| 8 | 卸载 | 安装目录清理干净；不动 `%APPDATA%\Nexthrum` 用户数据 |
| 9 | 覆盖升级安装 | 保留设置、消息与凭据 |

凭据核对路径：控制面板 → 凭据管理器 → Windows 凭据 → 目标 `Nexthrum/GotifyClientToken`。

数据路径核对：

```powershell
Get-Content "$env:APPDATA\Nexthrum\settings.json"     # 不应出现 client_token 明文
dir "$env:APPDATA\Nexthrum\nexthrum_messages.db"
```

## 6. 截图规范

- 单窗口：`Alt + PrtScn`（复制当前窗口）或 `Win + Shift + S` 框选
- 需要保留 DPI 信息：`Win + PrtScn`，整屏原分辨率存到 `图片\屏幕截图`
- 命名：`win-<主题>-<尺寸>-<场景>-<dpi>.png`，例如 `win-light-400x300-list-150dpi.png`
- 存放：仓库根目录 `screenshot/`（该目录不入库）
- 最少覆盖 8 个场景：引导页 / 消息列表 / 长标题消息 / 设置页 / 400×300 / 最大化 / 白天模式 / 连接错误态

每个场景深色、白天各一张，方便对比主题差异。

## 7. 问题反馈模板

```text
场景：设置页 / 白天 / 125% DPI / 800x600
期望：服务器令牌输入框有可见标签
实际：标签被输入框压住，只看到半行
复现：托盘 → 设置 → 观察第一组卡片
截图：screenshot/win-light-800x600-settings-125dpi.png
日志：（如有）粘贴 tauri dev 终端最后 10 行
```

不需要自行定位到代码；带场景与截图的反馈就足够修复。

## 8. 常见问题

### 8.1 窗口完全空白（优先排查）

先用**空白区域的背景色**判断问题层级：

| 看到的画面 | 含义 | 下一步 |
|---|---|---|
| 深蓝底（`#0f1724`）且无任何文字 | 前端脚本已加载，但 React 未挂载或抛错 | `Ctrl + Shift + I` 看 Console |
| 深蓝底 + 「正在加载 Nexthrum…」文字 | 前端资源根本没加载（Vite 未启动或端口不通） | 看终端与下表的连接类问题 |
| 纯白 / 纯黑，无任何内容 | WebView2 未渲染页面（运行时缺失或进程被杀） | 确认 WebView2 运行时，重启应用 |
| 深蓝底 + 「界面启动失败 / 界面未能启动」+ 堆栈 | 已捕获到具体错误 | 直接截图堆栈反馈 |

分层定位最快的一步：**绕过 Tauri，直接用浏览器验证前端**。

```powershell
cd apps\windows
npm run dev
```

然后在 Edge/Chrome 打开 `http://localhost:1520`（端口以 `DEV_PORT` 为准）：

- 浏览器能正常显示界面 → 前端没问题，问题在 Tauri 窗口或 WebView2（继续看 8.2）
- 浏览器也空白 → 前端运行时错误，浏览器 Console 会直接给出文件和行号

> 浏览器里只是**渲染预览**：标题栏会显示 `browser preview` 标记，三个窗口按钮为禁用态，
> 消息、设置保存等 Tauri IPC 调用会失败，这是预期行为，不要当作 bug 上报——浏览器只用来
> 验证布局、主题和样式。
>
> 如果在浏览器里看到 `Cannot read properties of undefined (reading 'metadata')`，说明代码里有
> 未做运行时判断就调用了 `@tauri-apps/api`（`window.__TAURI_INTERNALS__` 不存在），这属于要修的缺陷，
> 用 `isTauriRuntime()`（`src/utils/tauriEnv.ts`）做守卫。

### 8.2 端口与 Hyper-V 占用

`tauri.conf.json` 的 `devUrl` 是 `http://localhost:1430`，`vite.config.ts` 也必须用 `1430` 且保持
`strictPort: true`：两边不一致或端口被占时，窗口会指向空地址而显示空白。**不要**为了“先跑起来”把
`strictPort` 改成 `false`——Vite 会悄悄换端口，Tauri 仍加载旧端口。

> 历史上的默认端口是 1420，但 Windows 上 Hyper-V/WinNAT 会动态保留端口段，1420 经常落在保留区间内，
> 因此改为 1430。

#### 先判断端口被谁占

```powershell
# 1) 端口是否有 LISTEN 监听（TimeWait 不阻塞绑定，不用管）
Get-NetTCPConnection -LocalPort 1430 -State Listen -ErrorAction SilentlyContinue

# 2) 看是不是 Hyper-V/WinNAT 的保留段
netsh int ipv4 show excludedportrange protocol=tcp
netsh int ipv4 show dynamicport tcp
```

- 出现在 `excludedportrange` 列表里 → 是系统/Hyper-V 保留，**杀进程无效**，走下面「换端口」或「永久保留」
- 显示为本项目 `node.exe` → 残留的开发服务器，直接结束

```powershell
Get-NetTCPConnection -LocalPort 1430 -State Listen -ErrorAction SilentlyContinue |
  ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }
```

老系统没有 `Get-NetTCPConnection` 时：`netstat -ano | findstr :1430` 取最后一列 PID，
再 `taskkill /PID <PID> /F`。

#### 一键换端口（推荐）

仓库自带脚本会读取系统保留段与动态端口段，挑一个真正可绑定的端口，并把两处配置同时改写：

```powershell
# 只报告，不改文件
powershell -ExecutionPolicy Bypass -File scripts/dev-port.ps1 -DryRun

# 选定并写入 vite.config.ts + tauri.conf.json
powershell -ExecutionPolicy Bypass -File scripts/dev-port.ps1

# 指定端口（脚本会校验是否被系统保留或占用）
powershell -ExecutionPolicy Bypass -File scripts/dev-port.ps1 -Port 1620
```

输出示例：

```text
Reserved (Hyper-V / WinNAT):
  1336-1435
Dynamic range: 49152-65535
Chosen dev port: 1520 (HMR 1521)
Updated:
  apps/windows/vite.config.ts            -> const DEV_PORT = 1520;
  apps/windows/src-tauri/tauri.conf.json -> http://localhost:1520
```

脚本会改写两个受版本控制的文件，撤销：`git checkout -- apps/windows/vite.config.ts apps/windows/src-tauri/tauri.conf.json`。

#### 手动换端口（无需管理员权限）

`vite.config.ts` 顶部只有一个常量 `DEV_PORT`，`hmr.port` 自动取 `DEV_PORT + 1`，所以手动改也只需两处：

| 文件 | 改什么 |
|---|---|
| `apps/windows/vite.config.ts` | `const DEV_PORT = <端口>;` |
| `apps/windows/src-tauri/tauri.conf.json` | `build.devUrl` 里的端口 |

选端口前先跑 `netsh int ipv4 show excludedportrange protocol=tcp`，避开所有列出区间
（实测某台 Win11 上 1336–1435 整段被 Hyper-V 划走，原来的 1420/1430 都在里面）。

#### 永久保留端口（需要管理员 PowerShell）

让 Windows 不再把该端口分配给 Hyper-V/WinNAT，之后重启系统也不会漂移：

```powershell
# 以管理员身份运行
net stop winnat
netsh int ipv4 add excludedportrange protocol=tcp startport=1430 numberofports=10
net start winnat
```

这条命令把 1430–1439 从动态分配池中排除，本项目仍可绑定这些端口。撤销：

```powershell
netsh int ipv4 delete excludedportrange protocol=tcp startport=1430 numberofports=10
```

#### 现象对照

| 现象 | 处理 |
|---|---|
| 终端提示 `Port 1430 is already in use` | 按上面的命令确认是残留 node 进程（结束）还是 Hyper-V 保留段（换端口或永久保留） |
| Console 报 `ERR_CONNECTION_REFUSED` | Vite 没起来：确认终端里有 `Local: http://localhost:1430/` |
| 端口正常但仍空白（IPv6/IPv4 解析差异） | 把 `host: host \|\| false` 改成 `host: host \|\| "127.0.0.1"`，并把 `devUrl` 改成 `http://127.0.0.1:1430` |
| WebView2 缺失 | `winget install Microsoft.EdgeWebView2Runtime` |

### 8.3 构建与环境类

| 现象 | 处理 |
|---|---|
| `error: linker 'link.exe' not found` | 缺 VS C++ Build Tools，或未安装「使用 C++ 的桌面开发」工作负载 |
| 改了 Rust 不生效 | 删对应 crate 的 `target` 产物重编译 |
| 想回到「首次运行」状态 | 退出应用，删 `%APPDATA%\Nexthrum\`，再删凭据管理器里的 `Nexthrum/GotifyClientToken` |
| 中文字体发虚 | 记录缩放比例与截图，通常是 WebView2 缩放或字体回退问题 |
| dev 模式通知发送者为空/乱码 | dev 与安装包的 AppUserModelID 不同，属预期；以安装包为准 |

## 9. 测试完成后

1. 截图放入 `screenshot/`，按第 6 节命名
2. 在本手册对应条目后记录 `通过 / 失败 + 说明`
3. 失败项汇总成第 7 节模板列表回传
4. 修复后在本手册勾选复测结果，并在 `devlog/YYYY-MM-DD.md` 记录结论

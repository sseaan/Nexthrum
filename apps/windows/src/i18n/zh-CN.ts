const zh = {
  app: { title: "Nexthrum" },

  window: {
    minimize: "最小化",
    maximize: "最大化",
    restore: "向下还原",
    close: "关闭",
  },

  appearance: {
    title: "外观",
    theme: "主题",
    system: "跟随系统",
    light: "白天",
    dark: "深色",
    hint: "主题只影响本机显示；选择「跟随系统」时将随 Windows 外观自动切换。",
  },

  filter: {
    label: "按来源筛选",
    allApps: "全部应用",
  },

  message: {
    empty: "暂无消息",
    emptyHint: "在设置中配置 Gotify 服务器以开始接收消息",
    delete: "删除消息",
    markRead: "标记为已读",
    noTitle: "无标题",
    read: "已读",
    unread: "未读",
    openSettings: "打开设置",
  },

  tray: {
    messageList: "消息列表",
    settings: "设置",
    exit: "退出",
  },

  search: {
    placeholder: "搜索消息",
    clear: "清除搜索",
    noMatches: "没有匹配的消息"
  },

  update: {
    "available": "发现新版本 {version}",
    "downloading": "正在下载 {version}…",
    "downloadingPercent": "正在下载 {version}… {percent}%",
    "ready": "新版本已下载，重启即可完成更新",
    "failed": "更新失败，请稍后重试",
    "install": "下载更新",
    "restart": "立即重启",
    "dismiss": "暂不更新"
  },

  header: {
    title: "消息",
    unread: "{n} 条未读",
    cached: "已缓存 {n} 条",
    reconnect: "重新连接",
    settings: "设置",
    clearAll: "全部清空…"
  },

  onboarding: {
    welcome: "欢迎使用 Nexthrum",
    intro: "连接你的 Gotify 服务器后即可在此接收消息，只需一分钟。",
    step1Title: "服务器地址",
    step1Detail: "填写你在浏览器中打开的地址，例如 https://gotify.example.com",
    step2Title: "客户端令牌",
    step2Detail: "在 Gotify 网页端打开 Clients，创建或复制一个客户端令牌，然后粘贴到下方。",
    step3Title: "先测试，再保存",
    step3Detail: "建议先测试连接；保存后令牌会存入 Windows 凭据管理器。",
    serverUrl: "服务器地址",
    serverUrlPlaceholder: "https://gotify.example.com",
    clientToken: "客户端令牌",
    tokenPlaceholder: "粘贴你的客户端令牌",
    showToken: "粘贴客户端令牌…",
    test: "测试连接",
    testing: "测试中…",
    save: "保存并连接",
    skip: "暂时跳过",
    advanced: "高级设置…",
    missing: "请先填写服务器地址与客户端令牌。"
  },

  settings: {
    title: "设置",
    back: "返回",
    serverConnection: "服务器连接",
    serverUrl: "服务器地址",
    clientToken: "客户端令牌",
    tokenHint: "令牌保存在 Windows 凭据管理器，不会写入设置文件。",
    testConnection: "测试连接",
    testing: "测试中…",
    fillRequired: "请先填写服务器地址和客户端令牌",
    connected: "连接成功！Gotify 服务器版本：{version}",
    general: "启动与窗口",
    autoStart: "开机自启",
    autoStartHint: "登录 Windows 后自动启动 Nexthrum 并保持后台连接。",
    startHidden: "启动时隐藏到托盘",
    startHiddenHint: "启动后不显示主窗口，只在托盘显示图标。",
    minimizeToTray: "关闭窗口时隐藏到托盘",
    minimizeToTrayHint: "关闭按钮只隐藏窗口；请从托盘菜单退出应用。",
    retention: "消息保留天数",
    retentionHint: "超过该天数的本地消息会在后台定期清理。",
    dnd: "免打扰",
    dndHint: "在此时间段内不会弹出通知（消息仍会保存）",
    dndStart: "开始",
    dndEnd: "结束",
    dndTo: "至",
    save: "保存设置",
    saving: "保存中…",
    saved: "设置已保存！",
    savedButDisconnected: "设置已保存，但暂时无法连接到服务器：{error}",
    saveFailed: "保存失败：{error}",
  },

  clearAll: {
    title: "清空全部消息",
    confirm: "确定要清空全部消息吗？此操作无法撤销。",
    syncServer: "同时删除服务器上的全部消息",
    cancel: "取消",
    confirmButton: "全部清空",
  },

  connection: {
    connected: "已连接",
    disconnected: "未连接",
    connecting: "正在连接…",
  },

  language: {
    label: "语言 / Language",
    zhCN: "中文",
    enUS: "English",
  },
} as const;

export default zh;

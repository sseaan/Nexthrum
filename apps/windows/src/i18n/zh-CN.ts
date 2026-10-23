const zh = {
  app: { title: "Nexthrum" },

  toolbar: {
    clearAll: "清空全部",
    settings: "⚙ 设置",
    unread: "{n} 条未读",
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
  },

  tray: {
    messageList: "消息列表",
    settings: "设置",
    exit: "退出",
  },

  settings: {
    title: "设置",
    back: "← 返回",
    serverConnection: "服务器连接",
    serverUrl: "服务器地址",
    clientToken: "客户端令牌",
    testConnection: "测试连接",
    testing: "测试中...",
    fillRequired: "请先填写服务器地址和客户端令牌",
    connected: "连接成功！Gotify 服务器版本：{version}",
    general: "通用设置",
    autoStart: "开机自启",
    startHidden: "启动时隐藏到托盘",
    minimizeToTray: "关闭窗口时隐藏到托盘",
    retention: "消息保留天数",
    dnd: "免打扰",
    dndHint: "在此时间段内不会弹出通知（消息仍会保存）",
    dndStart: "开始",
    dndEnd: "结束",
    dndTo: "至",
    save: "保存设置",
    saving: "保存中...",
    saved: "设置已保存！",
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

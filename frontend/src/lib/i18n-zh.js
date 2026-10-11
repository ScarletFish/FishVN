export const DICT_ZH = {
  nav: {
    mylist: '我的列表',
    scan: '扫描',
    stats: '统计',
    settings: '设置',
  },
  mylist: {
    title: '我的列表',
    add: '添加作品',
    empty: '列表还是空的',
    emptyFiltered: '{label}里没有条目',
    error: '读取列表失败',
    filter: {
      all: '全部',
    },
  },
  common: {
    loading: '加载中',
    cancel: '取消',
    save: '保存',
    close: '关闭',
  },
  detail: {
    notFound: '作品不存在',
    voteCount: '{count} 人',
    error: '读取作品失败',
  },
  // 每个操作一组 title/message/confirm，调用处只传 action 名
  confirm: {
    deleteGame: {
      title: '删除作品',
      message: '《{name}》将从列表中移除，游玩记录一并删除',
      confirm: '删除',
    },
  },
  settings: {
    title: '设置',
    theme: '深色主题',
    themeHint: '切换浅色与深色两套配色',
  },
  gameForm: {
    title: '添加作品',
    namePlaceholder: '新的游戏',
    field: {
      name: '游戏名',
    },
    error: {
      nameRequired: '请填写游戏名',
    },
  },
  toast: {
    copied: '已复制',
    copyFailed: '复制失败',
  },
  status: {
    playing: '游玩中',
    wishlist: '计划中',
    finished: '已完成',
    dropped: '抛弃',
  },
};

# 前端 lib 契约 — Libs

## 事件与刷新

- **`events.js` 的 devhttp 订阅**：共享一条 EventSource，后端重启窗口期代理会以错误状态码应答，EventSource 因此进入 CLOSED 终态；`onerror` 里销毁并按 1 秒间隔重建，重挂全部监听，重建地址带 `?since=` 续传已收到的最大事件 id，避免重放旧事件。
- **库内容刷新分三级广播**：每写完一部发 `LIBRARY_ITEM_UPDATED`，载荷带 `game_id`——`onLibraryItemUpdated(handler)` 逐条立即回调、订阅方按 id 自行过滤（详情页与角色区只认自己那部，一部只刷一次）；`onLibraryItemsThrottled(handler, ms)` 把连续多部并成一批，列表与首页用 5 秒窗口。跑完再发 `LIBRARY_CHANGED`（列表 / 首页整体重拉、详情页重拉单条）与 `LIBRARY_TAGS_CHANGED`（标签云、`#` 标签池、角色区）。写库的入口一律发 `LIBRARY_CHANGED`：补全完结、表单保存、同步计划应用；抓取阶段不落库，不发。详情页与卡片的封面解析都按 `image_path` 判重，路径没变不重置 `coverUrl`，否则逐部刷新会每秒闪一次占位。完结回执由 App 在补全页之外弹一次 toast，补全页内的汇总落在运行日志的「汇总」行。

## 展示与格式

- **`format.js`**：评分在数据层一律是 0–100（VNDB 与本地的原值都如此）。卡片与详情两处精度不同是有意的 —— `formatScore` 出两位（详情要精确），`formatScoreShort` 出最短形式（卡片胶囊，整数不补 `.0`），改精度只改这一个文件；不要在组件里另写 `toFixed`。同步结果弹窗的 `ratingText` 另算一路：它显示的是差异报告，要能与 VNDB 原始数值对齐，所以取两位再去零。
- **状态枚举**：状态是跨表单下拉、列表分组、筛选条、同步报告共用的同一份顺序，定义只在 `status.js` 一处；Rust 侧 `src-tauri/src/types.rs` 的 `Status` 是它的镜像，加状态或调顺序两边一起改。

## 交互小件

- **`toast.svelte.js`**：渲染在 `components/Toaster.svelte`，挂在 `App.svelte`；计时器簿记不放进 `$state`（每次读写都会触发更新）；暂停要手动记剩余时长 —— 进度条的暂停靠 CSS `animation-play-state`，计时器在 JS 里，两者不通。
- **`confirm.svelte.js`**：`action` 必须在 `i18n-zh.js` 有 `confirm.<action>.{title,message,confirm}` 三条词条，缺了不做兜底（`t()` 会回落 key）。
- **`vndb-search.svelte.js`**：换词重搜先清空，表单每次打开也要清一遍；导出的是 getter 而不是 `$state` 本体（直接 `return` 只会把初始值交出去）。侧栏由 `Modal` 的 `aside` 渲染，关闭时那一段没有挂载，父组件的工具栏按钮因此调不到里面的东西。
- **`native-menu.js`**：`contextmenu` 在 `document` **冒泡阶段**兜底 `preventDefault`，桌面端不再露出 WebView2 那套「重新加载 / 另存为 / 打印 / 检查」。必须挂冒泡而不是捕获：自己的菜单（bits-ui 的 Trigger、`Toaster` 的右键复制）都在目标节点上 `preventDefault`，捕获先跑会把事件标成已处理，Trigger 见到 `defaultPrevented` 直接不弹。`input` / `textarea` / `contenteditable` 里放行——剪切粘贴是系统级输入能力，自建只会降级；判定按 `closest` 命中祖先，输入框内部的节点一道放行，`contenteditable="false"` 不算。
- **`form-tab-pref.js`**：编辑表单的 tab 位置记忆，localStorage 持久化到重启，全部作品共享一份。`resolveFormTab` 是纯函数（未知值回落 `basic`），测试不打 localStorage；读写两个入口只给 `Detail.svelte` 用——新建表单（`Dashboard` / `MyList`）不读记忆，永远落基本页。`GameForm` 的 `tab` 是 `$bindable` prop，表单内切页经绑定写回 `formTab`，持久化的 `$effect` 挂在 `Detail` 侧。
- **`frontend-log.js`**：前端异常、未处理的 Promise 拒绝、启动就绪时刻经 `log_frontend` 落到日志文件，`api.js` 的 IPC 失败也经它上报。直连 `invoke` 而不走 `api.call`，日志上报自身失败不会形成上报循环；非 Tauri 环境（浏览器 dev）整体 no-op，异常看 devtools。

## 比对契约

`char-merge.js` / `vn-merge.js` 分别是角色粒度与作品粒度的 VNDB 同步比对，两者同口径：本地优先，冲突逐个取舍；面板文案在组件侧按 key 查词条。

- 后端 `trimToNull` 会把空串变 `null`，两边要归一成同一个空值
- 比对字段表只收内容字段：`sort_order` 是位置，`id` 是存储身份
- 配对键优先取原名 —— 罗马字名重名会把两个不同角色判成同一个
- 角色草稿留着 `source`（原样取回的那一行），保存时铺在最前、只覆盖表单露出的字段，否则改一次名字就把 VNDB 的考据数据抹了
- 合并结果里 `adopted` 命中的冲突取 VNDB 那份，其余一律保留本地 —— 包括本地独有的角色，用户手建的同步后原样留着
- 顺序是角色的唯一真相，不另存序号；卡片名与详情页一致，原名在前

## runes 与文件扩展名

runes（`$state` `$derived` `$effect` `$props`）**只在 `.svelte` / `.svelte.js` / `.svelte.ts` 里可用**，普通 `.js` 写了会运行时抛 `rune_outside_svelte`（构建和 `check:frontend` 都不拦，只有打开页面才炸）。

共享响应式状态放 `.svelte.js`；跨模块导出 state 必须包一层对象，直接 `export let x = $state(0)` 会被编译器拒绝。

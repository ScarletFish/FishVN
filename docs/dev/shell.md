# 外壳与视图 — Shell

## 首页与视图切换

默认视图是首页（`App.svelte` 的 `current` 初值 `'home'`），模块按 `config.dashboard_layout` 的顺序渲染，只渲染认识的模块 id，配置里多余的 id 被忽略。模块的注册表、开关与调序的纯函数在 `lib/dashboard-layout.js`，设置弹窗与首页共用这一份。首页监听 `lib/config-events.js` 的 `CONFIG_CHANGED` 事件并整页重载——设置弹窗每次成功保存配置后必须广播，否则首页与统计口径不刷新。品牌页的就绪事件（`app:ready`）由「启动时实际挂载的那个视图」派发——MyList 与 Dashboard 的首次加载各自派发，监听侧 `{ once: true }`，重复派发无害；新增默认视图或改默认视图时必须确认这条链路有人接手。

## 启动页

窗口以 `visible: false` 创建，前端在「内容就绪」与「加载页该出现」两者先到者处 show，Rust 侧 1.5 秒兜底。窗口出现时必定已有内容：快机直接是完整首页，慢机是同一帧出现的加载页，中间没有空白窗口。窗口背景色在 setup 里按已存主题设置，先于 webview 首帧，深色用户看不到浅色底；主题选跟随系统时按窗口当前系统主题解析。

`splash` 是纯 HTML/CSS，JS 不执行也照样显示 —— 前端整体挂掉时 Rust 兜底 show 出来的是静态加载页，不是空窗。判定落在 `index.html` 的内联脚本里：内容在该阈值内就绪则窗口跳过加载页，否则窗口与加载页同时出现。加载页有最短可见时长，避免出现即被撤走。

品牌鱼形只有一份 `path`：`index.html` 的 `#fishvn-fish` 精灵。启动页大图标与标题栏单色鱼都靠 `<use>` 引用它，换鱼形只改这一处。同一份 path 有两种用法：启动页是负形（`--splash-accent` 底色、鱼挖成白），标题栏是正形（`color: var(--accent)`），不另画一套。

`frontend/public/favicon.svg` 与 `src-tauri/icons/*` 是构建期资源，拿不到运行时 token，因此是品牌的独立拷贝：底色写死浅色默认 `--accent`，鱼写死 `--fg-on-accent`，不跟主题或色系联动——外部识别锚定默认品牌色是有意为之。改图标用 `npx tauri icon frontend/public/favicon.svg -o src-tauri/icons` 重生成，多出来的 `android/`、`ios/` 删掉；鱼占方块宽度的比例写了两遍（favicon 的 `transform`、启动页 `#splash .brand-mark svg` 的 `width`），改一处要两处一起改。

## 弹窗

弹窗节点由 `portal.js` 搬到 body 级的 `#modal-root`，与 `#app` 平级 —— 背景要靠 `inert` 屏蔽，两者做成父子关系会把弹窗自己一起屏蔽。用 `inert` 而不是 `pointer-events`：后者挡不住 Tab，键盘仍会走进背景。`#globalTooltip` 也挂在 body 上且靠 `opacity` 显隐，不能跟背景一起模糊。

- 面板常驻 DOM，切 tab 只切 `display` —— 条件渲染会让已填内容在切走时丢失
- tab 栏不能放进 `.modal-panels`：它是滚动容器，会把标签一起滚走
- `.modal` 是 flex 列：外壳固定件不参与收缩，视口不足时只有 `.modal-panels` 滚动；内层 `form` 只负责回车隐式提交、布局交给 `.modal`，`dialog` 角色要挂在非交互元素上
- 并排双窗是一个遮罩里放两个 `.modal`：不能设 `min-width`（会先撑破遮罩），也不能加 `flex-wrap`（换行就不再并排）
- `Modal` 的 `onclose` 返回 `false` 可否决关闭，用于「保存中不许关」这类拦截
- `GameForm` 关闭时组件不卸载，字段要在关闭处重置；等待 VNDB 拉取时列表仍保持可读 —— 请求回来后直接进冲突面板，得留住用户点的那一条
- `GameForm` 的保存按钮在 tab 面板之外：从「基本信息」页提交时，若资料页的范围校验失败，要把 tab 切到资料页，否则用户看到的是「点了保存没反应」

## 水平分页

分页滚动由一条契约贯穿 JS 与 CSS：容器上写 `--card-w`（基准卡宽）与 `--cols`（整数列数），`lib/scroll-dots.svelte.js` 按基准卡宽与容器宽算出列数写回 `--cols`，CSS 据此把容器宽均分成 `--cols` 份，每屏恰好整数张完整卡片、铺满无空隙。两侧只要不同源，圆点数就会对不上实际能放几张。

- 基准卡宽是档位门槛，不是实际渲染宽：各分区按需覆盖 `--card-w`，渲染宽由均分结果决定
- `--card-w` 只写 px：JS 用 `parseFloat` 读取该值，`20rem` 会被截断成 20，列数暴涨、卡片塌缩
- 不量测任何渲染尺寸：容器隐藏后再显示不影响结果，窗口尺寸一变即重算
- 圆点数要算上末尾那张添加卡，漏计会让最后一屏跳不过去
- 展开态只换配色：动尺寸会触发分页重算，卡宽与 `--cols` 会在一帧里塌缩
- 立绘保留原始比例不裁，名字压在图底的 scrim 上；用户自选的图未必是卡的比例，裁的时候保头不保脚
- 表单里的封面预览与角色卡取同一均分式，改一边要跟着改另一边

卡宽、列数与窗口化的规格与实测数据见 [grid-layout.md](./grid-layout.md)。

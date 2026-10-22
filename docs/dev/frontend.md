# 前端开发规范 — Frontend

### 视觉参照

组件的类名、布局、动效与交互对齐 MyAnimeDocker 的实现。不过token采用我们项目的写法。参照之前先查 [visual-ref.md](./visual-ref.md) 里的 token 对照表与语法翻译表。

### 样式的三种形态

1. **Token**（`css/tokens.css`）— 颜色/间距/圆角/字体的唯一来源
2. **Svelte scoped style** — 组件私有样式
3. **普通 CSS 类**（`components/`、`views/`）— 跨组件复用的模式

归属判断：组件独有 → scoped；可能复用 → components/；视图专属 → views/；是变量 → tokens.css。

分层的依据是**控件本身**，不是界面上的分页或 tab：同一个控件被哪几页用到都不改变它的归属，通用控件一律进 `components/`，`views/` 只放该页面专属的样式。

### Token 分层

`tokens.css` 分三层：L1 原始值（只有色值，零语义）、L2 语义色槽（描述用途，不描述外观）、L3 尺度（间距/圆角/字号/字重/字体/动效）。

**组件只能连第 2、3 层** —— 直连 L1 的原始值不会随主题变。换主题只改 L2 取值，组件代码不动。

### 主题

浅色是 `:root` 默认值，深色在 `[data-theme="dark"]`。`index.html` 首帧前定 `data-theme`，无 `data-theme` 属性即浅色。

强调色系是正交的第二轴：`[data-accent="…"]` 挂色系名（amber/ocean/sakura/emerald/violet），绯红是默认、不打属性。深浅由 `data-theme` 管、色系由 `data-accent` 管，任意组合；深色下的色系值用 `[data-theme="dark"][data-accent="…"]` 双属性选择器覆盖浅色值。`index.html` 首帧前同时定 `data-theme` 与 `data-accent`。新色系补齐 tokens.css 的三处：L1 原值（accent 族与配套 fg 族）、浅色与深色两个覆盖块、设置面板色块的 `--swatch-*`；faint/bg/border/selected 等派生槽引用 `--accent`，自动跟随。

### 字体

拉丁走 DM Sans：可变字体拉丁子集打包在 `assets/fonts/dm-sans-latin.woff2`（OFL 许可文本同目录存放），`fonts.css` 声明 `@font-face`，字重轴 100–1000 全覆盖，新字重直接取用。CJK 走 `@fontsource-variable/noto-sans-sc`（OFL，族名 `Noto Sans SC Variable`）：101 个按 `unicode-range` 切片的可变字体，在 `fonts.css` 里 `@import`，构建时进 `dist`，浏览器按用到的字符取片；升级换版本即换包。`--font-sans` 中 CJK 族排在系统字体之前；两族之外的字符（斜体、扩展区汉字）回落字体栈里的系统字体。

#### 字号阶层

字号一律引用 `--text-*` token；SVG 内文字与 DOM 文字同样适用（SVG 按 1:1 渲染时 rem 即屏幕像素）。阶层与用途：

| token | 像素 | 用途 |
|---|---|---|
| `--text-2xs` | 12.5 | 最小注记 |
| `--text-xs` | 14.4 | 图表轴标签、徽章数字、图表数值标注、次要说明 |
| `--text-sm` | 15 | 卡片副标题、列表辅助行 |
| `--text-base` | 16.3 | 正文、卡标题 |
| `--text-md` | 17.5 | 表单主标签、强调正文 |
| `--text-lg` | 20 | 区块标题 |
| `--text-xl` | 24 | KPI 大数字 |
| `--text-2xl` | 26 | 弹窗标题 |
| `--text-3xl` | 36 | 详情页主标题 |

### 动画

GSAP 与 ScrollTrigger 以 vendor 全局脚本加载（`public/vendor/gsap/`，`index.html` 在模块脚本之前引入）。动画代码全程判空 `globalThis.gsap`，脚本缺失时界面照常渲染，动画只是增强层；`prefers-reduced-motion: reduce` 时入场动画直接跳过。

滚动入场（`lib/scroll-anim.js`）：区块 fade+rise、卡片按批 stagger，触发线在视口 92% 处、`once` 自毁；scroller 固定 `.shell__main`，它也是页面的唯一滚动容器。使用契约：视图持有一个实例，用数据签名 gate 驱动——`$effect` 里比较签名，变化后才 `tick().then(build)`，`build` 先 `kill` 旧 trigger 并清内联样式（防残留 `opacity: 0`），视图销毁时调 `kill()`。动画只动 transform 与 opacity。

主题与强调色切换经 `document.startViewTransition` 包裹（`theme.js` 的 `setTheme`/`setAccent`），整窗交叉淡入；启动同步（`syncTheme`/`syncAccent`）不走过渡。API 缺失或 `prefers-reduced-motion` 时直接生效。

封面 → 详情共享元素过渡（`lib/hero-transition.js`）：卡片点击时 `captureHero` 记录封面图的位置与已解码地址，详情视图挂载后 `playHero` 立即接管——先同步隐藏目标、rAF 后一次测量、GSAP 补间 0.35s。hero 的启动不依赖封面 URL 解析（`coverSrc` 走 IPC 异步返回，等它就永远赶不上首帧）；换手阶段等目标 `<img>` **出现并加载完成**（MutationObserver + 2s 兜底）再瞬间替换，异步加载期间由 hero 兜住视觉。无封面、GSAP 缺失或 `prefers-reduced-motion` 时跳过，详情瞬时打开。

### Token 引用规则

- 颜色/间距/圆角/字重一律引用 token，**禁止写死值**；token 单点定义在 `tokens.css`，禁止在别处重新声明
- 字号一律引用 `--text-*` token，阶层见「字体」节；SVG 内文字同样适用
- **组件禁止直连 L1 原始值**，只能引用 L2/L3
- **阴影与过渡只从 L2/L3 取**，不在组件里写死；要半透明色用 `color-mix(in srgb, var(--accent) 12%, transparent)`，不新增 `-rgb` 三元组

### "先找后写"三步协议

新增任何 CSS 前：

1. 查 `tokens.css` 有没有现成 token
2. 查 `components/` 有没有现成类
3. 都没有才新写，写进正确归属文件

### 自动检查

`npm run check:frontend` 跑注释检查、token 检查、死 CSS 检查、测试与构建；**勿只跑其中一条**，否则 dist 过期「改了没生效」。

死 CSS 只警告不阻断 —— 先写样式后写 markup 是常见顺序，开发中途见红会让人养成无视输出的习惯。token 检查强制 L1/L2/L3 纪律与裸色值禁令，尚未覆盖间距与字号的写死值。

### CSS 文件结构

`styles.css` 只做 `@import`，禁止写样式；样式写进哪个文件见上面的「样式的三种形态」。

### 布局

chrome 尺寸固定，不随视口缩放：侧栏 `--sidebar-w`，标题栏 `--titlebar-h`。

封面墙的卡宽、列数、窗口化约束见 [grid-layout.md](./grid-layout.md)。

模糊优先用 `filter`。`backdrop-filter: blur()` 只在背后存在未模糊内容时才有效果 —— 背后已是模糊色块时它是空转（实测加与不加像素一致），所以只有浮在原图之上的组件才值得考虑。

作品的聚合划分、消费方的字段归属判据、写权限分类、加载策略与数据流见 [view-contracts.md](./view-contracts.md)。

### 首页与视图切换

默认视图是首页（`App.svelte` 的 `current` 初值 `'home'`），模块按 `config.dashboard_layout` 的顺序渲染，只渲染认识的模块 id，配置里多余的 id 被忽略。模块的注册表、开关与调序的纯函数在 `lib/dashboard-layout.js`，设置弹窗与首页共用这一份。首页监听 `lib/config-events.js` 的 `CONFIG_CHANGED` 事件并整页重载——设置弹窗每次成功保存配置后必须广播，否则首页与统计口径不刷新。品牌页的就绪事件（`app:ready`）由「启动时实际挂载的那个视图」派发——MyList 与 Dashboard 的首次加载各自派发，监听侧 `{ once: true }`，重复派发无害；新增默认视图或改默认视图时必须确认这条链路有人接手。

## 启动页

窗口以 `visible: false` 创建，前端在「内容就绪」与「加载页该出现」两者先到者处 show，Rust 侧 1.5 秒兜底。窗口出现时必定已有内容：快机直接是完整首页，慢机是同一帧出现的加载页，中间没有空白窗口。窗口背景色在 setup 里按已存主题设置，先于 webview 首帧，深色用户看不到浅色底。

`splash` 是纯 HTML/CSS，JS 不执行也照样显示 —— 前端整体挂掉时 Rust 兜底 show 出来的是静态加载页，不是空窗。判定落在 `index.html` 的内联脚本里：内容在该阈值内就绪则窗口跳过加载页，否则窗口与加载页同时出现。加载页有最短可见时长，避免出现即被撤走。

启动页图标与 `frontend/public/favicon.svg` 是同源副本，换品牌图标要同步两处。

## 构建与开发服务器

`vite.config.js` 里几处不能动的配置：

- 测试态把 `resolve.conditions` 设为 `['browser']`：svelte 要解析到浏览器构建，SSR 构建里 `mount()` 直接抛错。非测试态**完全不写 `resolve` 键** —— 写成 `conditions: []` 同样会顶掉 vite 的默认条件，产物里会混进 svelte 的服务端模块（构建不报错，只有运行才炸）
- dev server 的 `host` / `port` 必须与 `tauri.conf.json` 的 `devUrl` 同协议族、同端口：写 `localhost` 会让 vite 只绑 `::1`，devUrl 的 IPv4 字面量就够不着；`strictPort: true` 让端口占用直接失败，不自动往上换 —— 换了端口 devUrl 就指空
- `/dev-http` 是唯一的跨源转发入口，指向开发期 HTTP 通道；生产构建里没有它

## 弹窗

弹窗节点由 `portal.js` 搬到 body 级的 `#modal-root`，与 `#app` 平级 —— 背景要靠 `inert` 屏蔽，两者做成父子关系会把弹窗自己一起屏蔽。用 `inert` 而不是 `pointer-events`：后者挡不住 Tab，键盘仍会走进背景。`#globalTooltip` 也挂在 body 上且靠 `opacity` 显隐，不能跟背景一起模糊。

- 面板常驻 DOM，切 tab 只切 `display` —— 条件渲染会让已填内容在切走时丢失
- tab 栏不能放进 `.modal-panels`：它是滚动容器，会把标签一起滚走
- `.modal` 是 flex 列：外壳固定件不参与收缩，视口不足时只有 `.modal-panels` 滚动；内层 `form` 只负责回车隐式提交、布局交给 `.modal`，`dialog` 角色要挂在非交互元素上
- 并排双窗是一个遮罩里放两个 `.modal`：不能设 `min-width`（会先撑破遮罩），也不能加 `flex-wrap`（换行就不再并排）
- `Modal` 的 `onclose` 返回 `false` 可否决关闭，用于「保存中不许关」这类拦截
- `GameForm` 关闭时组件不卸载，字段要在关闭处重置；等待 VNDB 拉取时列表仍保持可读 —— 请求回来后直接进冲突面板，得留住用户点的那一条
- `GameForm` 的保存按钮在 tab 面板之外：从「基本信息」页提交时，若资料页的范围校验失败，要把 tab 切到资料页，否则用户看到的是「点了保存没反应」

## 滚动条

`lib/scrollbars.js` 支撑三类：主视图与详情页是覆盖式、不占布局宽（占宽则竖条一出现整排卡片就变窄），指针进入竖带时加粗；弹窗表单（`.scroll-form`）整块 hover 才显示，移出即隐藏；内嵌小滚动区（`.scroll-thin`）走浏览器默认的细条。

- 插件不显式注册 `clickScroll` 就静默失效
- `autoHide: 'never'`：库自己的计时器会盖掉命中时的常驻；显隐由 `.is-visible` 控制，淡出时长在 CSS
- 弹窗不挂 `watchRightEdge`：那套坐标判定会把滚动条卡在加粗态回不去，加粗反馈只留给主视图
- 只记主视图的实例：`MyList` 靠它还原列表滚动位置，弹窗实例写进去会把它顶掉
- 命中判定按坐标，不走滚动条的 `:hover`：库给滚动条设了 `pointer-events: none`，放开命中后落在竖带上的滚轮会进库自己的 `wheel` 监听并被 `preventDefault`
- 竖带命中区取竖带右缘到容器右缘（两者之间还有死区）；首次移动指针前先算一次位置，否则 `edge=0` 会让到处都算「在边缘」
- 指针离开整个窗口要立即清状态：抬手发生在窗口外时收不到 `pointerup`，不清 `dragging` 会卡在加粗态

外观全量覆盖库内置主题（`theme` 参数只提供结构属性，尺寸与色值一律走我们的 token）：竖带、手柄、拉伸位的宽度都要写死 —— 库的 `:hover` / `:active` 默认会改宽度。`transition` 是整体替换而非叠加，位置项要一并列出，否则手柄跟随滚动会瞬移。内嵌细条只走标准属性 `scrollbar-width` / `scrollbar-color`：Chromium 一旦设了标准属性就禁用 `::-webkit-scrollbar-*`，混用会静默失效；且要挂在真正 `overflow-y: auto` 的元素上，套一层外壳会断掉高度约束。

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

## 组件与交互惯例

- **原名优先**：人名与作品名在界面上展示 `original`，罗马音 `name` 用作配对与检索身份。VNDB 同步填进表单的展示名按「官方中文标题 → 原文标题（alttitle）→ 主标题（title）」选择，别名放主标题；新增任何展示 VNDB 名字的位置都沿用这一优先级。
- **`GameForm` 的保存顺序是「内容先行，绑定殿后」**：安装、状态、元数据、角色全部写成功后，最后一步才写 `vndb_id`，绑定始终与已落库的内容一致。新建条目在创建返回后立刻切到更新通道，失败重试落在同一条上。选中搜索命中先查库内占用，已有条目绑定同一作品就当场提示，校验通过才进入拉取。
- **`Menu`**：关闭判定等焦点整个移出菜单 —— 点菜单项时 `blur` 先于 `click` 触发。
- **`SelectDd`**：下拉选择的唯一入口，选项以 `{ value, label }` 数组传入，开合状态由调用方持有（`open` + `onopenchange`），同一表单内多处下拉互斥靠调用方比较存的是「哪一处开着」。浮层随组件文档流定位，宿主容器不要再套 `overflow` 滚动，长列表交给面板级滚动。
- **`DetailSection`**：详情页分区的统一外壳（卡片 + 标题 + 计数 + 内边距）。`.section-card` 基类只有背景与边框，内边距由外壳统一给出；新增详情分区直接用组件，自己拼 `section.section-card` 会让标题与相邻分区错位。
- **`tooltip.js` / `side-tip.js`**：`data-tooltip` 的全局气泡替代原生 `title`；`sideTip` 是面向侧栏的变体，定位与外观不同，两者并存互不干扰。提示源被移除（切页、关弹窗）后浏览器不再对它发 `mouseout`，收尾由下一次 `mouseover` 探测 `isConnected` 完成；新增移除提示源的场景不需要额外清理。
- **`events.js` 的 devhttp 订阅**：共享一条 EventSource，后端重启窗口期代理会以错误状态码应答，EventSource 因此进入 CLOSED 终态；`onerror` 里销毁并按 1 秒间隔重建，重挂全部监听，重建地址带 `?since=` 续传已收到的最大事件 id，避免重放旧事件。
- **错误文案**：后端与网络错误不走 i18n，原样显示。
- **`theme.js`**：首帧由 `index.html` 的内联脚本定，运行时切换只在 `theme.js`；两边判断必须一致 —— 深色加 `data-theme` 属性、其余删属性，不往 `localStorage` 写 `'light'`。
- **`Detail`**：鼠标侧键 3 是后退，4 不接。
- **`MyList`**：打开表单前先落 `gameId`，`GameForm` 的 `$effect` 与 `open` 在同一帧读到的是最终值。
- **`DetailCharacters` / `CharacterEditor`**：立绘走封面那条 `coverSrc` 管线 —— 缩图与 asset 协议授权都在那边做过。
- **`VnMergeDialog`**：单一取舍面板承载两层分区——元数据字段行与角色冲突行，角色区立绘随元数据区一起预取，面板关掉时结果作废；两侧任一为空的单元格显示「未填写」占位。
- **`CharacterEditor`**：回选只在「角色列表变化」时发生（加载 / 增删 / 保存回填），用 `untrack` 让它只依赖 `characters`、不追踪 `selectedId`，用户手动点开或收起不会被抢回。
- **`vndb-search.svelte.js`**：换词重搜先清空，表单每次打开也要清一遍；导出的是 getter 而不是 `$state` 本体（直接 `return` 只会把初始值交出去）。侧栏由 `Modal` 的 `aside` 渲染，关闭时那一段没有挂载，父组件的工具栏按钮因此调不到里面的东西。
- **`frontend-log.js`**：前端异常、未处理的 Promise 拒绝、启动就绪时刻经 `log_frontend` 落到日志文件，`api.js` 的 IPC 失败也经它上报。直连 `invoke` 而不走 `api.call`，日志上报自身失败不会形成上报循环；非 Tauri 环境（浏览器 dev）整体 no-op，异常看 devtools。
- **`confirm.svelte.js`**：`action` 必须在 `i18n-zh.js` 有 `confirm.<action>.{title,message,confirm}` 三条词条，缺了不做兜底（`t()` 会回落 key）。
- **`PlaytimeField.svelte`**：总时长单输入框，失焦即 `update_game_state`。输入认 `12:30`、`12h30m` 与纯数字（按秒），右侧读数列显示秒；回填用无损写法（整分 `12:34`，带秒 `12:34:56`），认不出来的输入还原成原值。
- **`toast.svelte.js`**：渲染在 `components/Toaster.svelte`，挂在 `App.svelte`；计时器簿记不放进 `$state`（每次读写都会触发更新）；暂停要手动记剩余时长 —— 进度条的暂停靠 CSS `animation-play-state`，计时器在 JS 里，两者不通。

## 前端 lib 的比对契约

`char-merge.js` / `vn-merge.js` 分别是角色粒度与作品粒度的 VNDB 同步比对，两者同口径：本地优先，冲突逐个取舍；面板文案在组件侧按 key 查词条。

- 后端 `trimToNull` 会把空串变 `null`，两边要归一成同一个空值
- 比对字段表只收内容字段：`sort_order` 是位置，`id` 是存储身份
- 配对键优先取原名 —— 罗马字名重名会把两个不同角色判成同一个
- 角色草稿留着 `source`（原样取回的那一行），保存时铺在最前、只覆盖表单露出的字段，否则改一次名字就把 VNDB 的考据数据抹了
- 合并结果里 `adopted` 命中的冲突取 VNDB 那份，其余一律保留本地 —— 包括本地独有的角色，用户手建的同步后原样留着
- 顺序是角色的唯一真相，不另存序号；卡片名与详情页一致，原名在前

## 样式坑集

改样式前扫一眼这里，每一条都对应过一次「改完就坏」。

**卡片与浮层**

- `:hover` 的 `transform` 会生成层叠上下文，菜单展开时整卡抬到相邻卡之上
- 承载弹出菜单的区块要压得过它后面的兄弟：`.detail-banner` 与 `.detail-chars` 同为 `z-index: 1` 时，DOM 靠后的角色区盖住横幅里弹出的 `.menu__list` —— 菜单自己的 `--z-dropdown` 出不了父级的层叠上下文
- 可见表面与裁切都归 `.cover-card__frame`，菜单是它的兄弟而非子元素才逃得出裁切；卡片 `overflow: hidden` 同时承担浮层的底角，不能去掉

**详情页氛围底**

- 上内边距归零：氛围底 `absolute` 在 `.detail` 的 padding box 上，`padding-top` 会把它推下去
- 内容起点由 `padding-top` 决定，氛围底的高度与 mask 停点只决定它画到哪：上移内容不改变覆盖范围，但栏体可能落进实色区而不是淡出段，改这里要同时看 `.detail-info-line` 的落位
- 氛围底必须是 `.detail` 的直接子元素才能左右满幅，挪进 `.detail-banner` 会各留一条空白边
- mask 必须分段：压回两站时收口线附近仍接近不透明，底图会被切出硬边；实色边界 = 高度 × 停点%，改高度后要保持边界不变就得同步停点
- 模糊层 `scale 1.3` 是为了盖住 blur 向外溢出的透明边，否则四边露出未模糊的原图边缘；氛围底的中间停点与 mask 停点同高，改一处另一处要同步
- 详情页封面与列表卡封面（`.cover-card` / `.cover-card__frame`）的尺寸、圆角、占位要一起改
- 信息栏的边框是承重结构（底色 55% 不透明度且浮在氛围底的淡出段上），去掉边框栏体就塌；背后必须有 `.detail-ambience`，删掉它这层几乎不可见

**标签与胶囊**

- 胶囊几何共用、配色按角色分开；tag 走中性色是刻意的 —— 一屏几十个 tag 全用 accent 会稀释强调色也压过标题，accent 留给信息栏里的评分与信息项
- `.pill--more` 落在 `<button>` 上：它是靠 `font-family: inherit` 拿字体的（见上「字体继承」），边框也要显式清掉，去掉这两行会露出系统默认样式
- 简介靠固定行数夹住，依赖 `-webkit-box` 三件套，`-webkit-box-orient` 缺了 ellipsis 不出现
- 属性用流式键值对（值都很短，表格浪费竖向空间）；trait 分组保留表格式布局：标题列定宽，标签列吃剩余宽度；简介是连续散文，铺满整宽时一行长到读不回行首
- 角色卡与所在 section 底色同档会让边界糊掉，卡片要高一档
- vn-merge 的角色行标签列装的是名字，比字段名长，栅格与作品级一致、靠折行消化宽度

**控件**

- `字体继承`：原生 `button` / `input` / `select` / `textarea` **不继承** `font`（含 `font-family` / `font-size` / `font-weight`），UA 会给一套系统默认字体与尺寸，与页面其余 `--font-sans` 文字错位（标题栏返回按钮当初就因此与标题差 ~2px）。`tokens.css` 的全局 reset 已用标准写法 `font: inherit` 对这几类强制沿用父级字体与尺寸，从根上消除整类隐患；组件里仍可用 `font-family: var(--font-sans)` 显式写死（特异性更高，优先于全局规则）。依赖 `inherit` 拿到字体的按钮（如 `.pill--more`）删掉 `font-family: inherit` 会回落系统字体、露出默认样式
- `视觉对齐`：控件「看着不对齐 / 文字像浮起」，先查是不是漏了字体或字号声明（原生控件不继承，见上「字体继承」），不要上 `translate` / `margin` 之类的魔法偏移去补。本项目的「1px 上浮」当初就是被误读的字体错位——字体一对齐，几何上已经居中，这类魔法数字没有几何依据、后人看不懂也不敢动，应删掉；像素级补偿只作为带注释的最后手段
- `toggle`：hidden input 不能 `display: none`（会退出 Tab 序与无障碍树），只能压成 0 尺寸；滑块必须是它的紧邻兄弟，中间隔任何元素都会断开相邻选择器；input 是 0 尺寸，全局 `outline` 会画在看不见的地方，焦点环要转到滑块上；药丸小且边框淡，外描才看得出离开轨道
- `tags`：输入框必须在容器内、跟在芯片后面（放到容器外 placeholder 会把容器撑宽），它吃掉剩余宽度，最小宽度保证空标签时也能点进去打字
- `forms`：`:focus-visible` 要清 `outline`，否则键盘 Tab 会套上全局那圈重环；数量用文本输入框而非 `number`（原生微调与页面滚动、方向键冲突）；左缘竖条常态中性灰、勾选后转 accent，左侧 padding 必须比右侧窄，竖条才贴得住边
- `dropdowns`：开合用 `is-open` 类切换、不做 portal —— 菜单与触发器同处一个层叠上下文，走 portal 会与弹窗遮罩的 `z-index` 打架
- `menus`：菜单留在触发器的层叠上下文里，不移出 DOM；`.btn svg` 是按描边线性图标定的，实心点要自己定尺；裸图标变体浮在内容图上，必须自带底色
- `sidebar`：`stroke-width` 与尺寸绑定，图标放大后线条会相对变细；`.active:hover` 必须与 `.active` 一起清，否则悬停时半透明底色会回来
- `buttons`：`.btn-primary:hover` 用 `brightness` 提亮（不过滤会瞬间跳变）；覆盖 `.btn:hover` 的规则与它特异性相同，要逐项写全才不会漏掉被按顺序冲掉的属性

**表面层级与交互态**

表面层级只连 L2 的 `--bg-*` token，用途对应：

| 用途 | token |
|---|---|
| 页面底 / 侧栏 | `--bg-page` / `--bg-modal` |
| 常规表面 / 卡片 / 输入框底 | `--bg-surface` |
| 抬起的块（卡片常态、浮层锚） | `--bg-raised` |
| 浮层（菜单 / 下拉 / 弹窗内容） | `--bg-floating` |
| hover / 按下蒙版（压暗） | `--bg-hover` |
| 选中（accent 身份） | `--bg-selected` + `--accent-border` |

交互态写法：
- hover / active 用 `--bg-hover` 蒙版压暗，或只强化 `--border-strong` 边框。
- 浮层（菜单、下拉、弹窗）用 `--bg-floating`，取 ramp 最浅一档。
- 选中用 `--bg-selected` 淡染 + `--accent-border` 描边表达身份。
- 表单字段（输入框、标签输入框等）hover 只动边框，`focus` / `:focus-within` 才升 `--bg-raised` + `--shadow-sm`。
- 下拉 / 菜单：触发器（按钮）走 `--bg-hover` 压暗，弹出菜单走 `--bg-floating`。
- 标签页 / 段控选中指示用 accent 下划线或 accent 淡染。
- 已添加的「内容」类胶囊（tag 芯片）恒定背景用 `--bg-hover`，比容器深一档，hover 时仍保持边界（与只读 `.pill--tag` 同款）。

## 注释

代码里不写注释，前端惯例写进本文。规则与例外见 [comments.md](./comments.md)。

## 新增 UI 组件检查清单

- [ ] 新增与改动的行里没有注释（例外见 comments.md）
- [ ] `npm run check:dead-css` 没有输出死类
- [ ] 往本文档增行前先列条目过目，并按 30 秒判据筛过
- [ ] 跑过 `npm run check:frontend`

## 文案

界面文案一律走 `lib/i18n.js` 的 `t()`，词条按命名空间集中在 `lib/i18n-zh.js`，命名沿用 `ns.key`。locale 跟随系统语言，只在已登记语言中匹配；加语言只需扩 `i18n.js` 里的 `DICT`。

需要运行时切换 locale 时，把 `lib/i18n.js` 改名 `i18n.svelte.js` 并把状态包成对象 —— 见下。

### runes 与文件扩展名

runes（`$state` `$derived` `$effect` `$props`）**只在 `.svelte` / `.svelte.js` / `.svelte.ts` 里可用**，普通 `.js` 写了会运行时抛 `rune_outside_svelte`（构建和 `check:frontend` 都不拦，只有打开页面才炸）。

共享响应式状态放 `.svelte.js`；跨模块导出 state 必须包一层对象，直接 `export let x = $state(0)` 会被编译器拒绝。

## VNDB 标签/特征中文显示

tag 与 trait 的显示名走 `lib/tag-label.svelte.js`（`tagLabel` / `traitLabel`）与 `tag_dictionary` 命令，**不进 i18n 字典**：它们是随 VN 同步进来的动态数据，量级几千条，手写字典承载不了。trait 的分组标签（`detail.traitGroups.*`）是唯一走 i18n 的相关文案。

查表是一层：按 id 命中中文译名则显示中文，未命中或没有 id（表单手输标签）显示原名。

id 的形态与同步数据对齐：tag 是 `g`+数字、trait 是 `i`+数字。tag 与 trait 的 id 数字空间互相重叠（同一个数字在两张表中是不同的词条），词典分 `tags` / `traits` 两张表存放，查询必须按类型进行。

数据管线（开发期一次性生成，产物入 git）：

1. `src-tauri/resources/vndb/tags.json.gz` / `traits.json.gz` 是 VNDB 官方 dump，提供 id 与英文名的权威对应；
2. `src-tauri/resources/vndb/community/build_tag_zh.py` 按英文名（含 aliases）把 `community/vndb-tags-translation.json`（社区表，MIT）与 `community/gap-fill.json`（自补，优先）join 成 id→中文，换算出 `g`/`i` 前缀键，产出 `src-tauri/resources/vndb/tags_zh.json`；
3. 词典经 `include_str!` 编译进二进制（`src-tauri/src/tagdict.rs`），`tag_dictionary` 命令下发，`App.svelte` 启动时加载一次（失败静默，全量回退英文）。

改翻译只动 `community/gap-fill.json` 后重跑 `build_tag_zh.py`；数据来源链路、许可证与署名要求见 [vndb-localization.md](vndb-localization.md)，署名见 `src-tauri/resources/vndb/ATTRIBUTION.md`（随包分发）。

仪表盘词云的 `TagCount` 带 `id`：Rust 侧按 tag id 聚合并输出，无 id 的条目按 name 聚合，键加 `#` 前缀避免与真实 id 相撞。

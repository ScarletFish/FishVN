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

浅色是 `:root` 默认值，深色在 `[data-theme="dark"]`。用户可选浅色 / 深色 / 跟随系统三态，存的是选择而非解析结果：`index.html` 首帧前按 `fishvn.theme` 判定，值为 `system` 时看 `prefers-color-scheme`，解析成深色才挂 `data-theme`，无属性即浅色。运行时由 `theme.js` 的 `setTheme(mode)` / `syncTheme(mode)` 做同一套解析；选中 `system` 时 `theme.js` 常驻一个 `matchMedia` 监听，系统切换深浅就地重绘，不需要用户重启或重进设置。Rust 侧窗口背景色同样按「选择 → 解析」二段处理（`main.rs`）。

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

主题与强调色切换经 `document.startViewTransition` 包裹（`theme.js` 的 `setTheme`/`setAccent`），整窗交叉淡入；启动同步（`syncTheme`/`syncAccent`）不走过渡。API 缺失或 `prefers-reduced-motion` 时直接生效。跟随系统下的系统切换不套过渡，避免用户在别的应用里换主题时本窗跟着做一次交叉淡入。

封面 → 详情共享元素过渡（`lib/hero-transition.js`）：卡片点击时 `captureHero` 记录封面图的位置与已解码地址，详情视图挂载后 `playHero` 立即接管——先同步隐藏目标、rAF 后一次测量、GSAP 补间 0.35s。hero 的启动不依赖封面 URL 解析（`coverSrc` 走 IPC 异步返回，等它就永远赶不上首帧）；换手阶段等目标 `<img>` **出现并加载完成**（MutationObserver + 2s 兜底）再瞬间替换，异步加载期间由 hero 兜住视觉。无封面、GSAP 缺失或 `prefers-reduced-motion` 时跳过，详情瞬时打开。

详情页分波入场（`detail.css` 的 `detail-rise`/`detail-fade`）：CSS animation + `animation-fill-mode: backwards`，隐藏态由首帧关键帧兜住——不需要 show 类、不需要双 rAF，动画在元素插入时自动启动，天然适配异步分区（角色/制作数据晚于外壳到达时，各自的波在自己渲染时启动）。波次：氛围底/操作按钮 0s → 信息栏逐条 0.04–0.29s（延迟内联 `animation-delay`，属编排参数，不适用 token 规则）→ 角色区头 0.12s → 制作区 0.18s；封面不参与（归 hero 过渡接管）。判据：**多图分区（角色/剧集式）直接加载、不做逐卡编排**——编排延迟与图片加载时钟不同步，且卡片是点击锚点应尽早可交互。新增详情分区挂 `detail-rise` 并给波次延迟；`prefers-reduced-motion` 全量豁免。

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

窗口以 `visible: false` 创建，前端在「内容就绪」与「加载页该出现」两者先到者处 show，Rust 侧 1.5 秒兜底。窗口出现时必定已有内容：快机直接是完整首页，慢机是同一帧出现的加载页，中间没有空白窗口。窗口背景色在 setup 里按已存主题设置，先于 webview 首帧，深色用户看不到浅色底；主题选跟随系统时按窗口当前系统主题解析。

`splash` 是纯 HTML/CSS，JS 不执行也照样显示 —— 前端整体挂掉时 Rust 兜底 show 出来的是静态加载页，不是空窗。判定落在 `index.html` 的内联脚本里：内容在该阈值内就绪则窗口跳过加载页，否则窗口与加载页同时出现。加载页有最短可见时长，避免出现即被撤走。

品牌鱼形只有一份 `path`：`index.html` 的 `#fishvn-fish` 精灵。启动页大图标与标题栏单色鱼都靠 `<use>` 引用它，换鱼形只改这一处。同一份 path 有两种用法：启动页是负形（`--splash-accent` 底色、鱼挖成白），标题栏是正形（`color: var(--accent)`），不另画一套。

`frontend/public/favicon.svg` 与 `src-tauri/icons/*` 是构建期资源，拿不到运行时 token，因此是品牌的独立拷贝：底色写死浅色默认 `--accent`，鱼写死 `--fg-on-accent`，不跟主题或色系联动——外部识别锚定默认品牌色是有意为之。改图标用 `npx tauri icon frontend/public/favicon.svg -o src-tauri/icons` 重生成，多出来的 `android/`、`ios/` 删掉；鱼占方块宽度的比例写了两遍（favicon 的 `transform`、启动页 `#splash .brand-mark svg` 的 `width`），改一处要两处一起改。

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

滚动条只有两大族，按容器选其一：

- **OverlayScrollbars 族**（详情页/主区/面板，由 `lib/scrollbars.js` 驱动）：内容区、浮层、分栏统一用它，手柄尺寸与色值走 token，别处不再手写滚动条样式
- **`.scroll-thin` 族**（小表单/文本，纯 `::-webkit-scrollbar-*`）：只给 textarea、合并面板单元格这类纯文本滚动区

| 容器 | 用法 |
|---|---|
| 主滚动区 `.shell__main` | `useScrollbars(el)`，`autoHide: 'never'`，指针进入右边缘时手柄加粗到 12px |
| 弹窗面板、侧栏、标签框、详情页分栏、搜索下拉 | `use:panelScrollbars`，手柄 8px，指针离开 400ms 后淡出 |
| textarea、合并面板的单元格文本、任务面板的错误列表 | 类 `.scroll-thin` |

`panelScrollbars` 的宿主写法：

- 宿主用 `div`，承担 `overflow-y`、限高与内边距
- flex / grid 布局写在内层 `div` 上
- 正文元素（`p`、`ul`）放进宿主内层的 `div` 里

细则：

- `clickScroll` 插件显式注册
- overlay 滚动条浮在内容之上、不占布局位：右对齐的元素（数量、徽标）右内距要写成 `calc(<原内距> + var(--scrollbar-size))`，否则被手柄压住
- 主视图的 `autoHide` 取 `'never'`，显隐由 `.is-visible` 控制，淡出时长在 CSS；`opacity: 0` 的初始态只写在主视图自己的滚动条上（初始化时打 `.is-main`），不可用 `.shell__main .os-scrollbar` 这类后代选择器——会连面板滚动条一起压成透明
- `watchRightEdge` 取竖滚动条走 `instance.elements().scrollbarVertical.scrollbar`，不用 `querySelector`——面板滚动条在文档序里更靠前，会被先选中
- `watchRightEdge` 只挂主视图
- 实例只记主视图，`MyList` 靠它还原列表滚动位置
- 命中判定按坐标做，取竖带右缘到容器右缘；首次移动指针前先算一次位置
- 指针离开整个窗口时清掉 `dragging` 与 `lingering`

外观（`components/scrollbars.css`）：

- `theme` 只给结构，尺寸与色值走 token：竖带、手柄、拉伸位宽度写死，`transition` 列全位置项
- 手柄色值只有一个来源 `--scrollbar-thumb`，overlay 族与 `.scroll-thin` 族共用；它必须是不透明色（混入 `--bg-surface` 定明度）——带透明度会让手柄跟着所在容器的底色走，主区与面板滚出深浅差
- `.scroll-thin` 用 `::-webkit-scrollbar-*`：`width` 取 `--scrollbar-size`，`::-webkit-scrollbar-button` 置 `none`，轨道与角落透明，手柄悬停时取 `--scrollbar-thumb`；同一元素不写 `scrollbar-width` / `scrollbar-color`

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
- **`theme.js`**：首帧由 `index.html` 的内联脚本定，运行时切换只在 `theme.js`；两边判断必须一致 —— 把 `fishvn.theme` 解析成深浅（`system` 看 `prefers-color-scheme`）后，深色加 `data-theme` 属性、浅色删属性。`localStorage` 存的是三态选择本身，不是解析结果。
- **`Detail`**：鼠标侧键 3 是后退，4 不接。
- **`MyList`**：打开表单前先落 `gameId`，`GameForm` 的 `$effect` 与 `open` 在同一帧读到的是最终值。
- **`DetailCharacters` / `CharacterEditor`**：立绘走封面那条 `coverSrc` 管线 —— 缩图与 asset 协议授权都在那边做过。
- **`VnMergeDialog`**：单一取舍面板承载两层分区——元数据字段行与角色冲突行，角色区立绘随元数据区一起预取，面板关掉时结果作废；两侧任一为空的单元格显示「未填写」占位。
- **`CharacterEditor`**：回选只在「角色列表变化」时发生（加载 / 增删 / 保存回填），用 `untrack` 让它只依赖 `characters`、不追踪 `selectedId`，用户手动点开或收起不会被抢回。
- **`vndb-search.svelte.js`**：换词重搜先清空，表单每次打开也要清一遍；导出的是 getter 而不是 `$state` 本体（直接 `return` 只会把初始值交出去）。侧栏由 `Modal` 的 `aside` 渲染，关闭时那一段没有挂载，父组件的工具栏按钮因此调不到里面的东西。
- **`frontend-log.js`**：前端异常、未处理的 Promise 拒绝、启动就绪时刻经 `log_frontend` 落到日志文件，`api.js` 的 IPC 失败也经它上报。直连 `invoke` 而不走 `api.call`，日志上报自身失败不会形成上报循环；非 Tauri 环境（浏览器 dev）整体 no-op，异常看 devtools。
- **`confirm.svelte.js`**：`action` 必须在 `i18n-zh.js` 有 `confirm.<action>.{title,message,confirm}` 三条词条，缺了不做兜底（`t()` 会回落 key）。
- **`PlaytimeField.svelte`**：总时长单输入框，失焦即 `update_game_state`。输入认 `12:30`、`12h30m` 与纯数字（按秒），右侧读数列显示秒；回填用无损写法（整分 `12:34`，带秒 `12:34:56`），认不出来的输入还原成原值。
- **输入组**（输入框 + 右侧内嵌按钮）：容器单边框，输入框无框伸缩（`flex:1` + `min-width:0`），按钮坐右侧、中间一条分隔线；容器 `:focus-within` 接管焦点高亮，点输入框或点按钮整个框一起亮；按钮 `min-width` 兜住两种文字宽度，切换时输入框不抖帧；内嵌按钮做成无底无框的附属动作（hover 只变文字色 + 出 `--bg-hover` 底），不与表单行里的主按钮同权重。与 `.num-stepper` 同构，「框 + 附属动作」的组合沿用这一只。
- **成组输入**：几个短字段同属一个语义组（宽高、三围这类）时并成一条外壳，不要各占一行——外壳只画外框、格间断竖线，组内 input 自身无框；组内标签可以压成单个字母或单位前缀，但 `.sr-only` 要补出全名（可访问名含可见标签，WCAG 2.5.3）。焦点由整格底色加格底内线承担，不套 `--focus-ring`：编辑区的焦点是中性强调（`--bg-raised` / `--border-strong`），accent 只表示选中身份。容器用 `role="group"` + `aria-label`，`fieldset` 的 `legend` 会在上边框留一道缺口；外壳 `overflow: hidden`，方角底色才能沿圆角收口。
- **状态枚举**：状态是跨表单下拉、列表分组、筛选条、同步报告共用的同一份顺序，定义只在 `status.js` 一处；Rust 侧 `src-tauri/src/types.rs` 的 `Status` 是它的镜像，加状态或调顺序两边一起改。
- **chip 双选**：tag-pill 体系复用到非标签场景（同步导入名单这类「出现在表单里 = 选中」的清单）时，选中用实心、未选用幽灵（虚线 + 前景色透明化），`aria-pressed` 承载语义，点一下两态切换；chip 内只放条目名，状态与分组由组头承载，chip 里不重复。
- **`toast.svelte.js`**：渲染在 `components/Toaster.svelte`，挂在 `App.svelte`；计时器簿记不放进 `$state`（每次读写都会触发更新）；暂停要手动记剩余时长 —— 进度条的暂停靠 CSS `animation-play-state`，计时器在 JS 里，两者不通。
- **`TaskDock`**：全局任务状态面板，浮动在视窗底部居中（380px，非详情页展示），层级压在全部弹窗与 toast 之上。与 Toaster 一样经 portal 挂进 `#modal-root`：弹窗打开时 `body` 的直接子元素（除 `#modal-root` / `#globalTooltip`）整体 blur，浮条挂在 `#app` 里会跟着糊掉。当前承载资料补全任务——它是唯一在弹窗外跑的长任务。显隐语义：× 在任务在途时折叠成底部居中的小手柄（状态点 + 进度文案 + 展开箭头），任务 alive 期间手柄常驻，点击恢复完整面板；完结汇总始终自动唤起（含 × 折叠期间），带 6 秒倒计时自动隐藏，倒计时走完后的 × 才是真收起。新任务启动（列表按钮触发时派发 `backfill-refresh` 窗口事件、后台接续时经首个进度事件）无条件重新打开——收起不能让人错过下一轮进度。状态归约抽到纯模块 `dock-state.js`（与 Svelte/DOM 解耦、可单测）：后端 `vndb-backfill` 事件全部消费，阶段事件 `meta`/`cover`/`characters`/`images` 只更新 `phase`、不污染全局 `done`/`total`，`error` 收进 `errors[]`、面板给「N 条失败」展开；`run_id` 取代模块级「收起/关闭」标记，新一轮 `run_id` 不同即判为「这轮没折叠/关过」，上一轮完结汇总随新轮自然作废、无需手动 reset。挂载时只接在途任务，刷新后从进行中的进度续上，上一轮的完结汇总随刷新作废。× 改右上角独立角标，不再压住动作按钮。刷新分三级广播：每写完一部发 `LIBRARY_ITEM_UPDATED`，载荷带 `game_id`——`onLibraryItemUpdated(handler)` 逐条立即回调、订阅方按 id 自行过滤（详情页与角色区只认自己那部，一部只刷一次）；`onLibraryItemsThrottled(handler, ms)` 把连续多部并成一批，列表与首页用 5 秒窗口。跑完再发 `LIBRARY_CHANGED`（列表 / 首页整体重拉、详情页重拉单条）与 `LIBRARY_TAGS_CHANGED`（标签云、`#` 标签池、角色区）。写库的入口一律发 `LIBRARY_CHANGED`：补全完结、表单保存、同步计划应用；抓取阶段不落库，不发。详情页与卡片的封面解析都按 `image_path` 判重，路径没变不重置 `coverUrl`，否则逐部刷新会每秒闪一次占位。完结回执由 dock 自己在不可见时弹 toast，App 不再重复订阅该事件。结构上完整面板与手柄是同一个 fixed 容器的两种形态、原地切换（不挂方向相反 `fly`，折叠只是换形态、不闪），形态由 `dockView` 据「折叠/关闭/隐藏」意图推导。
- **`SyncResultDialog`**：重操作（VNDB 同步这类秒级拉取）点下即开弹窗进加载态，反馈先于结果出现，不等 await 返回才弹窗；加载态的阶段文案只消费后端进度事件，前端不虚构阶段。父级用序号令牌防迟到结果：每次触发自增序号，await 返回时序号已被新的触发或取消改掉就丢弃（`seq !== syncActive` 直接 return），旧请求的 plan / error 不会覆盖新状态；按钮的忙态只由最新一次在途调用置位与清除。计划内容分两步呈现：有冲突时第一屏只做裁决，第二屏是写入预览（推送与导入各自可取消勾选），底部按钮随步骤切换为「下一步」/「上一步 · 应用」；只有一类内容时不分步。应用阶段若涉及上传 VNDB，弹窗留在原地显示写入进度（进度事件 `phase` 为 `push`），期间 `onclose` 返回 `false`，Esc 与关闭按钮都不生效；按钮换成禁用的进度态，避免长任务中途留下半截状态。
- **`ImageViewer`**：单图查看画布（截图查看器的地基，见下「图片查看器地基」）。只管一张图怎么看：滚轮以指针为中心缩放、拖动图面平移、双击在适应窗口/100% 间切换、右上角缩放比、stage 尺寸变化时适应态保持整图可见。图源从外部注入 `src`，不关心图从哪来（封面缩略图或本地原图都行）；`maxZoom` prop 化——封面编辑器传 4（预览图最大边 560px，再大必糊），直开原图的场景可以传更大。`children` snippet 收 `{ toImage, toScreen, view }` 供浮层做坐标换算（取景框就浮在它上面）；单击图面（位移 < 4px）回调 `ontap(图坐标)`。viewport 纯函数在 `lib/image-viewer.js`（带单测）。**坑**：滚轮监听必须 `addEventListener('wheel', h, { passive: false })` 才能 `preventDefault`，Svelte 的 `onwheel` 属性写法不行——不加 passive 选项浏览器默认 passive，`preventDefault` 是静默无效的，页面会跟着滚。**边界**：多图导航、画廊、旋转、信息面板不长在这个组件里——等截图查看器的真实需求来了再长，现在猜接口必然猜错。
- **`CoverFocalDialog`**：封面展示焦点的取景框编辑器（表单「调整展示位置」打开），画布是 `ImageViewer`，自己只做焦点数学（`lib/cover-focal.js`）。约定：焦点不参与合并面板（`vn-merge.js` 的 `FIELDS` 不加它，远端没有这个概念）；换封面重置焦点由后端 `metadata::upsert` 收口，前端只在 picker 选图与合并接受两条路径同步重置显示值，别处不要再造重置逻辑；新增封面展示处要读焦点字段（清单见 [backend.md](./backend.md)「封面展示焦点」）。手势路由：拖框与拖图面是两个区域的手势——取景框自己收 pointer 事件并 `stopPropagation`（否则画布会跟着平移），框外区域归画布（拖动平移、单击移框）；方向键微调焦点（Shift 加速，无效轴忽略），弹窗打开即聚焦到根容器。
- **`InstallationList`**：安装行 = 单选圆点 + `.install-main`（路径框 + `.install-flags`）。flags 区两个开关各自成行，形态对齐设置页的 `toggle-switch` + 标签，不加说明文字（开关本身自明）：转区启动一行；Magpie 缩放一行，行右挂两态段控件（全屏 / 窗口），开关关闭时段控件禁用置灰——开关即 `magpie` 的 off，开启默认落 `full`。段控件复用设置页的 `seg-radio-group` / `seg-radio-item`（label 包隐藏 radio、纯自定义样式，不用原生控件外观），radio 的 `name` 按 `item.key` 隔离防多行串组。flags 块整体缩进 + 左竖线，表明从属于该安装行；圆点与移除按钮都只跟路径框行对齐（移除是 `btn btn-danger`，跟角色面板的移除同一按钮语言）。转区开着时参数框禁用、placeholder 换成「转区时无法附加参数」（LE 链路不透传；别在禁用 input 上挂 tooltip——浏览器不派发鼠标事件给 disabled 控件，hover 永远不触发）。
- **`SettingsDialog` 的「工具」标签**：登记 Magpie / Locale Emulator 两个工具路径。失焦即保存（`oncommit`）：trim 后为空发 `null` 注销，非空先 `path_exists` 校验、不存在出行内错误且不落库，通过了才 `update_config` + 广播 `CONFIG_CHANGED`。浏览选完文件同样触发提交——`PathField` 的 `oncommit` 在 pick 成功后也会调，不只是 blur。

## 图片查看器地基

截图查看/管理功能预计会做，查看器内核已按单图查看的边界抽好，复用时直接长多图层，不要动内核：

- `lib/image-viewer.js` 是纯函数：viewport = 图坐标到屏幕坐标的 `(scale, tx, ty)` 变换，提供适应窗口、指针不动点缩放、平移、双向坐标换算、1:1 视图。无 DOM 依赖，带单测。
- `components/ImageViewer.svelte` 是画布：滚轮缩放、拖动平移、双击适应/100% 切换、缩放比显示。图源外部注入 `src`，`maxZoom` prop 化。
- 做截图查看器时：多图导航（上一张/下一张/键盘左右）、画廊网格、旋转、信息面板、管理动作都是内核之外的层，挂在查看器外面；封面编辑器这种「查看器 + 浮层」的形态可以抄 `CoverFocalDialog` 的 snippet 接法。

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

**导入顺序**

- `styles.css` 的 `@import` 顺序就是层叠顺序：后导入文件里同等特异性的规则覆盖先导入的。给先导入体系的类（tag 胶囊这类）写变体时，用「基类 + 变体」双类前缀锚定（如 `.tag-pill--pick.sync-chip`），单类变体会被后导入体系里的同名规则按源序冲掉——症状是「改完不报错、视觉没变化」

**卡片与浮层**

- `:hover` 的 `transform` 会生成层叠上下文，菜单展开时整卡抬到相邻卡之上
- 承载弹出菜单的区块要压得过它后面的兄弟：`.detail-banner` 与 `.detail-chars` 同为 `z-index: 1` 时，DOM 靠后的角色区盖住横幅里弹出的 `.menu__list` —— 菜单自己的 `--z-dropdown` 出不了父级的层叠上下文
- 可见表面与裁切都归 `.cover-card__frame`，菜单是它的兄弟而非子元素才逃得出裁切；卡片 `overflow: hidden` 同时承担浮层的底角，不能去掉

**表单分组**

- `.form-group + .form-group` 自带一条分割线，线上 20px、线下 4px：下一块的首元素若是自带边框的卡片（`.form-row`），线会和卡片边框在 4px 内并排成双线；这类 group 加 `.form-group--flat` 去掉线，靠段间距分组
- **逻辑成组**：同属一个功能的若干项（如「设置项 + 动作 + 列表」）要视作一组、组间不留分割线时，保持首项为普通 `.form-group`，其余项加 `.form-group--flat`；VNDB 标签页的「同步心愿单 / 同步评分」两个开关行即此写法，设置页「数据库」标签页的「保留策略 / 创建备份 / 备份列表」也照此合并。改一组的视觉边界时，只动首项之后的 `--flat`，别给首项也加（那样会和上一组再隔开）
- **动作行**：组内若某项是「标题 + 描述 + 控件/按钮」的单行卡片，用 `.form-row`——左侧放 `.form-row-text`（`.form-row-title` 标题 + `.form-row-desc` 描述，文本块 `flex: 1` 把控件顶到右侧），右侧放控件或按钮；版式镜像 VNDB 的开关行（左开关、右 `.form-row-label`），只是左侧换成文字块。新增这种行不要手搓 flex + 内联 style，统一走 `.form-row` 家族
- **下拉**：设置里的选择一律用 `SelectDd`（见「组件与交互惯例」一节，已约定为下拉唯一入口），不要写原生 `<select>`；选项以 `{ value, label }` 数组传入，开合状态由调用方持有

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
- `视觉对齐`：控件「看着不对齐 / 文字像浮起」，先查是不是漏了字体或字号声明（原生控件不继承，见上「字体继承」），不要上 `translate` / `margin` 之类的魔法偏移去补。本项目的「1px 上浮」当初就是被误读的字体错位——字体一对齐，几何上已经居中，这类魔法数字没有几何依据、后人看不懂也不敢动，应删掉；像素级补偿只作为带注释的最后手段。唯一持例是与文字同行的状态点 `translateY(-1px)`：CJK 墨迹在行框内比几何中心高约 1px（可变版 Noto Sans SC 实测），圆点不补偿会显得压底线；判据是有实测依据、且只作用于与文字同行的装饰元素，其余场景仍是先查字体
- `toggle`：hidden input 不能 `display: none`（会退出 Tab 序与无障碍树），只能压成 0 尺寸；滑块必须是它的紧邻兄弟，中间隔任何元素都会断开相邻选择器；input 是 0 尺寸，全局 `outline` 会画在看不见的地方，焦点环要转到滑块上；药丸小且边框淡，外描才看得出离开轨道
- `tags`：输入框必须在容器内、跟在芯片后面（放到容器外 placeholder 会把容器撑宽），它吃掉剩余宽度，最小宽度保证空标签时也能点进去打字
- `forms`：`:focus-visible` 要清 `outline`，否则键盘 Tab 会套上全局那圈重环；数量用文本输入框而非 `number`（原生微调与页面滚动、方向键冲突）
- `键入清洗`：受控输入在 `oninput` 里过滤字符时，状态和框里的字会分叉——清洗结果与上一次状态相同（已是 `12` 又敲了个字母），Svelte 判等后不重绘，脏字符就留在框里了。清洗结果与原文不同时直接写回 input 元素，别只靠状态更新
- `dropdowns`：开合用 `is-open` 类切换、不做 portal —— 菜单与触发器同处一个层叠上下文，走 portal 会与弹窗遮罩的 `z-index` 打架
- `menus`：菜单留在触发器的层叠上下文里，不移出 DOM；`.btn svg` 是按描边线性图标定的，实心点要自己定尺；裸图标变体浮在内容图上，必须自带底色
- `sidebar`：`stroke-width` 与尺寸绑定，图标放大后线条会相对变细；`.active:hover` 必须与 `.active` 一起清，否则悬停时半透明底色会回来
- `buttons`：`.btn-primary` 常态纯色 `--accent`、hover 换 `--accent-soft`（扁平风，不用渐变与 brightness 滤镜）；覆盖 `.btn:hover` 的规则与它特异性相同，要逐项写全才不会漏掉被按顺序冲掉的属性

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

分隔符口径：` · ` 用于同义项并列（CV 列表、标签名），` | ` 用于结构化统计行（多项统计一行排开）。

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

仪表盘词云与库内标签清单都按显示名聚合并输出 `{id, tag, count}`：同步标签与手输的同名标签并成一条，代表 id 取该名下首个带 id 的条目。标签的可见性、排序与计数口径见 [tags.md](tags.md)。

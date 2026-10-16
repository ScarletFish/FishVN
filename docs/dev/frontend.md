# 前端开发规范 — Frontend

### 视觉参照

组件的类名、布局、动效与交互对齐 MyAnimeDocker 的实现。抄之前先查 [visual-ref.md](./visual-ref.md) 里的 token 对照表与语法翻译表。

### 样式的三种形态

1. **Token**（`css/tokens.css`）— 颜色/间距/圆角/字体的唯一来源
2. **Svelte scoped style** — 组件私有样式
3. **普通 CSS 类**（`components/`、`views/`）— 跨组件复用的模式

归属判断：组件独有 → scoped；可能复用 → components/；视图专属 → views/；是变量 → tokens.css。

### Token 分层

`tokens.css` 分三层，**组件只能连第 2、3 层**：

| 层 | 内容 | 命名示例 |
|---|---|---|
| L1 原始值 | 只有色值，零语义 | `--gray-100` `--red-600` `--ink-black` |
| L2 语义色槽 | 描述用途，不描述外观 | `--bg-surface` `--fg-muted` `--accent` |
| L3 尺度 | 间距/圆角/字号/字重/字体/动效 | `--space-4` `--radius-sm` `--text-base` |

换主题只改 L2 取值，组件代码不动。

### 主题

浅色是 `:root` 默认值，深色在 `[data-theme="dark"]`。`index.html` 首帧前定 `data-theme`，无 `data-theme` 属性即浅色。

### Token 引用规则

- 颜色/间距/圆角/字重一律引用 token，**禁止写死值**
- token 单点定义在 `tokens.css`，禁止在别处重新声明
- **组件禁止直连 L1 原始值**，只能引用 L2/L3 —— 直连原始值的组件不会随主题变
- **阴影与过渡只从 L2/L3 取**，不在组件里写死
- 要半透明色用 `color-mix(in srgb, var(--accent) 12%, transparent)`，不新增 `-rgb` 三元组

### "先找后写"三步协议

新增任何 CSS 前：

1. 查 `tokens.css` 有没有现成 token
2. 查 `components/` 有没有现成类
3. 都没有才新写，写进正确归属文件

完成后跑 `npm run check:frontend` 验证。

### 自动检查

`npm run check:frontend` = CSS token 检查（`scripts/check-css-tokens.js`）+ 死 CSS 检查（`scripts/check-dead-css.js`）+ 构建 `dist/`。
`npm run check:tokens` 与 `npm run check:dead-css` 可单独跑。

token 检查项：① `var(--x)` 必须已定义 ② 组件禁止引用 L1 原始值 ③ L2 内禁止裸色值 ④ 组件与 Svelte `<style>` 内禁止裸色值。

死 CSS 检查输出定义了却无引用的类，只警告不阻断。类在 Svelte 的 `class` / `class:foo`、任意 JS 字符串、或其他 CSS 复合选择器里出现都算有引用。

改完前端必跑，勿只跑其中一条，否则 dist 过期「改了没生效」。

### 检查器的判定范围

**死 CSS**（`check-dead-css.js`）：只警告不阻断 —— 先写样式后写 markup 是常见顺序，开发中途见红会让人养成无视输出的习惯。算作引用的来源：

- Svelte 的 `class` 属性与 `class:name` 指令（指令名本身就是类名）
- `class={cond ? 'a' : 'b'}` 这类表达式里的引号字面量
- 任意 JS 字符串
- 组件从别的属性收的类名，如 `<Modal variant="modal--merge">`
- 复合选择器里每个类都算彼此的引用，`:has()` 的参数同理 —— 库注入的类名（overlayscrollbars 的 `os-*`）在 markup 里找不到字面量，只有这条路兜得住

注释里的类名、`@import` 的文件名都不是声明，照原样抽会把它们算成死 CSS。

**token**（`check-css-tokens.js`）：四条规则见上。唯一的例外是 mask 渐变的颜色分量 —— `mask` 只读 alpha，写成 `black` / `transparent` / `rgb(0 0 0 / 50%)` 都不存在「该连哪个语义槽」的问题，检查时把整条声明剥掉再查。尚未覆盖的是间距与字号的写死值。

## 架构概要

### CSS 文件结构

```
src/css/
├── styles.css     # 仅入口，只做 @import，禁止写样式
├── tokens.css     # L1 原始值 / L2 语义色槽 / L3 尺度
├── views/         # 视图专属 <view>.css
├── components/    # 跨组件复用类
└── layouts/       # 主题/布局特有
```

### 布局

chrome 尺寸固定，不随视口缩放：侧栏 `3.75rem`（60px），标题栏 34px。

封面墙的卡宽、列数、窗口化约束见 [grid-layout.md](./grid-layout.md)。

模糊优先用 `filter`。`backdrop-filter: blur()` 只在背后存在未模糊内容时才有效果——背后已是模糊色块时它是空转（实测加与不加像素一致），所以只有浮在原图之上的组件才值得考虑。当前模糊取值见 `css/views/detail.css`。

作品的聚合划分、消费方的字段归属判据、写权限分类、加载策略与数据流见 [view-contracts.md](./view-contracts.md)。

## 构建与开发服务器

`vite.config.js` 里几处不能动的配置：

- 测试态把 `resolve.conditions` 设为 `['browser']`：svelte 要解析到浏览器构建，SSR 构建里 `mount()` 直接抛错。非测试态**完全不写 `resolve` 键** —— 写成 `conditions: []` 同样会顶掉 vite 的默认条件，产物里会混进 svelte 的服务端模块（构建不报错，只有运行才炸）
- dev server 的 `host` / `port` 必须与 `tauri.conf.json` 的 `devUrl` 同协议族、同端口：写 `localhost` 会让 vite 只绑 `::1`，devUrl 的 IPv4 字面量就够不着；`strictPort: true` 让端口占用直接失败，不自动往上换 —— 换了端口 devUrl 就指空
- `/dev-http` 是唯一的跨源转发入口，指向开发期 HTTP 通道；生产构建里没有它

## 弹窗

弹窗节点由 `portal.js` 搬到 body 级的 `#modal-root`，与 `#app` 平级。背景层靠 `inert` 屏蔽交互；两者做成父子关系会把弹窗自己一起屏蔽掉。用 `inert` 而不是 `pointer-events` 是因为后者挡不住 Tab，键盘仍会走进背景。`#globalTooltip` 也挂在 body 上且靠 `opacity` 显隐，不能跟背景一起模糊。

- 面板常驻 DOM，切 tab 只切 `display`：条件渲染会让已填内容在切走时丢失
- tab 栏不能放进 `.modal-panels` —— 它是滚动容器，会把标签一起滚走
- `.modal` 是 flex 列：外壳固定件不参与收缩，视口不足时只有 `.modal-panels` 滚动；内层 `form` 只做提交、布局交给 `.modal`，它自身只负责回车隐式提交，`dialog` 角色必须挂在非交互元素上
- 并排双窗是一个遮罩里放两个 `.modal`：旁窗加出来、主窗不让位，因此不能设 `min-width`（会先撑破遮罩），也不能加 `flex-wrap`（换行就不再并排）
- `Modal` 的 `onclose` 返回 `false` 可否决关闭，用于「保存中不许关」这类拦截
- `GameForm` 关闭时组件不卸载，字段要在关闭处重置；等待 VNDB 拉取时列表仍保持可读，请求回来后直接进冲突面板，所以要点亮用户当时点的那一条

## 滚动条

同一份 `lib/scrollbars.js` 支撑三类滚动条：

| 场景 | 形态 | 反馈 |
|---|---|---|
| 主视图 / 详情页（`useScrollbars`） | overlay 覆盖式，不占布局宽 | 指针进入竖带时加粗（`watchRightEdge` 加 `.is-visible`） |
| 弹窗表单（`attachScrollbars` + `.scroll-form`） | overlay | 整块 hover 时显示，移出即隐藏 |
| 内嵌小滚动区（`.scroll-thin`） | 原生细条 | 浏览器默认 |

- 覆盖式不占布局宽：占宽的话竖条一出现，整排卡片就变窄
- 插件不显式注册 `clickScroll` 就静默失效
- `autoHide: 'never'`：库自己的计时器会盖掉竖带命中时的常驻。显隐由 `.is-visible` 控制，淡出时长在 CSS（`--duration-normal`）
- 弹窗不挂 `watchRightEdge`：那套 edge/inside 坐标判定在弹窗里会把滚动条卡在加粗态回不去，加粗反馈只保留在主视图
- 只记主视图的实例：`MyList` 靠 `getScrollViewport` 还原列表滚动位置，弹窗实例写进去会把它顶掉；已销毁的实例直接忽略
- 命中判定按坐标做，不走滚动条的 `:hover`：库给滚动条整体设了 `pointer-events: none`，放开命中后落在竖带上的滚轮会进库自己的 `wheel` 监听并被 `preventDefault`，竖带上就滚不动
- 竖带命中区取竖带右缘到容器右缘的整段（两者之间还有死区）；首次移动指针前先算一次竖带位置，否则 `edge=0` 会让到处都判定为「在边缘」
- 指针离开整个窗口要立即收起并清状态：抬手发生在窗口外时收不到 `pointerup`，不清 `dragging` 会卡在加粗态；拖拽中移出不进缓冲

外观全量覆盖库内置主题：`theme` 参数只能选库内置的 `dark` / `light` / `none`，它们只提供结构属性，尺寸与色值一律走我们的 token。竖带宽度即 `--scrollbar-size-hover`，手柄常态 `--scrollbar-size`，三个宽度都要写死 —— 库的 `:hover` / `:active` 默认会改宽度，不钉死的话加粗就不只由 `is-edge` 一个来源触发。`transition` 是整体替换而非叠加，位置项要一并列出，否则手柄跟随滚动会瞬移。

内嵌细条只走标准属性 `scrollbar-width` / `scrollbar-color`：Chromium 一旦设了标准属性就禁用 `::-webkit-scrollbar-*`，两套混用会静默失效。平时全透明，移入元素才显色，并且要挂在真正 `overflow-y: auto` 的元素上 —— 套一层外壳会断掉高度约束。

## 水平分页

分页滚动由一个契约贯穿 JS 与 CSS：容器上写 `--card-w`（基准卡宽）与 `--cols`（整数列数），`lib/scroll-dots.svelte.js` 按基准卡宽与容器宽算出列数写回 `--cols`，CSS 据此把容器宽均分成 `--cols` 份，每屏恰好整数张完整卡片、铺满无空隙。两侧只要不同源，圆点数就会对不上实际能放几张，改任一侧都要同步另一侧。

- 基准卡宽是档位门槛，不是实际渲染宽：各分区按需覆盖 `--card-w`，渲染宽由均分结果决定
- 不量测任何渲染尺寸、无时序耦合：容器隐藏后再显示不影响结果，窗口尺寸一变（ResizeObserver）即重算
- 圆点数要算上末尾那张添加卡（角色区），漏计会让最后一屏跳不过去
- 展开态只换配色：动尺寸会触发分页重算，卡宽与 `--cols` 会在一帧里塌缩
- 卡片的 `overflow: hidden` 不能去掉 —— 浮层的两个底角靠它圆回来
- 立绘保留原始比例不裁，名字压在图底的 scrim 上；用户自选的图未必是卡的比例，裁的时候保头不保脚
- 表单里的封面预览与角色卡取同一均分式，改一边要跟着改另一边

卡宽、列数与窗口化的规格与实测数据见 [grid-layout.md](./grid-layout.md)。

## 组件与交互惯例

- **`Menu`**：`items` 是 `{ label, onclick }` 列表，`variant: btn` 用 `.btn.btn-ghost`、`plain` 是浮在内容上的裸图标。关闭判定等焦点整个移出菜单 —— 点菜单项时 `blur` 先于 `click` 触发。
- **`GameCard`**：可点区域是真按钮，与菜单互为兄弟；封面缺失时用游戏名首字符占位。
- **`StatusSection`**：状态分区容器 = 组头 + 网格。
- **`tooltip.js` / `side-tip.js`**：`data-tooltip` 的全局气泡替代原生 `title`，`import` 一次即生效；`sideTip` 是面向侧栏的变体，定位与外观不同，两者并存互不干扰。
- **错误文案**：后端与网络错误不走 i18n，原样显示。
- **`theme.js`**：首帧由 `index.html` 的内联脚本定，运行时切换只在 `theme.js`；两边判断必须一致 —— 深色加 `data-theme` 属性、其余删属性，不往 `localStorage` 写 `'light'`。
- **`Detail`**：鼠标侧键 3 是后退，4 不接。
- **`MyList`**：打开表单前先落 `gameId`，`GameForm` 的 `$effect` 与 `open` 在同一帧读到的是最终值。
- **`DetailCharacters` / `CharacterEditor`**：立绘走封面那条 `coverSrc` 管线，缩图与 asset 协议授权都在那边做过；缺失词条回落 key 会显示成 `detail.xxx`，靠它区分并保留原文。
- **`CharacterMergeDialog`**：两侧立绘一起预取，面板关掉时结果作废。
- **`CharacterEditor`**：回选只在「角色列表变化」时发生（加载 / 增删 / 保存回填），用 `untrack` 让它只依赖 `characters`、不追踪 `selectedId`，用户手动点开或收起不会被抢回。
- **`vndb-search.svelte.js`**：换词重搜先清空，避免旧结果与新请求之间的空窗显示错配；表单每次打开都要清一遍；导出的是 getter 而不是 `$state` 本体（直接 `return` 只会把初始值交出去）。侧栏内容由 `Modal` 的 `aside` 渲染，关闭时那一段没有挂载，父组件的工具栏按钮因此调不到里面的东西。
- **`confirm.svelte.js`**：`action` 必须在 `i18n-zh.js` 有 `confirm.<action>.{title,message,confirm}` 三条词条，缺了不做兜底（`t()` 会回落 key）。
- **`toast.svelte.js`**：渲染在 `components/Toaster.svelte`，挂在 `App.svelte`；计时器簿记不放进 `$state`（每次读写都会触发更新）；暂停要手动记剩余时长 —— 进度条的暂停靠 CSS `animation-play-state`，计时器在 JS 里，两者不通。

## 前端 lib 的比对契约

`char-merge.js` / `vn-merge.js` 分别是角色粒度与作品粒度的 VNDB 同步比对，两者同口径：本地优先，冲突逐个取舍。面板文案在组件侧按 key 查词条。

- 后端 `trimToNull` 会把空串变 `null`，两边都要归一成同一个空值
- `'80'` 与 `80` 判等；空值只与空值判等，`0` 是真实取值
- 集合语义：顺序不同按相等处理（标签与开发商）
- 比对字段表只收内容字段：`sort_order` 是位置，`id` 是存储身份
- 配对键优先取原名，罗马字名重名会把两个不同角色判成同一个
- 表单这边为空的不算冲突，直接取 VNDB 值；非空字段只在 `selected` 里才取 VNDB 的值
- `kept` 含「同名且内容一致」的本地角色：它们既不是新增也不需要取舍；合并结果里 `adopted` 命中的冲突取 VNDB 那份，其余一律保留本地 —— 包括本地独有的角色，用户手建的同步后原样留着
- 角色草稿留着 `source`（原样取回的那一行），保存时铺在最前、只覆盖表单露出的字段：本地编辑不露出的 traits / 三围 / 生日等靠它原样带回，否则改一次名字就把 VNDB 的考据数据抹了
- 性别在 VNDB 是按 release 逐条返回的数组，表单只暴露一个值给用户选
- 卡片名优先级与详情页一致：原名在前；顺序是角色的唯一真相，不另存序号
- 表单字段的形态与 VNDB 不同（逗号串 vs 对象数组），`read` 取出的值就是该填进表单的形态；`key` 同时是表单字段名与 i18n 词条的尾段

## 样式坑集

改样式前扫一眼这里，每一条都对应过一次「改完就坏」。

**高度与滚动**

- 高度链：`body` 定高 `100vh` → `.shell` 吃掉剩余 → `.shell__main` 内滚动

**卡片与浮层**

- `:hover` 的 `transform` 会生成层叠上下文，菜单展开时整卡抬到相邻卡之上
- 可见表面与裁切都归 `.cover-card__frame`，菜单是它的兄弟而非子元素才逃得出裁切
- 卡片 `overflow: hidden` 承担浮层底角，不能去掉

**详情页氛围底**

- 上内边距归零：氛围底 `absolute` 在 `.detail` 的 padding box 上，`padding-top` 会把它推下去
- 内容起点由 `padding-top` 决定，氛围底的高度与 mask 停点只决定它画到哪：上移内容不改变覆盖范围，但栏体可能落进实色区而不是淡出段，改这里要同时看 `.detail-info-line` 的落位
- 氛围底必须是 `.detail` 的直接子元素才能左右满幅，挪进 `.detail-banner` 会各留一条空白边
- mask 必须分段：压回两站时收口线附近仍接近不透明，底图会被切出硬边。实色边界 = 高度 × 停点%，改高度后要保持边界不变就得同步停点
- 模糊层 `scale 1.3` 是为了盖住 blur 向外溢出的透明边，否则四边露出未模糊的原图边缘
- 氛围底的中间停点与 mask 停点同高，改一处另一处要同步
- 标题组：别名用负边距往标题收紧，与标题算作一组（副标题用法）
- 详情页封面与列表卡封面（`.cover-card` / `.cover-card__frame`）的尺寸、圆角、占位要一起改
- 信息栏的边框是承重结构（底色 55% 不透明度且浮在氛围底的淡出段上），去掉边框栏体就塌；背后必须有 `.detail-ambience`，删掉它这层几乎不可见

**标签与胶囊**

- 胶囊几何共用、配色按角色分开；tag 走中性色是刻意的 —— 一屏几十个 tag 全用 accent 会稀释强调色也压过标题，accent 留给信息栏里的评分与信息项
- `.pill--more` 落在 `<button>` 上：按钮的边框与字体不继承，去掉这两行会露出系统默认样式
- 简介靠固定行数夹住，依赖 `-webkit-box` 三件套，`-webkit-box-orient` 缺了 ellipsis 不出现；正文以英文为主，同样行数装下的信息量多于中文
- 属性与 trait 分组共用表格式布局：标题列定宽，长值列吃剩余宽度；简介是连续散文，铺满整宽时一行长到读不回行首
- 角色卡与所在 section 底色同档会让边界糊掉，卡片要高一档
- vn-merge 的角色行标签列装的是名字，比字段名长，栅格与作品级一致、靠折行消化宽度

**控件**

- `toggle`：hidden input 不能 `display: none`（会退出 Tab 序与无障碍树），只能压成 0 尺寸；滑块必须是它的紧邻兄弟，中间隔任何元素都会断开相邻选择器；input 是 0 尺寸，全局 `outline` 会画在看不见的地方，焦点环转到滑块上；药丸小且边框淡，外描才看得出离开轨道
- `tags`：输入框必须在容器内、跟在芯片后面（放到容器外 placeholder 会把容器撑宽），它吃掉剩余宽度，最小宽度保证空标签时也能点进去打字
- `forms`：`:focus-visible` 要清 `outline`，否则键盘 Tab 会套上全局那圈重环；数量用文本输入框而非 `number`（原生微调与页面滚动、方向键冲突）；左缘竖条常态中性灰、勾选后转 accent，左侧 padding 必须比右侧窄，竖条才贴得住边
- `dropdowns`：开合用 `is-open` 类切换、不做 portal —— 菜单与触发器同处一个层叠上下文，走 portal 会与弹窗遮罩的 `z-index` 打架
- `menus`：菜单留在触发器的层叠上下文里，不移出 DOM；`.btn svg` 是按描边线性图标定的，实心点要自己定尺；裸图标变体浮在内容图上，必须自带底色
- `sidebar`：`stroke-width` 与尺寸绑定，图标放大后线条会相对变细；`.active:hover` 必须与 `.active` 一起清，否则悬停时半透明底色会回来
- `buttons`：`.btn-primary:hover` 用 `brightness` 提亮（不过滤会瞬间跳变）；覆盖 `.btn:hover` 的规则与它特异性相同，要逐项写全才不会漏掉被按顺序冲掉的属性

**列表与其它**

- 封面墙卡宽固定、列数由 `auto-fit` 依容器实测宽度决定；`toast` 用 `column-reverse`，新通知插在最底部、不推开已有的

## 注释

代码里不写注释，前端惯例写进本文。规则与例外见 [comments.md](./comments.md)。

## 新增 UI 组件检查清单

- [ ] 颜色/间距全部来自 token，无写死值
- [ ] 只引用 L2/L3，没有直连 L1 原始值
- [ ] 样式归属正确文件（scoped / components / views）
- [ ] 复用优先：先查过已有组件类
- [ ] 面向用户的文案走 `t()`，无硬编码
- [ ] 新增与改动的行里没有注释（例外见 comments.md）
- [ ] `npm run check:dead-css` 没有输出死类
- [ ] 新增小节是规则或语义，不是代码里读得到的实现描述
- [ ] 跑过 `npm run check:frontend`

## 文案

界面文案一律走 `lib/i18n.js` 的 `t()`，词条按命名空间集中在 `lib/i18n-zh.js`，命名沿用 `ns.key`（`nav.mylist` `mylist.title`）。`t()` 缺失词条回落 key，`t(key, params)` 按参数名替换 `{占位}`。

locale 跟随系统语言，只在已登记语言中匹配（全名 `zh-CN` → 基名 `zh` → fallback）。加语言只需扩 `i18n.js` 里的 `DICT`。

需要运行时切换 locale 时，把 `lib/i18n.js` 改名 `i18n.svelte.js` 并把状态包成对象—— 见下。

### runes 与文件扩展名

runes（`$state` `$derived` `$effect` `$props`）**只在 `.svelte` / `.svelte.js` / `.svelte.ts` 里可用**，普通 `.js` 写了会运行时抛 `rune_outside_svelte`（构建和 `check:frontend` 都不拦，只有打开页面才炸）。

共享响应式状态放 `.svelte.js`。跨模块导出 state 必须包一层对象，直接 `export let x = $state(0)` 会被编译器拒绝：

```js
// ✅ lib/store.svelte.js
export const locale = $state({ current: 'zh-CN' });

// ❌
export let locale = $state('zh-CN');
```


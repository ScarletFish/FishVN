# 前端开发规范 — Frontend

## 视觉参照

组件的类名、布局、动效与交互对齐 MyAnimeDocker 的实现。不过token采用我们项目的写法。参照之前先查 [visual-ref.md](./visual-ref.md) 里的 token 对照表与语法翻译表。

## 样式的三种形态

1. **Token**（`css/tokens.css`）— 颜色/间距/圆角/字体的唯一来源
2. **Svelte scoped style** — 组件私有样式
3. **普通 CSS 类**（`components/`、`views/`）— 跨组件复用的模式

归属判断：组件独有 → scoped；可能复用 → components/；视图专属 → views/；是变量 → tokens.css。

分层的依据是**控件本身**，不是界面上的分页或 tab：同一个控件被哪几页用到都不改变它的归属，通用控件一律进 `components/`，`views/` 只放该页面专属的样式。

## Token 分层

`tokens.css` 分三层：L1 原始值（只有色值，零语义）、L2 语义色槽（描述用途，不描述外观）、L3 尺度（间距/圆角/字号/字重/字体/动效）。

**组件只能连第 2、3 层** —— 直连 L1 的原始值不会随主题变。换主题只改 L2 取值，组件代码不动。

## Token 引用规则

- 颜色/间距/圆角/字重一律引用 token，**禁止写死值**；token 单点定义在 `tokens.css`，禁止在别处重新声明
- 字号一律引用 `--text-*` token，阶层见 [theme.md](./theme.md)「字号阶层」；SVG 内文字同样适用
- **组件禁止直连 L1 原始值**，只能引用 L2/L3
- **阴影与过渡只从 L2/L3 取**，不在组件里写死；要半透明色用 `color-mix(in srgb, var(--accent) 12%, transparent)`，不新增 `-rgb` 三元组

## "先找后写"三步协议

新增任何 CSS 前：

1. 查 `tokens.css` 有没有现成 token
2. 查 `components/` 有没有现成类
3. 都没有才新写，写进正确归属文件

## 自动检查

`npm run check:frontend` 跑注释检查、token 检查、死 CSS 检查、测试与构建；**勿只跑其中一条**，否则 dist 过期「改了没生效」。

死 CSS 只警告不阻断 —— 先写样式后写 markup 是常见顺序，开发中途见红会让人养成无视输出的习惯。token 检查强制 L1/L2/L3 纪律与裸色值禁令，尚未覆盖间距与字号的写死值。

## CSS 文件结构

`styles.css` 只做 `@import`，禁止写样式；样式写进哪个文件见上面的「样式的三种形态」。

## 布局

chrome 尺寸固定，不随视口缩放：侧栏 `--sidebar-w`，标题栏 `--titlebar-h`。

封面墙的卡宽、列数、窗口化约束见 [grid-layout.md](./grid-layout.md)。

模糊优先用 `filter`。`backdrop-filter: blur()` 只在背后存在未模糊内容时才有效果 —— 背后已是模糊色块时它是空转（实测加与不加像素一致），所以只有浮在原图之上的组件才值得考虑。

作品的聚合划分、消费方的字段归属判据、写权限分类、加载策略与数据流见 [view-contracts.md](./view-contracts.md)。

## 构建与开发服务器

`vite.config.js` 里几处不能动的配置：

- 测试态把 `resolve.conditions` 设为 `['browser']`：svelte 要解析到浏览器构建，SSR 构建里 `mount()` 直接抛错。非测试态**完全不写 `resolve` 键** —— 写成 `conditions: []` 同样会顶掉 vite 的默认条件，产物里会混进 svelte 的服务端模块（构建不报错，只有运行才炸）
- dev server 的 `host` / `port` 必须与 `tauri.conf.json` 的 `devUrl` 同协议族、同端口：写 `localhost` 会让 vite 只绑 `::1`，devUrl 的 IPv4 字面量就够不着；`strictPort: true` 让端口占用直接失败，不自动往上换 —— 换了端口 devUrl 就指空
- `/dev-http` 是唯一的跨源转发入口，指向开发期 HTTP 通道；生产构建里没有它

## 注释

代码里不写注释，前端惯例写进本文与索引里的各分册。规则与例外见 [comments.md](./comments.md)。

## 新增 UI 组件检查清单

- [ ] 新增与改动的行里没有注释（例外见 comments.md）
- [ ] `npm run check:dead-css` 没有输出死类
- [ ] 往 `docs/dev` 增行前先列条目过目，并按 30 秒判据筛过
- [ ] 跑过 `npm run check:frontend`

## 文档索引

| 要做什么 | 读哪份 |
|---|---|
| 配色、主题、字体与字号阶层 | [theme.md](./theme.md) |
| 动画、主题过渡、共享元素、分波入场 | [motion.md](./motion.md) |
| 布局外壳、启动页、弹窗、水平分页 | [shell.md](./shell.md) |
| 滚动条选型与外观 | [scrollbar.md](./scrollbar.md) |
| 组件惯例与交互契约 | [components.md](./components.md) |
| lib 模块契约、比对、刷新广播、runes | [libs.md](./libs.md) |
| 样式踩坑记录 | [styles.md](./styles.md) |
| 界面文案、标签与特征译名 | [i18n.md](./i18n.md) |
| 卡片宽度、列数与窗口化 | [grid-layout.md](./grid-layout.md) |
| MyAnimeDocker 的 token 对照 | [visual-ref.md](./visual-ref.md) |

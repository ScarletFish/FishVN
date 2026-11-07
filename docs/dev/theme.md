# 主题与字体 — Theme

## 主题

浅色是 `:root` 默认值，深色在 `[data-theme="dark"]`。用户可选浅色 / 深色 / 跟随系统三态，存的是选择而非解析结果：`index.html` 首帧前按 `fishvn.theme` 判定，值为 `system` 时看 `prefers-color-scheme`，解析成深色才挂 `data-theme`，无属性即浅色。运行时由 `theme.js` 的 `setTheme(mode)` / `syncTheme(mode)` 做同一套解析；选中 `system` 时 `theme.js` 常驻一个 `matchMedia` 监听，系统切换深浅就地重绘，不需要用户重启或重进设置。Rust 侧窗口背景色同样按「选择 → 解析」二段处理（`main.rs`）。

强调色系是正交的第二轴：`[data-accent="…"]` 挂色系名（amber/ocean/sakura/emerald/violet），绯红是默认、不打属性。深浅由 `data-theme` 管、色系由 `data-accent` 管，任意组合；深色下的色系值用 `[data-theme="dark"][data-accent="…"]` 双属性选择器覆盖浅色值。`index.html` 首帧前同时定 `data-theme` 与 `data-accent`。新色系补齐 tokens.css 的三处：L1 原值（accent 族与配套 fg 族）、浅色与深色两个覆盖块、设置面板色块的 `--swatch-*`；faint/bg/border/selected 等派生槽引用 `--accent`，自动跟随。

- **`theme.js`**：运行时切换只在 `theme.js`，首帧另由 `index.html` 内联脚本按同一套解析定（两边必须一致）。落定深浅的 `paint` 同时广播给 `subscribeTheme` 的订阅者：组件要知道当前深浅就订这个，别自己读 `data-theme`、也别另挂 `MutationObserver`，否则设置页改主题、系统深浅切换这两条路都会漏。侧栏的日夜按钮即此用法。
- **`card-prefs.js`**：封面卡片的标题常显开关，存 `fishvn.cardTitle`，生效方式是给根元素挂 `data-card-title`，样式只认这个属性。与 `theme.js` 同属本地偏好、不进后端配置——只影响本机展示，不需要跨端同步。关闭时删掉属性而不是写成 `'off'`，与 `theme.js` 浅色删 `data-theme` 同一写法，CSS 里也就只有 `'on'` 一个取值要判断。

## 字体

拉丁走 DM Sans：可变字体拉丁子集打包在 `assets/fonts/dm-sans-latin.woff2`（OFL 许可文本同目录存放），`fonts.css` 声明 `@font-face`，字重轴 100–1000 全覆盖，新字重直接取用。CJK 走 `@fontsource-variable/noto-sans-sc`（OFL，族名 `Noto Sans SC Variable`）：101 个按 `unicode-range` 切片的可变字体，在 `fonts.css` 里 `@import`，构建时进 `dist`，浏览器按用到的字符取片；升级换版本即换包。`--font-sans` 中 CJK 族排在系统字体之前；两族之外的字符（斜体、扩展区汉字）回落字体栈里的系统字体。

DM Sans 的 `@font-face` 带度量覆盖：`ascent-override: 106% / descent-override: 30% / line-gap-override: 0%`。显式行高下基线位置只由 ascent−descent 决定，0.76em 让 CJK 字形在行盒内视觉居中，中英混排错位在度量层一次修掉，不要在组件里打 translate 补丁；1.36em 总高是刻意保留，normal 行盒高度不变（项目全部显式行高，总高不参与布局）。改这三个值前先想清楚这层。

这份 DM Sans 子集不带 `tnum` 特性，`font-variant-numeric: tabular-nums` 对它无效——写了不报错，数字宽度仍随字形变（「1」明显窄于「0」）。需要数字定宽的地方（详情页评分这类会随数据整条切换的数值）逐字符渲染，每个字符给 `width: 1ch` 的格子，等于手工补等宽；不要改用固定 `rem` 的列宽兜——那只会把差额堆到列尾，表现为数字与后面标注之间偶尔多出一截空隙，切换时更像抖动。

## 动效与减少动画

减少动画的判定源统一在 `lib/motion.js`：应用内设置存 `fishvn.reduceMotion`，开启即强制减少；未开启跟随系统 `prefers-reduced-motion` 并实时同步系统变化。判定结果由它落到 `<html data-reduce-motion>`，CSS 只按这个属性降级（`motion-reduce.css`），不再各处写 media query；JS 消费方（`scroll-anim`、`hero-transition`、`theme.js` 的 `viewTransition`、`CharacterEditor`）读 `shouldReduceMotion()`，reduce 时直接跳过而不是降级。降级原则：去掉位移与缩放，保留淡入淡出；加载指示器（spinner、进度）是功能性状态，不参与减少。

## 字号阶层

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

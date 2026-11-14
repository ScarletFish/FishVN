# 图表 — Charts

图表统一走 Unovis。本文定接入方式、取色、交互与测试；单张图的形态另议。

## 依赖与导入

- `@unovis/svelte` 与 `@unovis/ts` 锁同一精确版本：`package.json` 写 `1.8.1`，升级时两个一起动（peer 是精确匹配）
- 导入走子路径：`@unovis/svelte/containers/xy-container`、`@unovis/svelte/components/line`、`@unovis/svelte/components/axis`
- 根导入的代价是连带打包 `maplibre-gl-worker`（约 484 kB）

## 分包

- 图表只在统计页出现，统计页是 `App.svelte` 里的动态导入，Unovis 因此只出现在异步 chunk 里
- 新增图表复用同一个导入点
- 主包口径：静态引入 Unovis 约 +165 kB / gz +52 kB，分包后约 +3 kB。换成 `sankey`、`graph`、`topojson-map` 这类重组件前，先量 chunk 增量

## 取色

| 用途 | 取值 |
|---|---|
| 数据主体（折线、柱、进度条、普通数据点） | `--chart-main` |
| 面积、淡底 | `--chart-fill` |
| 极值、榜首、交互命中 | `--accent` |
| UI 层（卡片、轴、网格、标签） | `--fg-*`、`--border-*` |

- 颜色写成 `var(--chart-*)`、`var(--accent)` 这类引用；Unovis 的 `color` 与 `--vis-color*` 接受字符串，可以直接把变量传进去
- `--chart-main` / `--chart-fill` 由 `var(--accent)` 派生，深浅各一套；`data-accent` 挂在 `documentElement`，6 套色系自动跟随
- `--accent` 每张图集中在一两处：极值带数值标注时，颜色就留给那一处

## 主题与 CSS 变量

- 覆盖集中写在图表容器上（如 `.activity-plot`），与 token 定义分开
- 常用变量：`--vis-color0`、`--vis-font-family`、`--vis-axis-tick-color`、`--vis-axis-tick-label-color` / `-font-size`、`--vis-axis-grid-color`、`--vis-axis-domain-color`、`--vis-scatter-point-label-text-*`、`--vis-tooltip-*`
- 暗色：Unovis 的暗色选择器原生包含 `html[data-theme="dark"]`，与项目挂载方式一致，主题切换直接生效，`theme.js` 保持原样
- 深色下 `--vis-dark-tooltip-background-color` / `-text-color` / `-border-color` 接管浮层，改浮层观感时连这三个一起覆盖
- tooltip 留在图表容器内，容器上的 `--vis-*` 覆盖才生效
- `VisXYLabels` 默认给每个标签垫一块背景色，柱顶、点旁的纯文字标签传 `backgroundColor="transparent"`

## 交互

- tooltip 由数据点触发：`triggers={{ [Scatter.selectors.point]: fn }}`，选择器从 `@unovis/ts/components/scatter` 取
- 浮层放该点的数值，月份交给 x 轴标签
- 交互用散点触发 + tooltip：命中严格，换来安静。crosshair 的竖线随指针移动、没有延迟项，适合沿线扫描的图；这类小图交给散点

## 测试

- 口径同 [testing.md](./testing.md)：断言落在可观察结果上
- 图表 DOM 测试走 `src/test/chart-dom.js` 的 `mountChart`：补齐 jsdom 缺的 API、处理 `flushSync` 与渲染等待，清理时恢复原型
- jsdom 需补：`ResizeObserver`、非零 `getBoundingClientRect`、`getBBox`、`getComputedTextLength`、`SVGElement.prototype.transform`
- 断言钉行为：数据点数与桶数一致、轴文本来自数据、切换筛选重新取数、交互节点挂载
- 样式不进断言：取色、字号、栏宽、hover 时序随设计调整，由人看运行中的应用判断
- jsdom 测不出真实布局下的裁切与浮层显隐，视觉判断以应用里的实际呈现为准

## 新增图表检查清单

- [ ] 子路径导入，量过 chunk 增量
- [ ] 取色只引用 `--chart-*` / `--accent` / UI 中性 token
- [ ] 轴、网格、字体、浮层覆盖项目 token
- [ ] 极值 accent 集中在一两处，带数值标注
- [ ] 用 `mountChart` 补一条行为用例
- [ ] 跑过 `npm run check:frontend`

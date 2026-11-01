# 滚动条 — Scrollbars

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

- `watchRightEdge` 与滚动实例都只对主视图生效（`MyList` 靠实例还原列表位置）
- overlay 滚动条浮在内容之上、不占布局位：右对齐的元素（数量、徽标）右内距要写成 `calc(<原内距> + var(--scrollbar-size))`，否则被手柄压住
- 主视图的 `autoHide` 取 `'never'`，显隐由 `.is-visible` 控制，淡出时长在 CSS；`opacity: 0` 的初始态只写在主视图自己的滚动条上（初始化时打 `.is-main`），不可用 `.shell__main .os-scrollbar` 这类后代选择器——会连面板滚动条一起压成透明
- `watchRightEdge` 取竖滚动条走 `instance.elements().scrollbarVertical.scrollbar`，不用 `querySelector`——面板滚动条在文档序里更靠前，会被先选中
- 指针离开整个窗口时清掉 `dragging` 与 `lingering`

外观（`components/scrollbars.css`）：

- `theme` 只给结构，尺寸与色值走 token：竖带、手柄、拉伸位宽度写死，`transition` 列全位置项
- 手柄色值只有一个来源 `--scrollbar-thumb`，overlay 族与 `.scroll-thin` 族共用；它必须是不透明色（混入 `--bg-surface` 定明度）——带透明度会让手柄跟着所在容器的底色走，主区与面板滚出深浅差
- `.scroll-thin` 用 `::-webkit-scrollbar-*`：`width` 取 `--scrollbar-size`，`::-webkit-scrollbar-button` 置 `none`，轨道与角落透明，手柄悬停时取 `--scrollbar-thumb`；同一元素不写 `scrollbar-width` / `scrollbar-color`

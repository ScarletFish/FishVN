# 前端开发规范 — Frontend

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

`npm run check:frontend` = CSS token 检查（`scripts/check-css-tokens.js`）+ 构建 dist/web/。
`npm run check:tokens` 可单独跑检查。

检查项：① `var(--x)` 必须已定义 ② 组件禁止引用 L1 原始值 ③ L2 内禁止裸色值 ④ 组件与 Svelte `<style>` 内禁止裸色值。

改完前端必跑，勿只跑其中一条，否则 dist 过期「改了没生效」。

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

库网格的卡宽、列数、窗口化约束见 [grid-layout.md](./grid-layout.md)。


## 新增 UI 组件检查清单

- [ ] 颜色/间距全部来自 token，无写死值
- [ ] 只引用 L2/L3，没有直连 L1 原始值
- [ ] 样式归属正确文件（scoped / components / views）
- [ ] 复用优先：先查过已有组件类
- [ ] 跑过 `npm run check:frontend`


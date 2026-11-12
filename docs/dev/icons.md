# 图标规范 — Icons

UI 图标的单一来源、几何参数与取用方式。

## 单一来源

- UI 图标只从 `components/Icon.svelte` 取用；组件不内联 path、不拼 SVG 字符串、不用 `{@html}` 注入图标。
- 新增图标前按 [frontend.md](./frontend.md)「先找后写」查现成资产；同一语义只保留一个图标，命中就复用。
- 图标以语义命名（`caret-left`、`check`、`calendar`），不以外观、颜色或所在页面命名。
- 品牌鱼形是品牌资产，唯一 path 见 [shell.md](./shell.md)。

## 几何与字重

- 图标沿用图标集自带的 viewBox 与填充方式：`phosphor-svelte` 为 256×256、`fill=currentColor` 的填充路径。不假设 24×24 网格或 stroke 描边。
- 常规 UI 图标统一 `regular` 字重；其他字重需在本文件登记用途后使用。
- 颜色恒为 `currentColor`，随 L2 语义色；组件与 CSS 不覆盖 `fill` / `stroke` / `viewBox` / `stroke-width`。
- 全局默认值由 `IconContext` 在应用根统一给定，各调用点不重复写 `weight` / `color`。

## 尺寸与颜色

- 显示尺寸取四档：12 / 16 / 20 / 24 px，分别引用 `--space-3` / `--space-4` / `--space-5` / `--space-6`，由 `.icon--*` 档位类给出；不用图标组件的 `size` prop 写数值。
- SVG 与 CSS data URI 里禁止写死色值。
- CSS `background-image` 里的 SVG 不继承 `currentColor`；需要随主题变色的图形用内联 SVG 或 `mask-image` 配 L2 语义槽。

## 状态与可访问性

- 状态由表面层级与 accent 表达，图标不随选中态换形态；实心仅用于图形本身语义要求的场合。
- 装饰性图标带 `aria-hidden="true"`；图标按钮的可访问名由按钮的 `aria-label` 或可见文案承担。
- 悬停、选中、禁用只改颜色与表面，不改图标本身。

## 例外

- 窗口控制按钮（最小化 / 最大化 / 还原 / 关闭）：14×14 自绘，保持桌面系统惯例，不套通用图标，也不受尺寸档位约束。
- 图表画布（`ActivityChart` / `EraChart` / `DetailPlayLog`）是图形不是图标。
- 测试里的占位数据不受本规范约束。

## 图标集登记

- `phosphor-svelte@3.1.1`，MIT，社区维护的 Svelte 5 封装；上游 Phosphor Icons 同为 MIT，分发时保留 LICENSE。
- 按 `phosphor-svelte/lib/XxxIcon` 单点导入，不用具名 barrel 导入。
- 整套界面只用这一套图标集，不混搭；新增图标优先从该集取。

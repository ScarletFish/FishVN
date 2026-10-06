# 视觉参照 — Visual Reference

组件的类名、布局、动效与交互都对齐 MyAnimeDocker 的实现，各弹窗与表单共用同一套命名。

源码位置 `D:\My Projects\MyAnimeDocker\frontend\src\`：

```
css/tokens.css          token 定义
css/components/         组件样式
css/light.css           浅色覆盖
components/  views/     组件结构与行为
```

## 抄之前先看浅色覆盖

MAD 的默认主题是深色：`tokens.css` 的 `:root` 是深色值，`[data-theme="light"]` 块与 `light.css` 是浅色值。我们的默认是浅色，对齐浅色一侧。

先在 `light.css` 里搜同名类，看有没有覆盖 —— 按钮底色、hover 边框色、输入框 focus 光环都在那里被改过。

## 语法翻译

| MAD | 我们 |
|---|---|
| `@utility .foo { }` | `.foo { }` |
| `&.active` / `&:hover` | 展平成 `.foo.active` / `.foo:hover` |
| `@apply flex items-center gap-2` | `display: flex; align-items: center; gap: var(--space-2)` |
| `border-border` | `border: 1px solid var(--border-default)` |
| `text-content-secondary` | `color: var(--fg-secondary)` |
| `rgba(var(--accent-rgb), var(--alpha-subtle))` | `color-mix(in srgb, var(--accent) 12%, transparent)` |
| `calc(1rem * var(--scale))` | `1rem` |

## token 对照

### 前景与 accent

两边等值。

| MAD | 我们 |
|---|---|
| `--fg-primary` | `--fg-default` |
| `--fg-secondary` | `--fg-secondary` |
| `--fg-muted` | `--fg-muted` |
| `--accent` | `--accent` |
| `--accent-soft` | `--accent-soft` |
| `--accent-gradient` | `--accent-gradient` |

### 表面

MAD 的浅色层级不单调 —— `--bg-elevated`（弹窗底）比 `--bg-surface`（按钮底）还暗。我们把层级拉直：弹窗底在 `--bg-surface` 档，其上浮起的元素在 `--bg-raised` 档。

| MAD | 角色 | 我们 |
|---|---|---|
| `--bg-deep` | 页面底 | `--bg-page` |
| `--bg-elevated` | 弹窗底 | `--bg-surface` |
| `--bg-surface` | 弹窗上浮起的按钮 | `--bg-raised` |
| `--bg-card` | 卡片 | `--bg-surface` |

### 边框与遮罩

| MAD | 我们 |
|---|---|
| `--border` | `--border-default` |
| `--border-hover` | `--accent-border` |
| `--divider` | `--border-default` |
| `--scrim` | `--scrim` |

### accent 透明度

MAD 用 `--alpha-*` 配 `--accent-rgb`，我们用 `--accent-*` 直接给值。

| MAD | 我们 |
|---|---|
| `--alpha-faint` 0.06 | `--accent-faint` |
| `--alpha-subtle` 0.12 | `--accent-bg` |
| `--alpha-emphasis` 0.22 | `--accent-border` |

### 尺度与动效

`space` / `radius` / `text` / `ease` / `duration` 全部等值，名字一致，直接用。

### 阴影

MAD 的阴影是单层大扩散值，我们只有三档。照层级对应：`--shadow-sm` / `--shadow-md` / `--shadow-raised`，不逐值复制。

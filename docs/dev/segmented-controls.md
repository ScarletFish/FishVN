# 分段控件 — Segmented Controls

分段控件只有两大族，按使用场景选其一：

- **设置用 `.seg-radio-group`**（`css/components/forms.css`）：设置项、表单字段、详情页横幅菜单里的单选。原生 `<input type="radio">` 撑底，项 `flex:1` 等宽填充；选中态靠 `:has(input:checked)` 提亮 + `box-shadow: var(--seg-thumb-shadow)`（凸起缩略块），并加粗；方向键导航与焦点管理原生免费，`role="radiogroup"`。
- **视图用 `.seg` / `<Seg>`**（`components/Seg.svelte` + `css/components/seg.css`）：视图工具条里的紧凑切换。`<button>` 组 + `aria-pressed`（`role="tablist"` 时转 `aria-selected`），宽度随内容、选中态中性高亮无凸起（`background: var(--bg-seg-thumb)`，无 `box-shadow`），支持 `count` 徽标 / `disabled` / `tooltip`，可直接 `bind:value` 连 store。

选型看**场景**而非外观：表单 / 设置项走 `.seg-radio-group`；视图内联切换（分组、作品/角色、范围/筛选 tab）走 `<Seg>`。两族共用同一套 token（`--border-default`、`--radius-sm`、`--bg-seg-thumb`、`--seg-thumb-shadow`、字号 `--text-xs` / `--text-sm`），尺寸与色值都走 token，禁止裸色值。

| 场景 | 用法 |
|---|---|
| 设置项（主题 / 时间单位 / 退出行为 / 剧透等级 / 角色范围 / VNDB 模式） | `.seg-radio-group` + `radiogroup`，每项 `<label class="seg-radio-item"><input type="radio">` |
| 详情页横幅菜单（氛围 源 / 模式） | `.seg-radio-group`，半受控，值由 meta 派生 |
| 安装行 Magpie（全屏 / 窗口） | `.seg-radio-group`，开关关闭时整体禁用置灰 |
| 列表分组（状态 / 平铺）、查询页（作品 / 角色） | `<Seg bind:value=...>` |
| 资料补全范围（聚焦 / 全部）、筛选 tab | `<Seg>`，范围带 `disabled` + `count`，筛选用 `role="tablist"` |

`<Seg>` 用法：

- props：`value`（`$bindable`）/ `options:[{value,label,disabled?,count?,tooltip?}]` / `role`（`'group'|'tablist'`）/ `label` / `onchange`
- 无副作用时直接 `bind:value` 连 store；有副作用（如 `setScope` / `setFilter` 带清理）走只读 `value` + `onchange` 转发原函数
- `role="tablist"` 自动切 `role="tab"` + `aria-selected`；`count` 自动渲染 `<b>`；`disabled` 拦截点击

外观细则：

- `.seg-radio-group` 选中态靠 `:has(input:checked)` 提亮 + `box-shadow` 凸起感；项间分隔线在相邻选中时淡出
- `.seg` 选中态 `class:is-active` + `background: var(--bg-seg-thumb)`，无 `box-shadow`（扁平感），项间分隔走 `border-right`
- 两者都不自绘 focus ring：`.seg-radio-item` 用 `:focus-visible` 的 accent outline；`<Seg>` 的 `<button>` 走全局 focus 样式

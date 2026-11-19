# 设置项与分区 — Settings

设置页是单条滚动流：六段分区排在同一列，左侧目录跟着滚动高亮、点击跳转，外壳与判定规则见 [shell.md](./shell.md)「设置页」。

## 一份配置的三处声明

| 层 | 位置 | 职责 |
|---|---|---|
| 字段清单 | `src-tauri/src/settings.rs` 的 `config_fields!` | 字段名、类型、默认值；结构与补丁合并由它生成 |
| 传输 | `ConfigPatch` / `ConfigView` | 写入是 `Option<T>` 补丁，读出是 `From<&AppConfig>` 投影 |
| 前端 | `lib/config-store.svelte.js` | 全量快照 `readConfig()` + 写回 `saveConfig()` |

`config_fields!` 分两组：`merged` 是同名直传的字段，`unmerged` 是需要变换的字段（令牌要混淆、工具路径要清洗），后者只进结构与默认值，写回逻辑手写在 `patch()` 里。

`ConfigPatch` 与 `ConfigView` 手写。加字段必须同时改这三处，`补丁字段覆盖配置字段` 与 `视图投影与配置字段一一对应` 两条用例会把漏改的挡下来。

前端只有 `config-store` 知道全部键，分区组件、详情页、查询页都从它读。

## 分区约定

- 分区组件直接 `readConfig()` 读快照、`saveConfig({...})` 写自己那几个键。
- 写回一律带单键补丁。`saveConfig` 先乐观合并，失败回滚并提示，成功后广播 `CONFIG_CHANGED`，其它视图据此重载。
- 分区自己的界面状态（令牌输入、同步计划、备份列表、更新进度）留在分区里。
- 写完由广播后的重载带回真实值：后端会清洗路径、把明文令牌换成 `has_vndb_token`。

## 外观分区的例外

主题与强调色有两个真源，双写是设计：`localStorage` 负责首屏不闪（`theme.js`），配置里那份供 Rust 建窗时读（`main.rs` 按 `settings::current().theme` 定窗口深浅色）。字号、卡片标题、减少动效只存 `localStorage`（`theme.js` / `card-prefs.js` / `motion.js`）。

## 新增一个设置项

1. `config_fields!` 的 `merged` 组加一行 `字段名: 类型 = 默认值,`。要变换或清洗的值才进 `unmerged`。
2. `ConfigPatch` 加同名字段 `Option<类型>`。「显式置空」语义靠 `double_option`：类型写 `Option<Option<T>>`，加 `#[serde(default, deserialize_with = "double_option")]`，前端发 `null` 才会落成清空。
3. `ConfigView` 加同名字段，`impl From<&AppConfig>` 里补一行。
4. 值需要清洗或校验时改 `patch()`，例如 `clean_tool_path`、`obfuscate_token`；改 VNDB 同步相关的键要让 `invalidate_plan` 跟着触发。
5. `config-store` 的 `emptyConfig()`（加载前初值）与 `normalize()`（越界回落）各补一行。
6. 在前端对应分区的 `<div class="settings-card">` 里加一个 `.form-group`。行间分隔线由 `.settings-card .form-group + .form-group` 自带；同组内连排的开关给后者加 `form-group--flat` 去线。
7. 控件照现有形态：开关走 `.toggle-switch` + `.form-row`，枚举走 `.seg-radio-group`，长列表走 `SelectDd`，路径走 `PathField`；文案进 `i18n-zh.js` 的 `settings.*`。
8. 跑 `npm run check:rust`、`npm test`、`npm run check:frontend`。

## 新增一个分区

1. `lib/settings-nav.js` 的 `SETTINGS_SECTIONS` 加 `{ id, label }`，`label` 是 i18n key。顺序即滚动流顺序，`settings-nav.test.js` 钉着现有 id 次序，加段要一起改。
2. `views/settings/` 下新建组件，根节点自己渲染 `<section class="settings-section" id="settings-<id>">` 与 `<h2 class="settings-section__title">`。锚点 id 必须与 `SETTINGS_SECTIONS` 的 `id` 对上——滚动跟随按它找节点。
3. `Settings.svelte` 的 `.settings__body` 里按顺序插入组件。窄档目录降级由外壳统一处理。
4. 分区自带的弹窗或浮层挂在本组件内即可：`Modal` 走 portal 到 `#modal-root`，层级由它接管。

## 检查清单

- [ ] 字段只在 `config_fields!` 写一次，`ConfigPatch` / `ConfigView` 同步
- [ ] 需要「置空」语义的字段用了 `double_option`
- [ ] `config-store` 的初值与归一化都补了
- [ ] 分区只读写自己的键
- [ ] 文案进 i18n
- [ ] 三条验证命令都过

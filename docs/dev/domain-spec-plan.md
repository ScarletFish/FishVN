# 领域规格重构执行台账

> 本文是**执行计划与进度台账**，不是产品规格。用途是保证长任务跨上下文不丢信息：接手者先读「当前状态」与「进度台账」，再读对应步骤的「做法」。
> 建立于 2026-11-07。授权：用户明确「全权负责、可分层提交、准备睡觉」，故文档闸门与提交过目在本任务内一次性豁免。

## 一、背景与判定

项目是单用户、单进程、本地优先的视觉小说库（Tauri 2 + Svelte 5 + rusqlite）。FDD 基本完成，**没有真实用户**，schema / IPC 协议 / FDD 均可修订——这解除了前几轮「兼容风险」的前提。

核心病因：同一个元数据概念在后端散成 6 份手写清单（20 / 20 / 18 / 18 / 18 / 11），前端另有 `vn-merge.js` 的 `FIELDS`（11 项）与 `form-metadata.js` 的补丁构造；四路写路径（批量补全 / 单部同步 / 账户同步 / 手动修正）各写一套「空值」「相等」「冲突」判定。已实锤的后果：`null` 清空契约断裂、`filled_labels` 漏 7 字段使 `patch_for` 存在静默丢补丁隐患、加批量匹配工作台时摩擦极大。

**判定**：框架本身（薄命令层 + 领域事务脚本 + 单连接 + 无环依赖）健康，不换、不整体重写。只重做「数据模型 / 合并管道 / 任务传输」三层。

同类调研结论（beets 的 `_fields`、Calibre 的 `FieldMetadata`+`NULL_VALUES`、EmulationStation 的 `MetaDataDecl`、tMM 的 `MediaMetadata.mergeFrom`、Jellyfin 的 `TaskProgress`、Picard 双栏匹配台）：**字段只有一份声明，空值/相等/合并/序列化都挂在字段上**；工作台形状（左清单、右修正台）本身是对的。

## 二、目标架构

| 层 | 内容 |
|---|---|
| L0 | `src-tauri/spec/metadata.json`：语言无关规格，唯一事实来源 |
| L1 | `src-tauri/build.rs`：构建期生成 Rust 与 JS 两侧代码；生成物入库 |
| L2 | `Observation { field, value, source, confidence, at }` → `merge(current, obs, policy) -> Decision`；四路写路径只是 policy 不同 |
| L3 | 任务事件携带增量，前端只做投影 |

**多源扩展的含义**：将来加 Bangumi / AniList 只是「新增一个 provider 产出 Observation + 一张来源优先级表」，不动字段模型。`source` / `editable` / `user_override 上锁` 必须在 L0 就声明，否则多源一来会盖掉用户手改的值。这套基建**不是为多源才做**——它先解决当前的清单不同步与合并策略分散，多源是免费的选项。

**不适用**：CRDT / 多端同步 / sidecar 导出 / 多 provider 插件 / Event Sourcing / ORM / 整体重写 / 集中式 store。理由见 `docs/fdd-game-library.md` 定位与 `AGENTS.md` YAGNI。

## 三、规格表设计

`src-tauri/spec/metadata.json`，`fields[]` 每项：

| 属性 | 含义 |
|---|---|
| `key` / `sql` | Rust 与 JS 字段名 / SQL 列名 |
| `ty` | `text integer real bool titles tags strings staff mode` |
| `clearable` | 是否 `Option<Option<T>>` 三态（`null`=清空、缺失=不动） |
| `blank` | `trim_empty` / `null_only` / `empty_list` / `not_applicable` |
| `label` | 补全报告中文标签；`null` 表示不参与报告（本地视觉字段） |
| `source` | `upstream` / `local` / `system` |
| `editable` | 用户是否可本地编辑 |
| `in_patch` | 是否属于 `MetadataPatch` |

当前 20 字段：`game_id`+`image_id` 为 system 且不在 patch；11 个 upstream 有 label；7 个 local 视觉字段无 label；18 个在 patch；17 个可编辑（仅 `titles` 只读展示）。

**关键**：`label: null` 不是「漏了」，是显式分类。`filled_labels()` 遍历规格表只取有 label 的，`is_set()` 由生成器覆盖全部 18 个 patch 字段，从此不可能出现「加字段忘了加进报告」。

## 四、现状盘点（改动前）

| 手写清单 | 位置 |
|---|---|
| DDL 20 列 | `src-tauri/src/db.rs:76` |
| `COLUMNS` 20 列名 | `src-tauri/src/metadata.rs:9` |
| `MetaRow` 20 字段 + `read_row` + `to_api` | `src-tauri/src/metadata.rs:12,231,256` |
| `GameMetadata` 20 字段 | `src-tauri/src/types.rs:202` |
| `MetadataPatch` 18 字段 | `src-tauri/src/types.rs:254` |
| `filled_labels` 11 项 | `src-tauri/src/types.rs:292` |
| `build_columns` 18 分支 | `src-tauri/src/metadata.rs:97` |
| `patch_for` 18 个 `fill(blank, map)` | `src-tauri/src/backfill.rs:358` |
| `FIELDS` 11 项 `{key,read,empty,same}` | `frontend/src/lib/vn-merge.js:82` |
| `buildMetadataPatch` 17 字段 | `frontend/src/lib/form-metadata.js:92` |
| 角色合并另一套 | `frontend/src/lib/char-merge.js` |
| 任务事件 | `src-tauri/src/tasks.rs:75` `Task`/`EventLog`/`app.emit`；前端 `events.js` 已把 payload 解析出来交给 `onEvent`，但 `backfill-store.svelte.js:73` 只 `schedule()` 丢掉它 |

## 五、分步计划

每步独立验证、独立 git 提交。验证统一为：`npm run check:rust` + `npm test` + `npm run check:frontend` + `git diff --check`。

### 步 0 · 基线固化 · 已完成于首轮切片

- 产出：`spec/metadata.json`、`build.rs` 生成逻辑、`src/generated/metadata.rs`、`types.rs::filled_labels` 委托、2 条回归用例、`Cargo.toml` 加 `serde_json` 到 build-dependencies。
- 验证：337 passed / 5 skipped；生成物 sha256 前后一致；故意改规格时 `FISHVN_GENERATED_CHECK=1` 能抓到且不改写文件。
- 待办：把 `FISHVN_GENERATED_CHECK=1 cargo check` 加进 npm scripts 或 CI。

### 步 1 · 规格生成 Rust 侧全部数据模型代码

**目标**：`types.rs` / `metadata.rs` / `db.rs` 里与元数据字段相关的手写清单全部由生成器产出。

**做法**（按风险从低到高，逐项跑验证）：
1. 生成 `pub const COLUMNS: &str`（20 列名）→ `metadata.rs` 删本地 `COLUMNS`，改 `use`。
2. 生成 `pub fn patch_columns(patch: &MetadataPatch) -> Result<(Vec<&'static str>, Vec<Value>)>`：按 `ty` + `clearable` 产出 18 个分支。映射规则：`text/mode→Value::Text`、`integer/bool→Value::Integer`（bool 用 `i64::from`）、`real→Value::Real`、`titles/tags/strings/staff→Value::Text(serde_json::to_string)`；`clearable` 时外层 `Some/None` 分别写值或 `Value::Null`。**必须保持 `metadata.rs` 现有分支顺序**（UPDATE 的列顺序影响测试与行为）。
3. 生成 `pub fn is_blank_current(meta: &GameMetadata, key: &str) -> bool`：按 `BlankRule` 产出；`backfill.rs::patch_for` 改用它，删 `blank_str` / `blank_list` / `fill` 三个手写 helper 中的重复判定（`fill` 可保留为泛型工具）。
4. 生成 `GameMetadata` / `MetadataPatch` struct 本体（含 18 处 `#[serde(default, deserialize_with = "double_option")]`），`types.rs` 改为 `pub use crate::generated::metadata::{GameMetadata, MetadataPatch};`，`filled_labels` 的 impl 一并搬到 generated 或保留在 types.rs（Rust 允许同 crate 内 impl）。
5. 生成 `MetaRow` / `read_row` / `to_api`：JSON 列与非 Option 字段需按 `ty` 区分 `from_json` 与 `row.get`。
6. 生成 DDL：规格表补 `sql_type` / `not_null` / `default` 三个属性，**先只写 parity 测试**（生成的 DDL 与 `db.rs` 现有 `CREATE TABLE` 逐字一致），parity 绿了再让 `db.rs` 引用生成的常量。

**降级路径**：5/6 风险偏高（涉及 schema 与 JSON 反序列化）。若 parity 无法快速做稳，本步保留这两处手写、但在规格表里补齐属性并加 parity 测试作为护栏，留到后续单独处理。**不允许为了「全生成」而让 schema 有丝毫行为变化。**

**验证**：后端全量 + 新增「生成 DDL 与现有 schema 一致」用例 + `patch_columns` 对 18 字段的等价性用例。

### 步 2 · 统一合并管道

- 新增 `src-tauri/src/merge.rs`：`Observation { field, value, source: ValueSource, confidence: u8, at: i64 }`、`MergePolicy { FillBlanks, UserChoice, LocalFirst, Override }`、`Decision { Applied, SkippedBlanked, SkippedLocked, Conflict, NoOp }`、`merge(current, observations, policy)`。
- L0 每字段补 `merge` 规则声明（`fill_blanks` / `user_choice` / `local_first` / `override` 的默认取值），生成器产出 `policy_for(field)`。
- 接入四路：`backfill.rs::patch_for` = `FillBlanks`；`sync.rs::judge`/`plan_ulist` = `LocalFirst`；手动修正 = `Override`；单部同步冲突面板 = `UserChoice`（JS 侧，步 3 对齐）。
- **`user_override` 上锁语义**：用户在表单改过的字段标记 locked，后续上游 Observation 一律 `SkippedLocked`。存储用 `game_metadata` 新增一列 `locked_fields TEXT`（JSON 数组，默认 `[]`）——因为无真实用户，可直接改 schema，无需迁移兼容。
- **跨语言边界**：Rust 与 JS 不可能共享同一个 `merge()` 实现。做法是**共享同一份规则声明、两侧各自生成实现**（Rust 由 build.rs 生成 `is_blank`/`is_same`，JS 由同一 spec 生成 `empty`/`same`），并用同一组策略矩阵用例分别验证。不要承诺「一份实现」。

**验证**：四路现有用例原样通过；新增策略矩阵用例（每字段 × 每 policy）；上锁字段不被上游覆盖的端到端用例。

### 步 3 · 生成 JS 侧字段契约

- 由同一 spec 生成 `frontend/src/lib/generated/fields.js`（或 `.json`）：字段清单、`empty`/`same` 判定类别、标签、可编辑性、来源。
- `vn-merge.js` 删除手写 `FIELDS`，`conflictRows` / `applyIncoming` 改用生成表 + 生成的判定函数；`FIELD_KEYS` 保留导出。
- `form-metadata.js::buildMetadataPatch` 的 17 字段构造改为遍历生成表（类型转换仍手写，规则从生成表取）。
- `VnMergeDialog.svelte` 与 `BackfillPanel.svelte` 的逐字段对比收敛为一个通用 `FieldDiff` 组件；`char-merge.js` 保持独立（角色是子实体，不属元数据字段表），但复用同一个对比组件。
- 生成物入库，前端同样加 `FISHVN_GENERATED_CHECK` 校验（Node 脚本，可与 Rust 侧共用一份生成器输出比对）。

**验证**：前端 309+ 例全绿；`npm run check:frontend`（含注释、CSS、vite build）；批量工作台与编辑表单手测路径的断言保持。

### 步 4 · 任务事件改增量投影

**重要核实结论**：`tasks.rs:123` 的 `push` 已经 `app.emit(&self.name, logged.event)`，`frontend/src/lib/events.js` 的 `wrapped` 也已把 `JSON.parse(e.data)` 完整交给 `onEvent`。**事件 payload 本来就带 `phase/done/total/detail/error/game_id/run_id`**，只是 `backfill-store.svelte.js:73` 的 `onEvent(ev) { schedule(); }` 把它丢了。

- 因此**不引 `Channel<T>`、不新建基建**：让 `onEvent` 直接把 `TaskEvent` 投影进 `backfillStore`（更新 `total/done`、当前 `active` 项、状态、错误信息），仅在需要全量清单时（首次进入、任务完结、手动修正后）才 `load()`。
- 删掉 `seq` 竞态、`pulling`/`pullAgain` 单飞、200ms 节流里不必要的全量重拉；`watchers` 引用计数保留（订阅生命周期仍需要）。
- `eventsSince` 日志拉取保留但降频。
- 若发现 `TaskEvent` 缺少投影所需字段（如逐条状态），在 `types.rs::TaskEvent` 补齐，规格驱动。

**验证**：全量测试 + 工作台暂停/停止/限流/手动修正的链路断言；`backfill-store` 的竞态相关单测改写为「事件序列 → 状态投影」表驱动。

### 步 5 · provenance 与文档收尾

- `Observation` 的 `source` / `confidence` / `at` 落库（`game_metadata` 加 `field_sources TEXT`，JSON 对象，默认 `{}`）。
- **FDD:413 的处理**：产品决策「批量仍保持仅空白」不改写。新增「改绑 VNDB 后可识别『自动填、未经用户确认』的值并重算」的**能力**（命令 + 工作台入口），默认行为完全不变，用户主动触发才生效。FDD 只补技术说明，不改产品语义。
- 写顶层约束文档（用户此前提出的薄文档第一节）：系统骨架与依赖方向、跨层不变量台账、改动影响面矩阵、性能预算指针。**先出条目清单再落盘**（若届时仍在本任务授权内，可直接写并在提交信息里列明）。
- 审计现有 `docs/dev/` 是否过期（尤其 `view-contracts.md:28` 描述了不存在的 selector 层）。

**验证**：新增错配改绑端到端用例；全量；文档条目列进提交信息。

## 六、默认决策与边界

- **不改**：FDD 产品语义、`commands.rs` 薄层、24 个模块职责与依赖方向、后端大文件（query/backfill/sync/dashboard）的目录结构。
- **不引**：ORM、CRDT、Event Sourcing、集中式 store、多 provider 插件、Tauri `Channel<T>`（现有 emit 已够）。
- **不做**：为多源预先写 provider 抽象；等真实场景出现再加。
- **代码规范**：零注释（`check-comments.mjs` 白名单只放行测试意图）；测试函数名用中文；生成代码同样零注释。
- **提交**：每步跑完全量验证后单独提交，标题沿 `type: 简述` + `-` 分段正文；提交信息里列清验证结果。

## 七、已知风险与缓解

| 风险 | 缓解 |
|---|---|
| 生成物入库但 Cargo 只监控 spec，手改生成文件不会被普通构建纠正 | `FISHVN_GENERATED_CHECK=1 cargo check` 进 CI；步 0 已验证能抓到 |
| 代码生成让报错信息与 IDE 跳转变差 | 生成物入库 + 保持生成的 struct/枚举是普通 Rust 源码 |
| 规格表变成新的「难读中心」 | 每字段单行 JSON、属性只有 9 个；不做嵌套分组 |
| 步 1 的 DDL / 行映射改动涉及 schema | parity 测试先行，不稳则降级保留手写 |
| 步 2 的 `locked_fields` / `field_sources` 加列 | 无真实用户，直接改 schema，无需迁移；但必须在 `db.rs::ensure_schema` 同步并跑全量 |
| 跨语言 merge 实现 | 共享规则声明、各自生成实现、同组用例双验，不承诺一份实现 |

## 八、进度台账

| 步 | 状态 | 提交 | 备注 |
|---|---|---|---|
| 0 | 切片已完成，待 CI 校验与提交 | — | 337 passed / 5 skipped |
| 1 | 未开始 | — | |
| 2 | 未开始 | — | |
| 3 | 未开始 | — | |
| 4 | 未开始 | — | |
| 5 | 未开始 | — | |

**接手指引**：先跑 `git log -6 --oneline` 与 `git status` 确认进度，再对照上表找到下一步；每步开始前重读该步「做法」与对应源码行号（行号会随改动漂移，按函数名定位）。

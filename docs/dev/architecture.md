# 系统骨架与约束 — Architecture

本文只写源码里看不出来的四样：依赖方向、不变量由谁守、改动要同步哪些位置、性能的已知边界。模块清单见 [backend.md](./backend.md)，视图与字段落位见 [view-contracts.md](./view-contracts.md)，产品决策见 [fdd-game-library.md](../fdd-game-library.md)。

## 骨架与依赖方向

| 层 | 职责 | 代表 |
|---|---|---|
| IPC 入口 | 一行转发，不含业务 | `commands.rs`；开发调试镜像 `devhttp.rs` |
| 领域 | 事务脚本与查询，按模块扁平组织 | `games` `metadata` `backfill` `sync` `query` `sessions` … |
| 数据 | 单条 `Arc<Mutex<Connection>>`，全进程共享 | `db.rs` |
| 契约 | IPC 结构体与数据库行映射同源 | `types.rs` + `generated/metadata.rs` |
| 前端 | 唯一 IPC 网关 + 视图自持状态 | `api.js` / `views/` / `*.svelte.js` |

允许的方向：入口 → 领域 → 数据；领域之间单向引用（编排层 `sessions` `backfill` 可引用叶子模块）。`settings` `tasks` `thumbs` `metadata` `characters` 是叶子，不反向引用入口。进程级静态只用于明确的缓存或单例（`settings::CACHE`、`sync::plan_cache`、`tagdict::DICT/GROUPS`），新增前先确认它不是可以随连接走的普通状态。

前端 `views/` 自取数、自持 `$state`；`lib/` 里的 `*.svelte.js` 是模块级单例，四套事件总线（`events` / `library-events` / `config-events` / `task-events`）各管一类广播。单窗口桌面应用不设集中式 store。

## 数据模型单一来源

`src-tauri/spec/metadata.json` 是元数据字段的**唯一事实来源**。`src-tauri/build.rs` 在构建期由它生成两侧代码，生成物入库：

| 生成物 | 内容 |
|---|---|
| `src-tauri/src/generated/metadata.rs` | `GameMetadata` / `MetadataPatch` / `MetaRow` / `FieldSpec` / `METADATA_FIELDS` / `COLUMNS` / `DDL_COLUMNS` / `patch_columns` / `read_row` / `to_api` / `is_set` / `is_blank_current` / `filled_labels` |
| `frontend/src/lib/generated/metadata-fields.js` | `METADATA_FIELDS` / `PATCH_KEYS` / `FORM_KEYS` |
| `src-tauri/src/generated/tag_key.rs` | `scope_of_root`（五根 → 范围） |
| `frontend/src/lib/generated/tag-key.js` | `TAG_ROOT_GROUPS` / `TAG_SCOPE` / `TAG_ROUND_ROBIN` / `TAG_GROUP_ORDER` / `tagRootCategory` / `scopeOfCategory` |

字段声明 13 个属性：`key sql ty nullable item col_def clearable blank label source editable in_patch storage`。其中 `source`（upstream / local / system）、`editable`、`blank` 同时是合并规则的输入；`label: null` 表示本地视觉字段不参与补全报告；`storage: "join"` 表示该字段不落列，由关联表组装（当前只有 `tags`）。

标签与特征的身份不在元数据列里：`tags` / `game_tag` / `traits` / `character_trait` 四张表在 `db.rs` 手写，字典标签 `g`+数字、本地标签 `u`+数字同表存放。聚合、查询、排除池一律按 id；译名与分类来自 `tags_zh.json` 快照，只进渲染。

同步校验：`npm run check:generated`（构建后比对生成物有无未提交改动）；CI 可用 `FISHVN_GENERATED_CHECK=1 cargo check --manifest-path src-tauri/Cargo.toml`，规格与生成物不一致时构建脚本直接失败。

## 跨层不变量台账

| 不变量 | 破了会怎样 | 谁在守 |
|---|---|---|
| `null` = 清空，字段缺失 = 不动 | 清空评分/备注静默失效 | `clearable` 声明生成 `double_option`；`types.rs` 三态用例；`tests/game_state.rs` 的 JSON→落库端到端 |
| 前后端补丁键名一致 | 字段静默丢失 | `form-metadata.test.js` 断言键集合等于生成的 `FORM_KEYS` |
| 生成物与规格同步 | 各处行为漂移 | `check:generated` 与 `FISHVN_GENERATED_CHECK` |
| 命令在正式/调试两条通道都可用 | 开发环境行为不同 | `devhttp` parity 测试 |
| 状态列表自洽 | 默认值不在主列表、报告顺序错 | `status.js` 由单一 `STATUS` 表派生 |
| 标签词典 `excluded` 有序 | 二分查找失效 | `tagdict` 加载时排序，由构造保证 |
| 任务中间进度不打断 UI | 每次事件都全量重拉 | `backfill-projection.js` 纯函数投影，`needsReload` 只在终态触发重拉 |
| 任务日志不重复、不丢序 | 推送与历史补读各写各的，日志重复或缺行 | `TaskEvent.seq` 由 `EventLog::push` 分配并随 Tauri emit 与 SSE 两条通道下发；`mergeLogRows` 按序号去重合并，`eventsSince` 只在首次订阅后补历史 |
| 备份还原中途失败不留下空连接 | 磁盘库被覆盖且内存库为空 | `backup.rs` 的 `Reopen` RAII 与 `restore_files` 顺序 |
| 四类同步 UPDATE 幽灵 id 一律报错 | 静默成功还计数 | `sync.rs::update_one` 统一检查 `changes == 0` |
| 补全页列表稳定、且任务从第一条往下跑 | 任务推进重建列表、选中丢失；进度不从顶部开始 | `listGames()` 全量按名称稳定排序为唯一列表源；`games.rs::GAME_ORDER_SQL` 单一常量锚定 `scan` 与 `items_by_ids`，`tests/backfill.rs` 断言三者 id 序列相等；`targetIds` 仅在 run_id 变化时冻结；任务状态只作 overlay；`backfill-list.test.js` 断言 pending→ok 时默认集合不变 |
| 库内条目删除后工作台同步 | 删了游戏列表还留着幽灵行 | `LIBRARY_CHANGED` 是列表成员变更唯一来源；三处 delete 补通知，`loadGames` 清理失效 overlay 与选中 |
| 猜测绑定能被集中复核 | 按名称自动绑定的条目跑完就找不到了，错配无从发现 | `backfill.rs::resolve_vndb_id` 以 `bind_source: search` 标记并记 `hit_count` / `candidates`；`backfill-list.js::visibleRows` 的 `review` 分支不受 `onlyNoId` 影响（自动绑定后 id 已写回库）；`accept()` 改绑 patch 把来源改成 `existing` 即自动退出；`BackfillPanel` 只对 `search` 行开放就地改绑 |

## 改动影响面矩阵

| 要改什么 | 必须同步的位置 | 必跑检查 |
|---|---|---|
| 加/改一个元数据字段 | 只改 `spec/metadata.json` 一行；生成器产出 Rust 与 JS 两侧。仍须手写上游值映射（`backfill.rs` 的 `map_*`）、表单控件（`GameForm.svelte`）、详情展示（`Detail.svelte`）——`FORM_KEYS` 契约测试与 `filled_labels` 会在字段漂移时失败 | `check:generated` + `npm test` + `check:frontend` |
| 改表结构 | 规格里的 `col_def`；其他表仍在 `db.rs` 手写 | `npm test`（建库路径全覆盖） |
| 改标签分类或范围 | 只改 `spec/tags.json`；生成器产出 Rust 与 JS 两侧，消费方读生成物 | `check:generated` + `npm test` + `check:frontend` |
| 加命令 | `commands.rs` + `generate_handler!` + `devhttp.rs` 镜像 + `api.js` | parity 测试 + `check:rust` |
| 改首屏/启动 | `index.html` 启动页、`main.js` 的 `app:ready`、`MyList` 首次加载 | `check:frontend` + FDD 第 4 节的性能口径 |
| 加查询条件 | `query.rs` 的条件构造与 `row_matches` | `npm test` |
| 改同步规则 | `sync.rs` 的判定与应用 + [vndb-sync.md](./vndb-sync.md) | `npm test` |
| 改标签展示/过滤 | [tags.md](./tags.md) 的规则与 `tagdict` | `check:frontend` + `npm test` |
| 改补全页列表或筛选 | `backfill-list.js` 纯函数 + `backfill-store.svelte.js` 的 games/targetIds/localOverlay + `Backfill.svelte`；新增库内成员变化入口要补 `notifyLibraryChanged` | `npm test -w frontend` + `check:frontend` |
| 改库内列表顺序 | `games.rs::GAME_ORDER_SQL` 单一常量（同时被 `scan` / `items_by_ids` 使用）；**全局影响** MyList / Dashboard / SearchBar，须确认任务顺序仍与列表一致 | `npm test`（含顺序一致性用例）+ `check:frontend` |

## 性能边界

FDD 第 4 节已实测：启动耗时集中在 webview 导航、bundle 解析与首屏渲染，**不在数据层**；`list_games` 随库规模的量级也在该节。FDD 第 4 节末尾列了几处**有意不优化**的取舍（全量摘要常驻内存、某些派生值不缓存、手动流程留空）——改之前先重测，别把取舍当 bug。

## 重审条件

- 引入第二个元数据源 → 需要来源优先级与字段锁，届时重新评估 `Observation` 与 provenance
- 引入窗口化渲染或虚拟列表 → 摘要投影常驻内存的前提要重评
- 后端引入推送通道 → 数据流里的重拉可换成订阅
- 元数据加入截图一类大字段 → 「重字段按需再取」的边界要重划

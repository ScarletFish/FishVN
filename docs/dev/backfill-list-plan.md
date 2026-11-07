# 批量匹配工作台：全量列表重构（执行台账）

> 本文件是执行台账，不是产品规格。记录背景判定、设计边界、分步做法与进度，供接手或上下文丢失时对照。

## 一、背景与判定

用户提出工作台应改为**全量游戏列表**：默认显示「未手动下载资料的条目」「同步来的仅有 id 条目」「当次成功的条目」，可通过筛选切到库内其它条目。核心制衡点有两轮讨论：

1. **本地优先、几乎全字段可改**，所以任何「字段完整性追踪」都不可判定。用户主动清空简介、只留自选封面、删掉角色，都算完整还是不完整？没有答案。结论：**不落 provenance、不加字段级来源、不用字段完整性做运行时成员资格判据**。
2. **双功能语义相反**：补全是无人值守批处理，修正是人机交互，修正需要列表稳定。现状两者共用同一个可变任务快照，一按「开始补全」就整体替换列表。
3. 用户澄清「不移除」有点绝对：**唯一允许的移出是「库里已经没有这条」**，由已有的 `LIBRARY_CHANGED` 事件驱动，不是「处理完了」的写操作。

### 六条设计边界（最终版）

1. 列表 = `listGames()` 全量，按名称排序（`games.rs::GAME_ORDER_SQL`），永不重建（除库内事实变化）
2. `BackfillStatus.items` 降为按 `game_id` 索引的 overlay，只决定徽章与修正台内容
3. 默认筛选「本次」= 冻结的目标集 id；成功/失败/未匹配全留在里面；「全部」= 全量；其余是只读条件
4. 任务推进只改 overlay，列表成员与顺序不动
5. 修正台可选中任意库内条目，局部 overlay 前端自己记，不写回后端 `items`
6. `LIBRARY_CHANGED` 是列表成员变更的唯一来源；补三处删除通知 + 工作台订阅 + 选中项失效与还原时的 overlay 清理

**步 0 判定时后端零改动（后续因排序反馈有追加，见第十一节）**：`ApplyReport.created_ids` 已存在（`SettingsDialog.svelte:496` 已用它调 `startBackfill(created)`），`list_games` 已有稳定排序，`start_backfill(game_ids)` 已支持传 id，`status_or_plan()` 已提供冻结的计划清单。

## 二、现状根因

| 位置 | 问题 |
|---|---|
| `backfill.rs:170` `status_or_plan()` | 首次 `scan()` 生成 `items` 后存进程单例，`status()` 只要 items 非空就返回缓存，库内变化不反映 |
| `commands.rs:575` `reset(&items)` | 一按开始补全就整体替换 items，选中项与顺序当场丢失 |
| `Backfill.svelte:40-47` | 筛选维度是任务状态（pending/active/ok…），任务推进时 `pending` 视图条目逐条消失 |
| `Backfill.svelte:301` | 列表直接渲染 `status.items`，不是库内视图 |
| `BackfillPanel.svelte:195` | 修正成功 `onchanged(patch)` 写回同一个 items，两功能互踩 |
| `library-events.js` + 三处 delete | 删除游戏只发 `notifyLibraryTagsChanged()`，漏发 `LIBRARY_CHANGED`；`Backfill.svelte` 未订阅任何 window 事件 |

`SCAN_SQL` 只在进入页面时被查询一次并冻结成目标集，这是**只读查询**不是「追踪完整性」；用户之后改字段不影响当前会话列表，下次打开重算。这条边界必须守住。

## 三、目标数据流

```
listGames()  ── 全量 games（按名称稳定排序，唯一列表源）──┐
                                                  ├─ mergeCatalog → rows → visibleRows → 列表 / 修正台
BackfillStatus.items ── overlay（按 id）───────────┤
localOverlay ── 修正非目标集条目的局部状态 ──────────┘

targetIds：load() 检测 run_id 变化时，从 status.items 冻结一次
任务进度事件：只更新 status（projectEvent）与 overlay，不动 games / targetIds
LIBRARY_CHANGED：唯一允许重取 games 的事件
BACKFILL_REFRESH：只重拉 status
```

## 四、数据模型

| 字段 | 类型 | 语义 |
|---|---|---|
| `games` | `GameListItem[]` | 全量库内条目，顺序固定，唯一列表源 |
| `status` | `BackfillStatus` | 任务进度 + overlay 源 |
| `targetIds` | `string[]` | 本次目标集，冻结；「本次」筛选的判据 |
| `localOverlay` | `{ [id]: patch }` | 修正非目标集条目的局部状态，不写回后端 |
| `selected` / `filter` / `onlyNoId` | 视图状态 | 与列表解耦 |

`mergeCatalog` 产出的每行形状保持 `BackfillItemState`（`game_id/name/vndb_id/state/hit_*/candidates/filled/characters/error/bind_source`），非目标集条目 `state: null`，视图不渲染徽章。`game_id/name/vndb_id` 以 catalog 为准；修正提交时就地更新 `games` 里的 `vndb_id`，不重排。

## 五、事件边界表

| 事件 | 对列表做什么 | 理由 |
|---|---|---|
| `LIBRARY_CHANGED`（新建/删除/改名/同步应用/还原/任务完成） | 重取 `listGames()`，被删条目自然消失 | 库内事实变了 |
| 任务进度事件、终态事件 | 只更新 overlay 的 state 与进度 | 功能二要稳定 |
| `BACKFILL_REFRESH` | 只重拉 `status` | 任务/配置变化，不是库变化 |
| 切换筛选 | 只改视图条件 | 用户主动，预期内 |
| 修正提交 | 只更新该条 overlay 与就地字段 | 不是移除动作 |

选中项被删时清空选中，这是唯一「列表变化影响选中」的情况，且必要。

## 六、分步实现

| 步 | 内容 | 验证 |
|---|---|---|
| 0 | 本台账落盘 | — |
| 1 | 新增 `backfill-list.js` 纯函数（`mergeCatalog` / `visibleRows` / `countsOf`）+ 表驱动测试，守住「pending→ok 时 todo 集合不变」 | `npm test -w frontend` |
| 2 | `backfill-store.svelte.js`：`load` 只重拉 status 并在 `run_id` 变化时冻结 `targetIds`；`loadGames` 重取全量并清理失效 overlay/选中；订阅 `LIBRARY_CHANGED`；`mergeBackfillItem` 改写局部 overlay | 前端测试 |
| 3 | `Backfill.svelte` 改用新数据流，筛选器改 `todo/all/ok/unmatched/failed`，补 i18n `todo` 与空态文案；三处 delete 补 `notifyLibraryChanged()` | `npm test -w frontend` + `check:frontend` |
| 4 | FDD:405-425 描述、FDD:933 收尾项、`architecture.md` 不变量台账更新 | 全量校验 |

每步跑完验证后单独提交，标题沿用 `type: 简述` + `-` 分段正文。

## 七、已知风险与缓解

| 风险 | 缓解 |
|---|---|
| 三处 delete 补通知后，`Detail.svelte:108` 的 `onLibrary → refreshMeta()` 会刷新已删除条目 | 核实 `refreshMeta` 自带 `try/catch` 与 `if (!g) return`，风险不成立；通知放在原位置 |
| 备份还原整体换库，overlay 里大批 id 失效 | 重取 `games` 时同步清掉 `localOverlay` 里不在新 games 的 id；后端幽灵 overlay 不会显示（合并视图只遍历 games） |
| 修正非目标集条目时后端 `set_item` 找不到会静默丢弃 | 全部走前端 `localOverlay`，不调后端 |
| 任务完成（TaskDock）也发 `LIBRARY_CHANGED`，可能让列表闪动 | 排序确定 + keyed each 用 `game_id`，仅字段更新，不重建其它节点 |
| 删除通知会让已挂载视图重复 `load()` 一次 | 幂等读库，可接受；不删原有显式 `load()` |
| 筛选器去掉 `pending/active` 后旧文案残留 | 同步更新 `i18n-zh.js` 与视图引用；`STATE_LABEL` 保留供徽章使用 |

## 八、进度台账

| 步 | 状态 | 提交 | 备注 |
|---|---|---|---|
| 0 | 完成 | `68c62eb` | 台账落盘 |
| 1 | 完成 | `339297c` | `backfill-list.js` + 10 条表驱动测试；前端 320→330 例 |
| 2+3 | 完成 | `7ac2d5f` | 原步 2/3 强耦合，合并为一个可运行提交：store 数据所有权、视图筛选、三处删除通知、i18n |
| 4 | 完成 | 本批 | FDD 407/413/419-421/933、`architecture.md` 不变量台账与影响面矩阵、本台账收尾 |

**执行调整**：步 2 与步 3 强耦合（store 导出变更后视图必须同时切换，单独提交会让 `vite build` 因引用旧导出失败），合并为一次可运行提交。

**风险核实**：`Detail.svelte:93` 的 `refreshMeta` 自带 `try/catch` 与 `if (!g) return`，三处 delete 补通知不会让已删除条目的刷新报错，通知放在原 `notifyLibraryTagsChanged()` 旁即可，无需调整派发顺序。

## 九、验收清单（交给用户验证）

1. 进入补全页，默认「本次」显示计划清单；切「全部」能看到库内所有条目
2. 开始补全后，进度推进时列表不重排、选中项不跳；成功的条目仍留在默认视图
3. 选中「全部」里任意一条，右栏能打开修正台并执行改词重搜 / 逐项覆盖
4. 在 MyList / Dashboard / Detail 删除一条游戏，补全页列表自动移出该条；若正选中则自动清空
5. 同步应用新建条目后，按 created ids 启动补全，默认视图切到这批
6. 备份还原后不出现幽灵徽章

前端表现（布局、文案、徽章样式）由用户验证后再细修正，本轮只保证行为正确。

## 十、收尾状态

全部完成。提交序列：`68c62eb`（步 0）→ `339297c`（步 1）→ `7ac2d5f`（步 2+3）→ 本批（步 4）。

最终基线：后端 **338 passed / 5 skipped**（本轮后端零改动）、前端 **32 文件 330 例**，`check:frontend` / `check:rust` / `check:generated` / `git diff --check` 全过。

明确未做：字段完整性追踪、provenance、队列持久化、localStorage、通用 merge 抽象、后端改动。列表成员资格只由冻结的目标集决定，唯一允许的移出是库内条目被删除（由 `LIBRARY_CHANGED` 驱动）。

接手指引：先看本文件第六、七节与 `backfill-list.js` 的纯函数契约，再跑 `git log -4 --oneline`；细修正前端表现时不要重新引入「从字段反推成员资格」的判据。

## 十一、追加变更：默认排序改名称并锚定执行顺序

用户验证阶段反馈「补全不是从列表第一条开始」。核实为 `list_games` 按 `created_at DESC, id` 排，而 `SCAN_SQL` / `items_by_ids` 没有 ORDER BY（实际按插入顺序），两者错位；且 `now()` 是秒级，同批新建条目 `created_at` 相同，列表按 UUID 排，视觉上像随机。用户推测「根据名称」，并指出若任务自上而下推进，成功条目自然落在上方，无需置顶。

- 默认排序改为按名称：`games.rs::GAME_ORDER_SQL = "ORDER BY g.name COLLATE NOCASE, g.id"`。ASCII 大小写不敏感，`id` 兜底保证确定性；中文/日文按 Unicode 码点（UTF-8 字节）排，不是拼音，未引入 ICU
- 该常量同时被 `list_games`、`backfill::scan`、`backfill::items_by_ids` 使用，任务从列表第一条往下推进；成功的条目原位变绿并集中在上方，列表全程不重排
- **全局影响**：`listGames()` 还被 MyList / Dashboard / SearchBar / GameForm / BackfillPanel 使用，这些视图的顺序也改为按名称
- `tests/backfill.rs` 新增 1 条用例：断言库内列表按名称排序，且 `scan` / `items_by_ids` 的 id 序列与 `list_games` 完全一致
- 验证：后端 339 passed / 5 skipped，前端 32 文件 330 例，`check:rust` / `check:frontend` 全过

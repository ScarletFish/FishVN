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

## 十二、追加变更：复核「猜测绑定」与右栏按来源分支

### 背景判定

用户指出批量补全实际有两条路径，后者的错配风险是**客观存在的复杂度，不是过度设计**：

| | 判据 | `resolve_vndb_id` 行为 | `bind_source` |
|---|---|---|---|
| 同步来的 id / 已有绑定 | `vndb_id.is_some()` | 直拉，不搜索 | `existing` |
| 本地目录 / 手输名 | 无 id | `vndb::search(name)` 取第一条自动绑定，并把 id 写回 `games` 表；记 `hit_count` / `hit_title` / `candidates` | `search` |

因此上一轮「删掉右栏 409 行、统一跳表单」的结论**只对第一条路径成立**：路径 2 猜错时，右栏是唯一的就地改绑入口（`fill_single` 要求已有 id、不重新搜索；补全页原本没有打开表单的入口），且它直接复用了本次 run 已经搜好的前 3 个候选与本地名，比去表单重新输词少两步。页面职责收敛为「无人值守填充 + 对猜测绑定的集中复核」，后者按 `bind_source` 条件出现，不是全体条目的默认负担。

### 实现清单

| 步 | 内容 | 验证 |
|---|---|---|
| 1 | `backfill-list.js` 加 `review` 筛选（`state === 'ok' && bind_source === 'search'`）与 `review` 计数；该判据**不受 `onlyNoId` 影响**（自动绑定后 id 已写回库，`onlyNoId` 会把最该复核的条目排掉）；表驱动测试守住「人工改绑后自动退出、列表成员不变」 | `npm test -w frontend` |
| 2 | `i18n-zh.js` 补 `filter.review` / `reviewHint` / `panelGuessed` / `panelDirect` / `openInForm`；`Backfill.svelte` 筛选器加「需复核」，review 时禁用「仅看无 ID」开关，列表行在 `hit_count > 1` 时显示命中数 | `check:frontend` |
| 3 | `BackfillPanel.svelte` 加 `guessed = bind_source === 'search'`：`search` 行保留搜索框/候选/逐字段覆盖，`existing` 与库内条目只给结果摘要；两者都加「在编辑表单中打开」，`Backfill.svelte` 局部挂载一份 `GameForm`（与 Dashboard/Detail/MyList 同构）；`retry()` 回传补回 `bind_source`，避免把待复核条目误标成已确认 | `check:frontend` |
| 4 | FDD:413 备注补复核入口、`architecture.md` 不变量台账加一行、本台账收尾 | 全量校验 |

### 边界

- **不加** schema、provenance、持久化、「标记已复核」写操作；自清洁靠 `accept()` 现成的 `bind_source: 'existing'` patch，任何非 `search` 值都表示这条不再是机器猜测
- **不重排**：需复核是筛选维度，不是置顶；列表稳定与 `GAME_ORDER_SQL` 锚定不变
- 后端零改动（`BindSource::Search` / `Existing`、`candidates`、`hit_count` 都已存在）
- 第 3 项（抽共享编排，消除 GameForm 与 BackfillPanel 的重复实现）**本轮不做**，等用户验证行为后再排

### 进度台账

| 步 | 状态 | 提交 | 备注 |
|---|---|---|---|
| 1 | 完成 | 本批 | `review` 筛选与计数 + 3 条表驱动用例；前端 330→333 例 |
| 2 | 完成 | 本批 | i18n 6 个 key、筛选器加「需复核」、review 时禁用「仅看无 ID」、列表行在 `hit_count > 1` 时显示命中数 |
| 3 | 完成 | 本批 | 右栏按 `bind_source` 分支（`search` / `existing` / 库内三态各有说明文案）、局部挂载 `GameForm`、`retry` 保住来源标记 |
| 4 | 完成 | 本批 | FDD:413 备注、`architecture.md` 不变量台账、本节收尾 |

**执行调整**：步 1~3 都落在同一批前端改动上，且 `BackfillPanel` 新增的 `onopenform` 与视图挂载必须同时存在才能跑通，因此合并为一次可运行提交（含台账落盘），不做三次拆分。

**实现细节**：
- `review` 判据放在 `onlyNoId` 之前并直接返回，因为自动绑定把 `vndb_id` 写回 `games` 表后，原本的「仅看无 ID」会把最该复核的条目排除掉；视图在 review 时禁用该开关，避免亮着却不生效
- `BackfillPanel.retry()` 回传补 `bind_source: item.bind_source ?? state.bind_source`：后端 `fill_single` 对不在目标集里的条目返回 `unwrap_or_default()`，来源为空，若不补会把待复核条目误标成已确认
- 「在编辑表单中打开」用补全页局部挂载一份 `GameForm`（与 Dashboard / Detail / MyList 同构），没有引入跨视图 store；表单保存会发 `LIBRARY_CHANGED`，补全页按既有机制重取全量

**文案修订（用户反馈）**：右栏两句只陈述「这条是怎么绑上的」，不预告出错、也不替用户担保：`search` 行「按游戏名搜索匹配」，`existing` 行「按绑定的 ID 匹配」。此前写过的「按已绑定的 ID 直接拉取，不会自动搜索」（描述实现机制）与「已绑定 VNDB 条目，作品身份确定，无需核对」（替用户担保，且与「不断言 id 一定正确」相冲突）都已撤回；改绑能力由右栏上方的候选列表本身体现，不靠文案预告。列表行标签「沿用」→「已绑定」、「搜索」→「搜索匹配」，同为一组口径统一。

核实后有一处不能写死：`vndb_id` 并非只来自账户同步，`GameForm.svelte:386`（用户手动选中）、`backfill.rs:569`（上一轮自动绑定写回）、`games.rs:161`（通用 update）都会写它。所以文案与 FDD:414 都不断言「来自同步」或「id 一定正确」，只表达「本次任务未对这条做匹配决策」——这与 `bind_source` 的实际语义一致，也仍然支持用户「这类条目无需复核」的心智。

最终基线：后端 **339 passed / 5 skipped**（本批后端零改动）、前端 **32 文件 333 例**，`check:frontend` / `check:rust` / `check:generated` / `git diff --check` 全过。接手时先看本节与 `backfill-list.js` 的 `review` 判据；第 3 项「抽共享编排消除 GameForm 与 BackfillPanel 的重复实现」仍未做，等用户验证行为后再排。

## 十三、抽取共同抽象：`createVnMergeFlow()`

用户确认执行第 3 项。核实后重复的边界是：

| 两边逐行相同 | 两边本就不同（保留） |
|---|---|
| `pulling` / `syncProgress` / `syncNotice` / `syncCancelled` 一组状态 | 搜索 UI（`VndbAside` vs `bf-search` + `bf-cands`） |
| `api.listGames()` 查 `vndb_id` 占用 + `vndbTaken` 提示 | `current` 来源（表单编辑态 vs 读库 `toFields`） |
| 订阅 `vndb-sync` → 进度/错误投影 → `off?.()` 清理 | 冲突确认后的写入（写表单态等保存 vs 立即写库 + 回传 overlay） |
| `fetchVnMetadata` → `conflictRows` → `planCharacters` | `coverFocalX/Y` 只在表单的取舍面板里传 |
| `applyIncoming` / `applyCharacters` 的应用与状态清理 | `retry()` 走 `vndb-backfill` 通道，不产生冲突行 |

**设计**：新增 `frontend/src/lib/vn-merge-flow.svelte.js`，与既有 `createVndbSearch()` 同风格的无头 rune 控制器。

- `run(channel, task, filter)`：单飞 + 订阅 + 进度/错误 + 取消 + `finally` 清理，三处拉取（GameForm `pickHit`、BackfillPanel `pick` 与 `retry`）共用
- `pick(id, load)`：`load()` 由宿主提供 `{ ownId, current, characters }`（可异步），内部完成占用检查、拉取、冲突计算；成功返回 `true`
- `accept(picked, charPicked)`：返回 `{ fields, characters, id, previous }`，**只计算不写入**，由宿主决定写表单还是写库
- `cancel()` / `reset()`；对话框开关仍由宿主各持一个 `$state`（`VnMergeDialog.open` 是 `$bindable`，不塞进控制器）

**一处刻意的行为统一**：原 GameForm 在无冲突时会打开一个空的合并对话框，BackfillPanel 会 toast「没有可应用的新数据」。统一为 toast，属于消除无意义交互，不是回归。

**验证**：`check:frontend`（含 339 例前端测试、注释/CSS/死 CSS/vite build）与 `git diff --check`；后端零改动，不跑 Rust 全量。

### 进度台账

| 步 | 状态 | 提交 | 备注 |
|---|---|---|---|
| 1 | 完成 | 本批 | `vn-merge-flow.svelte.js`（136 行）+ 6 条表驱动测试；前端 333→339 例 |
| 2 | 完成 | 本批 | `GameForm.svelte` 接入，删 74 行加 23 行；`pickHit` / `cancelSync` / `acceptMerge` 变薄 |
| 3 | 完成 | 本批 | `BackfillPanel.svelte` 接入，`pick` / `accept` / `retry` 都走 flow，删掉整套本地拉取状态 |
| 4 | 完成 | 本批 | 全量验证、本节收尾、提交 |

**收尾状态**：
- 三处拉取（GameForm `pickHit`、BackfillPanel `pick` 与 `retry`）共用 `run()` 的单飞 / 订阅 / 进度投影 / 取消 / `finally` 清理；`冲突计算 → 取舍 → 应用`只在 `vn-merge-flow.svelte.js` 一处，以后改 `vn-merge` / `char-merge` 的接入方式不用同步两个组件
- `GameForm` 与 `BackfillPanel` 仍各自保留搜索 UI 与最终写入，这是本就不同的部分，没有强行统一
- `VnMergeDialog.open` 是 `$bindable`，对话框开关仍由宿主各持一个 `$state`，没塞进控制器
- **行数净增**（新增控制器 136 行 + 测试 131 行，两个组件净减约 93 行）：换来的是原本零覆盖的拉取编排有了 6 条表驱动用例，以及规则单一来源
- 一处细微统一：原 GameForm 把所有非 error 事件都当进度，现与 BackfillPanel 一致只认带 `phase` 的事件；`TaskEvent` 都带 phase，无实际影响
- 最终基线：前端 **33 文件 339 例**，`check:frontend` / `git diff --check` 全过；后端零改动

至此第 3 项完成，本台账的全部计划项（0~4、排序锚定、复核筛选、共同抽象）均已落地。

# VNDB 账户同步设计（草案）

> 状态：设计草案，待定稿后并入 `docs/fdd-game-library.md` 的活动集。
> 配套端点细节（`get ulist` / `set ulist`）待补入 `docs/dev/vndb-api.md`。

## 1. 定位与字段集

账户同步把用户在 VNDB 的个人列表搬进本地「用户状态」组，与活动集二的元数据同步走不同通道：元数据同步改写元数据行（含 VNDB 的贝叶斯 `rating`），账户同步改写 `user_game_state` 的 `status` 与 `user_rating`。VNDB 的 `vote`（个人评分）映射到 `user_rating`；VNDB 的 `rating`（元数据）仍不触碰 `user_rating`（与 FDD 活动集二备注一致）。

同步字段首版为两个：`status`、`user_rating`。`notes` 暂不在首版范围，留待后续。身份组、本地安装组、元数据组不在账户同步范围。关联键 `vndb_id` 用于匹配，不参与覆盖语义。

## 2. 三种同步力度

三种力度共享一套三路比对内核，差别只在「采纳 / 保留」动作是否真正写入对方侧。内核依赖一份**基快照**：记录上次成功同步时 VNDB 在那个字段上的值。

对每条记录取三值：基快照 `base`、本地当前 `local`、远程当前 `remote`，逐字段（status 与 user_rating 各自独立）判定：

| base、local、remote 关系 | 判定 | 默认动作 |
|---|---|---|
| 三者相同 | 无变化 | 不动 |
| base == local ≠ remote | 仅 VNDB 改了 | 采纳 remote |
| base ≠ local == remote | 仅本地改了 | 保留 local |
| 三者各不相同 | 两边都改了 | 冲突，进面板 |

基快照的意义：用户上次坚持的本地值，只要与基快照不同，就不会被反复判为冲突；只有用户与 VNDB 都动了同一字段，才上升为冲突。

### 仅下载（首版默认力度）

数据从 VNDB 流向本地，本地从不写回 VNDB。

- VNDB 列表项在本地已存在 → 三路比对，按上表采纳或保留。
- VNDB 列表项在本地不存在 → 用 `title` 建最小壳：`name` + `vndb_id` + `status`，有 `vote` 时带 `user_rating`。这批新 `vndb_id` 归入「刚同步集」，供批量下载图片使用。
- 仅下载只处理 `status` 与 `user_rating`；`notes` 维持本地原值。
- 保留规则的效果：用户本地手改的状态 / 评分，只要不等于基快照，就不会被 VNDB 的旧值覆盖。

### 仅上传

数据从本地流向 VNDB，用 `set ulist` 写出 `status` 与 `vote`（= `user_rating`）。

- 三路比对同上；上传语境下「采纳 remote」读作「仍把本地推上去覆盖远程」。
- 本地有、VNDB 列表没有的条目 → 用 `set ulist` 在 VNDB 新建（带 status 与 vote），会在 vndb.org 真实列表新增条目，首次写入前需用户确认。
- 本地字段只由用户编辑与上传流程产生；VNDB 当前值仅参与基快照比对。

### 双向

两个方向同时生效：下载侧沿用仅下载的保留规则，上传侧沿用仅上传的保留规则。

- 冲突的「保留哪边」默认准则：以 `lastmod` 较新的一侧为准。VNDB 的 `ulist.lastmod` 给出每条修改时间；本地用状态 / 评分的最后修改时间比对，较新一侧的值胜出。
- 时间无法判定时（本地缺修改时间或 VNDB 缺 lastmod）默认保留本地并提示。
- 双向会同时增删两端列表项：本地新条目上传、VNDB 新条目下载建壳。

## 3. 状态与评分映射

### 状态

VNDB 用 labels 表达状态：`playing` / `finished` / `stalled` / `dropped` / `wishlist` / `planning`。本地 `status` 枚举在现有四个之外增加 `stalled`（搁置），其余与 VNDB 对齐：

| VNDB label | 本地 status |
|---|---|
| playing | 游玩中 |
| finished | 已完成 |
| dropped | 已弃坑 |
| stalled | 搁置（新增） |
| wishlist / planning | 计划中 |

VNDB 返回未知 label 时按 `计划中` 处理并提示。`status` 非空两端一致，无缺省歧义。

### 评分

`user_rating` 与 VNDB `vote` 同为 0–100 百分制（`vote` 量程 10–100，对应 1.0–10.0），直接 1:1 写入，不换算。`user_rating = 0`（未评分）对应 VNDB 无 `vote`；VNDB 有 `vote` 而本地为 0 时按「remote 有值」采纳。

## 4. 数据模型

- 用户状态表增加两个可空列，记录基快照：`synced_status`、`synced_rating`。
- 用户状态表增加一个修改时间列，记录 status 与 user_rating 的最后本地修改，供双向 `lastmod` 比对。
- `Status` 枚举增加 `stalled` 变体；字符串映射同步更新，`status` 列为 TEXT，旧数据不受影响，不写旧库迁移。
- 开发期直接改建表语句（项目约定）。

## 5. 端点、鉴权与限流

- 读取用 `get ulist`（Kana v2 已合并 vnlist / votelist / wishlist），一次返回 `vn`(id) + 状态 labels + `vote` + `notes` + `started`/`finished` + `lastmod`。首版只取 `vn, labels, vote, lastmod`。
- 写入用 `set ulist { status→labels, vote }`；无对应条目时即新建。
- 翻页用 `results`（上限 100）+ `page`，以响应 `more` 决定是否续页。限流按 `vndb-api.md` 保守口径（100 请求 / 5 分钟客户端节流）执行，500 部约 5 页，余量充足；429 按退避重排。
- 鉴权：请求头 `Authorization: token <token>`。
- token 存储：存本地 `config.json`，但以「固定掩码 XOR + hex」混淆落盘、**不存明文**。本地单机 + 只读凭证场景下，混淆只防配置文件被误提交进 git、被 dotfiles / 云盘同步、被截图或随手打开；防不住本机内存抓取型恶意软件。因此不引入加密依赖，零新增 crate。
- UI 入口在设置页，提供「测试连接」（发一次 `get ulist results=1`，401 即无效）与「断开」（清空）。前端只持有本次会话用户输入的原始 token，不碰落盘密文；打开设置时后端只回 `has_token` 布尔。UI 标注 token 等同只读凭证、只发往 `api.vndb.org`。

## 6. UI 落点

- 设置页「VNDB 账户」区：粘贴 token、测试连接、断开、选择同步力度（默认仅下载）、手动「同步」按钮。
- 冲突面板沿用活动集二的「VNDB 建议值、用户逐字段取舍」范式：并排显示本地与 VNDB 的 status、user_rating，默认预选（仅下载 / 上传语境预选保留本地，双向语境预选较新一侧）；支持「全部保留本地 / 全部采纳对方」；无冲突项即时落库，冲突项暂存非阻塞。
- 解决冲突后回写基快照，该字段下次不再被重复判为冲突。

## 7. 风险与边界

- 仅上传与双向会向 vndb.org 真实列表写入 status 与 vote；力度切换与首次写入前需用户确认。
- 鉴权失败（401）不抹本地数据，明确报错；网络中断时「已拉到的先落库 + 报告剩余」。
- 图片不属 ODbL，缓存随应用数据走，清应用数据即删（见 `vndb-api.md` 使用条款）。

## 附：批量下载图片（配套，与账户同步解耦）

- 独立触发入口（设置页或列表工具条），作用域二选一：全库（换机 / 清 `covers/`）或刚同步的这批。
- 复用现有图片管线（按 `vndb_id` 取封面 + 角色立绘），内容寻址 `cv…` / `ch…` 天然幂等：默认跳过已存在，提供「强制重下」勾选（清缓存场景）。
- 进度 UI + 单张失败跳过（活动集二已规定）；合规提示同上。

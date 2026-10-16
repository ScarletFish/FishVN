# 后端开发规范 — Backend

### 架构一览

- 后端是 Rust crate `src-tauri`，与前端同在 Tauri 进程里
- 前端通过 `invoke` 调 `#[tauri::command]`，命令层只转发，业务逻辑在 `games.rs` / `metadata.rs` / `characters.rs` / `installations.rs` 这些领域模块
- 数据类型只定义一次：`types.rs` 的 serde struct 同时喂给 rusqlite 的行映射与 IPC 序列化
- 改完必跑 `npm run check:rust` + `npm test`

```
src-tauri/
├── main.rs          # 窗口 + 状态 + invoke_handler
├── lib.rs           # data_dir 解析与 id、时间戳生成
├── commands.rs      # #[tauri::command] 转发层，不写业务
├── db.rs            # 连接与 ensure_schema()
├── error.rs         # AppError，过 IPC 时序列化成一句中文
├── types.rs         # 表行与 PATCH 参数
├── games.rs         # 作品与用户状态
├── metadata.rs      # 作品资料
├── characters.rs    # 角色与声优（集合，整包替换）
├── installations.rs # 本地目录
├── tasks.rs        # 长任务进度事件与取消
├── thumbs.rs        # 封面派生图与缓存
├── vndb.rs          # VNDB 上游客户端
├── devhttp.rs       # 开发期 HTTP 通道（commands 的镜像，仅 debug 构建）
└── fsinfo.rs        # 目录遍历
```

改哪块先读哪份：

| 改这些 | 先读 |
|---|---|
| `vndb.rs` | [vndb-api.md](./vndb-api.md)：上游字段形状、过滤与限流、图片编号 |
| `db.rs` / `types.rs` | 本文「数据模型」 |
| `thumbs.rs` / 前端 `cover-src.js` | 本文「封面派生图」与「本地封面授权」 |
| `tasks.rs` / 长任务 | 本文「任务事件」 |
| 前端组件与样式 | [frontend.md](./frontend.md) |

### 调用链

```
前端 api.js  →  invoke('list_games')  →  commands::list_games  →  games::list_games(&Connection)
```

每个 `#[tauri::command]` 都要端到端接通：领域函数 → `commands.rs` 转发 → `main.rs` 的
`generate_handler!` 注册 → 前端 `api.js` 方法 → devhttp 路由与 `COMMANDS` 登记。

devhttp 的 parity 测试会比对 `COMMANDS` 与 `commands.rs` 声明的命令，漏接路由会在 `cargo test` 期失败。以测试通过为准。

### 数据目录

| 构建 | 位置 | 理由 |
|---|---|---|
| `cargo run` / `tauri dev` | 仓库根 `data/` | 与开发时看到的数据一致 |
| 打包后 | 系统应用数据目录（`app_data_dir()`） | 仓库根不在用户机器上 |

开发期与打包期**两边不共用**，开发数据不会混进安装版。解析逻辑定义在 lib（bin 与 commands 都要用），由 `cfg(debug_assertions)` 选边。改这段先想清楚是哪一边在动。

### 命令层规则

- `Connection` 是 `Send` 不是 `Sync`，用 `Mutex<Connection>` 交给 `app.manage`
- 命令里必须走 `lock()` 取连接，锁毒化要转成错误上报，不能 `unwrap`
- 跨网络 I/O（同步、下封面）不持 `Connection` 锁：那是全局单锁，持着会卡住所有库操作
- reqwest 是 Tauri 的传递依赖，不带 TLS 后端：`native-tls` 要显式开在 `Cargo.toml`，去掉编译照过、运行期所有请求失败
- 一次同步取元数据与角色是两次请求，额度按请求数各扣一次
- `State` 是注入的，前端不传；其余参数由 serde 从 JSON 反序列化
- 参数类型本身就是校验：非法枚举值、错的字段类型在反序列化就进不来，不需要手写 typeof 检查

### 进程与状态

- 注册时传的是 `Arc<T>`，命令侧就必须是 `State<'_, Arc<T>>`；泛型按类型精确匹配，写错编译期不报，只在调用该命令时报 `state not managed`

### 任务事件

任何单条命令响应容纳不下的耗时操作（网络批量拉取、文件下载、后续库扫描、启动计时等）都走任务事件通道，不要阻塞命令等结果。任务对外只有三条结果，各走各的通道：

- **进度**走事件通道：按阶段上报，可计数阶段给绝对 `done/total`（定长条），不可计数阶段只给阶段名（不定长动画条）。
- **best-effort 的局部失败**走事件通道的非致命通知：任务整体仍可继续的步骤（如个别文件下载失败）经 `Progress::report_error` 上报，不 abort 整次任务；缺资源（数据源本就没有该文件）不算失败、不报。
- **致命失败与最终结果**走命令返回路径：整条数据查询的失败照旧上抛，前端按普通命令错误弹提示；成功则命令正常返回。

新增任务时遵守：

- 任务名是跨前后端字面量，前后各定义一份，命名约定同命令名。
- 用 `Progress` 上报：每个用户该看得见推进的可计阶段都要有发射点；不要为单文件、单响应伪造 0/1 计数器，要么合进父阶段、要么给不定长条。
- 取消：worker 在单元间隙查 `cancelled()`，命中即中断并以 `Cancelled` 收尾，下载类一并 abort 在途任务；前端置位标志后调 `cancel_task`，`await` 返回复查标志——取消与成功擦身而过时按取消处理，结果丢弃。
- 持 `Progress` 跨 `await` 时，trait 须标 `Sync`：引用它的 future 要满足 `Send`，否则检查失败只在异步 handler 边界报出。

通道约定：事件有两层传输（应用内 IPC 与开发期镜像 SSE），前端以统一订阅接口屏蔽差异；**新增任务不应要求前端改动传输层**，只在任务侧调 `report` / `report_error`。事件发送时入有界缓冲，订阅后才来的消费者先补读缓冲再收实时流；事件携绝对值，消费者以最新值覆盖即可续上，但补读区间有限，靠缓冲续命而非永久历史。

### 开发期 HTTP 通道

只在 debug 构建里存在，是 `commands.rs` 的镜像：加命令时这里要同步改。只绑 `127.0.0.1`，
且要设 `FISHVN_DEV_HTTP` 才开 —— 本机任何进程都能读写用户的库。端口与 vite 的 4545 分开，
撞上会自己请求自己。

请求体字段用命令原样的 snake_case：`invoke` 那条路会把 `gameId` 这类驼峰参数转成
snake_case，裸 HTTP 没有这层转换。

### 错误与 IPC

- 过 IPC 只给前端一句中文说明，错误种类留 Rust 侧自己判断
- 撞唯一约束（`SQLITE_CONSTRAINT_UNIQUE` 扩展码）转 `Conflict`，其余约束错误原样上抛
- `invoke` 会用原值 reject，`api.js` 包成 `Error`，调用方统一读 `.message`

### PATCH 的三态

可空字段用 `Option<Option<T>>`：外层 `None` 是「没给这个字段」，内层 `Some(None)` 是
「显式置空」。单层 `Option` 分不清这两者，PATCH 会写坏数据。

`status` 例外 —— 它是 `NOT NULL`，没有清空语义，所以是单层 `Option`，给 `null`
等于「没给」。

### 数据模型

作品身份、本地安装、VNDB 元数据、用户状态**各是一张表**；角色、声优与配音关系另成一组，见下文。
**一张表一个自动写者**；用户显式编辑优先，自动流程只在用户逐字段确认后写入。

| 表 | 内容 | 写者 |
|---|---|---|
| `games` | 作品身份 | 用户创建与编辑 |
| `user_game_state` | 状态、用户评分、笔记、默认安装、游玩时间与次数 | 用户；游玩时间与次数暂无写入方 |
| `installations` | 本地目录、入口 exe、候选入口、目录大小、顺序 | 用户在前端表单里整包写回 |
| `game_metadata` | 别名与本地化标题、封面、发行日期、评分与人数、简介、标签、开发商 | 用户直接编辑；同步时逐字段比对后由用户选覆盖或跳过 |

几条从表结构看不出来的语义与配套惯例：

- `vndb_id` 可空且 `UNIQUE`：NULL 之间不算重复，一部 VN 只对应一条作品；重复绑定报冲突
- `installations.game_id` 为 NULL 表示待认领。删作品走 `SET NULL` 而不是 CASCADE —— 目录还在盘上，下次扫描还会遇到它
- 状态取值定义在 `types.rs` 的 `Status` 枚举（wishlist / playing / finished / dropped），取值的合法集合只有这一份
- `dir_path` 存 `resolve_path` 绝对化后的原样路径，不做大小写归一 —— `COLLATE NOCASE` 会在 macOS 与 Linux 上把不同目录判成同一个
- `game_metadata` 除 `game_id` 外每列都可空：一部不绑 VNDB 的作品同样要有资料行，没有数据就是空而不是占位值
- 资料行可有可无，且行不存在与行存在但字段为空是两种状态（「没填过」与「填了空的」），调用方要分得清
- 作品正式名在 `games.name`，`game_metadata` 不重复存。VNDB 那份 `titles` 里已带 `official` / `main` 标记的正式标题，别名另存 `alias`
- 状态行随作品删除而清除（`CASCADE`）；安装行相反走 `SET NULL`
- `game_metadata.image_id` 是 VNDB 的 cv 编号，本项目**不写它**：下载判重靠 `covers/vndb/<cv 编号>.jpg` 是否存在，同步冲突靠 `image_path` 是否为空。`image_path` 是本地文件的位置，两者不是一回事
- JSON 文本列（`titles` / `tags` / `developers` / `aliases` / `traits` / `gender` / `exes`）：库里是文本，对外一律给数组或结构；要按其中某个键筛，只能整列取出后在 Rust 侧解
- `characters.id` 平时是 VNDB 的 c 编号，用户手建的行是 `local-<uuid>`，两种混排
- `characters.traits` 是 trait 全量数组，本地录入不给入口，只有 VNDB 同步会写
- `staffs` 是唯一跨作品共享的实体：同一个人换一部作品还是同一行。本地手填用 `local-<uuid>` 占位，无人引用时由触发器回收；两条删除来源（删作品、整包替换）共用这一条回收路径，触发器按 `staff_id` 反查，所以有 `character_va_staff` 索引
- `character_va.note` 空值一律存 `''`：SQLite 主键允许 NULL，同一条关系会被 NULL 与 `''` 写成两行
- 写角色是整包替换，删与插在同一个事务里，中途失败退回调用前的整套角色；建作品与它的初始状态行同理，同一事务
- 列表投影把 `game_metadata` 一并放进 `GROUP BY`：选了它的列却不在分组键里，SQLite 会取任意一行的值（它是主键，一行一作品，不改变行数）
- `SQLite` 的 `changes` 记的是命中行数而非真正变了的行数，`0` 就是没有这一行
- 安装的身份是「目录 + 入口」这对：同一目录下可以有多条不同入口的记录，同一对只留一条。唯一索引落在 `dir_path` 与 `IFNULL(entry_path, '')` 上 —— 唯一约束不比较 NULL，空入口直接比 `entry_path` 会让约束失效
- 建安装先按 `dir_path` 查占位行：拖同一个目录两次不必付出读整个目录的代价；唯一索引仍是最终保证
- 安装集合由表单整包写回：草稿带 `id` 的更新、不带 `id` 的新建、库里本作品有而草稿里没有的删除，全在一个事务里；`sort_order` 按数组下标派生，读取按 `sort_order, dir_path, entry_path` 排序
- 整包写回按「目录 + 入口」命中未认领行（`game_id` 为空）时认领它，命中别的作品的行报冲突；同批出现同一对也报冲突
- 一个目录只归一部作品：草稿写进已被别的作品占用的目录就报冲突；某目录被本作品接管后，该目录里没被认领到的未认领行随这次保存清掉，没被碰到的目录里的未认领行照旧留着
- `patch` 改入口时先看同目录有没有别的行占着这一对，占了就报冲突而不是漏成唯一索引错误
- 目录元信息分两档取：只要候选 exe 时下探三层（限深也截断了读取），连大小一起取时遍历整棵子树。整包写回走前者，`size_bytes` 留空；新建与换目录时重算候选，换目录顺带把失效的 `size_bytes` 置空；目录没变就不读盘，搬走的目录不挡保存
- `entry_path` 存相对 `dir_path` 的路径：空串、绝对路径与含 `..` 的路径都判非法输入
- 主键是带连字符的小写 uuid v4 形态

元数据每列都可由用户手改。同步不按「哪些列空了」判断该不该写，而是逐字段比对库里现值与
新值——不同的就是冲突，交给调用方逐字段选覆盖或跳过。用「只填空字段」的话，VNDB 换了封面
这类非空变更永远同步不过来，同步按钮第二次点就成空转。名称的取舍也在同一决策集里：用户选择采用官方标题时，同步一并更新 `games` 的 `name`。

表结构只写在 `ensure_schema()`，语句自身幂等（`CREATE TABLE` / `CREATE INDEX` 带 `IF NOT EXISTS`），
每次开库跑一遍即可。应用处在开发期，本地库随代码重建：改结构直接改建表语句，旧库删掉重建；
要保住本地的作品数据，先手工导出搬一次，这类操作留在本机，不进代码。

建表那句 `PRAGMA foreign_keys = ON` 看着多余（内建 SQLite 带 `SQLITE_DEFAULT_FOREIGN_KEYS=1`），
但不能删：建表全靠 `ON DELETE CASCADE`，去掉它等于把级联交给传递依赖的编译选项。

### 外键列不能收空串

`game_id` / `default_installation_id` 给 `""` 会被 SQLite 当成真实 id 去查，撞外键后
报成未处理的数据库错误。写入前先判空串，当成非法输入拒掉。

### 收图格式两份清单要手动对齐

文件选择器的白名单在 `frontend/src/lib/pick-path.js` 的 `IMAGE_EXT`，后端能解哪些格式由
`src-tauri/Cargo.toml` 里 image crate 的 feature 决定，两边必须一致：前端放行而后端解不开，
用户会在选完文件之后才撞上「封面格式不支持」。avif 要 libdav1d（C 库），gif 没开 feature、解不开，
两份清单都不收。`image` crate 关掉 default-features、只开这四种 —— 默认档会把 avif（libdav1d）
与 exr/rayon/rav1e 那套拖进来。改一边就同步改另一边——没有构建检查兜底。

封面从 `t.vndb.org` 拉取，落盘到 `covers/vndb/<cv 编号>.jpg`；该文件已存在即跳过下载。
网络读体中断时按 3 次退避重试（300ms 递增），仍失败才放弃该封面；HTTP 非 2xx 直接放弃、不重试。

### 本地封面授权

渲染本地封面走 asset 协议，scope 之外的路径不放行就拿到 403 空响应。dialog 插件只放行
`fs_scope`，不碰 asset 协议，所以渲染之前必须补一次授权。

授权只活在本次进程，重启后靠 `tauri.conf.json` 的 `assetProtocol.scope` 兜底 —— 但它只覆盖
`$HOME/**/*`：打包后数据目录在 `app_data_dir()`、落在覆盖内；开发期数据目录在仓库 `data/`，
scope 够不着，必须走 `allow_local_file`。

### 封面派生图

长边 560 = `--card-w`（277px）的 2 倍，覆盖 200% 缩放屏；再大只是白 decode。

缓存内容寻址：路径、size、mtime 变了就是新 key，旧缓存自然失效，不必显式删。mtime 哈希到
纳秒；size 一并参与哈希，兜底 mtime 精度粗的文件系统。

### 注释

代码里不写注释，知识写进本文与 [vndb-api.md](./vndb-api.md)。规则与例外见 [comments.md](./comments.md)。

## 新增命令检查清单

- [ ] 领域函数在 `games.rs` / `installations.rs`，`commands.rs` 只转发
- [ ] 挂进 `main.rs` 的 `generate_handler!`，前端 `api.js` 有对应方法
- [ ] 新列挂对了表：路径进 `installations`，元数据进 `game_metadata`，状态进 `user_game_state`
- [ ] 新增列有写入方
- [ ] 可空字段的 PATCH 用双层 `Option`，语义在 types.rs 上看得见
- [ ] 外键列判了空串
- [ ] 长任务经 `Progress` 播进度、经 `Tasks` 注册表收取消；best-effort 失败走 `report_error` 而非吞掉
- [ ] 新增测试指向临时库，文件系统用例有真目录树
- [ ] 注释过了 comments.md 的判据
- [ ] 跑过 `npm run check:rust` + `npm test`
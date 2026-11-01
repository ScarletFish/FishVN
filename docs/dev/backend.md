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
├── sessions.rs      # 启动与游玩记录（作业 + 计时回路）
├── dashboard.rs     # 首页统计与库内标签
├── inspect.rs       # 目录检查与候选入口扫描
├── settings.rs      # 设置持久化（config.json）
├── tagdict.rs       # 标签词典（tag/trait 中文译名）
├── sync.rs          # VNDB 账户同步（计划-应用两段式）
├── backfill.rs      # 资料补全（批量拉 VNDB 元数据填空白列）
├── backup.rs        # 整库备份、列表、还原与删除（data/backups/，VACUUM INTO 备库）
├── tasks.rs         # 长任务进度事件与取消
├── thumbs.rs        # 封面派生图与缓存
├── vndb/            # VNDB 上游客户端（入口 + client/dto/characters/images）
├── devhttp.rs       # 开发期 HTTP 通道（commands 的镜像，仅 debug 构建）
└── fsinfo.rs        # 目录遍历
```

改哪块先读哪份：

| 改这些 | 先读 |
|---|---|
| `vndb/` | [vndb-api.md](./vndb-api.md)：上游字段形状、过滤与限流、图片编号 |
| `db.rs` / `types.rs` | 本文「数据模型」 |
| `thumbs.rs` / 前端 `cover-src.js` | 本文「封面派生图」与「本地封面授权」 |
| `sessions.rs` / 启动与计时 | 本文「启动与游玩记录」 |
| `sync.rs` / 账户同步 | 本文「外部数据同步：计划-应用两段式」与 [vndb-sync.md](./vndb-sync.md) |
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

### 设置持久化

设置存数据目录的 `config.json`，与库同目录、随数据备份一起走。读取容错：文件缺失或损坏回落默认值，启动不被阻塞。启动时加载进进程内缓存供读（游玩结算在独立线程上，从缓存取口径、不碰磁盘），写操作落盘并刷新缓存。新设置项加入 `AppConfig`，字段标 `serde(default)` 让旧配置文件缺字段仍可读；前端经 `get_config` / `update_config` 读写，选择即保存。

`update_config` 收 `ConfigPatch`（各字段 `Option`）并在 Rust 侧与现值合并后整体落盘：设置项会持续增多，前端每次只发自己改的那项，互不覆盖。

面向已存凭证的命令（连接测试这类），凭证参数可缺省：缺省时回退设置里已存的值。用户验证「当前保存的这份」时不必重新粘贴一遍。

主题（`theme`）在 `AppConfig` 里，取值浅色 / 深色 / 跟随系统。setup 阶段读取后先解析成深浅（`system` 取窗口当前系统主题），再对主窗口调 `set_background_color`：深 `#121212`、浅 `#f1f0ec`，与前端 tokens 的 `--bg-page` 严格一致。setup 跑在 WebView 首帧之前，这是 WebView 外壳防露白的唯一一次机会；`tauri.conf.json` 里的静态 `backgroundColor` 是 config.json 读不出来时的兜底。系统主题在运行中变化时窗口底色不重涂，只有内容区边缘露底的瞬间可能差一档，webview 内的界面本身由前端 `matchMedia` 跟着换。

### 命令层规则

- `Connection` 是 `Send` 不是 `Sync`，用 `Mutex<Connection>` 交给 `app.manage`
- 命令里必须走 `lock()` 取连接，锁毒化要转成错误上报，不能 `unwrap`
- 跨网络 I/O（同步、下封面）不持 `Connection` 锁：那是全局单锁，持着会卡住所有库操作
- 网络类命令入口先过限流器：`Arc<RateLimiter>` 注册进 state，命令侧 `State<'_, Arc<RateLimiter>>` 取出、发请求前 `acquire`，速率口径见 [vndb-api.md](./vndb-api.md)
- reqwest 是 Tauri 的传递依赖，不带 TLS 后端：`native-tls` 要显式开在 `Cargo.toml`，去掉编译照过、运行期所有请求失败
- 一次同步取元数据与角色是两次请求，额度按请求数各扣一次
- `State` 是注入的，前端不传；其余参数由 serde 从 JSON 反序列化
- 参数类型本身就是校验：非法枚举值、错的字段类型在反序列化就进不来，不需要手写 typeof 检查

### 进程与状态

- 注册时传的是 `Arc<T>`，命令侧就必须是 `State<'_, Arc<T>>`；泛型按类型精确匹配，写错编译期不报，只在调用该命令时报 `state not managed`

### 启动与游玩记录

启动走 `sessions.rs`：解析入口 → 按安装项的启动方式拼 `Command` → 起进程 → 把进程放进 Job Object → 落一条进行中的 `play_sessions` → 后台线程按 1 秒节拍守着，进程退出后结算。

启动方式由安装项的三项决定（`le_launch` 开关 / `magpie` 模式 / `launch_args`，均是该安装项的属性，见 FDD 活动集三）：

- **直接启动**（默认）：附加参数按 MSVCRT 引号规则拆成数组透传（引号内为一个参数，引号内 `""` 是字面引号）；`bat` / `cmd` 走 `cmd /C`，`lnk` 走 `cmd /C start`（`CreateProcess` 不认识 `.lnk`），其余直接起；工作目录显式设为安装项的 `dir_path`。
- **转区启动**（`le_launch`）：以 Locale Emulator 主程序（设置项 `le_path`，未登记时启动被 `BadRequest` 拦住、不起进程）为父进程，入口路径作参数、`dir_path` 作工作目录拉起；只收 exe 入口（非 exe 直接报错），不透传附加参数，不提权、不暴露 LE 自身的区域参数。LEProc 作父进程后 Job 覆盖不缩水。
- **Magpie 缩放**（`magpie`，三态 `MagpieMode`：关 / 全屏 / 窗口，库里 0/1/2）：旁路链路，**不进 Job、不参与计时判定**——先把 Magpie 无参拉起（已在跑则跳过，5 秒内确认）→ 等前台切到游戏进程（60 秒超时）→ 停 2 秒 → 向类名 `Magpie_Hotkey` 的消息窗口 `PostMessageW(WM_HOTKEY)`，`wParam` 为 Magpie `ShortcutAction` 序号（全屏 = 0 `Scale`，窗口化 = 1 `WindowedModeScale`）。消息窗口分发不校验用户有没有绑过热键，所以两个动作都能直接发，不要求在 Magpie 里配热键。触发走消息窗口而非模拟按键：不依赖输入焦点、不要管理员权限、不抢用户键鼠（对标 PotatoVN 的 InputSimulator 方案）。三项可叠加：转区 + Magpie 同时开时 LE 作父进程，Magpie 照常旁路触发。

计时回路对两个工具做了适配：

- **前景时间按节拍累加**（命中一次加 1 秒），不是记两个时间戳相减 —— 后者会把系统休眠的整段算进前景时间。判定 = `GetForegroundWindow` → pid → `IsProcessInJob`。
- **前台白名单保缩放期间计时连续**：前台进程映像名是 `Magpie.exe` 时仍算前台（`QueryFullProcessImageNameW` 取文件名比字），用户在 Magpie 界面操作的那几秒同样计入，宁可多算这几秒也不要断档。
- **结束判定读作业的活跃进程数**（`QueryInformationJobObject` 的 `ActiveProcesses`），不用「作业句柄变有信号」。归零后要连续 3 个节拍仍为零才收尾：启动器型入口会「启动器退出、本体被它拉起」，计数会短暂归零，立刻收尾会把一次游玩砍成几秒。
- **不给 breakaway 许可**（不设 `JOB_OBJECT_LIMIT_BREAKAWAY_OK`），加上子进程默认继承作业成员身份，游戏拉起的子进程逃不出去。作业句柄得自己开：`std::process::Command` 只给 pid，要拿句柄得 `OpenProcess`。
- **落盘时机**：前台状态翻转时立刻写，另有 30 个节拍的心跳兜底。前景时间只在 FishVN 在场时才可能增长，所以断电最多丢最后一个心跳周期、不会虚高；存活时间由起止时间戳相减，FishVN 不在场的那段必然少算。两种偏差都朝「少算」。
- **孤儿收尾只在开库时做**（`sweep`）：把 `ended_at IS NULL` 的记录按 `last_seen_at` 结算。不能挪进 `launch_game` —— 那会把别的作品的进行中记录一起结算掉。
- **状态自动迁移只在越过静默窗后**做一次「计划中 → 游玩中」：spawn 成功不等于游戏真的跑起来了，点一条失效的路径不该改写状态。
- 依赖 `windows` crate（Job Objects / Threading / WindowsAndMessaging）：版本要与 tauri 的传递依赖对齐，错开会编两遍。裸 `HANDLE` 不是 `Send`，跨线程传作业句柄要包 newtype 并整体传（闭包按字段捕获会绕过包装）；`DuplicateHandle` / `DUPLICATE_SAME_ACCESS` 在 `Win32::Foundation` 不在 Threading。

### 任务事件

任何单条命令响应容纳不下的耗时操作（网络批量拉取、文件下载、后续库扫描等）都走任务事件通道，不要阻塞命令等结果。任务对外只有三条结果，各走各的通道：

- **进度**走事件通道：按阶段上报，可计数阶段给绝对 `done/total`（定长条），不可计数阶段只给阶段名（不定长动画条）。
- **best-effort 的局部失败**走事件通道的非致命通知：任务整体仍可继续的步骤（如个别文件下载失败）经 `Progress::report_error` 上报，不 abort 整次任务；缺资源（数据源本就没有该文件）不算失败、不报。
- **致命失败与最终结果**走命令返回路径：整条数据查询的失败照旧上抛，前端按普通命令错误弹提示；成功则命令正常返回。

常驻状态不进这条通道：游玩记录这类「一次动作开一段、由查询读」的状态落库，前端按需重取 —— 它没有进度可言，结束也不是靠取消。

新增任务时遵守：

- 任务名是跨前后端字面量，前后各定义一份，命名约定同命令名。
- 用 `Progress` 上报：每个用户该看得见推进的可计阶段都要有发射点；不要为单文件、单响应伪造 0/1 计数器，要么合进父阶段、要么给不定长条。
- 逐条目处理的任务在「这一条已写完」时调 `report_item`（`Progress` 的默认实现退化成 `report`，只有 `Task` 会把它落成 `game_id` 字段）：阶段事件说的是「正在处理」，不带 id；带 id 的是「这一条变了」，订阅方据此认领，不必为别人的进度空转。
- 取消：worker 在单元间隙查 `cancelled()`，命中即中断并以 `Cancelled` 收尾，下载类一并 abort 在途任务；前端置位标志后调 `cancel_task`，`await` 返回复查标志——取消与成功擦身而过时按取消处理，结果丢弃。
- 暂停：任务暴露暂停位，经状态查询读取、不进事件通道；worker 在单元间隙查暂停，暂停期间让出等待，取消可随时打断这段等待；暂停位在任务结束与重置时清除，残留的暂停位会让下一轮任务停在下一条目。
- 持 `Progress` 跨 `await` 时，trait 须标 `Sync`：引用它的 future 要满足 `Send`，否则检查失败只在异步 handler 边界报出。

通道约定：事件有两层传输（应用内 IPC 与开发期镜像 SSE），前端以统一订阅接口屏蔽差异；**新增任务不应要求前端改动传输层**，只在任务侧调 `report` / `report_error`。事件发送时入有界缓冲，订阅后才来的消费者先补读缓冲再收实时流；事件携绝对值，消费者以最新值覆盖即可续上，但补读区间有限，靠缓冲续命而非永久历史。

### 外部数据同步：计划-应用两段式

拉外部账户数据（VNDB 用户列表这类）落库的动作拆成两段，中间隔一次用户确认：

1. **计划段**：读上游、与本地比对、产出计划，全程零写库。计划是纯判定的结果——同样的上游条目与本地快照进来，产出同样的计划，与上次同步的结果无关，可反复重新生成。
2. **应用段**：用户在前端对计划逐条勾选后，一次提交整份决策，单事务落库。

配套约定：

- 上游拉取与库访问不放在同一个锁区间：网络 I/O 全程不持 `Connection` 锁，判定与落库才短暂持锁——这是「跨网络 I/O 不持锁」在同步链路上的完整落实姿势
- 判定分三路：本地现值、上次同步快照、上游现值。本地改过的字段进冲突集交用户逐字段选；上游相对快照变了、本地没动的进可对齐集；两侧一致的无操作
- 用户对每条计划的取舍是一次决策（勾选结果数组随命令一次性上行），逐条 invoke 会让「全选 / 全不选」这类批量操作退化成 N 次往返
- 「不动」与「清空」分开表达：可空字段用 `Option<Option<T>>`，外层是「这个字段没给」，内层是「显式置空」
- 上游侧无法映射到本地口径的条目（本地没有对应取值、纯附属数据）恒跳过，并在计划的警告区显式列出，由结果弹窗告知用户
- 两段式的适用边界是「会不会覆盖本地已有值」：会覆盖的批量写入走计划-确认-应用；只填空白列的写入没有可丢失的本地值，直接执行——资料补全属于这种

### 进程内短缓存

用户容易重复触发的只读网络查询（连点同步按钮、切回 tab 重取）在进程内缓存结果，设短 TTL（分钟级）。约定：

- 缓存返回克隆，调用方改不到缓存内容
- 失效条件随写动作与相关设置联动：落库类命令成功后、影响判定口径的设置项变更后、上游凭证更换后，缓存立即失效——TTL 只是兜底下限，不是唯一失效途径
- 缓存里嵌了本地侧快照的（如同步计划含库内现值），本地任何相关写动作都即使之失效：用户在表单里改个状态，缓存里的旧本地值就会把判定算错，这种缓存宁可没有
- 缓存只服务「重复触发」的读取，命令的响应语义不变：无缓存时行为与加缓存前一致

### 开发期 HTTP 通道

只在 debug 构建里存在，是 `commands.rs` 的镜像：加命令时这里要同步改。只绑 `127.0.0.1`，
且要设 `FISHVN_DEV_HTTP` 才开 —— 本机任何进程都能读写用户的库。端口与 vite 的 4545 分开，
撞上会自己请求自己。

请求体字段用命令原样的 snake_case：`invoke` 那条路会把 `gameId` 这类驼峰参数转成
snake_case，裸 HTTP 没有这层转换。前端日志命令（`log_frontend`）不经 devhttp——浏览器
会话的前端异常直接看 devtools，对应豁免记录在 devhttp 集成测试里。

### 日志

日志走 `tauri-plugin-log`，写入系统日志目录（Windows 为 `%LOCALAPPDATA%\{bundleIdentifier}\logs`），单文件 5MB 触发轮转并保留一个备份，时间戳用本地时区，级别默认 `info`，可用环境变量 `FISHVN_LOG_LEVEL` 覆盖（`off` / `error` / `warn` / `info` / `debug` / `trace`，写错或留空回落到 `info`）。开发与安装版写同一处，排查问题直接看这个目录。

进日志的内容与各自的 target：启动计时（`startup`，Rust 侧进程到后端就绪、前端侧 webview 导航到列表就绪）、任务生命周期（开始 / 完成 / 取消 / 失败，带耗时，记在 `fetch_vn_metadata` 外壳）、任务 `report_error` 的降级事件（封面单张下载失败这类不中断任务的错误）、图片下载失败、Rust panic（`panic`）、前端异常与 IPC 失败（`webview`，经 `log_frontend` 转发）、副作用命令失败（`command`，写库 / 落盘 / 起进程 / 网络写入这类命令，带错误类别，形如 `restore_backup 失败：[db] ...`）、游玩链路（`session`，开始、结束结算的秒数与计时口径、结算写库失败、启动时补账的孤儿会话）。

`command` 只把意外类（`db` / `io` / `json` / `upstream`）记成 warn；输入非法、找不到、冲突、取消、限流属于用户可预期的结果，默认级别下不进日志，`FISHVN_LOG_LEVEL=debug` 时以 debug 记一条。成功的 IPC 调用与高频进度事件不进日志；纯读命令失败也不进后端日志，靠前端那条 IPC 失败定位；`log_frontend` 把超长消息截断到 2000 字符。

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
| `user_game_state` | 状态、用户评分、笔记、默认安装、总时长 | 用户；总时长另有结算累加 |
| `play_sessions` | 一次游玩的起点、结束与两个时长 | 启动动作 |
| `installations` | 本地目录、入口 exe、候选入口、目录大小、顺序、转区开关 / Magpie 模式 / 附加参数 | 用户在前端表单里整包写回 |
| `game_metadata` | 别名与本地化标题、封面、发行日期、评分与人数、简介、标签、开发商 | 用户直接编辑；同步时逐字段比对后由用户选覆盖或跳过 |

几条从表结构看不出来的语义与配套惯例：

- `vndb_id` 可空且 `UNIQUE`：NULL 之间不算重复，一部 VN 只对应一条作品；重复绑定报冲突
- 删作品时安装记录随级联一并删除（`CASCADE`），删除的是库内登记，磁盘目录原样保留
- 状态取值定义在 `types.rs` 的 `Status` 枚举（wishlist / playing / finished / stalled / dropped），取值的合法集合只有这一份；消费方按五值全覆盖分支，漏一个就会在 UI 上丢一类条目
- `dir_path` 存 `resolve_path` 绝对化后的原样路径，不做大小写归一 —— `COLLATE NOCASE` 会在 macOS 与 Linux 上把不同目录判成同一个
- `game_metadata` 除 `game_id` 外每列都可空：一部不绑 VNDB 的作品同样要有资料行，没有数据就是空而不是占位值
- 资料行可有可无，且行不存在与行存在但字段为空是两种状态（「没填过」与「填了空的」），调用方要分得清
- 作品正式名在 `games.name`，`game_metadata` 不重复存。VNDB 那份 `titles` 里已带 `official` / `main` 标记的正式标题，别名另存 `alias`
- 状态行、安装行都随作品删除而清除（`CASCADE`）
- `game_metadata.image_id` 是 VNDB 的 cv 编号，本项目**不写它**：下载判重靠 `covers/vndb/<cv 编号>.jpg` 是否存在，同步冲突靠 `image_path` 是否为空。`image_path` 是本地文件的位置，两者不是一回事
- JSON 文本列（`titles` / `tags` / `developers` / `staff` / `aliases` / `traits` / `gender` / `exes`）：库里是文本，对外一律给数组或结构；要按其中某个键筛，只能整列取出后在 Rust 侧解
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
- 整包写回按「目录 + 入口」定位已有行：属于本作品的原地更新，属于别的作品的报冲突；同批出现同一对也报冲突
- 一个目录只归一部作品：草稿写进已被别的作品占用的目录就报冲突
- `patch` 改入口时先看同目录有没有别的行占着这一对，占了就报冲突而不是漏成唯一索引错误
- 目录元信息分两档取：只要候选 exe 时下探三层（限深也截断了读取），连大小一起取时遍历整棵子树。整包写回走前者，`size_bytes` 留空；新建与换目录时重算候选，换目录顺带把失效的 `size_bytes` 置空；目录没变就不读盘，搬走的目录不挡保存
- `entry_path` 存相对 `dir_path` 的路径：空串、绝对路径与含 `..` 的路径都判非法输入
- 安装项的三项启动控制都是该行的属性、不是全局设置：`le_launch`（布尔，默认 0）、`magpie`（模式整数，0 = 关 / 1 = 全屏 / 2 = 窗口，默认 0）与 `launch_args`（文本，可空；trim 后为空串按 NULL 存）。工具本体的路径不在 installations 里，在 `config.json` 的 `le_path` / `magpie_path`（均可空），启动时现取现验——没登记就 `BadRequest` 拦住、不起进程
- `user_game_state.play_seconds` 是作品的总时长：用户可随时改写，结算在既有值上累加。次数、最近游玩、前景时间、存活时间这四个聚合值由 `play_sessions` 在查询里聚合出来
- `user_game_state` 的同步基线三列：`synced_status` / `synced_rating` 存上次同步应用时写入的上游快照，`state_modified_at` 存本地状态与评分最后一次被改的时间。三路比对（本地现值 / 快照 / 上游）靠它们区分「本地改过」与「上游改过」：上游相对快照变了、本地没动，才进可对齐集；两者都动过的是冲突，交用户逐字段选。三列均可空，从没同步过的行是 NULL
- `play_sessions.ended_at` 为空即进行中；计时回路收尾与开库收尾都会把这次的前景时间累加进作品的总时长
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
失败不写文件（没有半张图留盘），下次拉取元数据——表单里选中搜索结果即拉取，或批量补全——会重下
该封面；单张失败只报错，不让整次同步失败。读超时与封面体积的约束见 [vndb-api.md](./vndb-api.md)
的「封面与图片」。

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

### 封面展示焦点

`game_metadata.cover_focal_x/y`（0-100，默认 50）是用户为封面选的展示位置，渲染成
`object-position: x% y%`。读它的展示处：卡片墙、详情页主封面、表单预览、合并面板缩略图、
首页继续游玩背景、详情页 FLIP 过渡；氛围背景层不跟（模糊装饰，取景无意义），角色立绘没有
该字段。新增封面展示处一律读这两个字段。

换封面重置焦点，收口在 `metadata::upsert`：本次 patch 改了 `image_path`（含清空再填入、
批量补全首次填封面）就强制把焦点写回 50/50，无视 patch 里带的焦点值——旧焦点对新图没有
意义。用户只调焦点不换图时正常生效。

### 注释

代码里不写注释，知识写进本文与 [vndb-api.md](./vndb-api.md)。规则与例外见 [comments.md](./comments.md)。

## 新增命令检查清单

- [ ] 领域函数在 `games.rs` / `installations.rs`，`commands.rs` 只转发
- [ ] 挂进 `main.rs` 的 `generate_handler!`，前端 `api.js` 有对应方法
- [ ] 新列挂对了表：路径进 `installations`，元数据进 `game_metadata`，状态进 `user_game_state`，游玩记录进 `play_sessions`
- [ ] 新增列有写入方
- [ ] 可空字段的 PATCH 用双层 `Option`，语义在 types.rs 上看得见
- [ ] 外键列判了空串
- [ ] 长任务经 `Progress` 播进度、经 `Tasks` 注册表收取消；best-effort 失败走 `report_error` 而非吞掉
- [ ] 新增测试指向临时库，文件系统用例有真目录树
- [ ] 注释过了 comments.md 的判据
- [ ] 跑过 `npm run check:rust` + `npm test`
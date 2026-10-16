# 注释去留对照表

对全项目 ~470 行注释的逐条分类。执行顺序：先补测试 → 搬知识文档 → 最后批量删注释并重写规范。

图例：`[ ]` 待处理 / `[x]` 已处理

判定基准（已定）：

- 测试文件的意图注释保留 —— 测试规格本身
- `tokens.css` 的 token 用途注释保留 —— 设计系统的一部分
- 普通代码默认零注释：需求与设计决策进 `docs/fdd-game-library.md`，栈特有事实与惯例进 `docs/dev/*`，复述与标准知识直接删

## 待保留（3 类，不删）

- [ ] 测试文件内意图注释（`tests/*.rs` ~40 条，如 `tests/characters.rs:58`「复现新增角色报 500」）——测试规格本身
- [ ] tokens.css 的 token 用途注释（~30 条）——规则文档，非代码旁注
- [ ] 真残渣 2 条，一行以内保留：
  - `fsinfo.rs:81` —— Rust 扩展名判定与 js `path.extname` 对齐，跨语言无法互测
  - `GameForm.svelte:352` —— 保存按钮在 tab 外，报错跨 tab 可见性，组件测试接不住

## → 测试（行为锁进断言，注释删）

| 状态 | 位置 | 内容 |
|---|---|---|
| [x] | `characters.rs:68` | 空数组 = 清库，不当坏请求 —— 已由 `空数组清空该作品的角色` 锁 |
| [x] | `characters.rs:86-88` / `199-200` | 删前留名字 —— 已由 `已有的声优行不被本地名字盖掉` 锁（触发器回收 + kept_staff）；跨作品共用行靠 ON CONFLICT 保名，新增 `别家已有的声优名不被新写入的盖掉` |
| [x] | `characters.rs:120` | 空串与 null 同义，让更全的填进来 —— 新增 `库里是空的声优名让传入的填进来`（空串 / null 两例） |
| [x] | `characters.rs:291` | 只按 character_id 查会捞出别作品配音 —— 已由 `同编号在两部作品的配音互不串台` 锁 |
| [x] | `vndb.rs:284` | 多 release 取首条会丢角色 —— 新增 `一个角色多条release时按入库档位挑而不是取首条`（`role_of`/`CharVn` 转 pub） |
| [x] | `db.rs:143` | ADD COLUMN 不幂等 —— 新建 `tests/db.rs`，`建表跑两遍不报错` |
| [x] | `metadata.rs:49` | 同步已绑作品不抹没同步的字段 —— 已由 `外层不给就不动内层置空才清掉` 锁 |
| [x] | `metadata.rs:126` | 标签整列替换 —— 已由 `标签整列替换` 锁（「删掉的标签不该因为 id 相同而复活」） |
| [x] | `games.rs:92` | 三表事务 —— 已由 `资料写失败时作品与状态都不留行` 锁 |
| [x] | `games.rs:108` | vndb_id 显式 null = 解绑 —— 已由 `patch_vndb_id给null是解绑之后另一条可以绑同一个号` + `patch只给vndb_id解绑不动name` 锁 |
| [x] | `games.rs:160` | 默认安装必须属于本作品 —— 已由 `默认安装只接受属于该作品的安装` 锁 |
| [x] | `thumbs.rs:68` | EXIF Rotate90 先缩放会算错边长 —— 新增 `exif方向在缩放前应用_不然算错目标边长`（`sample_with_exif` 手造方向 6 的 JPEG） |
| [x] | `GameForm.svelte:57` | 新增 `applyIncoming` 用例「多出来的键（characters / id / titles）不会进表单」——只搬 FIELDS 里的字段 |
| [x] | `GameForm.svelte:138` / `141` | 判为「坏了立刻可见」：少挡会 toast 报错且不回填，竞态会填错数据，不造测试，只删注释 |
| [x] | `GameForm.svelte:202` / `218` | 抽出 `cleanRating`（8..5 → 8.5、10.99 → 10.9、输入阶段不夹范围）与 `clampRating`（失焦夹 0..100），新增 `form-metadata.test.js` 两组用例 |
| [x] | `GameForm.svelte:340` | 抽出 `ratingRejected` / `votecountRejected`（没填不算错，越界与非数字算错），新增两组用例 |
| [x] | `DetailCharacters.svelte:34` | 空角色非失败 —— 后端那一半已由 `没有角色的作品读出来是空数组而不是报错` 锁，前端注释已删 |
| [x] | `DetailCharacters.svelte:72` / `89` | displayName 两边对拍 → 删组件本地 `charName`，改用 lib `displayName`（补 null 用例）；`[spoiler]` 连内容丢 → 删本地 `plain`，改用 lib `stripBbcode`（`vn-merge.test.js` 已锁） |
| [x] | `Modal.svelte:40` | 判为「坏了立刻可见」，直接删注释不造测试 |
| [x] | `confirm.svelte.js:25` | 新增 `confirm.test.js`，`已有确认框打开时再调会抛错，不共用上一个 promise` |
| [x] | `cover-src.js:4` | 缓存必须键最终 asset URL —— 新增 `cover-src.test.js`（4 例，核心是交给 convertFileSrc 的必须是缩图路径） |
| [x] | `form-metadata.js:16` / `vn-merge.js:81` | patch 不带 titles 不抹标题（已由 `不写 titles 与 image_id` 锁）；空串封面不覆盖已有（`VNDB 没下到封面就不动本地` 扩到 null + 空串两例） |
| [x] | `toast.svelte.js:38` / `51` / `75` | 新增 `toast.test.js`：`退场中的条目不算进上限，可见的始终不超过 5 条`、`退场结束移除的是那一条，不是最后一条`；`75` 那两个守卫查证为防御性、公共 API 不可达，不造测试 |
| [x] | `char-merge.js:111` | 排序稳定性：同档保持来源顺序 —— 已由 `同档内保持各自来源的原有顺序` 锁 |
| [x] | `scroll-dots.svelte.js:44` / `55` | 抽 `cardStep` 纯函数；新增 `scroll-dots.test.js`：`列数按基准卡宽与容器宽的纯函数算`、`容器隐藏时沿用已有的列数，不退回 1`、`每张卡的渲染宽与滚动步长` |
| [x] | `pick-path.js:5` | 判为「注释在说一条不存在的守卫」：知识搬进 `backend.md`「收图格式两份清单要手动对齐」，注释删掉（构建检查本身另立需求） |

## → backend.md（栈特有、罕见、难测）

| 状态 | 来源 | 条目 |
|---|---|---|
| [x] | `commands.rs:3-4` / `131` | Connection 单锁；不持锁跨网络 I/O；毒化转错误上报 |
| [x] | `commands.rs:165` + `lib.rs:35` + `cover-src.js:6` | asset 协议授权管线：默认白名单空 → 403；`$USERPROFILE` 在 scope 内不必另行授权；进程内授权 + 静态 scope 兜底 |
| [x] | `main.rs:42` | Tauri `Arc`/`State` 泛型精确匹配，写错编译不报 |
| [x] | `main.rs:11` + `devhttp.rs:5` | 只绑回环 + env 开关（本机进程可读写库） |
| [x] | `devhttp.rs:23` / `50` + `api.js:4` | dev 端口与 vite 4545 分开；camel→snake 转换只有 Tauri 侧有 |
| [x] | `error.rs:37` / `64` + `api.js:25` | IPC 只传一句中文；UNIQUE 扩展码单独报；invoke reject 包成 Error |
| [x] | `db.rs:21` | 外键那行「看着多余」不能删（级联依赖编译选项） |
| [x] | `db.rs:153` | table 拼 SQL，只能传字面量（注入面）——本文原有 `has_column` 一条已覆盖 |
| [x] | `metadata.rs:75` | SQLite changes 记命中行数而非变更行数 |
| [x] | `games.rs:13` | GROUP BY 必须含 metadata 主键，否则取任意行 |
| [x] | `thumbs.rs:1` | 封面派生管线：560 = 2× `--card-w`（`510` 那条实际在 `vndb.rs:515`，已移到 vndb-api 表） |
| [x] | `thumbs.rs:37` | 缓存残洞：同秒同尺寸仍中旧缓存（已知接受） |
| [x] | `characters.rs:19` 等 | 惯例条：库内 JSON 列对外数组、时间戳 unix 秒、主键 uuid v4 形态（`lib.rs:18`） |
| [x] | `types.rs:106` / `167` | PATCH 双层 Option 契约：外层没给 / 内层置空，空串 ≠ null —— 本文「PATCH 的三态」已覆盖 |
| [x] | `db.rs` 建表处 SQL `--` 25 行 | 表与字段语义：`vndb_id` 可空且 UNIQUE（NULL 之间不算重复）、状态行随作品删除、路径唯一由文件管理器给出规范写法 —— 归「数据模型」一节 |
| [x] | `installations.rs:68` | 先查重复再遍历目录：拖同一个目录两次不该付出读整个目录的代价；UNIQUE 仍是最终保证 |
| [x] | `metadata.rs:1` / `29` | 资料行可有可无；行不存在与字段为空是两种状态（同 FDD 条）——已收进本文「数据模型」，FDD 那条不再需要 |

另有几处纯复述或已被本文覆盖的 doc，随本批一并删：`db.rs:8/11/19/143/153`、`types.rs:93/114/152-153`、`installations.rs:12`、`characters.rs:83/95`、`games.rs:12/68`、`metadata.rs:72`、`commands.rs:149`（两次请求各扣一次额度，另行成条）。

本批已提交：`docs: 把后端栈特有知识搬进 backend.md，删掉对应注释`。`backend.md` 新增五节（进程与状态 / 开发期 HTTP 通道 / 错误与 IPC / 本地封面授权 / 封面派生图）、架构树的模块补全、「改哪块先读哪份」路由表、数据模型一节的一批语义条。

## → vndb-api.md（外部系统事实）

| 状态 | 来源 | 条目 |
|---|---|---|
| [x] | `vndb.rs:17` / `20` | 搜索 / 元数据两档字段 |
| [x] | `vndb.rs:21` | sexual / violence 刻意不取（过滤留设置项，别当漏做过滤补上） |
| [x] | `vndb.rs:23` / `381` | CV 只能从 va 取；va 有而 character 无的角色按无立绘处理 |
| [x] | `vndb.rs:26` | traits 要写进 fields 才带回名字 |
| [x] | `vndb.rs:30` | /character results 有上限，超了要翻页 |
| [x] | `vndb.rs:35` / `39` | 3 秒中断线对齐；图片另开客户端不受此限 |
| [x] | `vndb.rs:43` | 额度 200 次 / 5 分钟，按一半取留裕量 |
| [x] | `vndb.rs:47` | 立绘走 t.vndb.org CDN，与 api 限流是两套 |
| [x] | `vndb.rs:186` | null 率高，除 id/title 外一律 Option（计数句删） |
| [x] | `vndb.rs:207` | va note 是版本标记，按原样保留 |
| [x] | `vndb.rs:231` | 元数据与角色两次请求合并，无一次请求能给全 |
| [x] | `vndb.rs:251` / `452` | released 是 `[月, 日]` 形状，只有实测能确认；拍平存月-日 |
| [x] | `vndb.rs:464` | gender 按 release 逐条返回，多版本重复 |
| [x] | `vndb.rs:313` | sort 必带；search 过滤器传数组不报错而是静默失效 |
| [x] | `vndb.rs:370` | vn 过滤器只吃嵌套形状，写平是 400 |
| [x] | `vndb.rs:522` | 图编号（cv/ch）是永久身份，当文件名判断重下 |
| [x] | `vndb.rs:583` | 400 是我们请求写错，body 带 VNDB 指出的字段名 |
| [x] | `types.rs:144` / `204` / `226` | 出场定位取值、va note 多条已收进 vndb-api.md；「中文标题可能一条都没有」那半归 FDD 表 |
| [x] | `vndb.rs:515` | 下 url 不下 thumbnail：thumbnail 宽度不足以覆盖 2× 屏（要 554 宽） |
| [x] | `vndb.rs:140` | 搜索档只回 thumbnail，url / dims 恒为 None |

本批另外收进的：`vndb.rs:106`（take_slot 的 doc，纯复述）、`vndb.rs:308`（v 编号形态，`/vn` 字段表已给例）、`vndb.rs:418`（va 缺 staff 跳过）、`vndb.rs:458`（birthday 拍平成 `月-日`）、`vndb.rs:1-3`（文件头，本文开头已声明用途）、`vndb.rs:387-388`（va 有而 character 无）。

留在 `vndb.rs` 未动的三条属 FDD 表：`vndb.rs:259-260`（不收无立绘配角）、`298-299`（任一失败整体报错）、`475`（图下载失败只记日志）。

`vndb-api.md` 新增「客户端只取两档」「封面与图片」两节，并在 `/character`、限流两处补细节。

本批已提交：`docs: 把 VNDB 上游实测事实搬进 vndb-api.md，删掉对应注释`。

## → FDD（产品 / 设计决策）

| 状态 | 来源 | 条目 |
|---|---|---|
| [ ] | `characters.rs:1-4` / `56` | 整包替换 vs patch；顺序取数组下标（两个真相来源合一） |
| [ ] | `vndb.rs:280` | 不收无立绘配角（占位块 + 多下一批图） |
| [ ] | `vndb.rs:325` | 元数据与角色任一失败整体报错（只填一半像同步完成） |
| [ ] | `vndb.rs:512` | 图下载失败只记日志，不拖垮整次同步 |
| [x] | `metadata.rs:1` / `30` | 无行 = 没填过 ≠ 填了空 —— 已由 backend.md「数据模型」收回，注释在 backend 批次删掉 |
| [ ] | `types.rs:127` / `154` / `183` | 正式名单源 games.name；两种标签来源混列；c 编号按作品各存一行不建共享本体 |
| [ ] | `DetailCharacters.svelte:8` | trait 分组无序号字段，固定展示序，新分组排末尾 |
| [ ] | `DetailCharacters.svelte:105` | spoiler ≥ 1 不展示；成人向分组不进正文 |
| [ ] | `GameForm.svelte:31` | 字段级校验 vs toast 分工（操作失败走 toast） |
| [ ] | `GameForm.svelte:314` | 与作品级同口径：无取舍项不开面板，直接填新增角色 |
| [ ] | `GameForm.svelte:342` | 报错文案渲染在各字段旁，不共用 error |
| [ ] | `Detail.svelte:124` | 占位按钮对应「launch command 未实现」，功能列表状态 |
| [ ] | `characters.js:1-6` | 草稿留 `source` 原行：本地编辑不露出的 traits / 三围 / 生日要靠它原样带回，改一次名字就把 VNDB 考据数据抹了 |
| [ ] | `char-merge.js:27` / `50` | 比对字段表只收内容字段（位置与身份除外）；配对键取原名，罗马字重名会把两个角色判成一个 |
| [ ] | `vn-merge.js:3` | `[spoiler]` / `[raw]` 整块丢弃：前端不解析 BBCode，剥掉不等于丢数据 |
| [ ] | `Detail.svelte:27` / `37` | 评分按十分制展示（91 → 9.1，不补无信息量的 0）；标签按 rating 排、用户手打标签当 0 排在其后 |
| [ ] | `MyList.svelte:12` | 分区与筛选栏的展示顺序：游玩中的排最前 |

## → frontend.md（前端惯例与样式坑集）

| 状态 | 来源 | 条目 |
|---|---|---|
| [ ] | `Modal.svelte:15` + `portal.js:1` + `modal.css:8` / `83` / `92` | 弹窗基建：inert 挡 Tab；#modal-root 与 #app 平级；#globalTooltip 一起屏蔽；面板常驻 DOM 切 display；tab 栏不进滚动容器 |
| [ ] | `GameForm.svelte:74` / `118` | 弹窗关闭不卸载：字段要重置；实例随 open 挂与销毁 |
| [ ] | `GameForm.svelte:117` + `scrollbars.js:3/6/21/34/155/190` + `scrollbars.css:16/33/45` | 滚动条方案全套：覆盖式不占布局宽；插件注册；autoHide never；只记主视图；弹窗不挂 watchRightEdge；离窗清状态；transition 整体替换要列全位置项 |
| [ ] | `scroll-dots.svelte.js:1-5` + `hscroll.css:1` + `CharacterEditor.svelte:55` + `detail.css:307` | 布局契约一处写全：--card-w / --cols 纯函数计算；基准卡宽是档位门槛 |
| [ ] | `confirm.svelte.js:4` | 用 .svelte.js 后缀：要 $state，.js 里编译不了 rune |
| [ ] | `toast.svelte.js:1` / `7` / `64` | 渲染在 Toaster.svelte；计时器簿记不进 $state；暂停手动记剩余时长 |
| [ ] | `vndb-search.svelte.js:24` / `32` / `46` | 换词先清空；错误不走 i18n；返回 getter 不返回 $state 本体 |
| [ ] | `Menu.svelte:15` | blur 先于 click：焦点整个移出才关 |
| [ ] | `GameCard.svelte:52` | 可点区域独立真按钮，与菜单互为兄弟 |
| [ ] | `tooltip.js:1` + `side-tip.js:1` | 全局气泡替代 title；sideTip 与之并存 |
| [ ] | `i18n.js:1` + `DetailCharacters.svelte:96` | 缺失词条回落 key 保留原文 |
| [ ] | `theme.js:1` | 内联脚本与 JS 两侧判断必须一致 |
| [ ] | `Detail.svelte:92` | 鼠标侧键 3 = 后退，4 不接 |
| [ ] | `MyList.svelte:22` | 先落 gameId 再 open 的同帧时序 |
| [ ] | CSS 样式坑 ~25 条 | `toggle.css:1/19`、`detail.css:1/79/104/114/168/231/255/334/340`、`cover-grid.css:21/26` + `menus.css:1`、`dropdowns.css:1`、`forms.css:81/125`、`buttons.css:22/85`、`sidebar.css:67/82`、`tags.css:1/118` |
| [ ] | CSS 未列出的坑 | `detail.css` 10 条、`scrollbars.css` 8 条、`modal.css` 4 条、`vn-merge.css` 2 条、`menus.css:12/20`、`shell.css:1`、`forms.css:36`、`tags.css:31`、`toast.css:48`、`sidebar.css:129`、`tooltip.css:31` —— 与上一行合并成一份「样式坑集」 |
| [ ] | `frontend/scripts/check-css-tokens.js` / `check-dead-css.js` | 两个检查脚本的规则自述：三层架构与 L2 禁裸色值、只警告不报错的理由、复合选择器与 class 指令的判定 |
| [ ] | `char-merge.js:7/10/16/65` + `vn-merge.js:15/21/30` + `characters.js:40/90/95/104` | lib 的比对契约：空值两边归一、空值只与空值判等（0 是真实取值）、集合语义顺序无关、卡片名优先级、无名角色占掉一个名字 —— 归一份「前端 lib 惯例」 |
| [ ] | `Modal.svelte:26` + `GameForm.svelte:392` + `DetailCharacters.svelte:51` + `CharacterMergeDialog.svelte:16` + `CharacterEditor.svelte:39` + `Detail.svelte:56` | 零散惯例：onclose 可否决；后端与网络错误不走 i18n；立绘走 coverSrc 管线；untrack 只依赖角色列表 |

## → 删（复述类残留，未执行）

第二遍扫描补出的、没有任何知识含量的那些。与搬文档同批处理。

| 状态 | 范围 | 说明 |
|---|---|---|
| [ ] | 组件 props 一行说明 | `Menu.svelte:2`、`StatusSection.svelte:2`、`GameCard.svelte:2` —— 与类型签名重复 |
| [ ] | JSDoc 的参数表与返回值 | `scroll-dots.svelte.js:7-12/27-35`、`toast.svelte.js:20-26`、`confirm.svelte.js:16-23` —— 签名自明，其中「为什么」的部分归 frontend.md |
| [ ] | 复述实现的一行 doc | `scroll-dots.svelte.js:22`、`vn-merge.js:76/86/97` 等 |

## → 删（复述、标准知识、重复、工具指令）— 已执行

删的是「删了不丢信息」的那一类：复述代码、标准知识、重复、以及已有测试或文档兜底的条目。

| 状态 | 范围 | 说明 |
|---|---|---|
| [x] | `fsinfo.rs` 大部 | 删常量说明、遍历语义与 `resolve_path` doc；仅留 `95` 残渣 |
| [x] | `vndb.rs:71/240/494` | 复述 |
| [x] | `thumbs.rs:16/76/86` | q85 够用、Default 手动兜、不放大 |
| [x] | `devhttp.rs:42/67/175/222` | axum Clone、Value 透传、重复条目 |
| [x] | `types.rs` 字段级 doc、`error.rs` 变体名 | 类型签名自明 |
| [x] | `main.rs:1/34`、`games.rs:242` | 控制台窗口、data 目录（与 lib.rs 重复）、v14 格式 |
| [x] | `installations.rs:84/119` | 已由 `tests/installations.rs` 锁 |
| [x] | `metadata.rs:15` | JSON 文本对外数组（惯例条已收） |
| [x] | `MyList.svelte:28`、`CharacterEditor.svelte:89/154`、`BrandMark.svelte:2` | 复述 |
| [x] | `vn-merge.js:68`、`tooltip.js:24`、`vndb-cache.js:20`、`i18n-zh.js:62/168` | 复述或标准知识 |
| [x] | `api.js:24`、`characters.js:58` | 已由测试与 backend.md 条覆盖 |
| [x] | `form-metadata.js:10` | 已由 `tests/game_metadata.rs` 锁 |
| [x] | `Toaster.svelte:28` | 保留：svelte-ignore 是编译指令，不是注释 |
| [x] | CSS 文件头 / 分节横幅 / 一行复述 | `confirm.css`、`toast.css:12/86/117`、`sidebar.css:92/102`、`section-card.css`、`view-header.css`、`modal.css:1`、`shell.css:16`、`mylist.css:1`、`detail.css:278`、`cover-grid.css:75`、`buttons.css:59/71` |
| [x] | `styles.css:1` | 入口禁写样式（frontend.md 已有结构章节） |

### 原清单外

- `db.rs` 的 SQL `--` 注释（25 行，建表处的表/字段语义）——已归到 `→ backend.md` 的「数据模型」一行
- 内联约束说明，已改由代码自证消解（不留注释）：
  - `cover-grid.css` 圆角补偿 → 表面与裁切收拢进 `.cover-card__frame`，浏览器按「内圆角 = 外圆角 − 边框宽」自动裁子元素
  - `cover-grid.css` 标题让位按钮 → `--menu-plain-size`（tokens.css）与遮罩 `padding-right` 共用一份值
  - `vndb.rs` 睡 0 会忙等 → 抽 `take_slot` 纯函数 + 4 条测试
  - `vndb.rs` `local_path` 非 VNDB 字段 → `#[serde(skip_deserializing)]`
  - `fsinfo.rs` `resolve_path` 不 stat → 删 doc（函数体无 fs 调用，`ensure_dir` 紧跟其后）

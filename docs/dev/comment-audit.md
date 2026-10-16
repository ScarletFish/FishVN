# 注释去留对照表

对全项目 ~470 行注释的逐条分类。执行顺序：先补测试 → 搬知识文档 → 最后批量删注释并重写规范。

图例：`[ ]` 待处理 / `[x]` 已处理

## 待保留（3 类，不删）

- [ ] 测试文件内意图注释（`tests/*.rs` ~40 条，如 `tests/characters.rs:58`「复现新增角色报 500」）——测试规格本身
- [ ] tokens.css 的 token 用途注释（~30 条）——规则文档，非代码旁注
- [ ] 真残渣 2 条，一行以内保留：
  - `fsinfo.rs:95` —— Rust 扩展名判定与 js `path.extname` 对齐，跨语言无法互测
  - `GameForm.svelte:369` —— 保存按钮在 tab 外，报错跨 tab 可见性，组件测试接不住

## → 测试（行为锁进断言，注释删）

| 状态 | 位置 | 内容 |
|---|---|---|
| [ ] | `characters.rs:68` | 空数组 = 清库，不当坏请求 |
| [ ] | `characters.rs:86-88` / `199-200` | 删角色级联回收声优，删前要留名字（`tests/characters.rs:355` 已覆盖一半，补 kept_staff 用例） |
| [ ] | `characters.rs:120` | 空串与 null 同义，让更全的填进来 |
| [ ] | `characters.rs:291` | 只按 character_id 查会捞出别作品配音 |
| [ ] | `vndb.rs:284` | 多 release 取首条会丢角色（按入库档位挑条） |
| [ ] | `db.rs:143` | ADD COLUMN 不幂等（ensure_schema 跑两遍） |
| [ ] | `metadata.rs:49` | 同步已绑作品不抹没同步的字段 |
| [ ] | `metadata.rs:126` | 标签整列替换：删掉的标签要能存回去（round-trip） |
| [ ] | `games.rs:92` | 三表事务：建作品失败资料不落地 |
| [ ] | `games.rs:108` | vndb_id 显式 null = 解绑 |
| [ ] | `games.rs:160` | 默认安装必须属于本作品 |
| [ ] | `thumbs.rs:68` | EXIF Rotate90 先缩放会算错边长（带 EXIF fixture） |
| [ ] | `GameForm.svelte:57` | 角色混进 incoming 会被当作品字段 |
| [ ] | `GameForm.svelte:138` / `141` | 回填容错：角色读不出、作品被删不拖垮表单 |
| [ ] | `GameForm.svelte:202` / `218` | 输入清洗纯函数；失焦才夹范围（8..5、10 两个用例） |
| [ ] | `GameForm.svelte:340` | 评分越界与非数字挡住（百分制原值） |
| [ ] | `DetailCharacters.svelte:34` / `72` / `89` | 空角色非失败；displayName 两边对拍；`[spoiler]` 连内容丢（补进 `vn-merge.test.js`） |
| [ ] | `Modal.svelte:40` | 弹窗内部点击不等于点遮罩 |
| [ ] | `confirm.svelte.js:25` | 两次 confirm 不能共用一个 promise |
| [ ] | `cover-src.js:4` | 缓存必须键最终 asset URL，否则派生图白换 |
| [ ] | `form-metadata.js:16` / `vn-merge.js:81` | patch 不带 titles 不抹标题；空串封面不覆盖已有 |
| [ ] | `toast.svelte.js:38` / `51` / `75` | dismissing 不计上限；提前移除防 splice(-1)；防重复启动 |
| [ ] | `char-merge.js:111` | 排序稳定性：同档保持来源顺序 |
| [ ] | `scroll-dots.svelte.js:44` / `55` | 隐藏不重算（塌缩 9→5→3→2→1）；渲染宽公式断言 |
| [ ] | `pick-path.js:5` | 新增构建检查：Cargo.toml features 与清单比对（现状无守卫） |

## → backend.md（栈特有、罕见、难测）

| 状态 | 来源 | 条目 |
|---|---|---|
| [ ] | `commands.rs:3-4` / `131` | Connection 单锁；不持锁跨网络 I/O；毒化转错误上报 |
| [ ] | `commands.rs:165` + `lib.rs:35` + `cover-src.js:6` | asset 协议授权管线：默认白名单空 → 403；`$USERPROFILE` 在 scope 内不必另行授权；进程内授权 + 静态 scope 兜底 |
| [ ] | `main.rs:42` | Tauri `Arc`/`State` 泛型精确匹配，写错编译不报 |
| [ ] | `main.rs:11` + `devhttp.rs:5` | 只绑回环 + env 开关（本机进程可读写库） |
| [ ] | `devhttp.rs:23` / `50` + `api.js:4` | dev 端口与 vite 4545 分开；camel→snake 转换只有 Tauri 侧有 |
| [ ] | `error.rs:37` / `64` + `api.js:25` | IPC 只传一句中文；UNIQUE 扩展码单独报；invoke reject 包成 Error |
| [ ] | `db.rs:21` | 外键那行「看着多余」不能删（级联依赖编译选项） |
| [ ] | `db.rs:153` | table 拼 SQL，只能传字面量（注入面） |
| [ ] | `metadata.rs:75` | SQLite changes 记命中行数而非变更行数 |
| [ ] | `games.rs:13` | GROUP BY 必须含 metadata 主键，否则取任意行 |
| [ ] | `thumbs.rs:1` / `510` | 封面派生管线：560 = 2× `--card-w`；下 url 不下 thumbnail（2× 屏 554） |
| [ ] | `thumbs.rs:37` | 缓存残洞：同秒同尺寸仍中旧缓存（已知接受） |
| [ ] | `characters.rs:19` 等 | 惯例条：库内 JSON 列对外数组、时间戳 unix 秒、主键 uuid v4 形态（`lib.rs:18`） |
| [ ] | `types.rs:106` / `167` | PATCH 双层 Option 契约：外层没给 / 内层置空，空串 ≠ null |

## → vndb-api.md（外部系统事实）

| 状态 | 来源 | 条目 |
|---|---|---|
| [ ] | `vndb.rs:17` / `20` | 搜索 / 元数据两档字段 |
| [ ] | `vndb.rs:21` | sexual / violence 刻意不取（过滤留设置项，别当漏做过滤补上） |
| [ ] | `vndb.rs:23` / `381` | CV 只能从 va 取；va 有而 character 无的角色按无立绘处理 |
| [ ] | `vndb.rs:26` | traits 要写进 fields 才带回名字 |
| [ ] | `vndb.rs:30` | /character results 有上限，超了要翻页 |
| [ ] | `vndb.rs:35` / `39` | 3 秒中断线对齐；图片另开客户端不受此限 |
| [ ] | `vndb.rs:43` | 额度 200 次 / 5 分钟，按一半取留裕量 |
| [ ] | `vndb.rs:47` | 立绘走 t.vndb.org CDN，与 api 限流是两套 |
| [ ] | `vndb.rs:186` | null 率高，除 id/title 外一律 Option（计数句删） |
| [ ] | `vndb.rs:207` | va note 是版本标记，按原样保留 |
| [ ] | `vndb.rs:231` | 元数据与角色两次请求合并，无一次请求能给全 |
| [ ] | `vndb.rs:251` / `452` | released 是 `[月, 日]` 形状，只有实测能确认；拍平存月-日 |
| [ ] | `vndb.rs:464` | gender 按 release 逐条返回，多版本重复 |
| [ ] | `vndb.rs:313` | sort 必带；search 过滤器传数组不报错而是静默失效 |
| [ ] | `vndb.rs:370` | vn 过滤器只吃嵌套形状，写平是 400 |
| [ ] | `vndb.rs:522` | 图编号（cv/ch）是永久身份，当文件名判断重下 |
| [ ] | `vndb.rs:583` | 400 是我们请求写错，body 带 VNDB 指出的字段名 |
| [ ] | `types.rs:144` / `204` / `226` | 中文标题可能一条都没有；出场定位取值；va note 多条 |

## → FDD（产品 / 设计决策）

| 状态 | 来源 | 条目 |
|---|---|---|
| [ ] | `characters.rs:1-4` / `56` | 整包替换 vs patch；顺序取数组下标（两个真相来源合一） |
| [ ] | `vndb.rs:280` | 不收无立绘配角（占位块 + 多下一批图） |
| [ ] | `vndb.rs:325` | 元数据与角色任一失败整体报错（只填一半像同步完成） |
| [ ] | `vndb.rs:512` | 图下载失败只记日志，不拖垮整次同步 |
| [ ] | `metadata.rs:1` / `30` | 无行 = 没填过 ≠ 填了空 |
| [ ] | `types.rs:127` / `154` / `183` | 正式名单源 games.name；两种标签来源混列；c 编号按作品各存一行不建共享本体 |
| [ ] | `DetailCharacters.svelte:8` | trait 分组无序号字段，固定展示序，新分组排末尾 |
| [ ] | `DetailCharacters.svelte:105` | spoiler ≥ 1 不展示；成人向分组不进正文 |
| [ ] | `GameForm.svelte:31` | 字段级校验 vs toast 分工（操作失败走 toast） |
| [ ] | `GameForm.svelte:314` | 与作品级同口径：无取舍项不开面板，直接填新增角色 |
| [ ] | `GameForm.svelte:342` | 报错文案渲染在各字段旁，不共用 error |
| [ ] | `Detail.svelte:124` | 占位按钮对应「launch command 未实现」，功能列表状态 |

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

- `db.rs` 的 SQL `--` 注释（约 30 处，建表处的表/字段语义）——统计只匹配 `//` 时漏掉，待议归类
- 内联约束说明，已改由代码自证消解（不留注释）：
  - `cover-grid.css` 圆角补偿 → 表面与裁切收拢进 `.cover-card__frame`，浏览器按「内圆角 = 外圆角 − 边框宽」自动裁子元素
  - `cover-grid.css` 标题让位按钮 → `--menu-plain-size`（tokens.css）与遮罩 `padding-right` 共用一份值
  - `vndb.rs` 睡 0 会忙等 → 抽 `take_slot` 纯函数 + 4 条测试
  - `vndb.rs` `local_path` 非 VNDB 字段 → `#[serde(skip_deserializing)]`
  - `fsinfo.rs` `resolve_path` 不 stat → 删 doc（函数体无 fs 调用，`ensure_dir` 紧跟其后）

# 角色数据接入 — Characters

## 1. 范围

打通 VNDB 角色数据：随元数据同步落库，在详情页呈现。

**不包含**角色编辑的 UI/UX，形态另行讨论。

## 2. VNDB 端点

`POST https://api.vndb.org/kana/character`，字段定义见 `GET /kana/schema` 的 `api_fields["/character"]`。

### 取一部作品的角色

`vn` 过滤器只接受嵌套查询：

```json
{ "filters": ["vn", "=", ["id", "=", "v11"]], "results": 100 }
```

写成 `["vn","=",  "v11"]` 或对象值都是 400 `Invalid 'filters' member: Invalid query`。
过滤器里没有 `= v11` 这种简写，别照 `/vn` 的形状套。

### traits 的字段要靠 `_inherit` 展开

schema 里 `traits` 只声明了 `_inherit` / `lie` / `spoiler` 三项，`name` 与 `group_name` 是继承
`/trait` 得来的。`fields` 里显式写出来就能拿到：

```
traits{id,name,group_name,spoiler,lie}
```

不写出来只会返回 trait 的 id。

### 声优不在 `/character` 上

`/character` 的字段里没有声优。CV 是挂在作品上的关系，只存在于 `/vn` 的 `va`：

```
va{character{id},staff{id,name,original},note}
```

按 `character.id` 挂回角色。`note` 是版本标记（如作品的重制版），同一角色在不同版本下会出现多条
VA 记录，入库时按 `(character_id, staff_id, note)` 去重。

### 响应里会重复的字段

`gender` 按 release 逐条返回，同一角色的多个 release 会让数组出现重复值（实测 `["f","f"]`），
展示前要去重。

### 耗时特征

请求耗时由固定开销主导（约 2 s），与批量大小无单调关系，同一请求重复发出的耗时可差数倍。
因此：

- 一次请求取全量比按需分次取划算——分次取每次都要再付一次固定开销与抖动风险。
- 不要依据单次实测的耗时做设计取舍，只能参照量级。

### trait 分组

共 11 个，两个是成人向：`Engages in (Sexual)` 与 `Subject of (Sexual)`。过滤按分组名里是否含
`Sexual` 判断。`group_name` 没有序号字段，展示顺序靠一张固定顺序表，表外的新分组排到末尾。

---

## 3. 数据模型

### 3.1 三张表

```sql
-- 角色属于作品。VNDB 的实际数据形态就是每部作品各一个角色条目：
-- 同名角色在不同作品下是不同的 c 编号、数据与立绘各自独立
-- （例：Illya 在 Fate/stay night 与 Fake/ever since 就是两个条目）。
-- 因此本地不做共享本体，characters 直接挂 game_id，一部作品一套角色
CREATE TABLE IF NOT EXISTS characters (
  game_id     TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  id          TEXT NOT NULL,      -- VNDB c 编号；不同作品下可重复，各自成行
  name        TEXT,
  original    TEXT,
  aliases     TEXT,               -- JSON 数组
  description TEXT,
  gender      TEXT,               -- JSON 数组，入库前去重
  age         INTEGER,
  birthday    TEXT,
  blood_type  TEXT,
  height      INTEGER,
  weight      INTEGER,
  bust        INTEGER,
  waist       INTEGER,
  hips        INTEGER,
  cup         TEXT,
  role        TEXT,               -- main / primary / side / appears，取 vns 里当前 VN 那条
  image_id    TEXT,
  image_path  TEXT,
  sort_order  INTEGER NOT NULL DEFAULT 0,
  traits      TEXT,               -- JSON 数组
  PRIMARY KEY (game_id, id)
);

CREATE TABLE IF NOT EXISTS staffs (
  id       TEXT PRIMARY KEY,      -- VNDB s 编号
  name     TEXT,
  original TEXT
);

-- note 是版本标记（如 Realta Nua / Vita/Remastered），同一对角色与声优
-- 可带多条不同 note。note 允许为空，落库时空值统一存 ''
-- 复合外键指向 characters：character_va 的行跟着角色走，删角色即删配音关系
CREATE TABLE IF NOT EXISTS character_va (
  game_id      TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  character_id TEXT NOT NULL,
  staff_id     TEXT NOT NULL REFERENCES staffs(id) ON DELETE CASCADE,
  note         TEXT NOT NULL DEFAULT '',
  PRIMARY KEY (game_id, character_id, staff_id, note),
  FOREIGN KEY (game_id, character_id) REFERENCES characters(game_id, id) ON DELETE CASCADE
);
```

### 3.2 为什么独立建表，以及为什么不做共享本体

独立成表（而不是塞进 `game_metadata` 的 JSON 列）的理由只有一条：**按声优检索作品是这个项目的
常规需求**（全局搜索规划）。这类查询要从声优反查作品，JSON 列在 SQLite 里只能全表扫加解析，
建不上索引；规范化之后 `staffs → character_va → characters → games` 是一条带索引的 join。

不做共享本体（不建 `game_characters` 关联表）的理由是源数据形态：VNDB 上"同一角色跨作品出场"
大多以**不同角色条目**建模——c 编号不同、数据详略不同、立绘不同。共享本体是为源数据里
并不存在的"同一个实体"付间接层的成本。角色随作品删而 CASCADE 删除，重新同步即可再取回。

### 3.3 不变约束

- `characters` 的写者只有 VNDB 同步，同步以作品为单位整包替换该作品的角色集。
- 删除作品走 `CASCADE` 清掉 `characters` 与 `character_va` 的对应行；`staffs` 是全局表，
  不随作品删除。
- `character_va` 的主键含 `note`：同一对角色与声优带不同版本标记时各自保留。
- `gender` 入库前去重，库里不存重复值。
- 立绘在 `characters` 行上。VNDB 的 `/character` 只返回角色主图，与按哪部作品过滤无关；
  行挂在作品下，天然就是"这部作品用这张图"。

---

## 4. 写路径

### 4.1 两种写者，同一个替换语义

角色表有两个写者，写的功能是同一个 `characters::replace_all`：**以作品为单位整包替换**——
先删该作品的 `characters` 行（`character_va` 靠外键级联），再按数组顺序写回。

| 写者 | 触发 | 数据来源 |
|---|---|---|
| 用户 | 表单「角色」tab 保存（`save_game_characters`） | 前端草稿 |
| VNDB 同步 | 并入现有「同步元数据」动作（待实现） | `/character` + `/vn` 的 `va` |

### 4.2 VNDB 同步的比对口径

按**角色名**判定是不是同一个角色，优先比 `original`，缺失时退回 `name`——原名相同的概率
远低于罗马字名，更贴近"同一个角色"的语义。

只有**重名且内容有差异**才进冲突面板。重名是少数情况，因此面板行数保持在很小的量级。
面板呈现与封面同源：每个冲突角色一行，两侧各展示 `name` 与立绘缩略图，逐行选覆盖或跳过。

`fetch_vn_metadata` 内部并行发 `/vn`（现有 `META_FIELDS` 追加 `va`）与 `/character`——
实测追加 `va` 后耗时仍落在抖动区间内。

### 4.3 用户改动会被同步覆盖

整包替换下用户改过的字段会被 VNDB 的值盖回去，这是当前语义。收窄替换粒度
（保留用户编辑过的字段）要等真出现这个诉求再做。

---

## 5. 本地编辑形态（已实现）

入口是表单第四个 tab「角色」，形态照抄详情页那份：横向卡片条当索引，选中一张就地展开
编辑面板。**所有元数据都在表单里改**这条约定不变，因此详情页不做就地编辑，避免在同一屏
里出现第二套保存/取消语义。

### 5.1 卡宽 180px，一屏 4 张

`.hscroll-section` 的列数由 `floor((容器宽 + gap) / (基准卡宽 + gap))` 算出，卡片再按列数
弹性均分——**基准卡宽只是分页门槛，不是渲染宽度**。两处容器宽不同，卡宽必须分开定：

| 容器 | 内容宽 | 基准卡宽 | 每屏 | 渲染卡宽 |
|---|---|---|---|---|
| 详情页 | ~1340 | 220px | 4–5 | ~270 |
| 表单 | 852（VNDB 旁窗打开 792） | **180px** | 4 | 186–201 |

表单侧取 180 的来源是**表单里已有的封面预览**（`--cover-preview-w: 11.25rem`）——首版按纯
列数算取了 112px（6 列），上手偏小，改成与封面同档。要 852 与 792 都稳定落在 4 列，基准值
只能取 (157.6, 186]，180 在中部、且与现有封面视觉一致。

展开面板的左列立绘同样用 `--cover-preview-w`，与封面预览等宽。

**卡片高度不随渲染宽涨，靠叠字而不是页脚**。首版立绘区 `aspect-ratio: 208/300`，渲染宽
~201px 时立绘 290px、加独立页脚总高 ~355px，比封面预览（180×240）高出近一半。给页脚单独留
一条是最贵的做法——它的 65px 完全可以用叠字省掉：

- 立绘保留 `aspect-ratio: 208/300` 原比例不裁，名字绝对定位压在图底，用封面墙同款的
  `--cover-scrim` 渐变保证可读性，文字色 `--fg-on-accent`；CV 移到卡片 `title` 悬停提示。
- 总高 = 立绘高度 ~290px，图完整，只被 scrim 半透遮住底部一小条。

写字面量不写 `var(--cover-preview-w)`（虽然数值相同）：`--card-w` 要被 `getPropertyValue`
取出来 `parseFloat`，套一层 var 一旦拿不到替换后的值，会静默回落成默认 320px、直接塌成 2 列。
（面板左侧那列用 `--cover-preview-w` 做宽度是安全的，它不是 JS 读取的属性。）

### 5.1.1 「添加」按钮与分页

卡片条里只放角色卡，「添加」是头部常显的按钮，不参与分页。两个理由：**添加是高频动作**，
10 个角色时它排在最后一个就要滑三屏；**混合计算会污染分页**——`initScrollDots` 的页数是
`total - cols + 1`，把按钮也算进 `total` 会让 4 个角色（刚好一屏）凭空多出一页。

由此 `.scroll-dots` 的位置也要显式给宿主：`initScrollDots` 是把圆点 append 到 `dotsParent`
末尾的，若直接传头部元素，圆点会排在按钮右侧；头里放一个空的 `<span>` 宿主，圆点就落在
按钮左边。另外 0 个角色时圆点根本不生成，靠 `margin-left: auto` 定位的按钮会失去右对齐，
需要一个 `flex: 1` 的占位撑住。

### 5.2 只暴露手填得动的字段

开的是立绘 / 名字 / 原名 / 性别 / 年龄 / 身高 / 体重 / 定位 / CV / 简介。
不开 traits 与三围、生日、血型——这些只在 VNDB 同步时才有来源，手填既不现实也没意义。

不露出来的字段靠 `lib/characters.js` 里那份「原样带回去」的机制保住：草稿持有取回时的
整行（`source`），保存时铺在最前面，只覆盖表单露出的字段。否则用户改一次名字，
同步来的 traits 就没了。

### 5.3 本地 id

用户手建的角色与声优没有 VNDB 编号，`id` 生成 `local-<uuid>`，写在同一个 `id` 列里。
这样主键不变，两种来源混排，将来 VNDB 同步只需比对 c 编号。

### 5.4 排序不用拖拽

横向卡条是 `scroll-snap-type: x mandatory`，拖拽重排会和吸边打架。改成左移/右移按钮，
顺序的真相是数组顺序：后端写入时按数组下标覆盖 `sort_order`，客户端给的值不采信。

---

## 6. 落点与进度

### 6.1 已完成（本地编辑）

| 文件 | 改动 |
|---|---|
| `db.rs` | `ensure_schema()` 加 `characters` / `staffs` / `character_va` |
| `types.rs` | `GameCharacter` / `CharacterTrait` / `CharacterVa` |
| `characters.rs` | `list` / `replace_all`（事务内删+插） |
| `commands.rs` | `list_game_characters` / `save_game_characters` |
| `main.rs` / `devhttp.rs` | 命令注册（测试 `路由表覆盖全部命令` 会盯这两处同步） |
| `lib/characters.js` | 草稿与 payload 互转、`moveCharacters` |
| `components/CharacterEditor.svelte` | 卡片条 + 就地编辑面板 |
| `components/GameForm.svelte` | 第四个 tab；读取回填；保存时提交 |
| `lib/api.js` / `lib/i18n-zh.js` | 两个命令；角色与声优词条 |

### 6.2 待做

| 步骤 | 文件 |
|---|---|
| 详情页接真实数据 | `views/DetailCharacters.svelte` 接 `gameId` 自取；`views/Detail.svelte` 传 `gameId`；删 `lib/sample-characters.js` |
| VNDB 同步 | `vndb.rs` 的 `CHAR_FIELDS` / `fetch_characters()`、`META_FIELDS` 追加 `va`、立绘落盘；`lib/vn-merge.js` 的角色比对 |

详情页呈现沿用现有卡片横滑与展开面板：卡片显示立绘、`original`、首位 CV；展开后显示
性别 / 尺寸 / 年龄 / 声优与按分组归类的 traits。

---

## 7. 未决

- **用户改动与同步的关系**：见 4.3，当前是同步覆盖用户改动。
- **全局搜索的角色/声优检索面**：数据已落在可 join 的表里
  （`staffs → character_va → characters → games`），检索入口不在本轮范围。

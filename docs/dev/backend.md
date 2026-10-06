# 后端开发规范 — Backend

### 架构一览

- **Hono**（`hono` + `@hono/node-server`），无其他框架依赖
- **ESM + nodenext**：相对 import 一律带 `.js` 后缀
- **strict 全绿，禁止 `any`**；改完必跑 `npm run typecheck` + `npm test`
- 构建产物 → 根 `dist/server/`；运行时数据 → 根 `data/`

```
apps/server/
├── server.ts        # 只做：装配路由 + 静态托管 + 启动
├── constants.ts     # PORT/DEFAULT_HOST/API_PREFIX/DB_FILE 单点
├── db.ts            # getDb() 惰性单例 + ensureSchema()
├── lib/paths.ts     # ROOT/WEB_DIR/DATA_DIR 路径单点
├── routes/<域>.ts   # 一个域一个文件，导出 hono 路由
└── __tests__/       # node --test 原生，起真服打真请求
```

### 端口与监听

- 默认 `4545`，显式只听 `127.0.0.1`（不暴露局域网）；局域网访问用 `HOST=0.0.0.0` 覆盖
- **地址全项目统一字面量 `127.0.0.1`**：后端绑定、vite 代理、vite dev 绑定、测试、
  将来 Tauri 壳的 start_url。不用 `localhost`——本机解析先 `::1` 后 `127.0.0.1`，
  IPv6-first 客户端（Chromium 系）先连 `::1` 失败再回退

### 路由规则

- 新增端点：`routes/<域>.ts` 建文件 → `server.ts` 里 `app.route(API_PREFIX, ...)` 挂一行
- `/api` 前缀之外的路径全部属于前端（静态/SPA 兜底），API 404 返回 JSON
- 路径参数用 hono 原生 `app.get('/games/:id')`，不手写解析

### 路径与数据

- 端口/前缀/文件名/外部端点 → `constants.ts`；路径 → `lib/paths.ts`。**禁止散落魔法字符串**
- `ROOT` 从编译产物位置上探三层（`dist/server/lib/` → 项目根），移动构建目录时这里要跟着改
- 数据库单文件 `data/fishvn.db`，惰性 `getDb()` 进程内单例；表结构演进**只写在 `ensureSchema()`**，保持幂等
- 测试用 `FISHVN_DATA_DIR` 指向临时库，跑真服打真请求

### 数据模型

作品身份、本地安装、VNDB 元数据、用户状态、磁盘扫描事实**各是一张表**。
一张表一个自动写者；用户显式编辑优先，自动流程只填空字段。

```text
games              作品身份：id / name / vndb_id / created_at
user_game_state    1:1 用户状态、默认启动安装
installations      1:N 本地目录，一部作品可有多个（不同发行版、不同盘）
game_metadata      1:1 VNDB 与展示元数据
library_roots      配置：扫哪些目录
```

各表的列与写者：

| 表 | 内容 | 写者 |
|---|---|---|
| `games` | 作品身份 | 用户创建与编辑 |
| `user_game_state` | 状态、用户评分、笔记、默认启动安装、游玩时间与次数 | 用户；启动行为更新游玩时间与次数 |
| `installations` | 本地目录、入口 exe、候选入口、目录大小 | 用户建与认领；启动行为不写 |
| `game_metadata` | VNDB 标题、封面、评分、简介 | VNDB 同步填空字段；用户可改 |
| `library_roots` | 库根路径 | 用户；扫描记录时间 |

**`games` 四列**：新建条目只给 `name`，其余由后续流程填入。

- `vndb_id` 可空且 `UNIQUE`：NULL 之间不算重复，一部 VN 只对应一条作品；
  重复绑定回 409，提示该作品已存在
- 一部作品对应多个安装时（汉化版 / 原版 / 不同盘），`installations.dir_path` 唯一

### 游玩状态

取值 `wishlist` / `playing` / `finished` / `dropped`，与 VNDB 的状态集一致，双向同步时映射是恒等。
定义在 `types.ts` 的 `STATUSES`，路由层按它校验，表上不加 `CHECK`——SQLite 改约束要重建整张表。

新建作品时在同一事务里带上 `status = 'wishlist'` 的状态行，所以每部作品必然有状态，
读状态接口不需要处理「不存在」。删作品时状态随外键级联清除。

### 本地安装

`installations` 一行是一个本地目录，`game_id` 为 NULL 表示待认领。
`ON DELETE SET NULL` 而不是 CASCADE：删作品条目不带走目录记录，那个目录还在盘上，下次扫描还会遇到它。

建安装时 `lib/fsinfo.ts` 读一次目录，填入 `exes` / `size_bytes`：

- 候选入口是 `.exe` / `.bat` / `.cmd`，顶层记第 0 层、最深收到第 2 层，最多留 30 个，层数浅的在前
- 大小累加全部深度，与候选深度无关。同步全深度遍历适合单个目录；批量遍历要另走异步并带进度

候选只有一个时直接填 `entry_path`——没有选择余地。多个则留空，由用户选。

`dir_path` 由文件管理器给出规范写法，所以不做大小写归一：`COLLATE NOCASE` 会在 macOS 与 Linux 上
把两个真实不同的目录判成同一个。路径存绝对路径，`lib/fsinfo.ts` 之外没有其他地方碰文件系统。

### VNDB

`lib/vndb.ts` 是访问 VNDB 的唯一出口。`vndb_id` 的校验在读的一侧做：搜索接口返回的候选直接来自 VNDB，
同步元数据时按 `vndb_id` 取回的数据会与库里的值比对。写 `vndb_id` 本身是纯库内操作，编号由搜索结果带入。

### 端点

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/health` | 存活探测 |
| GET | `/api/games` | 列出全部作品，按创建时间倒序 |
| POST | `/api/games` | 新建；`name` 必填，`vndb_id` 可选 |
| PATCH | `/api/games/:id` | 改 `name` / `vndb_id`；`vndb_id: null` 解绑 |
| DELETE | `/api/games/:id` | 删作品，关联记录随外键级联清除 |
| GET | `/api/games/:id/state` | 读状态、评分、笔记、默认安装 |
| PATCH | `/api/games/:id/state` | 改 `status` / `user_rating` / `notes` / `default_installation_id` |
| GET | `/api/installations` | 列出全部安装；带 `?game_id=` 只看某作品的 |
| POST | `/api/installations` | 建安装；读目录自动填候选入口与大小 |
| PATCH | `/api/installations/:id` | 改 `entry_path` / `game_id`（认领） |
| DELETE | `/api/installations/:id` | 删安装记录 |

响应统一包一层：`{ games }` / `{ game }` / `{ error }`。DELETE 成功回 204 无正文。

### 新增接口检查清单

- [ ] 路由在 `routes/` 独立文件，`server.ts` 只多一行挂载
- [ ] 无 `any`，常量/路径没有散落
- [ ] 新列挂对了表：路径进 `installations`，元数据进 `game_metadata`，状态进 `user_game_state`
- [ ] 新增列有写入方，没有等着将来才填的空壳列
- [ ] 文件系统访问在 `lib/` 的模块里，路由不直接调 `fs`
- [ ] 外部服务走 `lib/` 里的模块，不在路由里内联 fetch
- [ ] 新增测试指向临时库，外部服务有假实现
- [ ] 跑过 `npm run typecheck` + `npm test`
- [ ] （涉及前端联调）vite 代理已覆盖，或直接请求 `/api/...` 验证过

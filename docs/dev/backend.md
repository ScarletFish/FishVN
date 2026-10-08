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

作品身份、本地安装、VNDB 元数据、用户状态、库根配置**各是一张表**。
**一张表一个自动写者**；用户显式编辑优先，自动流程只填空字段。

| 表 | 内容 | 写者 |
|---|---|---|
| `games` | 作品身份 | 用户创建与编辑 |
| `user_game_state` | 状态、用户评分、笔记、默认安装、游玩时间与次数 | 用户；启动行为更新游玩时间与次数 |
| `installations` | 本地目录、入口 exe、候选入口、目录大小 | 用户建与认领 |
| `game_metadata` | VNDB 标题、封面、评分、简介 | VNDB 同步填空字段；用户可改 |
| `library_roots` | 库根路径 | 用户；扫描记录时间 |

几条从表结构看不出来的语义：

- `vndb_id` 可空且 `UNIQUE`：NULL 之间不算重复，一部 VN 只对应一条作品；重复绑定回 409
- `installations.game_id` 为 NULL 表示待认领。删作品走 `SET NULL` 而不是 CASCADE —— 目录还在盘上，下次扫描还会遇到它
- 状态取值与 VNDB 一致，定义在 `types.ts` 的 `STATUSES`。表上不加 `CHECK`，SQLite 改约束要重建整张表
- `dir_path` 由文件管理器给出规范写法，不做大小写归一 —— `COLLATE NOCASE` 会在 macOS 与 Linux 上把不同目录判成同一个

响应统一包一层：`{ games }` / `{ game }` / `{ error }`。DELETE 成功回 204 无正文。

### 注释

规则见 [comments.md](./comments.md)。判据一句话：**删掉这条注释，会不会有人因此走错路**。

## 新增接口检查清单

- [ ] 路由在 `routes/` 独立文件，`server.ts` 只多一行挂载
- [ ] 无 `any`，常量/路径没有散落
- [ ] 新列挂对了表：路径进 `installations`，元数据进 `game_metadata`，状态进 `user_game_state`
- [ ] 新增列有写入方
- [ ] 文件系统与外部服务的访问在 `lib/` 的模块里，路由不直接调
- [ ] 新增测试指向临时库，外部服务有假实现
- [ ] 注释过了 comments.md 的判据
- [ ] 新增小节是规则或语义，不是代码里读得到的实现描述
- [ ] 跑过 `npm run typecheck` + `npm test`
- [ ] （涉及前端联调）vite 代理已覆盖，或直接请求 `/api/...` 验证过

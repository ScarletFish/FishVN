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

- 端口/前缀/文件名 → `constants.ts`；路径 → `lib/paths.ts`。**禁止散落魔法字符串**
- `ROOT` 从编译产物位置上探三层（`dist/server/lib/` → 项目根），移动构建目录时这里要跟着改
- 数据库单文件 `data/fishvn.db`，惰性 `getDb()` 进程内单例；表结构演进**只写在 `ensureSchema()`**，保持幂等

### 新增接口检查清单

- [ ] 路由在 `routes/` 独立文件，`server.ts` 只多一行挂载
- [ ] 无 `any`，常量/路径没有散落
- [ ] 跑过 `npm run typecheck` + `npm test`
- [ ] （涉及前端联调）vite 代理已覆盖，或直接请求 `/api/...` 验证过

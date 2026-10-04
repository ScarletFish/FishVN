# FishVN

Galgame 本地游戏管理库。本地为主，云端（VNDB）为主要元数据来源。
Svelte 5 SPA + Node.js HTTP server + Tauri v2 壳的桌面应用

## Workflow

| 场景 | 行为 |
|------|------|
| 功能需求 / 非平凡改动 | 先出需求与方案，用户过目同意后再实现 |
| 设计讨论（"怎么实现"/"哪个方案好"） | 先讨论定稿，不动代码 |
| 明确的小 bug（已知位置 + 已知改法） | 直接修，修完报告 |
| 微调（文案、样式等小改动，无逻辑变更，<30 行） | 直接执行，改完报告 |
| **改前端样式/UI** | **读 `docs/dev/frontend.md` 必读章节** |
| **改后端 / 新增接口** | **读 `docs/dev/backend.md`** |

**硬闸门：需求与方案未经用户过目确认，禁止进入实现。沉默、"随便"、"你决定"均视为未确认。**

改完跑对应验证：前端 `npm run check:frontend`；后端 `npm run typecheck` + `npm test`。
只在用户明确要求时 git 提交。

## 项目原则

1. **只列正向**：文档只陈述「是什么、要什么」；已否决的东西不进文档。豁免：纪律性的「禁止」（如禁止写死值）
2. **YAGNI**：不为尚未发生的需求预先写代码、留接口、写文档；等真实场景出现再做。

## 技术栈

- **前端**: Svelte 5+ Vite；样式为纯 CSS——手写 token（`tokens.css`）+ Svelte scoped style + 普通 CSS 类
- **后端**: Node server + **Hono**（`hono` + `@hono/node-server`）；TypeScript strict，禁止 `any`
- **数据库**: better-sqlite3（单文件，迁移走 `ensureSchema()`）
- **桌面壳**: Tauri v2；Rust 侧只负责启动外部命令
- **元数据源**: VNDB API（唯一）

## Commands

| 命令 | 说明 |
|------|------|
| `npm run dev` | 同时起前后端（concurrently）：Vite 5173 + 后端 4545 |
| `npm run dev:frontend` | 只起 Vite dev，`http://127.0.0.1:5173` |
| `npm run dev:server` | 后端 `tsc` 编译并起服，`127.0.0.1:4545` |
| `npm run typecheck` | 后端 TS strict 检查 |
| `npm test` | 后端测试（先 tsc 再 node --test） |
| `npm run check:frontend` | 前端：JS 语法 + CSS token + 构建 `dist/web` |
| `npm run build` | 前端生产构建 → `dist/web` |

## 注释与文档

1. **不写历史改动**：不留「何时/谁/改了什么」，历史归 git
2. **只在需要时写**：代码读得懂的零注释；只写特例、踩坑、非显而易见的决策
3. **中文注释**：注释与文档用中文

## Git 提交

- **标题（英文）**: `<type>: <英文简述>`，type 用约定式提交词汇 —— `feat` / `fix` / `refactor` / `docs` / `style` / `chore` / `perf` / `test`
- **正文（中文）**: 逐条说明改了什么，每行以 `- ` 开头；一行标题能说清的小改动可省略正文

```
feat: add game scan endpoint

- 新增 /api/scan 路由，扫描指定目录下的游戏文件夹
- 扫描结果写入 scanned-tree.json
- 补充路由冒烟测试
```

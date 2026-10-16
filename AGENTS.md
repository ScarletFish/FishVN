# FishVN

Galgame 本地游戏管理库。本地为主，云端（VNDB）为主要元数据来源。
Svelte 5 SPA + Rust 后端的 Tauri v2 桌面应用

## Workflow

| 场景 | 行为 |
|------|------|
| 功能需求 / 非平凡改动 | 先出需求与方案，用户过目同意后再实现 |
| 设计讨论（"怎么实现"/"哪个方案好"） | 先讨论定稿，不动代码 |
| 明确的小 bug（已知位置 + 已知改法） | 直接修，修完报告 |
| 微调（文案、样式等小改动，无逻辑变更，<30 行） | 直接执行，改完报告 |
| 开发中跑测试 | 只跑改动范围对应的套件/文件（见 Commands 的粒度命令），全量留到收尾或提交前 |
| **改前端样式/UI** | **读 `docs/dev/frontend.md` 必读章节** |
| **改后端 / 新增接口** | **读 `docs/dev/backend.md`** |

**硬闸门：需求与方案未经用户过目确认，禁止进入实现。沉默、"随便"、"你决定"均视为未确认。**

改完跑对应验证：开发中按改动范围跑粒度命令，一个改动收尾或提交前跑全量 —— 前端 `npm run check:frontend`；后端 `npm run check:rust` + `npm test`。跨模块影响只有全量能暴露。
只在用户明确要求时 git 提交。

## 项目原则

1. **只列正向**：文档只陈述「是什么、要什么」；已否决的东西不进文档。豁免：纪律性的「禁止」（如禁止写死值）
2. **YAGNI**：不为尚未发生的需求预先写代码、留接口、写文档；等真实场景出现再做。

## 技术栈

- **前端**: Svelte 5+ Vite；样式为纯 CSS——手写 token（`tokens.css`）+ Svelte scoped style + 普通 CSS 类
- **后端**: Rust crate `src-tauri`，`#[tauri::command]` 转发到领域函数
- **数据库**: rusqlite（单文件，建表与迁移走 `ensure_schema()`）
- **桌面壳**: Tauri v2；前后端同一进程
- **元数据源**: VNDB API（唯一）

## Commands

| 命令 | 说明 |
|------|------|
| `npm run dev` | `tauri dev`：起 Vite 4545 并拉起桌面窗口 |
| `npm run dev:frontend` | 只起 Vite 4545；`tauri dev` 通过 `beforeDevCommand` 调它，浏览器里单独跑没有 IPC |
| `npm run check:rust` | 后端 `cargo clippy --all-targets`（前面串上注释检查） |
| `npm test` | 后端 `cargo test` 全量套件（`--quiet`：每套件一行结果，失败才展开） |
| `cargo test --manifest-path src-tauri/Cargo.toml --test <套件>` | 后端只跑一个集成套件（编译面最小） |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib <模块路径>` | 后端只跑 lib 内单测，可用模块或用例名前缀收窄 |
| `npm test -w frontend -- <测试文件>` | 前端只跑一个测试文件；再加 `-t <用例名>` 收窄到一条 |
| `npm run test:watch -w frontend` | 前端测试监听模式 |
| `npm run check:comments` | 注释白名单检查：命中白名单外的注释就失败（已串在 `check:rust` 与 `check:frontend` 最前面） |
| `npm run check:frontend` | 前端全量：注释检查 + CSS token 检查 + 无用 CSS 检查 + 测试 + 构建 `frontend/dist` |
| `npm run build` | 前端生产构建 → `frontend/dist` |

限定目标要直接调 cargo：`npm test -- --test <套件>` 会被 npm 当成自己的未知 flag 拦掉。

## 注释与文档

**代码里不写注释。** 信息要么在代码里读得出来，要么写进文档；注释不是它的第三个去处。

- 不靠自觉：`npm run check:comments` 强制白名单（已串在 `check:rust` 与 `check:frontend` 上），例外只有测试用例的意图、`tokens.css` 的 token 用途、两条不可测的残渣、工具指令——清单与理由见 [docs/dev/comments.md](docs/dev/comments.md)
- 产品与设计决策进 `docs/fdd-game-library.md`；栈特有事实、惯例与踩过的坑进 `docs/dev/*`
- 改哪块先读哪份：`docs/dev/backend.md` 与 `docs/dev/frontend.md` 各有一张路由表
- 文档与注释用中文

## Git 提交

- **标题（中文）**: `<type>: <中文简述>`，type 用约定式提交词汇 —— `feat` / `fix` / `refactor` / `docs` / `style` / `chore` / `perf` / `test`
- **正文（中文）**: 逐条说明改了什么，每行以 `- ` 开头；一行标题能说清的小改动可省略正文
- **提交前**：审 `git diff` 里新增与改动行的注释，只审这部分

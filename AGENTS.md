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
| 开发中跑测试 | 只跑改动范围对应的套件/文件（见 Commands 的粒度命令），全量条件见下 |
| **改前端样式/UI** | **先看 `docs/dev/frontend.md` 的文档索引，再读对应分册** |
| **新增前端函数、组件或 store** | **先按 `docs/dev/frontend.md`「先找后写」查现成资产，再决定复用、扩展还是新建** |
| **改标签展示 / 过滤规则** | **读 `docs/dev/tags.md`** |
| **改动落哪一层 / 加字段 / 改表 / 依赖方向** | **先读 `docs/dev/architecture.md`** |
| **改后端 / 新增接口** | **读 `docs/dev/backend.md`** |

**硬闸门：需求与方案未经用户过目确认，禁止进入实现。沉默、"随便"、"你决定"均视为未确认。**

改完按改动面分层验证，纯文档改动不跑测试与构建：

| 改了什么 | 验证 |
|---|---|
| 前端源码 / CSS / 生成物 | 对应粒度用例；收尾或提交前 `npm run check:frontend` |
| 后端源码 / spec / 构建脚本 | 对应套件；收尾或提交前 `npm run check:rust` + `npm test` |
| 打包与脚本配置 | 只验配置自身（`npx tauri build --no-bundle`、`cargo check` 或改到的 script） |
| 只有文档、许可文本、忽略规则 | `git diff --check` + 引用校对 |

全量只在源码批次收尾或提交前跑，跨模块影响靠它暴露；`check:generated` 只在生成物或 spec 变动时跑。
只在用户明确要求时 git 提交。

## 项目原则

1. **只列正向**：文档只陈述「是什么、要什么」；已否决的东西不进文档。豁免：纪律性的「禁止」（如禁止写死值）
2. **YAGNI**：不为尚未发生的需求预先写代码、留接口、写文档；等真实场景出现再做。同一段逻辑第二次出现即真实场景，按 `docs/dev/frontend.md`「重复时是否抽象」判断。

## 技术栈

- **前端**: Svelte 5+ Vite；样式为纯 CSS——手写 token（`tokens.css`）+ Svelte scoped style + 普通 CSS 类
- **后端**: Rust crate `src-tauri`，`#[tauri::command]` 转发到领域函数
- **数据库**: rusqlite（单文件，建表走 `ensure_schema()`）
- **桌面壳**: Tauri v2；前后端同一进程
- **元数据源**: VNDB API（唯一）
- **构建档位**: 日常与发版两档，发版档另有优化处理；构建统一走 `start.bat`。

## Commands

| 跑什么 | 命令 |
|---|---|
| 后端一个集成套件 | `cargo test --manifest-path src-tauri/Cargo.toml --test <套件>` |
| 后端 lib 内单测（模块或用例名前缀可收窄） | `cargo test --manifest-path src-tauri/Cargo.toml --lib <模块路径>` |
| 前端一个测试文件（加 `-t <用例名>` 收窄到一条） | `npm test -w frontend -- <测试文件>` |

`npm test -- --test <套件>` 会被 npm 当成未知 flag 拦掉，限定目标直接调 cargo。`launch` 套件默认跳过，改动启动 / 会话链路时手动跑 `npm run test:launch`。`npm run dev:frontend` 只起 Vite，浏览器里单独跑没有 IPC。

## 注释与文档

**代码里不写注释**，例外清单见 [comments.md](docs/dev/comments.md)；`check:comments` 挂在 `check:rust` 与 `check:frontend` 上。

- 文档只写三类：决策与规格、不写会改坏的约束与坑、路由指针。打开源码 30 秒能看出来的不写。
- 产品与设计决策进 `docs/fdd-game-library.md`；栈特有事实与惯例进 `docs/dev/*`。
- **往 `docs/` 增行前先列条目过目**，点头才落盘。
- 改哪块先读哪份：`docs/dev/architecture.md`（分层与不变量）、`docs/dev/backend.md` 的路由表、`docs/dev/frontend.md` 的文档索引。
- 文档与注释用中文。

## Git 提交

- **标题（中文）**: `<type>: <中文简述>`，type 用约定式提交词汇 —— `feat` / `fix` / `refactor` / `docs` / `style` / `chore` / `perf` / `test`
- **正文（中文）**: 逐条说明改了什么，每行以 `- ` 开头；一行标题能说清的小改动可省略正文
- **提交前**：审 `git diff` 里新增与改动行的注释，只审这部分

# 注释规范 — Comments

**代码里不写注释。** 读得出来的东西不必解释；读不出来的东西属于文档，不属于代码旁边。

不靠自觉：`npm run check:comments` 扫全仓源码，命中白名单外的注释就失败。它已挂在 `check:rust` 与 `check:frontend` 上，所以常规验证顺手就查了。

## 例外：白名单

白名单在 `scripts/check-comments.mjs` 里，一共三类：

1. **测试文件的意图注释** —— 用例在复现或锁定什么行为，是测试规格本身。整个 `src-tauri/tests/**` 与 `frontend/src/**/*.test.js` 放开
2. **`tokens.css` 的 token 用途注释** —— 设计系统的一部分
3. **工具指令** —— `svelte-ignore` / `@vitest-environment` / `eslint-disable` / `prettier-ignore` / `@ts-*`，它们是被工具读的开关，不是注释

加例外就是改白名单，并在上面写清理由。

## 知识去哪

| 内容 | 去处 |
|---|---|
| 产品与展示决策（要做到什么、为什么这样做） | `docs/fdd-game-library.md` |
| 后端栈特有事实、数据模型、上游实测 | `docs/dev/backend.md`、`docs/dev/vndb-api.md` |
| 前端惯例、样式坑集、布局契约、构建配置 | 从 `docs/dev/frontend.md` 的索引进各分册；卡宽与列数见 `docs/dev/grid-layout.md` |
| 视图的数据契约与字段归属 | `docs/dev/view-contracts.md` |
| 界面文案的用词、量词、标点与语气 | `docs/dev/wording.md` |

改哪块先读哪份：`backend.md` 与 `frontend.md` 各有一张路由表。

## 写新代码时

不写注释，也不为「以后可能有用」留占位 —— 踩到的坑记进上表对应的文档，写在那儿。回答「这行为什么这么写」时去看文档，不要靠代码旁的一句话。

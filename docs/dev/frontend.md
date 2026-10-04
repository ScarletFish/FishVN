# 前端开发规范 — Frontend

### 样式的三种形态

1. **Token**（`css/tokens.css`）— 颜色/间距/圆角/字体的唯一来源
2. **Svelte scoped style** — 组件私有样式
3. **普通 CSS 类**（`components/`、`views/`）— 跨组件复用的模式

归属判断：组件独有 → scoped；可能复用 → components/；视图专属 → views/；是变量 → tokens.css。

### Token 引用规则

- 颜色/间距/圆角/字重一律引用 token，**禁止写死值**
- token 单点定义在 `tokens.css`，禁止在别处重新声明

### "先找后写"三步协议

新增任何 CSS 前：

1. 查 `tokens.css` 有没有现成 token
2. 查 `components/` 有没有现成类
3. 都没有才新写，写进正确归属文件

完成后跑 `npm run check:frontend` 验证。

### 自动检查

`npm run check:frontend` = JS 语法 + CSS token 检查 + 构建 dist/web/。
改完前端必跑，勿只跑其中一条，否则 dist 过期「改了没生效」。

## 架构概要

### CSS 文件结构

```
src/css/
├── styles.css     # 仅入口，只做 @import，禁止写样式
├── tokens.css     # 设计 token 单点（迁移待决）
├── views/         # 视图专属 <view>.css
├── components/    # 跨组件复用类
└── layouts/       # 主题/布局特有
```


## 新增 UI 组件检查清单

- [ ] 颜色/间距全部来自 token，无写死值
- [ ] 样式归属正确文件（scoped / components / views）
- [ ] 复用优先：先查过已有组件类
- [ ] 跑过 `npm run check:frontend`


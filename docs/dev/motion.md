# 动画 — Motion

GSAP 与 ScrollTrigger 以 vendor 全局脚本加载（`public/vendor/gsap/`，`index.html` 在模块脚本之前引入）。动画代码全程判空 `globalThis.gsap`，脚本缺失时界面照常渲染，动画只是增强层；`prefers-reduced-motion: reduce` 时入场动画直接跳过。

滚动入场（`lib/scroll-anim.js`）：区块 fade+rise、卡片按批 stagger，触发线在视口 92% 处、`once` 自毁；scroller 固定 `.shell__main`，它也是页面的唯一滚动容器。使用契约：视图持有一个实例，用数据签名 gate 驱动——`$effect` 里比较签名，变化后才 `tick().then(build)`，`build` 先 `kill` 旧 trigger 并清内联样式（防残留 `opacity: 0`），视图销毁时调 `kill()`。动画只动 transform 与 opacity。

主题与强调色切换经 `document.startViewTransition` 包裹（`theme.js` 的 `setTheme`/`setAccent`），整窗交叉淡入；启动同步（`syncTheme`/`syncAccent`）不走过渡。API 缺失或 `prefers-reduced-motion` 时直接生效。跟随系统下的系统切换不套过渡，避免用户在别的应用里换主题时本窗跟着做一次交叉淡入。

封面 → 详情共享元素过渡（`lib/hero-transition.js`）：点击卡片时 `captureHero` 记位置与已解码地址，详情挂载后 `playHero` 接管。hero 的启动不依赖封面 URL 解析——`coverSrc` 走 IPC 异步返回，等它就永远赶不上首帧；换手要等目标 `<img>` **出现并加载完成**再瞬间替换，异步期间由 hero 兜住视觉。无封面、GSAP 缺失或 `prefers-reduced-motion` 时跳过，详情瞬时打开。

详情页分波入场（`detail.css` 的 `detail-rise`/`detail-fade`）：CSS animation + `animation-fill-mode: backwards`，隐藏态由首帧关键帧兜住——不需要 show 类、不需要双 rAF，动画随元素插入自动启动，天然适配异步分区。波次自上而下：氛围底/操作 → 信息栏逐条 → 角色区 → 制作区，封面不参与（归 hero 过渡接管）；逐条延迟写在行内 `animation-delay`，属编排参数、不适用 token 规则。判据：**多图分区（角色/剧集式）直接加载、不做逐卡编排**——编排延迟与图片加载时钟不同步，且卡片是点击锚点应尽早可交互。新增详情分区挂 `detail-rise` 并给波次延迟；`prefers-reduced-motion` 全量豁免。

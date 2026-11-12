# 样式坑集 — Styles

改样式前扫一眼这里，每一条都对应过一次「改完就坏」。

## 导入顺序

- `styles.css` 的 `@import` 顺序就是层叠顺序：后导入文件里同等特异性的规则覆盖先导入的。给先导入体系的类（tag 胶囊这类）写变体时，用「基类 + 变体」双类前缀锚定（如 `.tag-pill--pick.sync-chip`），单类变体会被后导入体系里的同名规则按源序冲掉——症状是「改完不报错、视觉没变化」

## 卡片与浮层

- `:hover` 的 `transform` 会生成层叠上下文，菜单展开时整卡抬到相邻卡之上
- 承载弹出菜单的区块要压得过它后面的兄弟：`.detail-banner` 与 `.detail-chars` 同为 `z-index: 1` 时，DOM 靠后的角色区盖住横幅里弹出的 `.menu__list` —— 菜单自己的 `--z-dropdown` 出不了父级的层叠上下文
- 可见表面与裁切都归 `.cover-card__frame`，菜单是它的兄弟而非子元素才逃得出裁切；卡片 `overflow: hidden` 同时承担浮层的底角，不能去掉
- **评分两枚胶囊同一时刻只出一枚**：我的评分 `.cover-card__score--mine` 常驻右下，挂在 `.cover-card__frame` 外——要逃出裁切，也才能在悬停时上移让开菜单；社区评分 `.cover-card__score--vndb` 放在 overlay 里、标题上方。有社区分时用 `:has()` 把 mine 隐掉，不引 JS 状态：同一条规则要同时覆盖 `:hover` 与 `:focus-within`，写两遍选择器比维护一个 hovered 变量更不容易漏
- **社区胶囊按 hover 独立显隐，不靠 overlay 的淡入**：它若跟着 overlay 走，开着「标题常显」时 overlay 常亮，两枚评分就会同时出现。现在它自己吃 `opacity` + `visibility`，只在 `:hover` / `:focus-within` 时可见；`visibility` 而非 `display`，是为了留出那一行的空间，否则悬停时 overlay 向上长一截、遮罩跟着跳
- **标题常显开关写在根元素上**：`html[data-card-title='on']` 由 `lib/card-prefs.js` 同步，卡片不接 props——它在首页与「我的列表」两处复用，走 props 会让每个调用点都要传一遍
- **标题固定一行省略**（`nowrap` + `text-overflow`），不按悬停状态改行数：两行会让卡片底部在进出悬停时跳高度。overlay 的 `padding-right` 要把菜单宽度与评分胶囊宽度一起算进去，否则长标题会钻到胶囊底下

## 表单分组

- `.form-group + .form-group` 自带一条分割线，线上 20px、线下 16px：线下的内边距不能只给 4px，否则线贴住下一块的标签，读作标签的镶边而不是分隔；下一块的首元素若是自带边框的卡片（`.form-row`），线与卡片边框仍会并排成双线，这类 group 加 `.form-group--flat` 去掉线，靠段间距分组
- **逻辑成组**：同属一个功能的若干项（如「设置项 + 动作 + 列表」）要视作一组、组间不留分割线时，保持首项为普通 `.form-group`，其余项加 `.form-group--flat`；VNDB 标签页的「同步心愿单 / 同步评分」两个开关行即此写法，设置页「数据库」标签页的「保留策略 / 创建备份 / 备份列表」也照此合并。改一组的视觉边界时，只动首项之后的 `--flat`，别给首项也加（那样会和上一组再隔开）
- **动作行**：组内若某项是「标题 + 描述 + 控件/按钮」的单行卡片，用 `.form-row`——左侧放 `.form-row-text`（`.form-row-title` 标题 + `.form-row-desc` 描述，文本块 `flex: 1` 把控件顶到右侧），右侧放控件或按钮。开关行共用这层卡片，但文字用 `.form-row-label` 放在左端、开关收在右端（同样靠 `flex: 1` 顶过去），与同栏动作按钮落成一条竖线；安装列表那种不带卡片的紧凑标志位是例外，开关反而在行首排成一列，别把这条规则套过去。新增这种行不要手搓 flex + 内联 style，统一走 `.form-row` 家族
- **下拉**：设置里的选择一律用 `SelectDd`，不要写原生 `<select>`（入口与用法见 [components.md](./components.md)「SelectDd」）

## 详情页氛围底

- 上内边距归零：氛围底 `absolute` 在 `.detail` 的 padding box 上，`padding-top` 会把它推下去
- 内容起点由 `padding-top` 决定，氛围底的高度与 mask 停点只决定它画到哪：上移内容不改变覆盖范围，但栏体可能落进实色区而不是淡出段，改这里要同时看 `.detail-info-line` 的落位
- 氛围底必须是 `.detail` 的直接子元素才能左右满幅，挪进 `.detail-banner` 会各留一条空白边
- mask 必须分段：压回两站时收口线附近仍接近不透明，底图会被切出硬边；实色边界 = 高度 × 停点%，改高度后要保持边界不变就得同步停点
- 模糊层 `scale 1.3` 是为了盖住 blur 向外溢出的透明边，否则四边露出未模糊的原图边缘；氛围底的中间停点与 mask 停点同高，改一处另一处要同步
- 详情页封面与列表卡封面（`.cover-card` / `.cover-card__frame`）的尺寸、圆角、占位要一起改
- 信息栏的边框是承重结构（底色 55% 不透明度且浮在氛围底的淡出段上），去掉边框栏体就塌；背后必须有 `.detail-ambience`，删掉它这层几乎不可见

## 标签与胶囊

- 胶囊几何共用、配色按角色分开；tag 走中性色是刻意的 —— 一屏几十个 tag 全用 accent 会稀释强调色也压过标题，accent 留给信息栏里的评分与信息项
- `.pill--more` 落在 `<button>` 上：它是靠 `font-family: inherit` 拿字体的（见下「字体继承」），边框也要显式清掉，去掉这两行会露出系统默认样式
- 简介靠固定行数夹住，依赖 `-webkit-box` 三件套，`-webkit-box-orient` 缺了 ellipsis 不出现
- 属性用流式键值对（值都很短，表格浪费竖向空间）；trait 分组保留表格式布局：标题列定宽，标签列吃剩余宽度；简介是连续散文，铺满整宽时一行长到读不回行首
- 角色卡与所在 section 底色同档会让边界糊掉，卡片要高一档
- vn-merge 的角色行标签列装的是名字，比字段名长，栅格与作品级一致、靠折行消化宽度

## 控件

- `字体继承`：原生 `button` / `input` / `select` / `textarea` **不继承** `font`（含 `font-family` / `font-size` / `font-weight`），UA 会给一套系统默认字体与尺寸，与页面其余 `--font-sans` 文字错位（标题栏返回按钮当初就因此与标题差 ~2px）。`tokens.css` 的全局 reset 已用标准写法 `font: inherit` 对这几类强制沿用父级字体与尺寸，从根上消除整类隐患；组件里仍可用 `font-family: var(--font-sans)` 显式写死（特异性更高，优先于全局规则）。依赖 `inherit` 拿到字体的按钮（如 `.pill--more`）删掉 `font-family: inherit` 会回落系统字体、露出默认样式
- `视觉对齐`：控件「看着不对齐 / 文字像浮起」，先查是不是漏了字体或字号声明（原生控件不继承，见上「字体继承」），不要上 `translate` / `margin` 之类的魔法偏移去补。本项目的「1px 上浮」当初就是被误读的字体错位——字体一对齐，几何上已经居中，这类魔法数字没有几何依据、后人看不懂也不敢动，应删掉；像素级补偿只作为带注释的最后手段。唯一持例是与文字同行的状态点 `translateY(-1px)`：CJK 墨迹在行框内比几何中心高约 1px（可变版 Noto Sans SC 实测），圆点不补偿会显得压底线；判据是有实测依据、且只作用于与文字同行的装饰元素，其余场景仍是先查字体
- `toggle`：hidden input 不能 `display: none`（会退出 Tab 序与无障碍树），只能压成 0 尺寸；滑块必须是它的紧邻兄弟，中间隔任何元素都会断开相邻选择器；input 是 0 尺寸，全局 `outline` 会画在看不见的地方，焦点环要转到滑块上；药丸小且边框淡，外描才看得出离开轨道
- `tags`：输入框必须在容器内、跟在芯片后面（放到容器外 placeholder 会把容器撑宽），它吃掉剩余宽度，最小宽度保证空标签时也能点进去打字
- `forms`：`:focus-visible` 要清 `outline`，否则键盘 Tab 会套上全局那圈重环；数量用文本输入框而非 `number`（原生微调与页面滚动、方向键冲突）
- `键入清洗`：受控输入在 `oninput` 里过滤字符时，状态和框里的字会分叉——清洗结果与上一次状态相同（已是 `12` 又敲了个字母），Svelte 判等后不重绘，脏字符就留在框里了。清洗结果与原文不同时直接写回 input 元素，别只靠状态更新
- `dropdowns`：开合用 `is-open` 类切换、不做 portal —— 菜单与触发器同处一个层叠上下文，走 portal 会与弹窗遮罩的 `z-index` 打架
- `menus`：菜单留在触发器的层叠上下文里，不移出 DOM；裸图标变体浮在内容图上，必须自带底色
- `sidebar`：`.active:hover` 必须与 `.active` 一起清，否则悬停时半透明底色会回来
- `buttons`：`.btn-primary` 常态纯色 `--accent`、hover 换 `--accent-soft`（扁平风，不用渐变与 brightness 滤镜）；覆盖 `.btn:hover` 的规则与它特异性相同，要逐项写全才不会漏掉被按顺序冲掉的属性

## 测量

- `裁切测量`：`-webkit-line-clamp` 裁掉的不是溢出内容而是布局片段，被裁元素 `scrollHeight` 与 `clientHeight` 相等，判不出「有没有被截断」。只能临时解除裁切、量一次自然高度再还原；解除要用与展开态**同一个**类（`line-clamp: none` 还得配 `overflow: visible`），只改内联 `line-clamp` 不保证等效。还原在同一帧内完成，不会闪。
- `宽度晚定`：元素宽度会因兄弟内容晚到而变（封面图加载完、右栏才收窄），`window` 的 `resize` 收不到这类变化，依赖宽度的测量要挂在元素自己的 `ResizeObserver` 上；解除裁切也会改自身尺寸，回调里按 `clientWidth` 去重，否则自己触发自己。

## 表面层级与交互态

表面层级只连 L2 的 `--bg-*` token，用途对应：

| 用途 | token |
|---|---|
| 页面底 / 侧栏 | `--bg-page` / `--bg-modal` |
| 常规表面 / 卡片 / 输入框底 | `--bg-surface` |
| 抬起的块（卡片常态、浮层锚） | `--bg-raised` |
| 浮层（菜单 / 下拉 / 弹窗内容） | `--bg-floating` |
| hover / 按下蒙版（压暗） | `--bg-hover` |
| 选中（accent 身份） | `--bg-selected` + `--accent-border` |
| 段控选中格 | `--bg-seg-thumb` + `--seg-thumb-shadow` |

交互态写法：
- hover / active 用 `--bg-hover` 蒙版压暗，或只强化 `--border-strong` 边框。
- 浮层（菜单、下拉、弹窗）用 `--bg-floating`，取 ramp 最浅一档。
- 选中用 `--bg-selected` 淡染 + `--accent-border` 描边表达身份。
- 表单字段（输入框、标签输入框等）hover 只动边框，`focus` / `:focus-within` 才升 `--bg-raised` + `--shadow-sm`。
- 下拉 / 菜单：触发器（按钮）走 `--bg-hover` 压暗，弹出菜单走 `--bg-floating`。
- 标签页选中指示用 accent 下划线。段控的选中是取值不是身份，不吃 accent；accent 留给侧栏 active、焦点环与 VNDB 选中卡。段控容器不上色 —— 与周围同层，选中格吃 `--bg-seg-thumb` 并投 `--seg-thumb-shadow` 浮起，悬停格走 `--bg-hover`。两套主题的差别只在明度空间：深色底到选中格有 12 个 L\*，浅色只有 3，所以浅色靠阴影补、深色靠明度本身且阴影取 `none`（`--shadow-sm` 在深色是浮层用的 50% 黑，套到一条矮控件上会糊出一圈黑边）。别把容器压暗去做「槽」：那会让整条控件比周围低一层，读作凹槽；也别给选中格加描边，容器已有一圈边框，格内再来一道只有右边会露出来。
- 已添加的「内容」类胶囊（tag 芯片）恒定背景用 `--bg-hover`，比容器深一档，hover 时仍保持边界（与只读 `.pill--tag` 同款）。
- **可点击元素的 hover 分两套，按形态选**：文本流型（正文外链、角色声优名、文字按钮）用下划线——恒有下划线时由 35% 转 `currentColor`，静态不带的用 `text-decoration-color: transparent` 起手、hover 转实，两者都不改布局；块型（标签、特征、`+N` 药丸）用 `--bg-hover-strong` 底 + 字色升 `--fg-default`。别让正文里的链接只靠字色深浅区分，也别给行内文字加底色块。

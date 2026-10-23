# VNDB Kana API v2 — 响应格式

`https://api.vndb.org/kana`，沙箱 `https://beta.vndb.org/api/kana`。本文只记实测到的响应格式与
约束，供 `src-tauri/src/vndb/dto.rs` 反序列化时对照。

抽样口径：2026-10-09 抓取 400 条全字段 VN（`id > v25000` 与 `id > v17000` 各 200 条，25 条一页）
做逐字段统计，另单独抓搜索、TBA、未发售样本。

### 请求

POST JSON body，`Content-Type: application/json`。响应恒为 JSON，`Accept` 头不切换格式。
顶层 `id` 永远返回，不必写进 `fields`。

| 成员 | 类型 | 说明 |
|---|---|---|
| `filters` | array \| string | 三元素 `["字段","运算符","值"]`，或站点上的 compact 字符串 |
| `fields` | string | 逗号分隔；嵌套对象写点路径 `image.url`；无通配，`*` 会 400 |
| `sort` | string | 见下方 sort 清单 |
| `reverse` | bool | 排序取反 |
| `results` | int | 每页条数，上限 **100** |
| `page` | int | 从 1 开始 |
| `count` | bool | 附带总条数，官方注明有明显性能开销 |
| `user`, `token` | string | 鉴权用：`token` 走 `Authorization: token <t>` 请求头，只有 `/ulist` 等用户列表端点需要；`user` 认证后默认本人，省略即可 |

### 响应信封

```json
{ "more": false, "results": [ ... ], "count": 1234 }
```

`more` 表示还有下一页，翻页只需要它。`count` 仅在请求里带了 `count:true` 时出现。

### 客户端只取两档

`vndb.rs` 里只有两套 `fields`：

| 档 | 用途 | 特点 |
|---|---|---|
| `SEARCH_FIELDS` | 搜索结果，判断是哪部作品 | 不取 `description` 与 `tags`；`image` 只要 `id,thumbnail,thumbnail_dims`，所以 `url` 与 `dims` 恒为 `None` |
| `META_FIELDS` | 详情页真正渲染的字段 | `image{id,url,dims}`，封面用 `url` |

除 `id` 与 `title` 外一律按 `Option` 反序列化：null 率高（`rating` 238/400、`alttitle` 140/400，
`image` 罕见但确有 null），详见下表。

### `/vn` 字段清单

`GET /kana/schema` 的 `api_fields["/vn"]` 递归展开共 55 项，本项目用得到的部分：

| 字段 | JSON 形状 | null 实测 (n=400) | 空数组 | 说明 |
|---|---|---|---|---|
| `id` | string | 0 | — | `"v14"` |
| `title` | string | 0 | — | 站点主标题：日文原作为罗马字，中文原作为中文 |
| `alttitle` | string | 140 | — | 与主标题互补的另一种写法（日文原作是原文，中文原作常为罗马字），两种写法一致时为 null |
| `olang` | string | 0 | — | 原作语言，`zh-Hans` / `ja` / `en` … |
| `released` | string | 5 | — | 五种形状，见下节 |
| `devstatus` | int | 0 | — | 0 已完成 / 1 开发中 / 2 已取消 |
| `rating` | float | **238** | — | 10..100 贝叶斯分，**票数 < 5 时为 null** |
| `average` | float | 121 | — | 10..100 原始均分，零投票时为 null |
| `votecount` | int | 0 | — | |
| `popularity` | float | 0 | — | 0..51 量级的热度值 |
| `length` | int | 148 | — | 1..5，仅在投票不足时兜底 |
| `length_minutes` | int | 277 | — | |
| `length_votes` | int | 0 | — | 无人投票时是 `0`，不是 null |
| `description` | string | 82 | — | 含 BBCode，如 `[url=…]…[/url]` |
| `languages` | string[] | 0 | 5 | 不含机翻 |
| `platforms` | string[] | 0 | 5 | `win` / `lin` / `mac` / `web` / 主机 … |
| `titles` | object[] | 0 | **0** | 每语言一条，至少一条 |
| `staff` | object[] | 0 | 2 | 每条 `{role,name,original,note}`；同一人多行（职能或备注不同），`role` 取 `enums.staff_role`（scenario / director / chardesign / art / music / songs / translator / editor / qa / staff） |
| `developers` | object[] | 0 | 0 | 每条 `{id,name,original,type}` |
| `aliases` | string[] | 0 | 265 | |
| `tags` | object[] | 0 | 2 | |
| `image` | object | **1** | — | 罕见但确有 null（样本 `v25031`） |

其余字段（`screenshots` / `va` / `relations` / `editions` / `extlinks`）
本项目不取。

裁剪字段的收益（每页 25 条实测）：

| fields | 响应体积 | 每条 |
|---|---|---|
| 摘要档 13 项 | 8.5 KB | 342 B |
| 摘要档 + `description` | 20 KB | 805 B |
| 全字段 55 项 | 110 KB | 4.4 KB |

### 字段简写

嵌套字段可以用括号写在一起，`image{id,url}` 等价于 `image.id,image.url`。请求体短一大截，
字段多的场景优先用这个写法。

### 响应头没有任何限流信息

实测 200 响应的头只有 `Server` / `Date` / `Content-Type` / `Content-Length` / `Connection` /
`Strict-Transport-Security`：没有 `X-RateLimit-*`，没有 `RateLimit-*`，没有 `Retry-After`。
429 是否带 `Retry-After` 未实测（触发限流要真发 200 次请求，不值得）。

余量查不到，客户端只能自己按低于服务端的口径节流。

### `search` 传数组会静默失效

`search` 是多值过滤器，写 `["search","=",["clannad","air"]]` **不报错**，而是返回按 id 排序的
无条件结果（v1、v2、v3…）。比报错更危险：请求看起来成功了，最相关的作品却排不到前面。
`search` 只能写字符串值。

### `sort:"searchrank"` 要求顶层过滤器就是 `search`

顶层换成 `or` 组合或别的过滤器时带 `sort:"searchrank"` 会 400。搜索请求的形状不要自由拼。

### 单请求超时

服务端的 3 秒线只约束查询执行时间，网络传输不计入。客户端总超时要覆盖「服务端执行 +
传输」两段：代理链路下 106 KB 的角色响应实测传输就到 10 秒，照抄 3 秒会在读体中途
被掐断，reqwest 把这种超时包成「error decoding response body」。客户端总超时设 20 秒；
超时错误经 `is_timeout` 判别，提示文案与连接失败分开。

### 一次请求合并多部作品

`id` 的 flag 只有 `o` 没有 `m`，直接传数组会 400（`Invalid 'id' filter: Invalid value.`）。
用 `or` 组合可行，实测 8 条与 20 条都正常返回：

```json
{ "filters": ["or", ["id", "=", "v11"], ["id", "=", "v17"]],
  "fields": "id,title", "results": 100 }
```

同步一批作品合并成一个请求，而不是每部各发一次。

### 耗时由字段重量决定，不由批量大小决定

同样 20 个 id，只取需要的字段与取全字段：

| 档位 | 用时 | 体积 |
|---|---|---|
| 精简字段 | 1.60 s | 68 KB |
| 全字段（30 项，含 staff/va/screenshots/relations） | 3.95 s | 63 KB |

全字段版本已越过下面的 3 秒中断线。批量大小与耗时无单调关系（实测 10 条 5.3 s、50 条 1.15 s），
客户端无法据此预判，只能靠裁字段压执行时间。

### `/character` 端点

取一部作品的角色走 `/character`。`/vn` 没有返回完整角色列表的字段——它的 `va` 只是配音关系，
只列已配音的角色。作品元数据与角色集因此是两次请求；把元数据塞进角色的 `vns.*` 能凑成一次，
但每个角色都会重复带一份，且 `vns` 是该角色出现过的所有 VN，还得自己按 id 去重。

`vn` 过滤器只接受嵌套查询，写平了是 400 `Invalid 'filters' member: Invalid query`：

```json
{ "filters": ["vn", "=", ["id", "=", "v11"]], "results": 100 }
```

`traits` 的 `name` 与 `group_name` 继承自 `/trait`，`fields` 里不显式写出来只回 trait 的 id：

```
traits{id,name,group_name,spoiler,lie}
```

声优不在 `/character` 上，只存在于 `/vn` 的 `va`，按 `character.id` 挂回角色：

```
va{character{id},staff{id,name,original},note}
```

`note` 是版本标记（如 Realta Nua），同一角色在不同版本下会有多条。出场定位直接由
`vns.role` 给（`main` / `primary` / `side` / `appears`），不必反查角色属于哪部作品。

`gender` 按 release 逐条返回，同一角色多个 release 会让数组出现重复值，取用前去重。
`image` 没有 `thumbnail`，角色图固定限 256×300，缩图要走 `url` 自己派生。

`birthday` 是 `[月, 日]` 两元素整数数组（如 `[12, 15]`），没有值时为 `null`。
`/kana/schema` 对它不给类型声明，与 `released` 同性质，只能靠实测确认。
500 条样本的分布是 265 个 `null`、235 个两元素整数数组，未见标量或字符串。

`results` 同样有上限（100），角色多的作品要翻页。

`va` 里出现而 `/character` 里没有的角色，在本作品里按没有立绘与资料处理；`va` 里缺 `staff`
的条目直接跳过。

`birthday` 的 `[月, 日]` 落库拍平成 `月-日`：只公布一个分量时存已知的那个，两边都没有则该字段
留空。

### `/ulist` 端点（用户列表）

用户自己的 VN 列表，合并了旧 API 的 vnlist / votelist / wishlist。读取必须带 token
（`Authorization: token <t>` 头），`user` 参数认证后默认本人，省略即可。本节形状经官方文档
与多个客户端库交叉确认，未经本项目实测（本地无 token 可抽样）。

请求体与通用查询格式一致，本项目只取四个字段：

```json
{ "fields": "vn.id,labels{id,label},vote,lastmod", "results": 100, "page": 1 }
```

| 字段 | JSON 形状 | 说明 |
|---|---|---|
| `vn` | object | 固定取 `vn.id`，形如 `{"id":"v17"}` |
| `labels` | object[] | 每项 `{id,label}`；`label` 是首字母大写的显示名（`Playing` / `Finished` / `Stalled` / `Dropped` / `Wishlist`），id 小于 10 是预定义标签且对所有用户相同；`Voted`(7) 随 `vote` 自动加减 |
| `vote` | int \| null | 10..100，无评分为 null；与本地 `user_rating` 同为百分制、同刻度 |
| `lastmod` | int | unix 时间戳，该列表条目最后修改时间 |

翻页用 `results`（上限 100）+ `page`，响应 `more` 为 true 就续页；结果默认按 `id` 排序。

写操作（`PATCH /ulist/<id>` 带 `labels_set` / `labels_unset` / `vote`，删条目用 `DELETE`）属
仅上传与双向同步力度，设计见 [vndb-sync.md](./vndb-sync.md)，首版（仅下载）不实现。

### 封面与图片

封面一律下 `url` 不下 `thumbnail`：thumbnail 宽度不足以覆盖 2× 屏（要 554 宽）。

图片在 `t.vndb.org` 这个 CDN 上，与 `api.vndb.org` 的限流额度是两套，客户端另开一个连接
（`IMAGE_TIMEOUT` 8 秒，不受 3 秒中断线约束），并发数只受本机带宽与 CDN 礼貌限速约束。

图编号（VN 封面 `cv…` / 角色 `ch…`）是 VNDB 给每张图的永久身份，换图才换号，可以直接当文件名
判断要不要重下。

`image` 上的 `sexual` / `violence` 内容标记不取：默认全下载，过滤留给设置项，别当成漏做过滤补上。

### 嵌套与数组的元素结构

```jsonc
"image":     { "id": "cv88797", "url": "https://t.vndb.org/cv/97/88797.jpg",
               "thumbnail": "https://t.vndb.org/cv.t/97/88797.jpg",
               "dims": [674, 967], "thumbnail_dims": [256, 367],
               "sexual": 0, "violence": 0, "votecount": 14 }
"titles":    [ { "lang": "ru", "title": "Школьные Дни", "latin": "Shkol'nye Dni",
                 "official": true, "main": true } ]
"tags":      [ { "id": "g23", "name": "Plot Continuity", "category": "cont",
                 "rating": 2.46794867515564, "vn_count": 1204,
                 "spoiler": 0, "lie": false } ]
"relations": [ { "id": "v1193", "relation": "side", "relation_official": true } ]
"editions":  [ { "eid": 0, "lang": "en", "name": "…", "official": true } ]
"staff":     [ { "id": "s11993", "role": "staff", "note": "Sound production", "eid": null } ]
"extlinks":  [ { "id": "enwiki", "label": "Wikipedia (en)", "name": "enwiki", "url": "https://…" } ]
```

`tags.rating` 是 float32 精度噪声，会出现 `2.46794867515564` 这种值，取整显示即可。
`dims` / `thumbnail_dims` 是 `[宽, 高]`。
`tags` 的 `name` / `category` / `vn_count` 随同步整体落库，标签中文显示见 [frontend.md](./frontend.md) 的词典机制。

### `released` 的五种形状

| 形状 | 例 | 含义 |
|---|---|---|
| 完整日期 | `"2005-04-28"` | 绝大多数 |
| 部分日期 | `"2027-06"` | 只公布到月 |
| 年 | `"2027"` | 只公布到年 |
| `TBA` | `"TBA"` | 已定档但未公布日期 |
| null | `null` | 完全未定 |

400 条样本分布：381 / 5 / 9 / 0 / 5。filters 里 `TBA`、`unknown`（等价于 null）、
`today` 都是合法的 `released` 取值，实测可用：

```json
{ "filters": ["released", "=", "TBA"], "fields": "id,released", "results": 3 }
→ {"more":true,"results":[{"id":"v7720","released":"TBA"}, …]}
```

`["released","n","=",null]` 这种写法会 400（`Invalid predicate`），查 null 要用 `unknown`。

### 常用 filters

`id`(可排序) / `search`(多值，命中标题、别名与关联 release 标题) / `lang`(多值) / `olang` /
`platform`(多值) / `length`(可排序) / `released`(可排序、接受 null) / `rating`(可排序、可取反) /
`votecount`(可排序) / `has_description` / `has_anime` / `has_screenshot` / `has_review`
/ `devstatus` / `tag`(多值) / `dtag`(多值) / `anime_id` / `label` /
`release` / `character` / `staff` / `developer`。

标志位追加在过滤器条目上：`o` 可排序比较、`n` 接受 null、`m` 一条匹配多值、`i` 取反不等价。

`image` **不是** `/vn` 的 filter 名，没法按「有无封面」筛。

sort 可选：`id` / `title` / `released` / `rating` / `votecount` / `searchrank`。
搜索要按相关度排必须带 `sort:"searchrank"`。

### 错误

错误 body 是纯文本一行，不是 JSON。**400 意味着我们请求写错了，是 bug。**

| 状态码 | 实测文案 | 场景 |
|---|---|---|
| 400 | `Invalid 'id' filter: Invalid value.` | id 格式非法，如 `v99999999` |
| 400 | `Invalid 'fields' member: Field 'no_such_field' not found` | 字段名拼错 |
| 400 | `Invalid 'fields' member: Field '*' not found` | 试图用通配符 |
| 400 | `Invalid 'results' member.` | results 超 100 |
| 400 | `Invalid 'nope' filter: Unknown field.` | filter 名不存在 |
| 400 | `Invalid 'filters' member: Invalid compact encoded form` | compact 串坏了 |
| 400 | `Invalid 'titles' object requires specifying sub-field(s).` | 嵌套对象没点子字段 |
| 400 | `Sub-field specified for non-object 'official'.` | 对标量写了子字段 |
| 429 | — | 触发限流 |
| 5xx / 502 | — | 上游故障 |

格式合法但不存在的 id（`v9999999`）返回 `200 {"more":false,"results":[]}`，不是 404。

### 限流

200 请求 / 5 分钟，且每分钟 1 秒执行时间；单请求超过 3 秒被服务端中断。
官方文档说这些额度「足够大多数应用」，且余量不通过响应头暴露（见上节），所以客户端节流
要取比这更保守的值：实际按一半取（100 请求 / 5 分钟），图片另算。

### 使用条款

服务免费、非商业用途，best-effort，不保证稳定性。数据许可见
<https://vndb.org/d17#4>：库内信息整体是 ODbL 1.0 + DbCL 1.0；**图片与 VN 简介不在此列**，
原文是「Images, visual novel descriptions and character descriptions are gathered from various
online sources and may be subject to separate license conditions」。本地缓存元数据与缓存图片的
合规性因此不同。

### 复现

```powershell
curl.exe -s -o out.json -w "%{http_code}" -X POST https://api.vndb.org/kana/vn `
  -H "Content-Type: application/json" `
  --data-binary '{"filters":["id","=","v14"],"fields":"id,title,released","results":1}'
```

pwsh 的 `Invoke-WebRequest` 在 4xx 时会把 response dispose 掉，读不到错误正文，探错误响应一律用
`curl.exe`。
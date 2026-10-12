# VNDB Kana API v2 — 响应格式

`https://api.vndb.org/kana`，沙箱 `https://beta.vndb.org/api/kana`。本文只记实测到的响应格式与
约束，供 `src-tauri/src/vndb.rs` 反序列化时对照。

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
| `user`, `token` | string | 鉴权用，本地只读不需要 |

### 响应信封

```json
{ "more": false, "results": [ ... ], "count": 1234 }
```

`more` 表示还有下一页，翻页只需要它。`count` 仅在请求里带了 `count:true` 时出现。

### `/vn` 字段清单

`GET /kana/schema` 的 `api_fields["/vn"]` 递归展开共 55 项，本项目用得到的部分：

| 字段 | JSON 形状 | null 实测 (n=400) | 空数组 | 说明 |
|---|---|---|---|---|
| `id` | string | 0 | — | `"v14"` |
| `title` | string | 0 | — | 主标题（原文） |
| `alttitle` | string | 140 | — | 罗马字标题，常为 null |
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
| `aliases` | string[] | 0 | 265 | |
| `tags` | object[] | 0 | 2 | |
| `image` | object | **1** | — | 罕见但确有 null（样本 `v25031`） |

其余字段（`screenshots` / `staff` / `va` / `relations` / `editions` / `extlinks` / `developers`）
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

### 嵌套与数组的元素结构

```jsonc
"image":     { "id": "cv88797", "url": "https://t.vndb.org/cv/97/88797.jpg",
               "thumbnail": "https://t.vndb.org/cv.t/97/88797.jpg",
               "dims": [674, 967], "thumbnail_dims": [256, 367],
               "sexual": 0, "violence": 0, "votecount": 14 }
"titles":    [ { "lang": "ru", "title": "Школьные Дни", "latin": "Shkol'nye Dni",
                 "official": true, "main": true } ]
"tags":      [ { "id": "g23", "rating": 2.46794867515564, "spoiler": 0, "lie": false } ]
"relations": [ { "id": "v1193", "relation": "side", "relation_official": true } ]
"editions":  [ { "eid": 0, "lang": "en", "name": "…", "official": true } ]
"staff":     [ { "id": "s11993", "role": "staff", "note": "Sound production", "eid": null } ]
"extlinks":  [ { "id": "enwiki", "label": "Wikipedia (en)", "name": "enwiki", "url": "https://…" } ]
```

`tags.rating` 是 float32 精度噪声，会出现 `2.46794867515564` 这种值，取整显示即可。
`dims` / `thumbnail_dims` 是 `[宽, 高]`。

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
要取比这更保守的值。

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
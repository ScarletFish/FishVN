# 第三方数据与翻译来源声明 (Third-Party Attribution)

本目录下的 VNDB 标签 / 特征（角色特征）数据与中文翻译，来自以下项目。
打包进 FishVN 时遵循各自许可证的署名与共享义务，随应用分发时本文件须一并保留。

---

## 1. VNDB 官方标签 / 特征数据 (`tags.json.gz`、`traits.json.gz`)

- **来源**：VNDB database dump（<https://dl.vndb.org/dump/>，含 `vndb-tags-latest.json.gz` / `vndb-traits-latest.json.gz`）
- **许可**：Open Database License (**ODbL**) 1.0 + Database Contents License (**DbCL**) 1.0
- **署名义务**：数据库内容的再分发须**署名**（保留出处）并以**相同方式共享**（ShareAlike）。
- **说明**：仅包含事实性元数据（标签名、分类 `cont/ero/tech`、父子层级、别名、挂载量）与
  `description` 英文描述——该字段在官方 dump 内，同受 ODbL + DbCL，可随包分发，须署名。
  注意：VN 简介 / 角色简介 / 图片**不在此列**，VNDB 对它们不作授权，见第 5 节。

---

## 2. 中文翻译表 (`community/vndb-tags-translation.json`)

- **来源**：VNDBTranslatorLib / VNDBTranslatorLib_（greasyfork **445990** / **472825**）
- **作者**：待定（发包前核定）
- **许可**：**MIT License**
- **说明**：该翻译表由 PotatoVN（GoldenPotato137/PotatoVN，Apache-2.0）从用户脚本抽取打包
  为 `Assets/Data/vndb-tags-translation.json`，其仓库 LICENSE 的 Third Party Notices 已明确将
  数据归属上游。FishVN 复用这一抽取结果（英文名 → 中文 的扁平字典，约 5438 条）。

---

## 3. 社区汉化脚本源（`community/raw/`，仅作溯源 / 繁简归一参考，非直接打包数据）

| 文件 | 脚本 (greasyfork) | 许可 |
|---|---|---|
| `vndb_tw.js` | VNDB_TW (487612) | MIT（繁体，最完整） |
| `vndbtranslatorlib.js` | VNDBTranslatorLib (472825) | MIT（简体源） |
| `vndb_ruili.js` | VNDB优先原文和中文化 (442355) | GPL-3.0（薄封装，`@require` 引用 VNDBTranslatorLib） |

---

## 4. FishVN 团队自补翻译

- **范围**：`community/gap_missing.json` 中由 FishVN 团队补充的未覆盖条目。
- **许可**：团队独立创作，保留全部权利（独立 IP），不受上述数据许可限制。

---

## 5. VNDB API 运行时数据（VN 简介 / 角色简介 / 图片）——不在 ODbL 覆盖范围

- **依据**：VNDB Data License（<https://vndb.org/d17#4>）原文：
  「Images, visual novel descriptions and character descriptions are gathered from various
  online sources and may be subject to separate license conditions」——这三类被明确排除在
  ODbL/DbCL 之外，VNDB 不主张对它们有再授权权利，实际版权归原作方（官方文案、画师、发行商）。
- **图片的单独口径**（<https://vndb.org/d14#6>）：图片**集合**走 ODbL，**单张图片的使用视为
  "fair use"** —— 即 VNDB 不授予单张图片的许可。
- **FishVN 的用法**：这三类一律经 API 运行时按需拉取 + 本地缓存展示，**不打包、不再分发**，
  非商业用途，与 PotatoVN 等 VNDB 客户端的通行做法一致。
- **红线**：不得将简介/图片汇成可再分发的数据包随应用分发，也不得商用。

---

## MIT License（VNDBTranslatorLib / VNDB_TW 等脚本适用）

```
Copyright (c) 待定（VNDBTranslatorLib 作者，发包前核定）

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## ODbL 1.0（VNDB 官方 dump 适用）

- 文本：<https://opendatacommons.org/licenses/odbl/>
- 核心义务：署名（Attribution）+ 以相同方式共享（ShareAlike）。再分发衍生数据库须提供本声明并同样以 ODbL 开放。

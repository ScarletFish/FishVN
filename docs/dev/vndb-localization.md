# VNDB 标签/特征本地化

> 范围：VN 标签（tag）与角色特征（trait）中文显示的数据来源、许可证与维护约束。
> 运行机制（查表、消费点、启动加载）见 [frontend.md](frontend.md)「VNDB 标签/特征中文显示」。

## 数据链路

自上游到产物：

1. **greasyfork 445990 VNDBTranslatorLib**（MIT）—— 翻译数据本体。
2. **PotatoVN 仓库 `Assets/Data/vndb-tags-translation.json`** —— 上游数据的扁平 `英文名 → 中文` 抽取表（5438 键）。
3. **`src-tauri/resources/vndb/community/vndb-tags-translation.json`** —— 拷贝自 PotatoVN 的抽取表。
4. **`src-tauri/resources/vndb/community/gap-fill.json`** —— FishVN 自补翻译，优先级最高。
5. **`community/build_tag_zh.py`** 将 2/3/4 与官方 dump join 成 `resources/vndb/tags_zh.json`（id → 中文），经 `tagdict.rs` 的 `include_str!` 编译进二进制。

产物中的社区翻译来自 PotatoVN 抽取表。`community/raw/` 下的三个脚本是溯源存档：

| 文件 | greasyfork ID | 许可 | 标签/特征页数据量 | 用途 |
|---|---|---|---|---|
| `vndb_tw.js` | 487612 | MIT | 5588 键 | 繁体完整版 |
| `vndbtranslatorlib.js` | 472825 | MIT | 1261 键 | 同族分发版 |
| `vndb_ruili.js` | 442355 | GPL-3.0 | 0（数据 `@require` 自 445990） | 薄封装，证明 445990 是数据本体 |

## 许可证与署名

| 数据 | 许可 | 义务 |
|---|---|---|
| 官方 dump（`tags.json.gz` / `traits.json.gz`，含名字、描述、层级、统计） | ODbL 1.0 + DbCL 1.0 | 署名 VNDB；再分发衍生数据时 ShareAlike |
| 社区翻译表（PotatoVN 抽取表，上游 greasyfork 445990 VNDBTranslatorLib） | MIT | 保留版权声明与许可声明 |
| `gap-fill.json`（FishVN 自补） | FishVN 独立 IP | 无 |

署名义务由 `src-tauri/resources/vndb/ATTRIBUTION.md` 承载，须随应用分发。

引用写双引：**PotatoVN**（`GoldenPotato137/PotatoVN`，直接来源）与 **greasyfork 445990 VNDBTranslatorLib**（上游源头）。作者署名留待发包前核定。

## 维护

- **改翻译的唯一入口是 `community/gap-fill.json`**（英文名 → 中文，优先级高于社区表，也是修正社区表译法的通道），改完重跑 `python community/build_tag_zh.py`。产物 `tags_zh.json` 入 git。
- **纯英文值的放行条件是 `build_tag_zh.py` 的 `ASCII_TERMS` 白名单**（BDSM、NEET、DJ、Vocaloid 等公认可直接用英文的术语）。公认术语可写纯英文值，白名单外的新术语先登记再写；`tagdict.rs` 的测试持同名白名单守门，纯英文伪值（社区表未翻译条目）会被拒收。
- **覆盖率现状**（重跑脚本时打印，应与此同量级）：tags 约 98.7%、traits 约 99.2%（applicable 条目口径）。残余缺口是刻意跳过的敏感词条，补齐时走 `gap-fill.json`。
- **官方 dump 随官方发布更新即可**：重跑生成脚本，已同步的游戏数据不受影响（同步侧存的是 id，显示时才查表）。

#!/usr/bin/env python3
"""生成 tag_groups.json：VNDB 标签 id -> 展示分组根 的映射表。

数据来源：
  - 官方 dump（tags.json.gz，ODbL，parents 字段携带父子层级，parents[0] 为主父）

规则：
  - 对每个 applicable（可直接挂载）标签，沿 parents[0] 上行至根标签；
  - 根标签共五个：Theme(1) / Character(20) / Setting(22) / Plot(24) / Style(674)；
  - 排除（不进 roots，计入 excluded）：
    1. 根为 Style 且二级父为 Presentation(399) 或 Design(675)；
    2. 直接父为 Presentation/Design（DAG 多父挂靠，主父链落在内容支）；
    3. QUIRK_EXCLUDED 名单：主父链落在内容支、但属于技术功能的标签；
  - 键的形态与数据库同步数据对齐：dump 中 id 为纯数字，产物键为 "g"+数字。
    前端按该键查同步标签的 tag.id（"g96" 形态）；未命中的同步标签回落「其他」组。

产物：resources/vndb/tag_groups.json，{"roots": {标签: 根}, "excluded": [标签]}。
excluded 按字符串序排列（Rust 侧二分查找依赖该顺序）。
用法：python build_tag_groups.py（可重复跑，stdlib only）
"""
import gzip
import json
import os
from collections import Counter

ROOT = os.path.dirname(os.path.abspath(__file__))
PROJ = os.path.dirname(ROOT)  # resources/vndb
OUT = os.path.join(PROJ, "tag_groups.json")

ROOTS = {1, 20, 22, 24, 674}
EXCLUDED_SECOND_LEVEL = {399, 675}  # Style 支下的 Presentation / Design

# 主父链不经过技术支、但属于技术功能的 applicable 标签
QUIRK_EXCLUDED = {
    451,    # Skippable Gameplay（主父链：Gameplay Elements → Other Gameplay Elements → Style）
    452,    # Skip Scenes（主父链：Routes → Plot）
    904,    # Super Deformed CGs（主父 Comedy → Theme，副父 Event CGs → Graphics → Presentation）
    1267,   # Super Deformed Sprites（主父 Comedy → Theme，副父指向 Presentation 支）
}


def load_gz(p):
    with gzip.open(p, "rt", encoding="utf-8") as f:
        return json.load(f)


def chain(by_id, tid):
    out = []
    t = by_id.get(tid)
    while t:
        out.append(t)
        parents = t.get("parents") or []
        t = by_id.get(parents[0]) if parents else None
    return out


def main():
    tags = load_gz(os.path.join(PROJ, "tags.json.gz"))
    by_id = {t["id"]: t for t in tags}

    roots = {}
    excluded = []
    stats = Counter()
    for t in tags:
        if not t.get("applicable"):
            continue
        if t["id"] in QUIRK_EXCLUDED:
            excluded.append(f"g{t['id']}")
            continue
        c = chain(by_id, t["id"])
        root = c[-1]
        if root["id"] not in ROOTS:
            raise SystemExit(f"未知根标签：{root['id']} {root['name']}")
        key = f"g{t['id']}"
        if root["id"] == 674:
            second = c[-2]["id"] if len(c) >= 2 else None
            if second in EXCLUDED_SECOND_LEVEL:
                excluded.append(key)
                continue
        if any(p in EXCLUDED_SECOND_LEVEL for p in (t.get("parents") or [])):
            excluded.append(key)
            continue
        roots[key] = f"g{root['id']}"
        stats[root["name"]] += 1

    result = {"roots": roots, "excluded": sorted(excluded)}
    with open(OUT, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, separators=(",", ":"))
    print(f"roots {len(roots)} 条，按根分组：{dict(stats)}")
    print(f"excluded {len(excluded)} 条（Style 的 Presentation/Design 支 + 直接父挂靠 + 技术功能名单）")
    print(f"已写出 {OUT} ({os.path.getsize(OUT)} bytes)")


if __name__ == "__main__":
    main()

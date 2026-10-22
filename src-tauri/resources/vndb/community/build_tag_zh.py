#!/usr/bin/env python3
"""生成 tags_zh.json：VNDB 标签/特征的 id -> 中文名 纯 id 映射。

数据来源：
  - 官方 dump（tags.json.gz / traits.json.gz，ODbL，权威 id <-> name <-> aliases）
  - community/vndb-tags-translation.json（PotatoVN 抽取的社区翻译表，MIT，英文名 -> 中文）
  - community/gap-fill.json（FishVN 自补翻译，英文名 -> 中文，优先级最高；文件不存在则跳过）

join 规则（离线一次性换算，社区表只有英文名键，只能按 name/aliases 对）：
  英文名命中 gap-fill > 英文名命中社区表 > aliases 命中社区表；未命中不入产物，运行时回退英文。
产物仅含 applicable（可直接挂载）条目；tag 与 trait 的 id 空间互相重叠，必须分表存放。
键的形态与数据库同步数据对齐：VNDB API/数据库中的 tag id 是 "g"+数字、trait id 是 "i"+数字，
而官方 dump 中是纯数字，故按前缀规则换算后作为键。
取值过滤：值为空、或值是不在白名单内的纯英文（社区表未翻译的伪值）则拒收；
白名单 ASCII_TERMS 内的术语（BDSM、NEET、DJ 等公认可直接用英文的词）允许纯英文值。
用法：python build_tag_zh.py（可重复跑，stdlib only）
"""
import gzip
import json
import os

ROOT = os.path.dirname(os.path.abspath(__file__))
PROJ = os.path.dirname(ROOT)  # resources/vndb
OUT = os.path.join(PROJ, "tags_zh.json")
TRANS = os.path.join(ROOT, "vndb-tags-translation.json")
GAP = os.path.join(ROOT, "gap-fill.json")

ASCII_TERMS = {
    "bdsm", "neet", "dj", "vocaloid", "adhd", "ptsd", "ocd",
    "cg", "sd", "dc", "marvel", "free!",
    "bl", "gl", "yaoi", "lgbt", "lgbtq", "jk", "jd",
    "fps", "tps", "rpg", "mmorpg", "vr", "d&d",
}


def load_gz(p):
    with gzip.open(p, "rt", encoding="utf-8") as f:
        return json.load(f)


def accepted(zh):
    if not zh:
        return False
    if any(ord(ch) > 127 for ch in zh):
        return True
    return zh.strip().lower() in ASCII_TERMS


def build(items, gap, trans, label, prefix):
    out = {}
    name_hit = alias_hit = gap_hit = ascii_hit = 0
    for it in items:
        if not it.get("applicable"):
            continue
        name = it.get("name", "")
        aliases = it.get("aliases") or []
        if name in gap:
            zh = gap[name]
            gap_hit += 1
        elif name in trans:
            zh = trans[name]
            name_hit += 1
        else:
            hit = next((a for a in aliases if a in trans), None)
            if hit is None:
                continue
            zh = trans[hit]
            alias_hit += 1
        if not accepted(zh):
            continue
        if all(ord(ch) <= 127 for ch in zh):
            ascii_hit += 1
        out[f"{prefix}{it['id']}"] = zh
    print(
        f"{label}: {len(out)}/{sum(1 for x in items if x.get('applicable'))} applicable "
        f"(gap-fill {gap_hit} / name {name_hit} / alias {alias_hit}"
        f"{f', ascii-term {ascii_hit}' if ascii_hit else ''})"
    )
    return out


def main():
    tags = load_gz(os.path.join(PROJ, "tags.json.gz"))
    traits = load_gz(os.path.join(PROJ, "traits.json.gz"))
    trans = json.load(open(TRANS, encoding="utf-8"))
    gap = json.load(open(GAP, encoding="utf-8")) if os.path.exists(GAP) else {}
    if gap:
        print(f"gap-fill 自补条目: {len(gap)}")

    result = {
        "tags": build(tags, gap, trans, "tags", "g"),
        "traits": build(traits, gap, trans, "traits", "i"),
    }
    with open(OUT, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, separators=(",", ":"))
    print(f"已写出 {OUT} ({os.path.getsize(OUT)} bytes)")


if __name__ == "__main__":
    main()

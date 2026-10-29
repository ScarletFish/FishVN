use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TraitEntry {
    pub name: String,
    pub group: String,
}

#[derive(Debug, Deserialize)]
struct Dictionary {
    tags: HashMap<String, String>,
    traits: HashMap<String, TraitEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagGroups {
    pub roots: HashMap<String, String>,
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagScope {
    Hidden,
    Detail,
    Full,
}

impl TagGroups {
    pub fn scope(&self, id: Option<&str>) -> TagScope {
        let Some(id) = id else {
            return TagScope::Full;
        };
        if self.excluded.binary_search_by(|e| e.as_str().cmp(id)).is_ok() {
            return TagScope::Hidden;
        }
        match self.roots.get(id).map(String::as_str) {
            Some("g22" | "g24" | "g674") => TagScope::Detail,
            _ => TagScope::Full,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TagDictionary {
    pub tags: &'static HashMap<String, String>,
    pub traits: &'static HashMap<String, TraitEntry>,
    pub tag_groups: &'static TagGroups,
}

pub fn dictionary() -> TagDictionary {
    static DICT: OnceLock<Dictionary> = OnceLock::new();
    let d = DICT.get_or_init(|| {
        serde_json::from_str(include_str!("../resources/vndb/tags_zh.json"))
            .expect("tags_zh.json 损坏：反序列化失败")
    });
    static GROUPS: OnceLock<TagGroups> = OnceLock::new();
    let g = GROUPS.get_or_init(|| {
        serde_json::from_str(include_str!("../resources/vndb/tag_groups.json"))
            .expect("tag_groups.json 损坏：反序列化失败")
    });
    TagDictionary {
        tags: &d.tags,
        traits: &d.traits,
        tag_groups: g,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 词典两条目数量级正确() {
        let d = dictionary();
        assert!(d.tags.len() > 2000, "tags 条目过少：{}", d.tags.len());
        assert!(d.traits.len() > 2500, "traits 条目过少：{}", d.traits.len());
    }

    const ASCII_TERMS: &[&str] = &[
        "BDSM", "NEET", "DJ", "Vocaloid", "ADHD", "PTSD", "OCD", "CG", "SD", "DC", "Marvel",
        "Free!", "BL", "GL", "Yaoi", "LGBT", "LGBTQ", "JK", "JD", "FPS", "TPS", "RPG", "MMORPG",
        "VR", "D&D", "NTR", "NTL", "Cosplay",
    ];

    #[test]
    fn 词典所有值均为中文或公认英文术语() {
        let d = dictionary();
        let check = |kind: &str, entries: Vec<(&str, &str)>| {
            let bad: Vec<_> = entries
                .iter()
                .filter(|(_, name)| name.is_ascii() && !ASCII_TERMS.contains(name))
                .map(|(k, name)| format!("{kind}/{k}={name}"))
                .collect();
            assert!(bad.is_empty(), "存在非中文且非公认术语的译名：{bad:?}");
        };
        check(
            "tags",
            d.tags.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect(),
        );
        check(
            "traits",
            d.traits
                .iter()
                .map(|(k, v)| (k.as_str(), v.name.as_str()))
                .collect(),
        );
    }

    #[test]
    fn 代表性词条与分表隔离() {
        let d = dictionary();
        assert_eq!(d.tags.get("g2").map(String::as_str), Some("奇幻"));
        assert_ne!(d.traits.get("i2").map(|e| e.name.as_str()), Some("奇幻"));
        assert_eq!(d.tags.get("g4").map(String::as_str), Some("魔法"));
        assert_eq!(d.traits.get("i4").map(|e| e.name.as_str()), Some("黑色"));
    }

    #[test]
    fn 特征均带顶层分组() {
        let d = dictionary();
        let missing: Vec<_> = d
            .traits
            .iter()
            .filter(|(_, e)| e.group.trim().is_empty())
            .map(|(k, _)| k.clone())
            .collect();
        assert!(missing.is_empty(), "存在未带分组的特征：{missing:?}");
    }

    #[test]
    fn 显示范围按根与排除表分档() {
        let groups = dictionary().tag_groups;
        assert_eq!(groups.scope(Some("g32")), TagScope::Hidden);
        assert_eq!(groups.scope(Some("g2")), TagScope::Full);
        assert_eq!(groups.scope(Some("g64")), TagScope::Full);
        assert_eq!(groups.scope(Some("g47")), TagScope::Detail);
        assert_eq!(groups.scope(Some("g148")), TagScope::Detail);
        assert_eq!(groups.scope(Some("g31")), TagScope::Detail);
        assert_eq!(groups.scope(Some("g999999")), TagScope::Full);
        assert_eq!(groups.scope(None), TagScope::Full);
    }

    #[test]
    fn 分组表覆盖五个根且排除技术支() {
        let d = dictionary();
        let groups = d.tag_groups;
        assert!(groups.roots.len() > 2000, "roots 条目过少：{}", groups.roots.len());
        assert_eq!(groups.roots.get("g2").map(String::as_str), Some("g1"));
        assert_eq!(groups.roots.get("g133").map(String::as_str), Some("g20"));
        assert_eq!(groups.roots.get("g148").map(String::as_str), Some("g24"));
        assert_eq!(groups.roots.get("g35").map(String::as_str), Some("g674"));
        assert!(!groups.excluded.contains(&"g35".to_string()), "玩法类型不应被排除");
        assert!(groups.excluded.contains(&"g32".to_string()), "ADV 应被排除");
        assert!(groups.excluded.contains(&"g452".to_string()), "Skip Scenes 应被排除");
        assert!(groups.excluded.contains(&"g904".to_string()), "Super Deformed CGs 应被排除");
        assert!(groups.excluded.contains(&"g1474".to_string()), "Ending List 应被排除");
        assert!(!groups.excluded.contains(&"g96".to_string()), "Romance 不应被排除");
        assert!(!groups.excluded.contains(&"g136".to_string()), "Multiple Protagonists 不应被排除");
        assert!(
            groups.excluded.windows(2).all(|w| w[0] < w[1]),
            "excluded 需按字符串序排列（Rust 侧二分查找依赖该顺序）"
        );
        for (id, root) in groups.roots.iter() {
            assert!(
                matches!(root.as_str(), "g1" | "g20" | "g22" | "g24" | "g674"),
                "{id} 的根 {root} 不在五组之内"
            );
            assert!(
                !groups.excluded.contains(id),
                "{id} 同时出现在 roots 与 excluded"
            );
        }
    }
}

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Dictionary {
    tags: HashMap<String, String>,
    traits: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct TagDictionary {
    pub tags: &'static HashMap<String, String>,
    pub traits: &'static HashMap<String, String>,
}

pub fn dictionary() -> TagDictionary {
    static DICT: OnceLock<Dictionary> = OnceLock::new();
    let d = DICT.get_or_init(|| {
        serde_json::from_str(include_str!("../resources/vndb/tags_zh.json"))
            .expect("tags_zh.json 损坏：反序列化失败")
    });
    TagDictionary {
        tags: &d.tags,
        traits: &d.traits,
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
        "VR", "D&D",
    ];

    #[test]
    fn 词典所有值均为中文或公认英文术语() {
        let d = dictionary();
        for (kind, map) in [("tags", d.tags), ("traits", d.traits)] {
            let bad: Vec<_> = map
                .iter()
                .filter(|(_, v)| v.is_ascii() && !ASCII_TERMS.contains(&v.as_str()))
                .map(|(k, v)| format!("{kind}/{k}={v}"))
                .collect();
            assert!(bad.is_empty(), "存在非中文且非公认术语的译名：{bad:?}");
        }
    }

    #[test]
    fn 代表性词条与分表隔离() {
        let d = dictionary();
        assert_eq!(d.tags.get("g2").map(String::as_str), Some("奇幻"));
        assert_ne!(d.traits.get("i2").map(String::as_str), Some("奇幻"));
        assert_eq!(d.tags.get("g4").map(String::as_str), Some("魔法"));
        assert_eq!(d.traits.get("i4").map(String::as_str), Some("黑色"));
    }
}

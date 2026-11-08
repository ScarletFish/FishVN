use std::collections::HashMap;

use super::dto::{CharRow, CharVn, VnVa};
use crate::types::{CharacterVa, GameCharacter};

const SYNCED_ROLES: [&str; 2] = ["main", "primary"];

pub(super) struct CharacterImage {
    pub character_id: String,
    pub image_id: String,
    pub url: String,
}

pub fn role_in<'a>(vns: Option<&'a [CharVn]>, vn_id: &str) -> Option<&'a str> {
    vns.and_then(|list| {
        list.iter()
            .filter(|v| v.id.as_deref() == Some(vn_id))
            .find_map(|v| v.role.as_deref().filter(|role| SYNCED_ROLES.contains(role)))
    })
}

pub(super) fn map_characters(
    rows: Vec<CharRow>,
    va: &[VnVa],
    vn_id: &str,
) -> (Vec<GameCharacter>, Vec<CharacterImage>) {
    let va_by_character = index_va(va);
    let mut out = Vec::with_capacity(rows.len());
    let mut images = Vec::new();
    for row in rows {
        let Some(role) = role_in(row.vns.as_deref(), vn_id).map(str::to_owned) else {
            continue;
        };
        if let Some((Some(url), Some(id))) = row.image.as_ref().map(|i| (&i.url, &i.id)) {
            images.push(CharacterImage {
                character_id: row.id.clone(),
                image_id: id.clone(),
                url: url.clone(),
            });
        }
        let entry = va_by_character.get(&row.id).map(Vec::as_slice);
        out.push(to_game_character(row, role, entry));
    }
    (out, images)
}

type VaIndex = HashMap<String, Vec<CharacterVa>>;

fn index_va(va: &[VnVa]) -> VaIndex {
    let mut index: VaIndex = HashMap::new();
    for v in va {
        let Some(c) = v.character.as_ref().filter(|c| !c.id.is_empty()) else {
            continue;
        };
        let Some(staff) = v.staff.as_ref().filter(|s| !s.id.is_empty()) else {
            continue;
        };
        index.entry(c.id.clone()).or_default().push(CharacterVa {
            staff_id: staff.id.clone(),
            name: staff.name.clone(),
            original: staff.original.clone(),
            note: v.note.clone(),
        });
    }
    index
}

fn to_game_character(row: CharRow, role: String, va: Option<&[CharacterVa]>) -> GameCharacter {
    GameCharacter {
        id: row.id,
        name: row.name,
        original: row.original,
        aliases: row
            .aliases
            .map(|list| list.into_iter().flatten().collect::<Vec<_>>())
            .filter(|list| !list.is_empty()),
        description: row.description,
        gender: dedupe(row.gender.unwrap_or_default().into_iter().flatten().collect()),
        age: row.age,
        birthday: format_birthday(row.birthday),
        blood_type: row.blood_type,
        height: row.height,
        weight: row.weight,
        bust: row.bust,
        waist: row.waist,
        hips: row.hips,
        cup: row.cup,
        role: Some(role),
        image_id: row.image.as_ref().and_then(|i| i.id.clone()),
        image_path: None,
        sort_order: 0,
        traits: row
            .traits
            .map(|list| {
                list.into_iter()
                    .filter(|t| t.name.as_deref().is_some_and(|n| !n.trim().is_empty()))
                    .collect::<Vec<_>>()
            })
            .filter(|list| !list.is_empty()),
        va: va.map(<[CharacterVa]>::to_vec),
    }
}

fn format_birthday(parts: Option<Vec<Option<i64>>>) -> Option<String> {
    let joined: Vec<String> = parts
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .map(|n| n.to_string())
        .collect();
    (!joined.is_empty()).then(|| joined.join("-"))
}

fn dedupe(values: Vec<String>) -> Option<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    let out: Vec<String> = values
        .into_iter()
        .filter(|v| seen.insert(v.clone()))
        .collect();
    (!out.is_empty()).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vndb::dto::{VnVaCharacter, VnVaStaff};

    fn row(id: &str, gender: Vec<&str>, roles: Vec<&str>) -> CharRow {
        CharRow {
            id: id.to_owned(),
            gender: Some(gender.into_iter().map(|s| Some(s.to_owned())).collect()),
            vns: Some(
                roles
                    .into_iter()
                    .map(|r| CharVn {
                        id: Some("v1".into()),
                        role: Some(r.to_owned()),
                    })
                    .collect(),
            ),
            ..CharRow::default()
        }
    }

    #[test]
    fn gender按release重复要去掉() {
        let c = to_game_character(
            row("c1", vec!["f", "f", "m"], vec!["main"]),
            "main".into(),
            None,
        );
        assert_eq!(c.gender, Some(vec!["f".to_owned(), "m".to_owned()]));
    }

    #[test]
    fn 没有性别的角色留空而不是空数组() {
        let c = to_game_character(CharRow::default(), "main".into(), None);
        assert_eq!(c.gender, None);
    }

    #[test]
    fn birthday只拼接存在的月日分量() {
        for (parts, expected) in [
            (Some(vec![Some(12), Some(15)]), Some("12-15")),
            (Some(vec![Some(12), None]), Some("12")),
            (Some(vec![None, Some(15)]), Some("15")),
            (Some(vec![None, None]), None),
            (None, None),
            (Some(vec![]), None),
        ] {
            let shown = format!("{parts:?}");
            assert_eq!(format_birthday(parts).as_deref(), expected, "{shown}");
        }
    }

    #[test]
    fn role只取主要档且首条是配角时继续往后找() {
        for (roles, expected) in [
            (vec!["main", "side"], Some("main")),
            (vec!["main"], Some("main")),
            (vec!["primary"], Some("primary")),
            (vec!["side"], None),
            (vec!["appears"], None),
            (vec!["side", "primary"], Some("primary")),
        ] {
            let shown = format!("{roles:?}");
            let r = row("c1", vec![], roles);
            assert_eq!(role_in(r.vns.as_deref(), "v1"), expected, "{shown}");
        }
        assert_eq!(role_in(None, "v1"), None);
        assert_eq!(role_in(Some(&[]), "v1"), None);
    }

    #[test]
    fn role只看当前作品的档位() {
        let r = CharRow {
            id: "c1".into(),
            vns: Some(vec![
                CharVn {
                    id: Some("v1".into()),
                    role: Some("side".into()),
                },
                CharVn {
                    id: Some("v2".into()),
                    role: Some("main".into()),
                },
            ]),
            ..CharRow::default()
        };
        assert_eq!(role_in(r.vns.as_deref(), "v1"), None);
        assert_eq!(role_in(r.vns.as_deref(), "v2"), Some("main"));
    }

    #[test]
    fn va按角色分组且保留各版本note() {
        let va = vec![
            VnVa {
                note: Some("Realta Nua".into()),
                staff: Some(VnVaStaff {
                    id: "s1".into(),
                    name: Some("彩音".into()),
                    original: None,
                }),
                character: Some(VnVaCharacter { id: "c1".into() }),
            },
            VnVa {
                note: Some("KO版".into()),
                staff: Some(VnVaStaff {
                    id: "s2".into(),
                    name: None,
                    original: None,
                }),
                character: Some(VnVaCharacter { id: "c1".into() }),
            },
        ];
        let index = index_va(&va);
        let c = to_game_character(
            row("c1", vec![], vec!["main"]),
            "main".into(),
            index.get("c1").map(Vec::as_slice),
        );
        let got = c.va.unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].note, Some("Realta Nua".into()));
        assert_eq!(got[1].staff_id, "s2");
    }

    #[test]
    fn 没有对应va的角色va留空() {
        let c = to_game_character(row("c1", vec![], vec!["primary"]), "primary".into(), None);
        assert_eq!(c.va, None);
        assert_eq!(c.sort_order, 0);
    }

    #[test]
    fn 缺声优的va条目不留空编号() {
        let va = vec![
            VnVa {
                note: None,
                staff: None,
                character: Some(VnVaCharacter { id: "c1".into() }),
            },
            VnVa {
                note: None,
                staff: Some(VnVaStaff::default()),
                character: Some(VnVaCharacter { id: "c2".into() }),
            },
        ];
        assert!(index_va(&va).is_empty());
    }
}

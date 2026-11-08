use std::collections::HashMap;

use super::dto::{CharRow, CharVn, VnVa};
use crate::settings::CharacterScope;
use crate::types::{CharacterVa, GameCharacter};

pub(super) struct CharacterImage {
    pub character_id: String,
    pub image_id: String,
    pub url: String,
}

pub fn role_in<'a>(
    vns: Option<&'a [CharVn]>,
    vn_id: &str,
    scope: CharacterScope,
) -> Option<&'a str> {
    vns.and_then(|list| {
        list.iter()
            .filter(|v| v.id.as_deref() == Some(vn_id))
            .filter_map(|v| v.role.as_deref().filter(|role| scope.allows(role)))
            .min_by_key(|role| crate::generated::character_roles::role_tier(Some(*role)))
    })
}

pub(super) fn map_characters(
    rows: Vec<CharRow>,
    va: &[VnVa],
    vn_id: &str,
    scope: CharacterScope,
) -> (Vec<GameCharacter>, Vec<CharacterImage>) {
    let va_by_character = index_va(va);
    let mut out = Vec::with_capacity(rows.len());
    let mut images = Vec::new();
    for row in rows {
        let Some(role) = role_in(row.vns.as_deref(), vn_id, scope).map(str::to_owned) else {
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
    out.sort_by_key(|c| crate::generated::character_roles::role_tier(c.role.as_deref()));
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
    fn role只取当前档位内的角色且首条不合时继续往后找() {
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
            assert_eq!(
                role_in(r.vns.as_deref(), "v1", CharacterScope::MainOnly),
                expected,
                "{shown}"
            );
        }
        assert_eq!(role_in(None, "v1", CharacterScope::MainOnly), None);
        assert_eq!(role_in(Some(&[]), "v1", CharacterScope::MainOnly), None);
    }

    #[test]
    fn 含配角档放行side但客串两档都不收() {
        for (roles, main_only, with_side) in [
            (vec!["main"], Some("main"), Some("main")),
            (vec!["side"], None, Some("side")),
            (vec!["appears"], None, None),
            (vec!["side", "primary"], Some("primary"), Some("primary")),
        ] {
            let shown = format!("{roles:?}");
            let r = row("c1", vec![], roles);
            assert_eq!(
                role_in(r.vns.as_deref(), "v1", CharacterScope::MainOnly),
                main_only,
                "{shown}"
            );
            assert_eq!(
                role_in(r.vns.as_deref(), "v1", CharacterScope::WithSide),
                with_side,
                "{shown}"
            );
        }
    }

    #[test]
    fn 同一作品多条档位时取最高档() {
        let r = row("c1", vec![], vec!["side", "main"]);
        assert_eq!(
            role_in(r.vns.as_deref(), "v1", CharacterScope::WithSide),
            Some("main")
        );
        let r = row("c1", vec![], vec!["primary", "main"]);
        assert_eq!(
            role_in(r.vns.as_deref(), "v1", CharacterScope::MainOnly),
            Some("main")
        );
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
        assert_eq!(role_in(r.vns.as_deref(), "v1", CharacterScope::MainOnly), None);
        assert_eq!(role_in(r.vns.as_deref(), "v2", CharacterScope::MainOnly), Some("main"));
        assert_eq!(role_in(r.vns.as_deref(), "v1", CharacterScope::WithSide), Some("side"));
        assert_eq!(role_in(r.vns.as_deref(), "v2", CharacterScope::WithSide), Some("main"));
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

    #[test]
    fn 批量补全的默认顺序先主要后次要() {
        let (got, images) = map_characters(
            vec![
                row("c1", vec![], vec!["side"]),
                row("c2", vec![], vec!["main"]),
                row("c3", vec![], vec!["primary"]),
            ],
            &[],
            "v1",
            CharacterScope::WithSide,
        );
        assert_eq!(
            got.iter().map(|c| c.role.as_deref()).collect::<Vec<_>>(),
            vec![Some("main"), Some("primary"), Some("side")]
        );
        assert!(images.is_empty());
    }

    #[test]
    fn 仅主要档丢掉配角且客串永不入库() {
        let (got, _) = map_characters(
            vec![
                row("c1", vec![], vec!["side"]),
                row("c2", vec![], vec!["appears"]),
                row("c3", vec![], vec!["main"]),
            ],
            &[],
            "v1",
            CharacterScope::MainOnly,
        );
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].role.as_deref(), Some("main"));

        let (got, _) = map_characters(
            vec![row("c1", vec![], vec!["appears"])],
            &[],
            "v1",
            CharacterScope::WithSide,
        );
        assert!(got.is_empty());
    }
}

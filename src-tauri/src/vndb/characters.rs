use std::collections::HashMap;

use super::dto::{CharRow, CharVn, VnVa};
use super::images::download_character_images;
use crate::error::Result;
use crate::tasks::Progress;
use crate::types::{CharacterVa, GameCharacter};

const SYNCED_ROLES: [&str; 2] = ["main", "primary"];

pub fn role_of(vns: Option<&[CharVn]>) -> Option<&str> {
    vns.and_then(|list| {
        list.iter()
            .find_map(|v| v.role.as_deref().filter(|role| SYNCED_ROLES.contains(role)))
    })
}

pub(super) async fn map_characters(
    rows: Vec<CharRow>,
    va: &[VnVa],
    data_dir: &std::path::Path,
    progress: &dyn Progress,
) -> Result<Vec<GameCharacter>> {
    let va_by_character = index_va(va);
    let mut out = Vec::with_capacity(rows.len());
    let mut jobs = Vec::new();
    for row in rows {
        let Some(role) = role_of(row.vns.as_deref()).map(str::to_owned) else {
            continue;
        };
        if let Some((Some(url), Some(id))) = row.image.as_ref().map(|i| (&i.url, &i.id)) {
            jobs.push((out.len(), url.clone(), id.clone()));
        }
        let entry = va_by_character.get(&row.id).map(Vec::as_slice);
        out.push(to_game_character(row, role, entry));
    }
    download_character_images(out, jobs, data_dir, progress).await
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
    fn birthday拍成月减日() {
        assert_eq!(
            format_birthday(Some(vec![Some(12), Some(15)])),
            Some("12-15".to_owned())
        );
    }

    #[test]
    fn birthday只有月时只留月() {
        assert_eq!(
            format_birthday(Some(vec![Some(12), None])),
            Some("12".to_owned())
        );
    }

    #[test]
    fn birthday只有日時只留日() {
        assert_eq!(
            format_birthday(Some(vec![None, Some(15)])),
            Some("15".to_owned())
        );
    }

    #[test]
    fn birthday两个分量都空时该字段留空() {
        assert_eq!(format_birthday(Some(vec![None, None])), None);
    }

    #[test]
    fn birthday整体缺失时该字段留空() {
        assert_eq!(format_birthday(None), None);
        assert_eq!(format_birthday(Some(vec![])), None);
    }

    #[test]
    fn 多条release取入库档位() {
        let r = row("c1", vec![], vec!["main", "side"]);
        assert_eq!(role_of(r.vns.as_deref()), Some("main"));
    }

    #[test]
    fn 主要角色的两档都入库() {
        for role in ["main", "primary"] {
            let r = row("c1", vec![], vec![role]);
            assert_eq!(role_of(r.vns.as_deref()), Some(role));
        }
    }

    #[test]
    fn 配角与登场不取() {
        for role in ["side", "appears"] {
            let r = row("c1", vec![], vec![role]);
            assert_eq!(role_of(r.vns.as_deref()), None);
        }
        assert_eq!(role_of(None), None);
        assert_eq!(role_of(Some(&[])), None);
    }

    #[test]
    fn 多条release里首条是配角时往后找主要档() {
        let r = row("c1", vec![], vec!["side", "primary"]);
        assert_eq!(role_of(r.vns.as_deref()), Some("primary"));
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

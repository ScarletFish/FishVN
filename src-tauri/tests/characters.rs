mod common;

use common::{variant, Db};
use fishvn::characters::{
    list_all_staff, list, merge_staff, replace_all, update_staff, StaffMerge, StaffUpdate,
};
use fishvn::games::{create_game, delete_game};
use fishvn::types::{CharacterVa, GameCharacter, NewGame};
use serde_json::json;

fn new_game(db: &Db, name: &str) -> String {
    create_game(
        db,
        &NewGame {
            name: name.into(),
            vndb_id: None,
            metadata: None,
        },
    )
    .expect("建作品失败")
    .id
}

fn character(id: &str, name: &str) -> GameCharacter {
    GameCharacter {
        id: id.into(),
        name: Some(name.into()),
        ..Default::default()
    }
}

fn with_va(id: &str, name: &str, credits: &[(&str, &str, &str)]) -> GameCharacter {
    GameCharacter {
        va: Some(
            credits
                .iter()
                .map(|(staff_id, staff_name, note)| CharacterVa {
                    staff_id: (*staff_id).into(),
                    name: Some((*staff_name).into()),
                    original: None,
                    note: Some((*note).into()),
                })
                .collect(),
        ),
        ..character(id, name)
    }
}

fn count(db: &Db, table: &str, game_id: &str) -> i64 {
    db.query_row(
        &format!("SELECT COUNT(*) FROM {table} WHERE game_id = ?"),
        [game_id],
        |row| row.get(0),
    )
    .expect("统计失败")
}

#[test]
fn 前端新建角色的精简payload能反序列化() {
    // 复现「新增角色时报 500」：前端 toPayload 对新建角色只发它认识的字段，
    // 不带 source 合并，因此 aliases/birthday/traits/sort_order 等都缺。
    // 后端必须容忍缺字段（GameCharacter 加了 #[serde(default)]），否则 serde 报
    // missing field 直接 500。
    let payload = json!({
        "id": "local-abc",
        "name": null,
        "original": null,
        "gender": null,
        "age": null,
        "height": null,
        "weight": null,
        "role": null,
        "image_path": null,
        "description": null,
        "va": []
    });
    let parsed: GameCharacter = serde_json::from_value(payload).expect("精简 payload 反序列化失败");
    assert_eq!(parsed.id, "local-abc");
    assert!(parsed.aliases.is_none());
    assert!(parsed.traits.is_none());
    assert!(parsed.va.is_some_and(|v| v.is_empty()));
}

#[test]
fn 保存后角色按数组顺序读出() {
    let db = Db::new();
    let id = new_game(&db, "待建角色");

    replace_all(
        &db,
        &id,
        &[character("c1", "先登记的"), character("c2", "后登记的")],
    )
    .expect("写角色失败");

    let got = list(&db, &id).expect("读角色失败");
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].id, "c1");
    assert_eq!(got[1].id, "c2");
    assert_eq!(got[0].sort_order, 0);
    assert_eq!(got[1].sort_order, 1);
}

#[test]
fn 重排后顺序按新的数组下标() {
    let db = Db::new();
    let id = new_game(&db, "重排");
    let roles = vec![character("c1", "甲"), character("c2", "乙")];
    replace_all(&db, &id, &roles).expect("写角色失败");

    // 客户端给的顺序就是新顺序，入参里的 sort_order 一律重排
    replace_all(&db, &id, &[character("c2", "乙"), character("c1", "甲")]).expect("重排失败");

    let got = list(&db, &id).expect("读角色失败");
    assert_eq!(got[0].id, "c2");
    assert_eq!(got[0].sort_order, 0);
    assert_eq!(got[1].id, "c1");
}

#[test]
fn 整包替换会带走数组里没出现的角色() {
    let db = Db::new();
    let id = new_game(&db, "删一个角色");
    replace_all(&db, &id, &[character("c1", "甲"), character("c2", "乙")]).expect("写角色失败");

    replace_all(&db, &id, &[character("c1", "甲")]).expect("整包替换失败");

    let got = list(&db, &id).expect("读角色失败");
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].id, "c1");
}

#[test]
fn 空数组清空该作品的角色() {
    let db = Db::new();
    let id = new_game(&db, "清空");
    replace_all(&db, &id, &[character("c1", "甲")]).expect("写角色失败");

    replace_all(&db, &id, &[]).expect("清空失败");

    assert!(list(&db, &id).expect("读角色失败").is_empty());
    assert_eq!(count(&db, "characters", &id), 0);
}

#[test]
fn 没有角色的作品读出来是空数组而不是报错() {
    let db = Db::new();
    let id = new_game(&db, "没同步过角色");

    assert!(list(&db, &id).expect("读角色不该失败").is_empty());
}

#[test]
fn 同一编号在不同作品各存一行() {
    let db = Db::new();
    let first = new_game(&db, "前作");
    let second = new_game(&db, "续作");
    replace_all(&db, &first, &[character("c33", "甲的形象")]).expect("写前作失败");
    replace_all(&db, &second, &[character("c33", "乙的形象")]).expect("写续作失败");

    assert_eq!(
        list(&db, &first).expect("读前作失败")[0].name.as_deref(),
        Some("甲的形象")
    );
    assert_eq!(
        list(&db, &second).expect("读续作失败")[0].name.as_deref(),
        Some("乙的形象")
    );
}

#[test]
fn 重复编号被拒() {
    let db = Db::new();
    let id = new_game(&db, "重复");
    let err = replace_all(
        &db,
        &id,
        &[character("c1", "甲"), character("c1", "还是甲")],
    )
    .expect_err("重复编号竟然通过了");

    assert_eq!(variant(&err), "BadRequest");
    assert_eq!(count(&db, "characters", &id), 0);
}

#[test]
fn 缺编号被拒() {
    let db = Db::new();
    let id = new_game(&db, "缺 id");
    let err =
        replace_all(&db, &id, &[character("  ", "没名字也算缺")]).expect_err("空编号竟然通过了");

    assert_eq!(variant(&err), "BadRequest");
}

#[test]
fn 缺声优编号的配音被拒() {
    let db = Db::new();
    let id = new_game(&db, "缺声优");
    // 空编号会落成一条 id 为空的声优行，所有无名声优还会挤成同一行
    let err = replace_all(&db, &id, &[with_va("c1", "甲", &[("  ", "没编号", "")])])
        .expect_err("空声优编号竟然通过了");

    assert_eq!(variant(&err), "BadRequest");
    assert_eq!(count(&db, "characters", &id), 0);
    assert_eq!(count(&db, "character_va", &id), 0);
}

#[test]
fn 读不存在的作品报_not_found() {
    let db = Db::new();
    let err = list(&db, common::ghost()).expect_err("不存在的作品竟然有角色");
    assert_eq!(variant(&err), "NotFound");
}

#[test]
fn 声优随角色读出并按角色归组() {
    let db = Db::new();
    let id = new_game(&db, "带配音");
    replace_all(
        &db,
        &id,
        &[
            with_va("c1", "甲", &[("s1", "声优甲", "")]),
            with_va(
                "c2",
                "乙",
                &[("s2", "声优乙", "重制版"), ("s3", "声优丙", "")],
            ),
        ],
    )
    .expect("写配音失败");

    let got = list(&db, &id).expect("读角色失败");
    let credited = got[1].va.clone().expect("乙没有配音记录");
    assert_eq!(credited.len(), 2);
    // 无版本标记的主配音排在版本限定的那条之前：'' 小于任何版本名
    assert_eq!(credited[0].staff_id, "s3");
    assert_eq!(credited[0].note.as_deref(), Some(""));
    assert_eq!(credited[1].staff_id, "s2");
    assert_eq!(credited[1].note.as_deref(), Some("重制版"));
    assert_eq!(credited[1].name.as_deref(), Some("声优乙"));
}

#[test]
fn 同编号在两部作品的配音互不串台() {
    let db = Db::new();
    let first = new_game(&db, "前作");
    let second = new_game(&db, "续作");
    replace_all(
        &db,
        &first,
        &[with_va("c33", "甲", &[("s1", "前作声优", "")])],
    )
    .expect("写前作失败");
    replace_all(
        &db,
        &second,
        &[with_va("c33", "乙", &[("s2", "续作声优", "")])],
    )
    .expect("写续作失败");

    let got = list(&db, &first).expect("读前作失败");
    let credited = got[0].va.clone().expect("前作没有配音记录");
    assert_eq!(credited.len(), 1);
    assert_eq!(credited[0].name.as_deref(), Some("前作声优"));
}

#[test]
fn 声优属于所属角色不会串到别的角色上() {
    let db = Db::new();
    let id = new_game(&db, "串声优");
    replace_all(
        &db,
        &id,
        &[
            with_va("c1", "甲", &[("s1", "声优甲", "")]),
            character("c2", "乙"),
        ],
    )
    .expect("写角色失败");

    let got = list(&db, &id).expect("读角色失败");
    assert!(got[1].va.clone().expect("乙应该有空列表").is_empty());
}

#[test]
fn 同一角色同一声优的不同版本各自保留() {
    let db = Db::new();
    let id = new_game(&db, "多版本");
    replace_all(
        &db,
        &id,
        &[with_va(
            "c1",
            "甲",
            &[("s1", "声优甲", "Realta Nua"), ("s1", "声优甲", "重制版")],
        )],
    )
    .expect("写多版本失败");

    let got = list(&db, &id).expect("读角色失败");
    assert_eq!(got[0].va.clone().expect("没有配音记录").len(), 2);
}

#[test]
fn 已有的声优行不被本地名字盖掉() {
    let db = Db::new();
    let id = new_game(&db, "同名声优");
    replace_all(&db, &id, &[with_va("c1", "甲", &[("s1", "全称 A", "")])]).expect("首次写入失败");

    replace_all(&db, &id, &[with_va("c2", "乙", &[("s1", "简称 B", "")])]).expect("二次写入失败");

    let kept: String = db
        .query_row("SELECT name FROM staffs WHERE id = 's1'", [], |row| {
            row.get(0)
        })
        .expect("查声优失败");
    assert_eq!(kept, "全称 A");
}

#[test]
fn 别家已有的声优名不被新写入的盖掉() {
    let db = Db::new();
    let a = new_game(&db, "甲作");
    let b = new_game(&db, "乙作");
    replace_all(&db, &a, &[with_va("c1", "甲", &[("s1", "全称 A", "")])]).expect("写甲失败");

    // 乙作此前没有 s1，取不到 kept_staff，只能靠 ON CONFLICT 保住甲作的名字
    replace_all(&db, &b, &[with_va("c1", "乙", &[("s1", "简称 B", "")])]).expect("写乙失败");

    let kept: String = db
        .query_row("SELECT name FROM staffs WHERE id = 's1'", [], |row| {
            row.get(0)
        })
        .expect("查声优失败");
    assert_eq!(kept, "全称 A");
}

#[test]
fn 库里是空的声优名让传入的填进来() {
    for blank in ["空串", "null"] {
        let db = Db::new();
        let a = new_game(&db, "甲作");
        let b = new_game(&db, "乙作");
        replace_all(&db, &a, &[with_va("c1", "甲", &[("s1", "旧名", "")])]).expect("写甲失败");
        replace_all(&db, &b, &[with_va("c2", "乙", &[("s1", "旧名", "")])]).expect("写乙失败");
        db.execute(
            match blank {
                "空串" => "UPDATE staffs SET name = '' WHERE id = 's1'",
                _ => "UPDATE staffs SET name = NULL WHERE id = 's1'",
            },
            [],
        )
        .expect("置空失败");

        replace_all(&db, &b, &[with_va("c2", "乙", &[("s1", "补上的名字", "")])])
            .expect("写乙失败");

        let name: Option<String> = db
            .query_row("SELECT name FROM staffs WHERE id = 's1'", [], |row| {
                row.get(0)
            })
            .expect("查声优失败");
        assert_eq!(
            name.as_deref(),
            Some("补上的名字"),
            "{blank} 的库里名字要能被传入的填上"
        );
    }
}

fn staff_count(db: &Db, staff_id: &str) -> i64 {
    db.query_row(
        "SELECT COUNT(*) FROM staffs WHERE id = ?",
        [staff_id],
        |row| row.get(0),
    )
    .expect("查声优失败")
}

#[test]
fn 删作品连带清掉角色配音关系与只属于它的声优() {
    let db = Db::new();
    let id = new_game(&db, "待删");
    replace_all(&db, &id, &[with_va("c1", "甲", &[("s1", "声优甲", "")])]).expect("写角色失败");

    delete_game(&db, &id).expect("删除失败");

    assert_eq!(count(&db, "characters", &id), 0);
    assert_eq!(count(&db, "character_va", &id), 0);
    assert_eq!(staff_count(&db, "s1"), 0);
}

#[test]
fn 删作品不动别家也在用的声优() {
    let db = Db::new();
    let a = new_game(&db, "甲作");
    let b = new_game(&db, "乙作");
    let credit = [("s1", "声优甲", "")];
    replace_all(&db, &a, &[with_va("c1", "甲", &credit)]).expect("写甲失败");
    replace_all(&db, &b, &[with_va("c2", "乙", &credit)]).expect("写乙失败");

    delete_game(&db, &a).expect("删除失败");

    assert_eq!(staff_count(&db, "s1"), 1);
    assert_eq!(count(&db, "character_va", &b), 1);
}

#[test]
fn 整包替换换掉角色也回收它的声优() {
    let db = Db::new();
    let id = new_game(&db, "替换");
    replace_all(&db, &id, &[with_va("c1", "甲", &[("s1", "声优甲", "")])]).expect("写角色失败");

    // 同一编号、换一位声优：老声优失去最后一个引用，该跟着走
    replace_all(&db, &id, &[with_va("c1", "甲", &[("s2", "声优乙", "")])]).expect("写角色失败");

    assert_eq!(staff_count(&db, "s1"), 0);
    assert_eq!(staff_count(&db, "s2"), 1);
}

fn staff_patch(id: &str, name: &str, original: &str) -> StaffUpdate {
    StaffUpdate {
        id: id.into(),
        name: name.into(),
        original: original.into(),
    }
}

fn staff_merge(target: &str, source: &str) -> StaffMerge {
    StaffMerge {
        target_id: target.into(),
        source_id: source.into(),
    }
}

fn staff_display(db: &Db, staff_id: &str) -> StaffDisplay {
    let hit = list_all_staff(db)
        .expect("列声优失败")
        .into_iter()
        .find(|s| s.id == staff_id)
        .unwrap_or_else(|| panic!("声优 {staff_id} 不在列表里"));
    StaffDisplay {
        name: hit.display_name.unwrap_or_default(),
        original: hit.display_original.unwrap_or_default(),
        roles: hit.roles,
    }
}

struct StaffDisplay {
    name: String,
    original: String,
    roles: i64,
}

#[test]
fn 改名写进主名并带出列表显示() {
    let db = Db::new();
    let id = new_game(&db, "改名");
    replace_all(&db, &id, &[with_va("c1", "甲", &[("s1", "旧署名", "")])]).expect("写角色失败");

    let updated = update_staff(&db, &staff_patch("s1", "新主名", "新原文")).expect("改名失败");

    assert_eq!(updated.display_name.as_deref(), Some("新主名"));
    assert_eq!(updated.display_original.as_deref(), Some("新原文"));
    let shown = staff_display(&db, "s1");
    assert_eq!(shown.name, "新主名");
    assert_eq!(shown.original, "新原文");
    let credited = list(&db, &id).expect("读角色失败")[0]
        .va
        .clone()
        .expect("没有配音记录");
    assert_eq!(credited[0].name.as_deref(), Some("新主名"));
}

#[test]
fn 署名证据不随改名被涂改() {
    let db = Db::new();
    let id = new_game(&db, "证据");
    replace_all(&db, &id, &[with_va("c1", "甲", &[("s1", "旧署名", "")])]).expect("写角色失败");

    update_staff(&db, &staff_patch("s1", "新主名", "")).expect("改名失败");

    let evidence: String = db
        .query_row(
            "SELECT staff_name FROM character_va WHERE game_id = ?",
            [&id],
            |row| row.get(0),
        )
        .expect("查署名证据失败");
    assert_eq!(evidence, "旧署名");
}

#[test]
fn 空主名被拒() {
    let db = Db::new();
    let id = new_game(&db, "空名");
    replace_all(&db, &id, &[with_va("c1", "甲", &[("s1", "声优甲", "")])]).expect("写角色失败");

    let err = update_staff(&db, &staff_patch("s1", "   ", "")).expect_err("空主名竟然通过了");

    assert_eq!(variant(&err), "BadRequest");
}

#[test]
fn 改不存在的声优报_not_found() {
    let db = Db::new();
    let err = update_staff(&db, &staff_patch("s404", "无名", "")).expect_err("竟然改成功了");

    assert_eq!(variant(&err), "NotFound");
}

#[test]
fn 合并把别处的署名并到目标() {
    let db = Db::new();
    let a = new_game(&db, "甲作");
    let b = new_game(&db, "乙作");
    replace_all(&db, &a, &[with_va("c1", "甲", &[("s1", "目标", "")])]).expect("写甲失败");
    replace_all(&db, &b, &[with_va("c2", "乙", &[("s2", "来源", "")])]).expect("写乙失败");

    let merged = merge_staff(&db, &staff_merge("s1", "s2")).expect("合并失败");

    assert_eq!(merged.roles, 2);
    assert_eq!(staff_count(&db, "s2"), 0);
    assert_eq!(
        count(&db, "character_va", &b),
        1,
        "乙作的署名应换指向而不是消失"
    );
    let owner: String = db
        .query_row(
            "SELECT staff_id FROM character_va WHERE game_id = ?",
            [&b],
            |row| row.get(0),
        )
        .expect("查署名归属失败");
    assert_eq!(owner, "s1");
}

#[test]
fn 合并时同角色同版本的重复署名去重() {
    let db = Db::new();
    let id = new_game(&db, "去重");
    replace_all(
        &db,
        &id,
        &[with_va(
            "c1",
            "甲",
            &[("s1", "目标", ""), ("s2", "来源", "")],
        )],
    )
    .expect("写角色失败");

    merge_staff(&db, &staff_merge("s1", "s2")).expect("合并失败");

    let got = list(&db, &id).expect("读角色失败");
    let credited = got[0].va.clone().expect("没有配音记录");
    assert_eq!(credited.len(), 1, "同角色同 note 只能留一条");
    assert_eq!(credited[0].staff_id, "s1");
}

#[test]
fn 合并把目标空缺的名字从来源补上() {
    let db = Db::new();
    let a = new_game(&db, "甲作");
    let b = new_game(&db, "乙作");
    replace_all(&db, &a, &[with_va("c1", "甲", &[("s1", "目标", "")])]).expect("写甲失败");
    replace_all(&db, &b, &[with_va("c2", "乙", &[("s2", "占位", "")])]).expect("写乙失败");
    db.execute(
        "UPDATE staffs SET name = '来源名', original = '来源原文' WHERE id = 's2'",
        [],
    )
    .expect("设来源名失败");
    db.execute("UPDATE staffs SET name = '', original = '' WHERE id = 's1'", [])
        .expect("置空失败");

    let merged = merge_staff(&db, &staff_merge("s1", "s2")).expect("合并失败");

    assert_eq!(merged.display_name.as_deref(), Some("来源名"));
    assert_eq!(merged.display_original.as_deref(), Some("来源原文"));
}

#[test]
fn 不能把声优并入它自己() {
    let db = Db::new();
    let id = new_game(&db, "自并");
    replace_all(&db, &id, &[with_va("c1", "甲", &[("s1", "声优甲", "")])]).expect("写角色失败");

    let err = merge_staff(&db, &staff_merge("s1", "s1")).expect_err("自己并自己竟然通过了");

    assert_eq!(variant(&err), "BadRequest");
}

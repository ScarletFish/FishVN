//! devhttp 的路由是命令层的镜像，两边手写维护。加命令时漏了路由，
//! 编译照过，浏览器里只看到一个看不出原因的 404

#![cfg(debug_assertions)]

use std::collections::BTreeSet;

/// 从 commands.rs 里抠出 #[tauri::command] 标注的函数名
fn declared_commands() -> BTreeSet<String> {
    let src = include_str!("../src/commands.rs");
    let mut out = BTreeSet::new();
    let mut pending = false;
    for line in src.lines() {
        let line = line.trim();
        if line == "#[tauri::command]" {
            pending = true;
            continue;
        }
        if !pending {
            continue;
        }
        let rest = line
            .strip_prefix("pub fn ")
            .or_else(|| line.strip_prefix("pub async fn "));
        if let Some(rest) = rest {
            if let Some(name) = rest.split('(').next() {
                out.insert(name.to_owned());
            }
        }
        pending = false;
    }
    assert!(!out.is_empty(), "commands.rs 里没找到命令，解析写坏了");
    out
}

#[test]
fn 路由表覆盖全部命令() {
    let declared = declared_commands();
    let routed: BTreeSet<String> = fishvn::devhttp::COMMANDS
        .iter()
        .map(|s| (*s).to_owned())
        .collect();

    let missing: Vec<_> = declared.difference(&routed).collect();
    assert!(missing.is_empty(), "命令没进 devhttp 路由：{missing:?}");

    let extra: Vec<_> = routed.difference(&declared).collect();
    assert!(extra.is_empty(), "devhttp 里有 commands.rs 没有的命令：{extra:?}");
}
#![allow(non_ascii_idents)]

mod common;

use std::path::Path;

use common::Tree;
use fishvn::fsinfo::{read_dir_info, resolve_path};

fn symlink_file(target: &Path, link: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link)
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
    }
}

#[test]
fn 候选入口只收三层_第四层开始不算() {
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[
        ("top.exe", "a"),
        ("d1/one.exe", "b"),
        ("d1/d2/two.exe", "c"),
        ("d1/d2/d3/three.exe", "d"),
    ]);

    let info = read_dir_info(&dir);
    let exes = info.exes.clone();
    assert!(exes.contains(&"top.exe".to_owned()), "顶层要收");
    assert!(exes.contains(&"d1/one.exe".to_owned()), "第 1 层要收");
    assert!(exes.contains(&"d1/d2/two.exe".to_owned()), "第 2 层要收");
    assert!(
        !exes.iter().any(|e| e.starts_with("d1/d2/d3/")),
        "第 3 层不该收，实际收了 {exes:?}"
    );
}

#[test]
fn 超过上限就截断() {
    let mut tree = Tree::new();
    let files: Vec<(String, &str)> = (0..35).map(|i| (format!("e{i}.exe"), "x")).collect();
    let borrowed: Vec<(&str, &str)> = files.iter().map(|(a, b)| (a.as_str(), *b)).collect();
    let dir = tree.make_dir(&borrowed);

    // 截断按遍历顺序发生，readdir 顺序不定，所以只能断言条数不能断言是哪 30 个
    assert_eq!(read_dir_info(&dir).exes.len(), 30);
}

#[test]
fn 符号链接既不递归也不计大小() {
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("real.exe", "abc"), ("sub/inner.exe", "d")]);
    let size_before = read_dir_info(&dir).size_bytes;

    let target = dir.join("real.exe");
    let link = dir.join("link.exe");
    if symlink_file(&target, &link).is_err() {
        // 建符号链接在 Windows 上要管理员权限或开发者模式
        eprintln!("跳过：建符号链接需要管理员权限或开发者模式");
        return;
    }

    let info = read_dir_info(&dir);
    assert!(
        !info.exes.iter().any(|e| e == "link.exe"),
        "链接不该进候选清单，实际 {:?}",
        info.exes
    );
    assert_eq!(info.size_bytes, size_before, "链接不该计入大小");

    let dir_link = dir.join("sub-link");
    if symlink_file(&dir.join("sub"), &dir_link).is_ok() {
        let info = read_dir_info(&dir);
        assert_eq!(
            info.exes.len(),
            2,
            "目录链接不该被跟进去，实际 {:?}",
            info.exes
        );
    }
}

#[test]
fn 扩展名取不到的情况不算候选() {
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[
        (".exe", "a"),      // 点开头的文件名，取不到扩展名
        ("trailing.", "b"), // 结尾是点，也取不到
        ("run.cmd", "c"),
        ("upper.EXE", "d"),
        ("plain.txt", "e"),
    ]);

    let exes = read_dir_info(&dir).exes.clone();
    assert!(exes.contains(&"run.cmd".to_owned()));
    assert!(exes.contains(&"upper.EXE".to_owned()), "扩展名该大小写无关");
    assert!(!exes.contains(&".exe".to_owned()), "点开头不该算");
    assert!(!exes.contains(&"trailing.".to_owned()), "结尾是点不该算");
    assert!(!exes.contains(&"plain.txt".to_owned()));
}

#[test]
fn 候选按层数排序_同层按路径() {
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[
        ("d1/z.exe", "1"),
        ("d1/a.exe", "1"),
        ("top.exe", "1"),
        ("d2/only.exe", "1"),
    ]);

    let exes = read_dir_info(&dir).exes;
    assert_eq!(exes[0], "top.exe", "顶层必须排最前");
    assert_eq!(exes[1], "d1/a.exe", "同层按路径排，不按创建顺序");
    assert_eq!(exes[2], "d1/z.exe");
    assert_eq!(exes[3], "d2/only.exe");
}

#[test]
fn 大小累加全部深度() {
    let mut tree = Tree::new();
    // 三层深，第 3 层已超出候选入口上限，但仍要计入大小
    let dir = tree.make_dir(&[
        ("a.exe", "12345"),
        ("d1/b.exe", "123"),
        ("d1/d2/c.exe", "12"),
        ("d1/d2/d3/d.txt", "1"),
    ]);

    let info = read_dir_info(&dir);
    assert_eq!(info.size_bytes, 5 + 3 + 2 + 1);
    assert_eq!(info.exes.len(), 3, "候选只到第 2 层");
}

#[test]
fn 相对路径归一化到绝对() {
    let base = std::env::current_dir().expect("取不到工作目录");
    let resolved = resolve_path("a/b/../c");
    assert!(
        resolved.is_absolute(),
        "结果应是绝对路径，实际 {resolved:?}"
    );
    assert_eq!(resolved, base.join("a").join("c"));
}

#[test]
fn 读不进的子目录跳过而不中断() {
    let mut tree = Tree::new();
    let dir = tree.make_dir(&[("ok.exe", "abc")]);
    // 造一个 read_dir 会失败的子目录：指向已删目标的符号链接指向它
    let broken = dir.join("broken");
    if symlink_file(&dir.join("real.exe"), &broken).is_err() {
        eprintln!("跳过：建符号链接需要管理员权限或开发者模式");
        return;
    }

    let info = read_dir_info(&dir);
    assert!(
        info.exes.contains(&"ok.exe".to_owned()),
        "别因为一个子目录失败就丢掉整棵树"
    );
}

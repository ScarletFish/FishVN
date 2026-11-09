use crate::tagdict::TagScope;

pub fn scope_of_root(root: &str) -> TagScope {
    match root {
        "g22" | "g24" | "g674" => TagScope::Detail,
        _ => TagScope::Full,
    }
}

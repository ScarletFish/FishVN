use std::{env, fs, path::PathBuf};

const HEADER: &str = r#"#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Integer,
    Real,
    Bool,
    Titles,
    Tags,
    Strings,
    Staff,
    Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlankRule {
    TrimEmpty,
    NullOnly,
    EmptyList,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    Upstream,
    Local,
    System,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldSpec {
    pub key: &'static str,
    pub sql: &'static str,
    pub ty: FieldType,
    pub clearable: bool,
    pub blank: BlankRule,
    pub label: Option<&'static str>,
    pub source: ValueSource,
    pub editable: bool,
    pub in_patch: bool,
}

use crate::error::Result;
use crate::types::{LocalizedTitle, MetadataStaff, MetadataTag};
use rusqlite::types::Value;

"#;

fn main() {
    tauri_build::build();
    println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}\\common-controls.manifest",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    );
    generate_metadata_spec();
    generate_character_roles();
    generate_display_titles();
    generate_module_index();
}

fn text<'a>(field: &'a serde_json::Value, key: &str) -> &'a str {
    field[key]
        .as_str()
        .unwrap_or_else(|| panic!("字段缺少字符串属性 {key}"))
}

fn flag(field: &serde_json::Value, key: &str) -> bool {
    field[key]
        .as_bool()
        .unwrap_or_else(|| panic!("字段缺少布尔属性 {key}"))
}

fn ident(raw: &str, variants: &[(&str, &str)]) -> String {
    variants
        .iter()
        .find(|(value, _)| *value == raw)
        .map(|(_, name)| (*name).to_owned())
        .unwrap_or_else(|| panic!("未知枚举值 {raw}"))
}

fn base_ty(field: &serde_json::Value) -> String {
    let ty = text(field, "ty");
    match ty {
        "text" | "mode" => "String".to_owned(),
        "integer" => "i64".to_owned(),
        "real" => "f64".to_owned(),
        "bool" => "bool".to_owned(),
        "titles" | "tags" | "strings" | "staff" => {
            format!("Vec<{}>", text(field, "item"))
        }
        _ => panic!("未知类型 {ty}"),
    }
}

fn row_ty(field: &serde_json::Value) -> String {
    let base = base_ty(field);
    if flag(field, "nullable") {
        format!("Option<{base}>")
    } else {
        base
    }
}

fn patch_ty(field: &serde_json::Value) -> String {
    let base = base_ty(field);
    if flag(field, "clearable") {
        format!("Option<Option<{base}>>")
    } else {
        format!("Option<{base}>")
    }
}

fn value_expr(ty: &str, clearable: bool) -> &'static str {
    match (ty, clearable) {
        ("text" | "mode", true) => "Value::Text(v.clone())",
        ("integer", true) => "Value::Integer(*v)",
        ("real", true) => "Value::Real(*v)",
        ("titles" | "tags" | "strings" | "staff", true) => {
            "Value::Text(serde_json::to_string(v)?)"
        }
        ("mode", false) => "Value::Text(raw.clone())",
        ("bool", false) => "Value::Integer(i64::from(*raw))",
        _ => panic!("无法生成 SQL 值转换：{ty} clearable={clearable}"),
    }
}

fn blank_expr(key: &str, rule: &str) -> Option<String> {
    let body = match rule {
        "trim_empty" => format!("{key}.as_deref().is_none_or(|v| v.trim().is_empty())"),
        "null_only" => format!("{key}.is_none()"),
        "empty_list" => format!("{key}.as_ref().is_none_or(|v| v.is_empty())"),
        "not_applicable" => return None,
        _ => panic!("未知空值规则 {rule}"),
    };
    Some(body)
}

fn generate_metadata_spec() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let spec = root.join("spec/metadata.json");
    println!("cargo:rerun-if-changed={}", spec.display());

    let doc: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&spec).expect("读取 domain spec 失败"))
            .expect("domain spec 不是合法 JSON");
    let fields = doc["fields"].as_array().expect("fields 必须是数组");

    let types = [
        ("text", "Text"),
        ("integer", "Integer"),
        ("real", "Real"),
        ("bool", "Bool"),
        ("titles", "Titles"),
        ("tags", "Tags"),
        ("strings", "Strings"),
        ("staff", "Staff"),
        ("mode", "Mode"),
    ];
    let blanks = [
        ("trim_empty", "TrimEmpty"),
        ("null_only", "NullOnly"),
        ("empty_list", "EmptyList"),
        ("not_applicable", "NotApplicable"),
    ];
    let sources = [
        ("upstream", "Upstream"),
        ("local", "Local"),
        ("system", "System"),
    ];

    let mut out = String::from(HEADER);

    out.push_str("#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]\npub struct GameMetadata {\n");
    for field in fields {
        out.push_str(&format!(
            "    pub {}: {},\n",
            text(field, "key"),
            row_ty(field)
        ));
    }
    out.push_str("}\n\n");

    out.push_str("#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]\npub struct MetadataPatch {\n");
    for field in fields.iter().filter(|f| flag(f, "in_patch")) {
        if flag(field, "clearable") {
            out.push_str("    #[serde(default, deserialize_with = \"crate::types::double_option\")]\n");
        }
        out.push_str(&format!(
            "    pub {}: {},\n",
            text(field, "key"),
            patch_ty(field)
        ));
    }
    out.push_str("}\n\n");

    out.push_str("impl MetadataPatch {\n    pub fn filled_labels(&self) -> Vec<&'static str> {\n        METADATA_FIELDS\n            .iter()\n            .filter_map(|field| field.label.filter(|_| is_set(self, field.key)))\n            .collect()\n    }\n}\n\n");

    out.push_str("pub const METADATA_FIELDS: &[FieldSpec] = &[\n");
    for field in fields {
        let label = field
            .get("label")
            .and_then(|v| v.as_str())
            .map(|s| format!("Some(\"{s}\")"))
            .unwrap_or_else(|| "None".to_owned());
        out.push_str(&format!(
            "    FieldSpec {{\n        key: \"{}\",\n        sql: \"{}\",\n        ty: FieldType::{},\n        clearable: {},\n        blank: BlankRule::{},\n        label: {label},\n        source: ValueSource::{},\n        editable: {},\n        in_patch: {},\n    }},\n",
            text(field, "key"),
            text(field, "sql"),
            ident(text(field, "ty"), &types),
            flag(field, "clearable"),
            ident(text(field, "blank"), &blanks),
            ident(text(field, "source"), &sources),
            flag(field, "editable"),
            flag(field, "in_patch"),
        ));
    }
    out.push_str("];\n\n");

    out.push_str("pub fn is_set(patch: &crate::types::MetadataPatch, key: &str) -> bool {\n    match key {\n");
    for field in fields.iter().filter(|f| flag(f, "in_patch")) {
        let key = text(field, "key");
        out.push_str(&format!("        \"{key}\" => patch.{key}.is_some(),\n"));
    }
    out.push_str("        _ => false,\n    }\n}\n\n");

    out.push_str("pub fn field(key: &str) -> Option<&'static FieldSpec> {\n    METADATA_FIELDS.iter().find(|f| f.key == key)\n}\n\n");

    let column_list = fields
        .iter()
        .map(|f| text(f, "sql"))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("pub const COLUMNS: &str = \"{column_list}\";\n\n"));

    out.push_str("pub const DDL_COLUMNS: &str = r#\"");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            out.push_str(",\n");
        }
        out.push_str(&format!(
            "          {:<12} {}",
            text(field, "sql"),
            text(field, "col_def")
        ));
    }
    out.push_str("\n\"#;\n\n");

    out.push_str("pub fn patch_columns(patch: &MetadataPatch) -> Result<(Vec<&'static str>, Vec<Value>)> {\n    let mut columns: Vec<&'static str> = Vec::new();\n    let mut values: Vec<Value> = Vec::new();\n");
    for field in fields.iter().filter(|f| flag(f, "in_patch")) {
        let key = text(field, "key");
        let sql = text(field, "sql");
        let clearable = flag(field, "clearable");
        let expr = value_expr(text(field, "ty"), clearable);
        out.push_str(&format!(
            "    if let Some(raw) = &patch.{key} {{\n        columns.push(\"{sql}\");\n"
        ));
        if clearable {
            out.push_str(&format!(
                "        values.push(match raw {{\n            Some(v) => {expr},\n            None => Value::Null,\n        }});\n"
            ));
        } else {
            out.push_str(&format!("        values.push({expr});\n"));
        }
        out.push_str("    }\n");
    }
    out.push_str("    Ok((columns, values))\n}\n\n");

    out.push_str("pub fn is_blank_current(meta: &GameMetadata, key: &str) -> bool {\n    match key {\n");
    for field in fields.iter().filter(|f| flag(f, "in_patch")) {
        let key = text(field, "key");
        if let Some(body) = blank_expr(&format!("meta.{key}"), text(field, "blank")) {
            out.push_str(&format!("        \"{key}\" => {body},\n"));
        }
    }
    out.push_str("        _ => false,\n    }\n}\n\n");

    out.push_str("pub struct MetaRow {\n");
    for field in fields {
        let ty = text(field, "ty");
        let row_type = if matches!(ty, "titles" | "tags" | "strings" | "staff") {
            "Option<String>".to_owned()
        } else {
            row_ty(field)
        };
        out.push_str(&format!(
            "    pub {}: {row_type},\n",
            text(field, "key")
        ));
    }
    out.push_str("}\n\n");

    out.push_str("pub fn read_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MetaRow> {\n    Ok(MetaRow {\n");
    for field in fields {
        out.push_str(&format!(
            "        {}: row.get(\"{}\")?,\n",
            text(field, "key"),
            text(field, "sql")
        ));
    }
    out.push_str("    })\n}\n\n");

    out.push_str("pub fn to_api(row: &MetaRow) -> Result<GameMetadata> {\n    Ok(GameMetadata {\n");
    for field in fields {
        let key = text(field, "key");
        let ty = text(field, "ty");
        let expr = match ty {
            "titles" | "tags" | "strings" | "staff" => {
                format!("from_json::<Vec<{}>>(&row.{key})?", text(field, "item"))
            }
            "text" | "mode" => format!("row.{key}.clone()"),
            _ => format!("row.{key}"),
        };
        out.push_str(&format!("        {key}: {expr},\n"));
    }
    out.push_str("    })\n}\n\n");

    out.push_str("fn from_json<T: serde::de::DeserializeOwned>(raw: &Option<String>) -> Result<Option<T>> {\n    match raw {\n        Some(text) => Ok(Some(serde_json::from_str(text)?)),\n        None => Ok(None),\n    }\n}\n");

    let out_path = root.join("src/generated/metadata.rs");
    write_if_changed(&out_path, &out, "generated/metadata.rs");

    let mut js = String::from("export const METADATA_FIELDS = [\n");
    for field in fields {
        let label = field
            .get("label")
            .map(|v| serde_json::to_string(v).unwrap())
            .unwrap_or_else(|| "null".to_owned());
        js.push_str(&format!(
            "  {{ key: \"{}\", type: \"{}\", blank: \"{}\", label: {}, source: \"{}\", editable: {}, clearable: {}, inPatch: {} }},\n",
            text(field, "key"),
            text(field, "ty"),
            text(field, "blank"),
            label,
            text(field, "source"),
            flag(field, "editable"),
            flag(field, "clearable"),
            flag(field, "in_patch"),
        ));
    }
    js.push_str("];\n\n");
    js.push_str("export const PATCH_KEYS = METADATA_FIELDS.filter((field) => field.inPatch).map((field) => field.key);\n\n");
    js.push_str("export const FORM_KEYS = METADATA_FIELDS.filter((field) => field.inPatch && field.editable).map((field) => field.key);\n");

    let js_path = root
        .parent()
        .unwrap()
        .join("frontend/src/lib/generated/metadata-fields.js");
    write_if_changed(&js_path, &js, "frontend generated/metadata-fields.js");
}

fn role_id(v: &serde_json::Value) -> &str {
    text(v, "id")
}

fn scope_variant(v: &serde_json::Value) -> &str {
    text(v, "variant")
}

fn scope_rank(v: &serde_json::Value) -> u8 {
    v["rank"].as_u64().expect("rank 必须是非负整数") as u8
}

fn role_tier_value(v: &serde_json::Value) -> u64 {
    v["tier"].as_u64().expect("tier 必须是非负整数")
}

fn min_scope(v: &serde_json::Value) -> Option<&str> {
    v.get("minScope").and_then(serde_json::Value::as_str)
}

fn generate_character_roles() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let spec = root.join("spec/character-roles.json");
    println!("cargo:rerun-if-changed={}", spec.display());

    let doc: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&spec).expect("读取 character roles spec 失败"))
            .expect("character roles spec 不是合法 JSON");
    let scopes = doc["scopes"].as_array().expect("scopes 必须是数组");
    let roles = doc["roles"].as_array().expect("roles 必须是数组");

    let mut rs = String::new();
    rs.push_str("#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]\npub enum CharacterScope {\n");
    for scope in scopes {
        if flag(scope, "default") {
            rs.push_str("    #[default]\n");
        }
        rs.push_str(&format!(
            "    #[serde(rename = \"{}\")]\n    {},\n",
            role_id(scope),
            scope_variant(scope)
        ));
    }
    rs.push_str("}\n\n");
    rs.push_str("#[derive(Debug, Clone, Copy)]\npub struct RoleSpec {\n    pub id: &'static str,\n    pub tier: usize,\n    pub min_scope: Option<CharacterScope>,\n}\n\n");
    rs.push_str("pub const ROLE_SPECS: &[RoleSpec] = &[\n");
    for role in roles {
        let min = match min_scope(role) {
            Some(id) => {
                let scope = scopes
                    .iter()
                    .find(|s| role_id(s) == id)
                    .unwrap_or_else(|| panic!("未知 minScope {id}"));
                format!("Some(CharacterScope::{})", scope_variant(scope))
            }
            None => "None".to_owned(),
        };
        let tier = role_tier_value(role);
        rs.push_str(&format!(
            "    RoleSpec {{ id: \"{}\", tier: {tier}, min_scope: {min} }},\n",
            role_id(role)
        ));
    }
    rs.push_str("];\n\n");
    rs.push_str(&format!("pub const UNKNOWN_ROLE_TIER: usize = {};\n\n", roles.len()));
    rs.push_str("impl CharacterScope {\n    pub fn rank(self) -> u8 {\n        match self {\n");
    for scope in scopes {
        rs.push_str(&format!(
            "            Self::{} => {},\n",
            scope_variant(scope),
            scope_rank(scope)
        ));
    }
    rs.push_str("        }\n    }\n\n");
    rs.push_str("    pub fn allows(self, role: &str) -> bool {\n        ROLE_SPECS\n            .iter()\n            .find(|spec| spec.id == role)\n            .and_then(|spec| spec.min_scope)\n            .is_some_and(|min| min.rank() <= self.rank())\n    }\n}\n\n");
    rs.push_str("pub fn role_tier(role: Option<&str>) -> usize {\n    role.and_then(|id| ROLE_SPECS.iter().find(|spec| spec.id == id).map(|spec| spec.tier))\n        .unwrap_or(UNKNOWN_ROLE_TIER)\n}\n");

    let rs_path = root.join("src/generated/character_roles.rs");
    write_if_changed(&rs_path, &rs, "generated/character_roles.rs");

    let mut js = String::from("export const CHARACTER_SCOPES = [\n");
    for scope in scopes {
        js.push_str(&format!(
            "  {{ id: \"{}\", rank: {}, default: {} }},\n",
            role_id(scope),
            scope_rank(scope),
            flag(scope, "default")
        ));
    }
    js.push_str("];\n\n");
    let default_scope = scopes
        .iter()
        .find(|s| flag(s, "default"))
        .unwrap_or_else(|| panic!("必须有一个 default scope"));
    js.push_str(&format!(
        "export const DEFAULT_CHARACTER_SCOPE = \"{}\";\n\n",
        role_id(default_scope)
    ));
    js.push_str("export const ROLE_SPECS = [\n");
    for role in roles {
        let min = min_scope(role)
            .map(|id| format!("\"{id}\""))
            .unwrap_or_else(|| "null".to_owned());
        let tier = role_tier_value(role);
        js.push_str(&format!(
            "  {{ id: \"{}\", tier: {tier}, minScope: {min} }},\n",
            role_id(role)
        ));
    }
    js.push_str("];\n\n");
    js.push_str(&format!(
        "export const UNKNOWN_ROLE_TIER = {};\n\n",
        roles.len()
    ));
    js.push_str("export function roleTier(role) {\n  const id = String(role ?? \"\").trim();\n  return ROLE_SPECS.find((spec) => spec.id === id)?.tier ?? UNKNOWN_ROLE_TIER;\n}\n");

    let js_path = root
        .parent()
        .unwrap()
        .join("frontend/src/lib/generated/character-roles.js");
    write_if_changed(&js_path, &js, "frontend generated/character-roles.js");
}

fn string_array(values: &[serde_json::Value]) -> String {
    let items: Vec<String> = values
        .iter()
        .map(|v| format!("\"{}\"", v.as_str().expect("语言标签必须是字符串")))
        .collect();
    format!("[{}]", items.join(", "))
}

fn generate_display_titles() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let spec = root.join("spec/display-titles.json");
    println!("cargo:rerun-if-changed={}", spec.display());

    let doc: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&spec).expect("读取 display titles spec 失败"))
            .expect("display titles spec 不是合法 JSON");
    let zh = string_array(doc["zhLangs"].as_array().expect("zhLangs 必须是数组"));
    let ja = string_array(doc["jaLangs"].as_array().expect("jaLangs 必须是数组"));
    let official_first = doc["zhOfficialFirst"]
        .as_bool()
        .expect("zhOfficialFirst 必须是布尔");
    let ja_olang = doc["jaOlang"].as_str().expect("jaOlang 必须是字符串");
    let latin_field = doc["latinField"].as_str().expect("latinField 必须是字符串");

    let mut rs = String::new();
    rs.push_str(&format!("pub const ZH_LANGS: &[&str] = &{zh};\n"));
    rs.push_str(&format!("pub const JA_LANGS: &[&str] = &{ja};\n"));
    rs.push_str(&format!(
        "pub const ZH_OFFICIAL_FIRST: bool = {official_first};\n\n"
    ));
    rs.push_str("#[derive(Debug, Clone, Default)]\npub struct TitleCandidate {\n    pub lang: String,\n    pub title: Option<String>,\n    pub official: Option<bool>,\n}\n\n");
    rs.push_str(&format!("#[derive(Debug, Clone, Default)]\npub struct TitleInput {{\n    pub titles: Vec<TitleCandidate>,\n    pub alttitle: Option<String>,\n    pub olang: Option<String>,\n    pub {latin_field}: String,\n}}\n\n"));
    rs.push_str("fn push_unique(out: &mut Vec<String>, raw: Option<&str>) {\n    let text = raw.unwrap_or_default().trim().to_owned();\n    if !text.is_empty() && !out.iter().any(|used| used == &text) {\n        out.push(text);\n    }\n}\n\n");
    rs.push_str("pub fn title_chain(input: &TitleInput) -> Vec<String> {\n    let mut out = Vec::new();\n    let zh: Vec<&TitleCandidate> = input.titles.iter().filter(|t| ZH_LANGS.contains(&t.lang.as_str())).collect();\n    let preferred = if ZH_OFFICIAL_FIRST {\n        zh.iter().find(|t| t.official != Some(false)).copied().or_else(|| zh.first().copied())\n    } else {\n        zh.first().copied()\n    };\n    if let Some(t) = preferred {\n        push_unique(&mut out, t.title.as_deref());\n    }\n    for t in input.titles.iter().filter(|t| JA_LANGS.contains(&t.lang.as_str())) {\n        push_unique(&mut out, t.title.as_deref());\n    }\n");
    rs.push_str(&format!("    if input.olang.as_deref() == Some(\"{ja_olang}\") {{\n        push_unique(&mut out, input.alttitle.as_deref());\n    }}\n"));
    rs.push_str(&format!(
        "    push_unique(&mut out, Some(input.{latin_field}.as_str()));\n    out\n}}\n"
    ));

    let rs_path = root.join("src/generated/display_titles.rs");
    write_if_changed(&rs_path, &rs, "generated/display_titles.rs");

    let mut js = String::new();
    js.push_str(&format!("export const ZH_LANGS = {zh};\n"));
    js.push_str(&format!("export const JA_LANGS = {ja};\n"));
    js.push_str(&format!(
        "export const ZH_OFFICIAL_FIRST = {official_first};\n\n"
    ));
    js.push_str("export function titleChain(input) {\n  const out = [];\n  const push = (raw) => {\n    const text = String(raw ?? \"\").trim();\n    if (text && !out.includes(text)) out.push(text);\n  };\n  const titles = input.titles ?? [];\n  const zh = titles.filter((t) => ZH_LANGS.includes(t.lang));\n  const preferred = ZH_OFFICIAL_FIRST\n    ? zh.find((t) => t.official !== false) ?? zh[0]\n    : zh[0];\n  if (preferred) push(preferred.title);\n  for (const t of titles.filter((t) => JA_LANGS.includes(t.lang))) push(t.title);\n");
    js.push_str(&format!(
        "  if (input.olang === \"{ja_olang}\") push(input.alttitle);\n"
    ));
    js.push_str(&format!("  push(input.{latin_field});\n  return out;\n}}\n"));

    let js_path = root
        .parent()
        .unwrap()
        .join("frontend/src/lib/generated/display-titles.js");
    write_if_changed(&js_path, &js, "frontend generated/display-titles.js");
}

fn generate_module_index() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let path = root.join("src/generated/mod.rs");
    write_if_changed(
        &path,
        "pub mod character_roles;\npub mod display_titles;\npub mod metadata;\n",
        "generated/mod.rs",
    );
}

fn write_if_changed(path: &std::path::Path, content: &str, label: &str) {
    let unchanged = fs::read_to_string(path)
        .map(|old| old == content)
        .unwrap_or(false);
    if !unchanged {
        if env::var_os("FISHVN_GENERATED_CHECK").is_some() {
            panic!("{label} 与 spec/metadata.json 不一致");
        }
        fs::create_dir_all(path.parent().unwrap()).unwrap_or_else(|_| panic!("创建 {label} 目录失败"));
        fs::write(path, content).unwrap_or_else(|_| panic!("写入 {label} 失败"));
    }
}

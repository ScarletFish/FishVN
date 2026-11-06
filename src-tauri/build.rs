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

"#;

fn main() {
    tauri_build::build();
    println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}\\common-controls.manifest",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    );
    generate_metadata_spec();
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

    out.push_str("pub fn field(key: &str) -> Option<&'static FieldSpec> {\n    METADATA_FIELDS.iter().find(|f| f.key == key)\n}\n");

    let out_path = root.join("src/generated/metadata.rs");
    let unchanged = fs::read_to_string(&out_path)
        .map(|old| old == out)
        .unwrap_or(false);
    if !unchanged {
        if env::var_os("FISHVN_GENERATED_CHECK").is_some() {
            panic!("generated/metadata.rs 与 spec/metadata.json 不一致");
        }
        fs::create_dir_all(out_path.parent().unwrap()).expect("创建 generated 目录失败");
        fs::write(&out_path, out).expect("写入 generated/metadata.rs 失败");
    }
}

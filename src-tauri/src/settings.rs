use std::path::Path;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};

use crate::error::Result;

pub use crate::generated::character_roles::CharacterScope;

const FILE_NAME: &str = "config.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeMetric {
    #[default]
    Foreground,
    Alive,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpoilerLevel {
    #[default]
    HideAll,
    HideMajor,
    ShowAll,
}

impl SpoilerLevel {
    pub fn hides(self, spoiler: i64) -> bool {
        match self {
            SpoilerLevel::HideAll => spoiler >= 1,
            SpoilerLevel::HideMajor => spoiler >= 2,
            SpoilerLevel::ShowAll => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VndbSyncMode {
    #[default]
    Download,
    Upload,
    Both,
}

impl VndbSyncMode {
    pub fn downloads(self) -> bool {
        matches!(self, Self::Download | Self::Both)
    }

    pub fn uploads(self) -> bool {
        matches!(self, Self::Upload | Self::Both)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuitBehavior {
    #[default]
    Quit,
    MinimizeToTray,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    #[default]
    Default,
    Amber,
    Ocean,
    Sakura,
    Emerald,
    Violet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardModule {
    pub id: String,
    pub enabled: bool,
}

pub fn default_dashboard_layout() -> Vec<DashboardModule> {
    vec![
        DashboardModule { id: "stats".into(), enabled: true },
        DashboardModule { id: "continue_play".into(), enabled: true },
        DashboardModule { id: "local_library".into(), enabled: true },
    ]
}

pub const EXCLUDED_TAGS_PRESET: &[&str] = &[
    "g520", "g23", "g133", "g96", "g134", "g2", "g147", "g148", "g728", "g304", "g214", "g1094",
    "g202", "g373",
];

pub fn default_excluded_tags() -> Vec<String> {
    EXCLUDED_TAGS_PRESET.iter().map(|id| (*id).to_owned()).collect()
}

pub fn default_dashboard_include_uninstalled() -> bool {
    true
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub time_metric: TimeMetric,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub accent: Accent,
    #[serde(default = "default_dashboard_layout")]
    pub dashboard_layout: Vec<DashboardModule>,
    #[serde(default = "default_dashboard_include_uninstalled")]
    pub dashboard_include_uninstalled: bool,
    #[serde(default)]
    pub hide_r18_tags: bool,
    #[serde(default)]
    pub spoiler_level: SpoilerLevel,
    #[serde(default = "default_excluded_tags")]
    pub excluded_tags: Vec<String>,
    #[serde(default)]
    pub character_scope: CharacterScope,
    #[serde(default)]
    pub vndb_sync_mode: VndbSyncMode,
    #[serde(default)]
    pub vndb_sync_wishlist: bool,
    #[serde(default = "default_true")]
    pub vndb_sync_rating: bool,
    #[serde(default)]
    pub vndb_token_obf: Option<String>,
    #[serde(default)]
    pub magpie_path: Option<String>,
    #[serde(default)]
    pub le_path: Option<String>,
    #[serde(default)]
    pub quit_behavior: QuitBehavior,
    #[serde(default = "default_true")]
    pub minimize_on_launch: bool,
    #[serde(default)]
    pub backup_retention_days: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct ConfigPatch {
    pub time_metric: Option<TimeMetric>,
    pub theme: Option<Theme>,
    pub accent: Option<Accent>,
    pub dashboard_layout: Option<Vec<DashboardModule>>,
    pub dashboard_include_uninstalled: Option<bool>,
    pub hide_r18_tags: Option<bool>,
    pub spoiler_level: Option<SpoilerLevel>,
    pub excluded_tags: Option<Vec<String>>,
    pub character_scope: Option<CharacterScope>,
    pub vndb_sync_mode: Option<VndbSyncMode>,
    pub vndb_sync_wishlist: Option<bool>,
    pub vndb_sync_rating: Option<bool>,
    pub vndb_token: Option<String>,
    pub magpie_path: Option<String>,
    pub le_path: Option<String>,
    pub quit_behavior: Option<QuitBehavior>,
    pub minimize_on_launch: Option<bool>,
    pub backup_retention_days: Option<Option<u64>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigView {
    pub time_metric: TimeMetric,
    pub theme: Theme,
    pub accent: Accent,
    pub dashboard_layout: Vec<DashboardModule>,
    pub dashboard_include_uninstalled: bool,
    pub hide_r18_tags: bool,
    pub spoiler_level: SpoilerLevel,
    pub excluded_tags: Vec<String>,
    pub character_scope: CharacterScope,
    pub vndb_sync_mode: VndbSyncMode,
    pub vndb_sync_wishlist: bool,
    pub vndb_sync_rating: bool,
    pub has_vndb_token: bool,
    pub magpie_path: Option<String>,
    pub le_path: Option<String>,
    pub quit_behavior: QuitBehavior,
    pub minimize_on_launch: bool,
    pub backup_retention_days: Option<u64>,
}

static CACHE: OnceLock<RwLock<Option<AppConfig>>> = OnceLock::new();

fn cache() -> &'static RwLock<Option<AppConfig>> {
    CACHE.get_or_init(|| RwLock::new(None))
}

pub fn init(dir: &Path) {
    let config = read_file(dir).unwrap_or_else(fallback);
    if let Ok(mut guard) = cache().write() {
        *guard = Some(config);
    }
}

pub fn current() -> AppConfig {
    cache()
        .read()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_else(fallback)
}

pub fn update(dir: &Path, config: AppConfig) -> Result<()> {
    write_file(dir, &config)?;
    if let Ok(mut guard) = cache().write() {
        *guard = Some(config);
    }
    Ok(())
}

pub fn patch(dir: &Path, patch: ConfigPatch) -> Result<AppConfig> {
    let mut config = current();
    if let Some(value) = patch.time_metric {
        config.time_metric = value;
    }
    if let Some(value) = patch.theme {
        config.theme = value;
    }
    if let Some(value) = patch.accent {
        config.accent = value;
    }
    if let Some(value) = patch.dashboard_layout {
        config.dashboard_layout = value;
    }
    if let Some(value) = patch.dashboard_include_uninstalled {
        config.dashboard_include_uninstalled = value;
    }
    if let Some(value) = patch.hide_r18_tags {
        config.hide_r18_tags = value;
    }
    if let Some(value) = patch.spoiler_level {
        config.spoiler_level = value;
    }
    if let Some(value) = patch.excluded_tags {
        config.excluded_tags = value;
    }
    if let Some(value) = patch.character_scope {
        config.character_scope = value;
    }
    if patch.vndb_sync_mode.is_some()
        || patch.vndb_sync_wishlist.is_some()
        || patch.vndb_sync_rating.is_some()
        || patch.vndb_token.is_some()
    {
        crate::sync::invalidate_plan();
    }
    if let Some(value) = patch.vndb_sync_mode {
        config.vndb_sync_mode = value;
    }
    if let Some(value) = patch.vndb_sync_wishlist {
        config.vndb_sync_wishlist = value;
    }
    if let Some(value) = patch.vndb_sync_rating {
        config.vndb_sync_rating = value;
    }
    if let Some(raw) = patch.vndb_token {
        config.vndb_token_obf = obfuscate_token(&raw);
    }
    if let Some(raw) = patch.magpie_path {
        config.magpie_path = clean_tool_path(&raw);
    }
    if let Some(raw) = patch.le_path {
        config.le_path = clean_tool_path(&raw);
    }
    if let Some(value) = patch.quit_behavior {
        config.quit_behavior = value;
    }
    if let Some(value) = patch.minimize_on_launch {
        config.minimize_on_launch = value;
    }
    if let Some(value) = patch.backup_retention_days {
        config.backup_retention_days = value;
    }
    update(dir, config)?;
    Ok(current())
}

fn clean_tool_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

const TOKEN_MASK: &[u8] = b"FishVN-vndb-token-obf-2026";

pub fn obfuscate_token(raw: &str) -> Option<String> {
    if raw.trim().is_empty() {
        return None;
    }
    let bytes = raw.as_bytes();
    let masked: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ TOKEN_MASK[i % TOKEN_MASK.len()])
        .collect();
    Some(hex_encode(&masked))
}

pub fn deobfuscate_token(obf: &str) -> Option<String> {
    let bytes = hex_decode(obf).ok()?;
    let raw: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ TOKEN_MASK[i % TOKEN_MASK.len()])
        .collect();
    String::from_utf8(raw).ok()
}

pub fn has_vndb_token(config: &AppConfig) -> bool {
    config
        .vndb_token_obf
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty())
}

pub fn view() -> ConfigView {
    let config = current();
    let token_set = has_vndb_token(&config);
    ConfigView {
        time_metric: config.time_metric,
        theme: config.theme,
        accent: config.accent,
        dashboard_layout: config.dashboard_layout,
        dashboard_include_uninstalled: config.dashboard_include_uninstalled,
        hide_r18_tags: config.hide_r18_tags,
        spoiler_level: config.spoiler_level,
        excluded_tags: config.excluded_tags,
        character_scope: config.character_scope,
        vndb_sync_mode: config.vndb_sync_mode,
        vndb_sync_wishlist: config.vndb_sync_wishlist,
        vndb_sync_rating: config.vndb_sync_rating,
        has_vndb_token: token_set,
        magpie_path: config.magpie_path.clone(),
        le_path: config.le_path.clone(),
        quit_behavior: config.quit_behavior,
        minimize_on_launch: config.minimize_on_launch,
        backup_retention_days: config.backup_retention_days,
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode(text: &str) -> std::result::Result<Vec<u8>, ()> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Err(());
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let val = |c: u8| match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(()),
    };
    for pair in bytes.chunks(2) {
        out.push((val(pair[0])? << 4) | val(pair[1])?);
    }
    Ok(out)
}

fn read_file(dir: &Path) -> Option<AppConfig> {
    let text = std::fs::read_to_string(dir.join(FILE_NAME)).ok()?;
    serde_json::from_str(&text).ok()
}

fn fallback() -> AppConfig {
    serde_json::from_str("{}").expect("{} 一定能反序列化成 AppConfig")
}

fn write_file(dir: &Path, config: &AppConfig) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let text = serde_json::to_string_pretty(config)?;
    std::fs::write(dir.join(FILE_NAME), text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::MutexGuard;

    static CONFIG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn exclusive() -> MutexGuard<'static, ()> {
        CONFIG_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn 排除池预置均为可显示标签且有中文译名() {
        let dict = crate::tagdict::dictionary();
        for id in EXCLUDED_TAGS_PRESET {
            assert_ne!(
                dict.tag_groups.scope(Some(*id)),
                crate::tagdict::TagScope::Hidden,
                "{id} 是技术向标签，不应出现在预置里"
            );
            assert!(dict.tags.contains_key(*id), "{id} 缺少中文译名");
        }
    }

    #[test]
    fn 没有配置文件时取默认值() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert_eq!(current(), fallback());
    }

    #[test]
    fn 更新后写盘且缓存生效() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        update(
            tmp.path(),
            AppConfig {
                time_metric: TimeMetric::Alive,
                theme: Theme::Dark,
                accent: Accent::Ocean,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(current().time_metric, TimeMetric::Alive);
        assert_eq!(current().theme, Theme::Dark);
        assert_eq!(current().accent, Accent::Ocean);

        init(tmp.path());
        assert_eq!(current().time_metric, TimeMetric::Alive);
        assert_eq!(current().theme, Theme::Dark);
        assert_eq!(current().accent, Accent::Ocean);
        let text = std::fs::read_to_string(tmp.path().join(FILE_NAME)).unwrap();
        assert!(text.contains("\"alive\""));
        assert!(text.contains("\"dark\""));
        assert!(text.contains("\"ocean\""));
    }

    #[test]
    fn 局部补丁只改给到的字段() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        update(
            tmp.path(),
            AppConfig {
                time_metric: TimeMetric::Alive,
                theme: Theme::Dark,
                accent: Accent::Default,
                backup_retention_days: Some(30),
                ..Default::default()
            },
        )
        .unwrap();

        let config = patch(
            tmp.path(),
            ConfigPatch {
                theme: Some(Theme::Light),
                accent: Some(Accent::Amber),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(config.time_metric, TimeMetric::Alive);
        assert_eq!(config.theme, Theme::Light);
        assert_eq!(config.accent, Accent::Amber);
        assert_eq!(config.backup_retention_days, Some(30));
        assert_eq!(current().time_metric, TimeMetric::Alive);

        let off = patch(
            tmp.path(),
            ConfigPatch {
                backup_retention_days: Some(None),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(off.backup_retention_days, None);
    }

    #[test]
    fn 布局缺省时按默认顺序补全且补丁可整列替换() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert_eq!(
            current().dashboard_layout,
            vec![
                DashboardModule { id: "stats".into(), enabled: true },
                DashboardModule { id: "continue_play".into(), enabled: true },
                DashboardModule { id: "local_library".into(), enabled: true },
            ]
        );

        let config = patch(
            tmp.path(),
            ConfigPatch {
                dashboard_layout: Some(vec![
                    DashboardModule { id: "stats".into(), enabled: true },
                    DashboardModule { id: "continue_play".into(), enabled: false },
                ]),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(
            config.dashboard_layout,
            vec![
                DashboardModule { id: "stats".into(), enabled: true },
                DashboardModule { id: "continue_play".into(), enabled: false },
            ]
        );
        init(tmp.path());
        assert_eq!(current().dashboard_layout[0].id, "stats");
        assert!(!current().dashboard_layout[1].enabled);
    }

    #[test]
    fn 损坏的配置文件回落默认() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(FILE_NAME), "not json").unwrap();
        init(tmp.path());
        assert_eq!(current(), fallback());
    }

    #[test]
    fn r18标签隐藏开关默认关闭且局部补丁可开启() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert!(!current().hide_r18_tags);

        let config = patch(tmp.path(), ConfigPatch { hide_r18_tags: Some(true), ..Default::default() }).unwrap();
        assert!(config.hide_r18_tags);
        assert!(current().hide_r18_tags);

        init(tmp.path());
        assert!(current().hide_r18_tags);
        let text = std::fs::read_to_string(tmp.path().join(FILE_NAME)).unwrap();
        assert!(text.contains("hide_r18_tags"));
    }

    #[test]
    fn 剧透级别默认隐藏全部且补丁可改() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert_eq!(current().spoiler_level, SpoilerLevel::HideAll);
        assert!(SpoilerLevel::HideAll.hides(1));
        assert!(SpoilerLevel::HideMajor.hides(2));
        assert!(!SpoilerLevel::HideMajor.hides(1));
        assert!(!SpoilerLevel::ShowAll.hides(2));

        let config = patch(
            tmp.path(),
            ConfigPatch { spoiler_level: Some(SpoilerLevel::HideMajor), ..Default::default() },
        )
        .unwrap();
        assert_eq!(config.spoiler_level, SpoilerLevel::HideMajor);

        init(tmp.path());
        assert_eq!(current().spoiler_level, SpoilerLevel::HideMajor);
        let text = std::fs::read_to_string(tmp.path().join(FILE_NAME)).unwrap();
        assert!(text.contains("spoiler_level"));
    }

    #[test]
    fn 排除池默认含预设且补丁可整列替换() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert_eq!(current().excluded_tags, default_excluded_tags());
        assert!(current().excluded_tags.contains(&"g133".to_owned()));

        let config = patch(
            tmp.path(),
            ConfigPatch {
                excluded_tags: Some(vec!["g96".to_owned(), "手输备注".to_owned()]),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(config.excluded_tags, vec!["g96".to_owned(), "手输备注".to_owned()]);
        assert!(!config.excluded_tags.contains(&"g133".to_owned()));

        init(tmp.path());
        assert_eq!(current().excluded_tags, config.excluded_tags);
        let text = std::fs::read_to_string(tmp.path().join(FILE_NAME)).unwrap();
        assert!(text.contains("excluded_tags"));
    }

    #[test]
    fn 旧配置无排除池字段时回落预设() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(FILE_NAME), r#"{"theme":"dark"}"#).unwrap();
        init(tmp.path());
        assert_eq!(current().excluded_tags, default_excluded_tags());
    }

    #[test]
    fn vndb_token_混淆可逆且空串清空() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());

        patch(
            tmp.path(),
            ConfigPatch {
                vndb_token: Some("secret-token-123".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let obf = current().vndb_token_obf.clone();
        assert!(obf.is_some());
        assert_ne!(obf.as_deref().unwrap(), "secret-token-123");
        assert_eq!(deobfuscate_token(obf.as_deref().unwrap()).unwrap(), "secret-token-123");
        assert!(has_vndb_token(&current()));

        let config = patch(
            tmp.path(),
            ConfigPatch {
                vndb_token: Some("".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(config.vndb_token_obf.is_none());
        assert!(!has_vndb_token(&current()));
        assert!(!view().has_vndb_token);
    }

    #[test]
    fn vndb_sync_mode_默认仅下载且补丁可改() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert_eq!(current().vndb_sync_mode, VndbSyncMode::Download);

        let config = patch(
            tmp.path(),
            ConfigPatch {
                vndb_sync_mode: Some(VndbSyncMode::Both),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(config.vndb_sync_mode, VndbSyncMode::Both);
        assert_eq!(view().vndb_sync_mode, VndbSyncMode::Both);
        let text = std::fs::read_to_string(tmp.path().join(FILE_NAME)).unwrap();
        assert!(text.contains("\"both\""));
    }

    #[test]
    fn vndb_sync_wishlist_默认不同步且补丁可改() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert!(!current().vndb_sync_wishlist);

        let config = patch(
            tmp.path(),
            ConfigPatch {
                vndb_sync_wishlist: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(config.vndb_sync_wishlist);
        assert!(view().vndb_sync_wishlist);

        init(tmp.path());
        assert!(current().vndb_sync_wishlist);
        let text = std::fs::read_to_string(tmp.path().join(FILE_NAME)).unwrap();
        assert!(text.contains("vndb_sync_wishlist"));
    }

    #[test]
    fn vndb_sync_rating_默认同步且补丁可关() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert!(current().vndb_sync_rating);

        let config = patch(
            tmp.path(),
            ConfigPatch {
                vndb_sync_rating: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!config.vndb_sync_rating);
        assert!(!view().vndb_sync_rating);

        init(tmp.path());
        assert!(!current().vndb_sync_rating);
    }

    #[test]
    fn 工具路径补丁去空白设置_空串清空_只动给到的字段() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert!(current().magpie_path.is_none());
        assert!(current().le_path.is_none());

        let config = patch(
            tmp.path(),
            ConfigPatch {
                magpie_path: Some("  D:\\Tools\\Magpie\\Magpie.exe  ".into()),
                le_path: Some("D:\\Tools\\LE\\LEProc.exe".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            config.magpie_path.as_deref(),
            Some("D:\\Tools\\Magpie\\Magpie.exe")
        );
        assert_eq!(config.le_path.as_deref(), Some("D:\\Tools\\LE\\LEProc.exe"));
        assert_eq!(
            view().magpie_path.as_deref(),
            Some("D:\\Tools\\Magpie\\Magpie.exe")
        );

        let config = patch(
            tmp.path(),
            ConfigPatch { magpie_path: Some("   ".into()), ..Default::default() },
        )
        .unwrap();
        assert!(config.magpie_path.is_none());
        assert!(view().magpie_path.is_none());
        assert_eq!(config.le_path.as_deref(), Some("D:\\Tools\\LE\\LEProc.exe"));

        init(tmp.path());
        assert!(current().magpie_path.is_none());
        assert_eq!(current().le_path.as_deref(), Some("D:\\Tools\\LE\\LEProc.exe"));
        let text = std::fs::read_to_string(tmp.path().join(FILE_NAME)).unwrap();
        assert!(text.contains("LEProc.exe"));
        assert!(text.contains("\"magpie_path\": null"));
    }

    #[test]
    fn 角色范围默认仅主要且客串始终排除() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert_eq!(current().character_scope, CharacterScope::MainOnly);
        assert!(CharacterScope::MainOnly.allows("main"));
        assert!(CharacterScope::MainOnly.allows("primary"));
        assert!(!CharacterScope::MainOnly.allows("side"));
        assert!(!CharacterScope::MainOnly.allows("appears"));
        assert!(CharacterScope::WithSide.allows("side"));
        assert!(!CharacterScope::WithSide.allows("appears"));

        let config = patch(
            tmp.path(),
            ConfigPatch {
                character_scope: Some(CharacterScope::WithSide),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(config.character_scope, CharacterScope::WithSide);
        assert_eq!(view().character_scope, CharacterScope::WithSide);

        init(tmp.path());
        assert_eq!(current().character_scope, CharacterScope::WithSide);
        let text = std::fs::read_to_string(tmp.path().join(FILE_NAME)).unwrap();
        assert!(text.contains("\"with_side\""));
    }

    #[test]
    fn 旧配置无角色范围字段时回落默认() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(FILE_NAME), r#"{"theme":"dark"}"#).unwrap();
        init(tmp.path());
        assert_eq!(current().character_scope, CharacterScope::MainOnly);
    }
}

use std::path::Path;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};

use crate::error::Result;

const FILE_NAME: &str = "config.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeMetric {
    #[default]
    Foreground,
    Alive,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub time_metric: TimeMetric,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub accent: Accent,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct ConfigPatch {
    pub time_metric: Option<TimeMetric>,
    pub theme: Option<Theme>,
    pub accent: Option<Accent>,
}

static CACHE: OnceLock<RwLock<Option<AppConfig>>> = OnceLock::new();

fn cache() -> &'static RwLock<Option<AppConfig>> {
    CACHE.get_or_init(|| RwLock::new(None))
}

pub fn init(dir: &Path) {
    let config = read_file(dir).unwrap_or_default();
    if let Ok(mut guard) = cache().write() {
        *guard = Some(config);
    }
}

pub fn current() -> AppConfig {
    cache()
        .read()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_default()
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
    update(dir, config)?;
    Ok(current())
}

fn read_file(dir: &Path) -> Option<AppConfig> {
    let text = std::fs::read_to_string(dir.join(FILE_NAME)).ok()?;
    serde_json::from_str(&text).ok()
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

    #[test]
    fn 没有配置文件时取默认值() {
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert_eq!(current(), AppConfig::default());
    }

    #[test]
    fn 更新后写盘且缓存生效() {
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        update(
            tmp.path(),
            AppConfig {
                time_metric: TimeMetric::Alive,
                theme: Theme::Dark,
                accent: Accent::Ocean,
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
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        update(
            tmp.path(),
            AppConfig {
                time_metric: TimeMetric::Alive,
                theme: Theme::Dark,
                accent: Accent::Default,
            },
        )
        .unwrap();

        let config = patch(
            tmp.path(),
            ConfigPatch {
                time_metric: None,
                theme: Some(Theme::Light),
                accent: Some(Accent::Amber),
            },
        )
        .unwrap();

        assert_eq!(config.time_metric, TimeMetric::Alive);
        assert_eq!(config.theme, Theme::Light);
        assert_eq!(config.accent, Accent::Amber);
        assert_eq!(current().time_metric, TimeMetric::Alive);
    }

    #[test]
    fn 损坏的配置文件回落默认() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(FILE_NAME), "not json").unwrap();
        init(tmp.path());
        assert_eq!(current(), AppConfig::default());
    }
}

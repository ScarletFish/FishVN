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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardModule {
    pub id: String,
    pub enabled: bool,
}

pub fn default_dashboard_layout() -> Vec<DashboardModule> {
    vec![
        DashboardModule { id: "continue_play".into(), enabled: true },
        DashboardModule { id: "stats".into(), enabled: true },
    ]
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
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct ConfigPatch {
    pub time_metric: Option<TimeMetric>,
    pub theme: Option<Theme>,
    pub accent: Option<Accent>,
    pub dashboard_layout: Option<Vec<DashboardModule>>,
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
    update(dir, config)?;
    Ok(current())
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
                ..Default::default()
            },
        )
        .unwrap();

        let config = patch(
            tmp.path(),
            ConfigPatch {
                time_metric: None,
                theme: Some(Theme::Light),
                accent: Some(Accent::Amber),
                dashboard_layout: None,
            },
        )
        .unwrap();

        assert_eq!(config.time_metric, TimeMetric::Alive);
        assert_eq!(config.theme, Theme::Light);
        assert_eq!(config.accent, Accent::Amber);
        assert_eq!(current().time_metric, TimeMetric::Alive);
    }

    #[test]
    fn 布局缺省时按默认顺序补全且补丁可整列替换() {
        let _guard = exclusive();
        let tmp = tempfile::tempdir().unwrap();
        init(tmp.path());
        assert_eq!(
            current().dashboard_layout,
            vec![
                DashboardModule { id: "continue_play".into(), enabled: true },
                DashboardModule { id: "stats".into(), enabled: true },
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
}

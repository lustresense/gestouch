use crate::gesture::{Direction, GestureKind, Recognized};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};

pub const SCHEMA: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub schema_version: u32,
    pub enabled: bool,
    pub start_at_login: bool,
    pub rules: Vec<GestureRule>,
}
impl Default for Config {
    fn default() -> Self { Self { schema_version: SCHEMA, enabled: true, start_at_login: false, rules: vec![] } }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GestureRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger: Trigger,
    pub action: Action,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trigger {
    pub kind: GestureKind,
    pub fingers: u8,
    pub direction: Direction,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag="type", rename_all="snake_case")]
pub enum Action {
    VolumeUp,
    VolumeDown,
    VolumeMute,
    MediaNext,
    MediaPrevious,
    MediaPlayPause,
    KeyCombo { keys: String },
}
impl Config {
    pub fn matching(&self, r: Recognized) -> Option<&Action> {
        if !self.enabled { return None; }
        self.rules.iter().find(|rule| rule.enabled && rule.trigger.fingers == r.fingers &&
            rule.trigger.kind == r.kind && rule.trigger.direction == r.direction).map(|rule| &rule.action)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA { return Err(format!("Unsupported schema {}, expected {}", self.schema_version, SCHEMA)); }
        if self.rules.len() > 256 { return Err("Too many rules".into()); }
        let mut ids = std::collections::HashSet::new();
        for r in &self.rules {
            if r.id.is_empty() || r.id.len() > 80 || !ids.insert(&r.id) { return Err(format!("Invalid/duplicate rule ID {}", r.id)); }
            if r.name.len() > 120 || !(2..=5).contains(&r.trigger.fingers) { return Err(format!("Invalid trigger {}",r.id)); }
            if r.trigger.kind == GestureKind::Tap && r.trigger.direction != Direction::None { return Err(format!("Tap requires direction none: {}",r.id)); }
            if r.trigger.kind == GestureKind::Swipe && r.trigger.direction == Direction::None { return Err(format!("Swipe requires direction: {}",r.id)); }
            if let Action::KeyCombo { keys } = &r.action {
                if keys.len() > 64 || keys.trim().is_empty() { return Err("Invalid shortcut".into()); }
            }
        }
        Ok(())
    }
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .unwrap_or_else(|| std::env::temp_dir());
    base.join("Gestouch")
}
pub fn config_path() -> PathBuf { data_dir().join("config.toml") }
pub fn load() -> Result<Config, String> {
    let path = config_path();
    if !path.exists() { return Ok(Config::default()); }
    let input = fs::read_to_string(&path).map_err(|e| format!("Read {}: {e}",path.display()))?;
    let config: Config = toml::from_str(&input).map_err(|e| format!("Invalid TOML: {e}"))?;
    config.validate()?;
    Ok(config)
}
pub fn save(config: &Config) -> Result<(), String> {
    config.validate()?;
    let path = config_path();
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let contents = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    atomic_save(&path, contents.as_bytes()).map_err(|e| e.to_string())
}
fn atomic_save(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temp = path.with_extension("tmp");
    let backup = path.with_extension("bak");
    fs::write(&temp, bytes)?;
    if backup.exists() { fs::remove_file(&backup)?; }
    if path.exists() { fs::rename(path, &backup)?; }
    if let Err(err) = fs::rename(&temp, path) {
        if backup.exists() { let _ = fs::rename(&backup, path); }
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn config_roundtrip_and_validation() {
        let c=Config::default();c.validate().unwrap();
        let encoded=toml::to_string(&c).unwrap();let decoded:Config=toml::from_str(&encoded).unwrap();
        assert!(decoded.rules.is_empty());
    }
    #[test] fn invalid_schema_rejected() {let mut c=Config::default();c.schema_version=3;assert!(c.validate().is_err());}
}

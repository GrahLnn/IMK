use std::collections::HashSet;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct InputConfig {
    pub eng: HashSet<String>,
    pub chinese: HashSet<String>,
    pub caps_interceptor: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            eng: HashSet::new(),
            chinese: HashSet::new(),
            caps_interceptor: default_caps_interceptor(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct RawConfig {
    #[serde(default, rename = "ENG")]
    eng: Vec<String>,
    #[serde(default, rename = "CHINESE")]
    chinese: Vec<String>,
    #[serde(default = "default_caps_interceptor", rename = "CAPS_INTERCEPTOR")]
    caps_interceptor: bool,
}

impl InputConfig {
    pub fn from_json_str(s: &str) -> anyhow::Result<Self> {
        let raw: RawConfig = serde_json::from_str(s).context("parse config json")?;
        Ok(Self {
            eng: raw.eng.into_iter().collect(),
            chinese: raw.chinese.into_iter().collect(),
            caps_interceptor: raw.caps_interceptor,
        })
    }

    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("read config: {}", path.as_ref().display()))?;
        Self::from_json_str(&text)
    }

    pub fn to_json_pretty(&self) -> anyhow::Result<String> {
        let raw = RawConfig {
            eng: self.eng.iter().cloned().collect(),
            chinese: self.chinese.iter().cloned().collect(),
            caps_interceptor: self.caps_interceptor,
        };
        Ok(serde_json::to_string_pretty(&raw)?)
    }
}

pub fn load_config_or_default(path: impl AsRef<Path>) -> InputConfig {
    InputConfig::from_file(path).unwrap_or_default()
}

pub fn save_config(path: impl AsRef<Path>, cfg: &InputConfig) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = cfg.to_json_pretty()?;
    std::fs::write(path, text)?;
    Ok(())
}

pub fn update_caps_interceptor(path: impl AsRef<Path>, enabled: bool) -> anyhow::Result<()> {
    let mut cfg = load_config_or_default(&path);
    cfg.caps_interceptor = enabled;
    save_config(path, &cfg)
}

fn default_caps_interceptor() -> bool {
    true
}

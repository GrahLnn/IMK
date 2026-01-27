use std::collections::HashSet;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct InputConfig {
    pub eng: HashSet<String>,
    pub chinese: HashSet<String>,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            eng: HashSet::new(),
            chinese: HashSet::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct RawConfig {
    #[serde(default, rename = "ENG")]
    eng: Vec<String>,
    #[serde(default, rename = "CHINESE")]
    chinese: Vec<String>,
}

impl InputConfig {
    pub fn from_json_str(s: &str) -> anyhow::Result<Self> {
        let raw: RawConfig = serde_json::from_str(s).context("parse config json")?;
        Ok(Self {
            eng: raw.eng.into_iter().collect(),
            chinese: raw.chinese.into_iter().collect(),
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
        };
        Ok(serde_json::to_string_pretty(&raw)?)
    }
}

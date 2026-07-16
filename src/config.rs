use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub show_notification: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_notification: true,
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let config = toml::from_slice(&bytes)?;
        Ok(config)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let contents = toml::to_string(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

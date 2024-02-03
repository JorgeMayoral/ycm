use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use eyre::Result;
use serde::{Deserialize, Serialize};

type ConfigName = String;
type ConfigPath = PathBuf;

const CONFIGS_DB_VERSION: u32 = 0;
pub const CONFIGS_DB_NAME: &str = "configs_db.json";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    /// Where the configuration file is stored
    source: ConfigPath,
    /// Where the configuration file is linked to
    target: ConfigPath,
}

impl Config {
    pub fn create_link(&self) -> Result<()> {
        log::info!(
            "Creating link from {} to {}",
            self.source.display(),
            self.target.display()
        );
        std::fs::copy(&self.target, &self.source)?;
        std::fs::remove_file(&self.target)?;
        std::fs::hard_link(&self.source, &self.target)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigsDB {
    version: u32,
    configs: HashMap<ConfigName, Vec<Config>>,
}

impl ConfigsDB {
    pub fn try_from_path(path: &PathBuf) -> Result<Self> {
        let db_json = std::fs::read_to_string(path)?;
        let db = serde_json::from_str(&db_json)?;
        Ok(db)
    }

    pub fn add_config(&mut self, name: ConfigName, source: ConfigPath, target: ConfigPath) {
        let config = Config { source, target };
        let entry = self.configs.entry(name).or_default();
        if !entry.contains(&config) {
            entry.push(config);
        }
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let db_json = serde_json::to_string_pretty(&self)?;
        let path = path.join(CONFIGS_DB_NAME);
        std::fs::write(path, db_json)?;
        Ok(())
    }

    pub fn get_configs(&self, name: &str) -> Option<&Vec<Config>> {
        self.configs.get(name)
    }

    pub fn get_all_configs(&self) -> Vec<Config> {
        self.configs.values().flatten().cloned().collect()
    }
}

impl Default for ConfigsDB {
    fn default() -> Self {
        Self {
            version: CONFIGS_DB_VERSION,
            configs: HashMap::new(),
        }
    }
}

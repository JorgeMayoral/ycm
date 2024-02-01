use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use eyre::{ContextCompat, Ok, Result};

type ConfigName = String;
type ConfigPath = PathBuf;
type TargetConfigs = HashMap<ConfigName, Vec<ConfigPath>>;

#[derive(Debug, serde::Deserialize)]
struct Config {
    name: ConfigName,
    source_dir: Option<ConfigPath>,
    filename: Option<ConfigPath>,
}

impl Config {
    fn target_config(&self) -> Result<(ConfigName, Vec<ConfigPath>)> {
        let name = self.name.clone();
        let mut config_paths = Vec::new();
        if let Some(filename) = &self.filename {
            config_paths.push(filename.clone());
        };
        if let Some(source_dir) = &self.source_dir {
            for entry in std::fs::read_dir(source_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    config_paths.push(path);
                }
            }
        };

        Ok((name, config_paths))
    }
}

#[derive(Debug, serde::Deserialize)]
struct ConfigFile {
    version: u32,
    configs: Vec<Config>,
}

impl ConfigFile {
    fn from_path(path: PathBuf) -> Result<Self> {
        let rdr = std::io::BufReader::new(std::fs::File::open(path)?);
        let config_file: Self = serde_json::from_reader(rdr)?;
        Ok(config_file)
    }

    fn target_configs(&self) -> Result<TargetConfigs> {
        let mut target_configs = HashMap::new();
        for config in &self.configs {
            let (name, config_paths) = config.target_config()?;
            target_configs.insert(name, config_paths);
        }
        Ok(target_configs)
    }
}

fn main() -> Result<()> {
    let config_file = ConfigFile::from_path(fs::canonicalize("./examples/config.json")?)?;
    let target_configs = config_file.target_configs()?;
    generate_links(target_configs)?;

    Ok(())
}

const CONFIG_MANAGER_FOLDER: &str = "config-manager";

fn create_config_dir(name: &str) -> Result<()> {
    let base_dirs = directories::BaseDirs::new().context("Could not get config directory")?;
    let config_dir = base_dirs.config_dir();
    let config_manager_dir = config_dir.join(CONFIG_MANAGER_FOLDER).join(name);
    std::fs::create_dir_all(config_manager_dir)?;
    Ok(())
}

fn copy_configs(saved_config_path: &Path, config_paths: &Vec<ConfigPath>) -> Result<()> {
    for config_path in config_paths {
        std::fs::copy(
            config_path,
            saved_config_path.join(config_path.file_name().unwrap()),
        )?;
    }
    Ok(())
}

fn generate_links(target_configs: TargetConfigs) -> Result<()> {
    for (name, config_paths) in target_configs {
        let base_dirs = directories::BaseDirs::new().context("Could not get config directory")?;
        let config_dir = base_dirs.config_dir();
        let saved_configs_dir = config_dir.join(CONFIG_MANAGER_FOLDER).join(name.clone());
        create_config_dir(&name)?;
        copy_configs(&saved_configs_dir, config_paths.as_ref())?;
        for config_path in config_paths {
            let saved_config_path = saved_configs_dir.join(config_path.file_name().unwrap());
            let config_path = config_path.canonicalize()?;
            std::fs::remove_file(&config_path)?;
            std::fs::hard_link(saved_config_path, config_path)?;
        }
    }

    Ok(())
}

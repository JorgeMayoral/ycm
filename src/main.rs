use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use configs_db::ConfigsDB;
use eyre::{ContextCompat, Result};

mod configs_db;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(short, long)]
        name: String,
        #[command(flatten)]
        input: Input,
    },
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct Input {
    #[arg(short, long)]
    dir: Option<PathBuf>,
    #[arg(short, long)]
    file: Option<PathBuf>,
}

const CONFIG_MANAGER_FOLDER: &str = "config-manager";

type ConfigDirPath = PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command;

    let config_manager_dir = init_configs_dir()?;
    let mut db = init_configs_db(&config_manager_dir)?;

    match command {
        Commands::Add { name, input } => {
            let Input { dir, file } = input;
            if let Some(dir) = dir {
                let entries = std::fs::read_dir(dir)?;
                for entry in entries {
                    let entry = entry?;
                    let file = entry.path();
                    write_config(&mut db, &name, file, &config_manager_dir)?;
                }
            }
            if let Some(file) = file {
                write_config(&mut db, &name, file, &config_manager_dir)?;
            }
            let configs = db.get_configs(&name).unwrap();
            for config in configs {
                config.create_link()?;
            }
        }
    }

    Ok(())
}

fn init_configs_dir() -> Result<ConfigDirPath> {
    let base_dirs = directories::BaseDirs::new().context("Could not get config directory")?;
    let config_dir = base_dirs.config_dir();
    let config_manager_dir = config_dir.join(CONFIG_MANAGER_FOLDER);
    if config_manager_dir.exists() {
        return Ok(config_manager_dir);
    }
    std::fs::create_dir_all(&config_manager_dir)?;
    Ok(config_manager_dir)
}

fn init_configs_db(path: &Path) -> Result<ConfigsDB> {
    let db_filename = configs_db::CONFIGS_DB_NAME;
    let db_path = path.join(db_filename);
    if db_path.exists() {
        let db = ConfigsDB::try_from_path(&db_path)?;
        return Ok(db);
    }
    let db = configs_db::ConfigsDB::default();
    let db_json = serde_json::to_string_pretty(&db)?;
    std::fs::write(db_path, db_json)?;
    Ok(db)
}

fn write_config(
    db: &mut ConfigsDB,
    config_name: &str,
    file: PathBuf,
    config_manager_dir: &Path,
) -> Result<()> {
    let filename = file.file_name().context("Could not get file name")?;
    let config_dir = config_manager_dir.canonicalize()?.join(config_name);
    if !config_dir.exists() {
        std::fs::create_dir(&config_dir)?;
    }
    let source_path = config_dir.join(filename);
    let file = file.canonicalize()?;
    db.add_config(config_name.to_string(), source_path, file);
    db.write(config_manager_dir)?;
    Ok(())
}

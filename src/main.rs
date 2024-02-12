use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use configs_db::ConfigsDB;
use eyre::{ContextCompat, Result};

mod configs_db;
mod git;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(short, long)]
        name: String,
        #[command(flatten)]
        input: Input,
    },
    Push,
    Pull,
    Clone {
        #[arg(short, long)]
        url: String,
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

const CONFIG_MANAGER_FOLDER: &str = "ycm";

type ConfigDirPath = PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command;

    let logger_level = cli.verbose.log_level_filter();
    pretty_env_logger::formatted_builder()
        .filter_level(logger_level)
        .init();

    match command {
        Commands::Add { name, input } => {
            let (config_manager_dir, mut db) = initialize()?;
            log::info!("Adding config: {name}");
            let Input { dir, file } = input;
            if let Some(dir) = dir {
                write_dir(&mut db, &name, dir, &config_manager_dir)?;
            }
            if let Some(file) = file {
                write_config(&mut db, &name, file, &config_manager_dir)?;
            }
            let configs = db.get_configs(&name).unwrap();
            for config in configs {
                config.create_link()?;
            }
        }
        Commands::Push => {
            let (config_manager_dir, _) = initialize()?;
            if !git::is_git_repo(&config_manager_dir)? {
                git::init_repository(&config_manager_dir)?;
            }
            if !git::has_remote(&config_manager_dir)? {
                git::create_remote(&config_manager_dir)?;
            }
            git::commit_all(&config_manager_dir)?;
            git::push_to_remote(&config_manager_dir)?;
        }
        Commands::Pull => {
            let (config_manager_dir, _) = initialize()?;
            git::pull_from_remote(&config_manager_dir)?;
        }
        Commands::Clone { url } => {
            let dir = get_base_dir()?;
            git::clone_repo(&url, CONFIG_MANAGER_FOLDER, &dir)?;
            let db = ConfigsDB::try_from_path(
                &dir.join(CONFIG_MANAGER_FOLDER)
                    .join(configs_db::CONFIGS_DB_NAME),
            )?;
            let configs = db.get_all_configs();
            for config in configs {
                config.create_link()?;
            }
        }
    }

    Ok(())
}

fn initialize() -> Result<(PathBuf, ConfigsDB)> {
    let config_manager_dir = init_configs_dir()?;
    let db = init_configs_db(&config_manager_dir)?;
    Ok((config_manager_dir, db))
}

fn get_base_dir() -> Result<PathBuf> {
    let base_dirs = directories::BaseDirs::new().context("Could not get config directory")?;
    let config_dir = base_dirs.config_dir();
    Ok(config_dir.to_path_buf())
}

fn init_configs_dir() -> Result<ConfigDirPath> {
    let config_dir = get_base_dir()?;
    let config_manager_dir = config_dir.join(CONFIG_MANAGER_FOLDER);
    if config_manager_dir.exists() {
        return Ok(config_manager_dir);
    }
    log::warn!("Config manager directory does not exist, creating it");
    std::fs::create_dir_all(&config_manager_dir)?;
    Ok(config_manager_dir)
}

fn init_configs_db(path: &Path) -> Result<ConfigsDB> {
    let db_filename = configs_db::CONFIGS_DB_NAME;
    let db_path = path.join(db_filename);
    if db_path.exists() {
        log::info!("Loading existing config database");
        let db = ConfigsDB::try_from_path(&db_path)?;
        return Ok(db);
    }
    log::warn!("Config database does not exist, creating it");
    let db = configs_db::ConfigsDB::default();
    let db_json = serde_json::to_string_pretty(&db)?;
    std::fs::write(db_path, db_json)?;
    Ok(db)
}

fn write_dir(
    db: &mut ConfigsDB,
    config_name: &str,
    dir: PathBuf,
    config_manager_dir: &Path,
) -> Result<()> {
    log::info!("Adding configs from directory: {}", dir.display());
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            write_dir(db, config_name, entry.path(), config_manager_dir)?;
            continue;
        }
        let file = entry.path();
        write_config(db, config_name, file, config_manager_dir)?;
    }

    Ok(())
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
        log::warn!("Config directory does not exist, creating it");
        std::fs::create_dir(&config_dir)?;
    }
    log::info!("Writing config: {}", filename.to_string_lossy());
    let source_path = config_dir.join(filename);
    let file = file.canonicalize()?;
    db.add_config(config_name.to_string(), source_path, file);
    db.write(config_manager_dir)?;
    Ok(())
}

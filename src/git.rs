use std::path::Path;

use eyre::Result;
use git2::Repository;

use crate::utils::log_cmd_output;

const REMOTE_NAME: &str = "origin";

pub fn get_repository(dir: &Path) -> Result<Repository> {
    match git2::Repository::open(dir) {
        Ok(repo) => Ok(repo),
        Err(_) => {
            log::warn!("Config manager directory is not a git repository, initializing it");
            let repo = git2::Repository::init(dir)?;
            std::process::Command::new("git")
                .arg("branch")
                .arg("-M")
                .arg("main")
                .current_dir(dir)
                .status()?;
            Ok(repo)
        }
    }
}

pub fn create_remote(repo: &Repository) -> Result<()> {
    match repo.find_remote(REMOTE_NAME) {
        Ok(_) => {}
        Err(_) => {
            log::warn!("No remote named 'origin', adding it");
            let remote = inquire::Text::new("Remote URL: (https://github.com/user/configs.git) > ")
                .prompt()?;
            repo.remote(REMOTE_NAME, &remote)?;
        }
    };
    Ok(())
}

fn get_commit_msg() -> String {
    let now = chrono::Local::now();
    let date = now.format("%Y-%m-%d %H:%M:%S");
    format!("Configs updated - {date}")
}

pub fn commit_all(repo: &Repository) -> Result<()> {
    log::info!("Committing changes");
    let output = std::process::Command::new("git")
        .arg("add")
        .arg("-A")
        .current_dir(repo.workdir().unwrap())
        .output()?;
    log_cmd_output("Git add", output);
    let output = std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(get_commit_msg())
        .current_dir(repo.workdir().unwrap())
        .output()?;
    log_cmd_output("Git Commit", output);
    Ok(())
}

pub fn push_to_remote(repo: &Repository) -> Result<()> {
    log::info!("Pushing changes to remote");
    let output = std::process::Command::new("git")
        .arg("push")
        .arg(REMOTE_NAME)
        .arg("main")
        .current_dir(repo.workdir().unwrap())
        .output()?;
    log_cmd_output("Git Push", output);
    Ok(())
}

use std::path::Path;

use eyre::Result;

use crate::utils::log_cmd_output;

const REMOTE_NAME: &str = "origin";

pub fn is_git_repo(dir: &Path) -> Result<bool> {
    std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(dir)
        .status()
        .map(|status| status.success())
        .map_err(Into::into)
}

pub fn init_repository(dir: &Path) -> Result<()> {
    log::info!("Initializing git repository");
    let output = std::process::Command::new("git")
        .arg("init")
        .current_dir(dir)
        .output()?;
    log_cmd_output("Git Init", output);
    let output = std::process::Command::new("git")
        .arg("branch")
        .arg("-M")
        .arg("main")
        .current_dir(dir)
        .output()?;
    log_cmd_output("Git Branch", output);
    Ok(())
}

pub fn has_remote(dir: &Path) -> Result<bool> {
    std::process::Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg(REMOTE_NAME)
        .current_dir(dir)
        .status()
        .map(|status| status.success())
        .map_err(Into::into)
}

pub fn create_remote(dir: &Path) -> Result<()> {
    log::warn!("No remote named 'origin', adding it");
    let remote =
        inquire::Text::new("Remote URL: (https://github.com/user/configs.git) > ").prompt()?;
    let output = std::process::Command::new("git")
        .arg("remote")
        .arg("add")
        .arg(REMOTE_NAME)
        .arg(remote)
        .current_dir(dir)
        .output()?;
    log_cmd_output("Git Remote Add", output);
    Ok(())
}

fn get_commit_msg() -> String {
    let now = chrono::Local::now();
    let date = now.format("%Y-%m-%d %H:%M:%S");
    format!("Configs updated - {date}")
}

pub fn commit_all(dir: &Path) -> Result<()> {
    log::info!("Committing changes");
    let output = std::process::Command::new("git")
        .arg("add")
        .arg("-A")
        .current_dir(dir)
        .output()?;
    log_cmd_output("Git add", output);
    let output = std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(get_commit_msg())
        .current_dir(dir)
        .output()?;
    log_cmd_output("Git Commit", output);
    Ok(())
}

pub fn push_to_remote(dir: &Path) -> Result<()> {
    log::info!("Pushing changes to remote");
    let output = std::process::Command::new("git")
        .arg("push")
        .arg(REMOTE_NAME)
        .arg("main")
        .current_dir(dir)
        .output()?;
    log_cmd_output("Git Push", output);
    Ok(())
}

pub fn pull_from_remote(dir: &Path) -> Result<()> {
    log::info!("Pulling changes from remote");
    if !is_git_repo(dir)? {
        log::error!("Not a git repository, skipping pull");
        return Ok(());
    }
    if !has_remote(dir)? {
        log::error!("No remote named 'origin', skipping pull");
        return Ok(());
    }
    let output = std::process::Command::new("git")
        .arg("pull")
        .arg(REMOTE_NAME)
        .arg("main")
        .current_dir(dir)
        .output()?;
    log_cmd_output("Git Pull", output);
    Ok(())
}

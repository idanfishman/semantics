use anyhow::{Ok, Result};
use git2::Repository;
use std::path::Path;

use crate::commit_analyzer::analyzer::Analyzer;
use crate::config::Config;
use crate::git::detect_current_branch;
use crate::release_channel;

pub fn analyze(config_path: &Path) -> Result<()> {
    let repo = Repository::open(".")?;
    let branch_name = detect_current_branch(&repo)?;

    let cfg = Config::from_file(config_path)?;

    let commit_analyzer = Analyzer::new()?;

    let release_channel =
        release_channel::find_release_channel_by_branch(&branch_name, cfg.release_channels())?;

    Ok(())
}

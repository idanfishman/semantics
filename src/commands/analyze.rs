use std::path::PathBuf;

use anyhow::{Ok, Result};
use git2::Repository;

use crate::config::Config;
use crate::release_channel::resolve_target_channel;

pub struct AnalyzeArgs {
    pub channel: Option<String>,
    pub config: PathBuf,
    pub repo: PathBuf,
}

pub fn analyze(args: AnalyzeArgs) -> Result<()> {
    let config = Config::from_file(&args.config)?;
    let repo = Repository::open(&args.repo)?;
    let channel = resolve_target_channel(&repo, &config.release_channels, args.channel.as_deref())?;

    Ok(())
}

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command, ConfigCommand};

mod cli;
mod commands;
mod commit_analyzer;
mod config;
mod git;
mod release_channel;
mod utils;

#[cfg(test)]
mod test_helpers;

fn main() -> Result<()> {
    let args = Cli::parse();

    let config_path = args.config;

    match args.cmd {
        Command::Analyze {
            repo,
            release_channel,
        } => {
            commands::analyze::analyze(&config_path, &repo, release_channel.as_deref())?;
        }
        Command::Config(ConfigCommand::Init { force }) => {
            commands::config::init(&config_path, force)?;
        }
        Command::Config(ConfigCommand::Show) => {
            commands::config::show(&config_path)?;
        }
    }

    Ok(())
}

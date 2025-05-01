use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command, ConfigCommand};

mod changelog_generator;
mod cli;
mod commands;
mod commit_analyzer;
mod config;
mod git;
mod release_channel;
mod semver;

#[cfg(test)]
mod test_helpers;

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Analyze {
            channel,
            config,
            repo,
        } => {}
        Command::Bump {
            channel,
            dry_run,
            config,
            repo,
            subcommand,
        } => {}
        Command::Changelog {
            channel,
            dry_run,
            config,
            repo,
        } => {}
        Command::Release {
            channel,
            dry_run,
            config,
            repo,
        } => {}
        Command::Config { subcommand } => match subcommand {
            ConfigCommand::Init { output, force } => {
                commands::config::init(&output, force)?;
            }
            ConfigCommand::Show { config } => {
                commands::config::show(&config)?;
            }
        },
    }

    Ok(())
}

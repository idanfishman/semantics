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

use anyhow::{Ok, Result};

pub use cli::Cli;
use cli::{Command, ConfigCommand};

pub fn run(args: Cli) -> Result<()> {
    match args.command {
        Command::Analyze {
            channel,
            config,
            repo,
        } => {
            commands::analyze::analyze(commands::analyze::AnalyzeArgs {
                channel,
                config,
                repo,
            })?;
        }
        Command::Bump {
            channel,
            dry_run,
            config,
            repo,
            subcommand,
        } => {
            commands::bump::bump(commands::bump::BumpArgs {
                channel,
                dry_run,
                config,
                repo,
            })?;
        }
        Command::Changelog {
            channel,
            dry_run,
            config,
            repo,
        } => {
            commands::changelog::changelog(commands::changelog::ChangelogArgs {
                channel,
                dry_run,
                config,
                repo,
            })?;
        }
        Command::Release {
            channel,
            dry_run,
            config,
            repo,
        } => {
            commands::release::release(commands::release::ReleaseArgs {
                channel,
                dry_run,
                config,
                repo,
            })?;
        }
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

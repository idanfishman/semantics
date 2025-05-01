use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

/// Command-line interface for semantics.
#[derive(Parser)]
#[command(version, author, about, long_about = None)]
pub struct Cli {
    /// The command to execute.
    #[clap(subcommand)]
    pub command: Command,
}

/// Available commands for semantics CLI.
#[derive(Subcommand)]
pub enum Command {
    /// Analyze commit history and suggest the next version bump.
    Analyze {
        /// Release channel to operate on.
        #[arg(long)]
        channel: Option<String>,

        /// Path to the configuration file.
        #[arg(long, default_value = ".semantics.json")]
        config: PathBuf,

        /// Path to the repository root.
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Create the next version tag.
    Bump {
        /// Release channel to operate on.
        #[arg(long)]
        channel: Option<String>,

        /// Preview the result without applying changes.
        #[arg(long)]
        dry_run: bool,

        /// Path to the configuration file.
        #[arg(long, default_value = ".semantics.json")]
        config: PathBuf,

        /// Path to the repository root.
        #[arg(long, default_value = ".")]
        repo: PathBuf,

        /// Subcommand to specify the type of version bump.
        #[command(subcommand)]
        subcommand: BumpSubcommand,
    },
    /// Generate and save a changelog from commit history.
    Changelog {
        /// Release channel to operate on.
        #[arg(long)]
        channel: Option<String>,

        /// Preview the changelog without saving.
        #[arg(long)]
        dry_run: bool,

        /// Path to the configuration file.
        #[arg(long, default_value = ".semantics.json")]
        config: PathBuf,

        /// Path to the repository root.
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Prepare and publish a full release.
    Release {
        /// Release channel to operate on.
        #[arg(long)]
        channel: Option<String>,

        /// Preview release notes without tagging.
        #[arg(long)]
        dry_run: bool,

        /// Path to the configuration file.
        #[arg(long, default_value = ".semantics.json")]
        config: PathBuf,

        /// Path to the repository root.
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Manage the CLI configuration file.
    Config {
        /// Subcommand to manage configuration settings.
        #[command(subcommand)]
        subcommand: ConfigCommand,
    },
}

/// Subcommands for the `bump` command.
///
/// The `BumpSubcommand` enum defines the specific types of version bumps that can be performed.
#[derive(Subcommand)]
pub enum BumpSubcommand {
    /// Increase major version (e.g., 1.0.0 → 2.0.0).
    Major,
    /// Increase minor version (e.g., 1.0.0 → 1.1.0).
    Minor,
    /// Increase patch version (e.g., 1.0.0 → 1.0.1).
    Patch,
    /// Increase prerelease (e.g., 1.0.0-alpha.1 → 1.0.0-alpha.2).
    Prerelease,
}

/// Subcommands for the `config` command.
///
/// The `ConfigCommand` enum defines operations for managing the CLI configuration file.
#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Generate and save a config file.
    Init {
        /// Path to the configuration file.
        #[arg(long, default_value = ".semantics.json")]
        output: PathBuf,

        /// Overwrite the configuration file if it already exists.
        #[arg(long, action = ArgAction::SetTrue)]
        force: bool,
    },
    /// Display loaded configuration settings.
    Show {
        /// Path to the configuration file.
        #[arg(long, default_value = ".semantics.json")]
        config: PathBuf,
    },
}

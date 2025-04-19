use std::path::PathBuf;

use clap::{ArgAction, Args, Parser};

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
pub struct Cli {
    /// Path to the config file
    #[arg(global = true, long, default_value = ".semantics.json")]
    pub config: PathBuf,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    Analyze {
        /// Path to the repository
        #[arg(long, default_value = ".")]
        repo: PathBuf,

        #[arg(long)]
        release_channel: Option<String>,
    },

    #[command(subcommand)]
    Config(ConfigCommand),
}

#[derive(Parser, Debug)]
pub enum ConfigCommand {
    /// Create a config file
    Init {
        /// Force overwrite existing config file
        #[arg(long, action = ArgAction::SetTrue)]
        force: bool,
    },

    /// Display the contents of the config file
    Show,
}

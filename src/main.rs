use anyhow::Result;
use clap::Parser;
use semantics::{Cli, run};

fn main() -> Result<()> {
    let args = Cli::parse();

    run(args)
}

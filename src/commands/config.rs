use std::path::Path;

use anyhow::{Ok, Result};
use serde_json::to_writer_pretty;

use crate::config::Config;

pub fn init(path: &Path, force: bool) -> Result<()> {
    if !force && path.exists() {
        println!(
            "config file already exists at {:?}. use the '--force' flag to overwrite it.",
            path
        );
        return Ok(());
    }

    let cfg = Config::default();
    cfg.save_to_file(path)?;
    println!("config file created at {:?}", path);

    Ok(())
}

pub fn show(path: &Path) -> Result<()> {
    let cfg = Config::from_file(path)?;
    to_writer_pretty(std::io::stdout(), &cfg)?;

    Ok(())
}

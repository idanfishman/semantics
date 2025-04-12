use std::path::Path;

use crate::config::Config;

pub fn init(path: &Path, force: bool) {
    if !force && path.exists() {
        println!(
            "config file already exists at {:?}. use the '--force' flag to overwrite it.",
            path
        );
        return;
    }

    let cfg = Config::default();
    match cfg.save_to_file(path) {
        Ok(_) => println!("config saved to {:?}", path),
        Err(e) => eprintln!("error: {:#}", e),
    }
}

pub fn show(path: &Path) {
    let cfg = Config::from_file(path);
    match cfg {
        Ok(cfg) => serde_json::to_writer_pretty(std::io::stdout(), &cfg).unwrap(),
        Err(e) => eprintln!("error: {:#}", e),
    }
}
